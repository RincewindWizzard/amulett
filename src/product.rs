//! Product firmware: friends are shown as a colour (saturation = how many,
//! brightness = how close).

use alloc::{format, string::String, vec::Vec};
use core::cell::RefCell;

use amulett_core::{
    beacon::Beacon,
    color::{target_brightness, target_saturation, ColorController},
    friends::{FriendTable, Measures},
    params,
};
use embedded_graphics::{pixelcolor::Rgb565, prelude::RgbColor};
use embedded_hal_bus::i2c::RefCellDevice;
use esp_hal::{
    delay::Delay,
    dma_rx_buffer, dma_tx_buffer,
    gpio::{Input, InputConfig, Level, Output, OutputConfig, Pull},
    i2c::master::{Config as I2cConfig, I2c},
    spi::{
        master::{Config as SpiConfig, Spi},
        Mode,
    },
    time::{Instant, Rate},
};
use esp_println::println;

use crate::{
    display::{self, Co5300, Framebuffer},
    expander::Tca9554,
    input::{Gesture, TouchEvent},
    touch::{self, Cst9217},
    ui, wireless,
};

/// Fake friends of the demo mode: RSSI values from near to far.
const DEMO_RSSI: [i8; 6] = [-48, -58, -68, -78, -55, -65];
const DEMO_BEACON_INTERVAL_MS: u32 = 200;
const OVERLAY_INTERVAL_MS: u32 = 250;
const REPORT_INTERVAL_MS: u32 = 1000;

fn now_ms() -> u32 {
    Instant::now().duration_since_epoch().as_millis() as u32
}

/// Device id: low four bytes of the factory MAC address.
fn own_id() -> u32 {
    let mac = esp_hal::efuse::base_mac_address();
    let b = mac.as_bytes();
    u32::from_le_bytes([b[2], b[3], b[4], b[5]])
}

struct Demo {
    active: bool,
    friends: usize,
    last_beacon_ms: u32,
}

impl Demo {
    const BASE_ID: u32 = 0xD000_0000;

    fn feed(&mut self, table: &mut FriendTable, now: u32) {
        if !self.active || now.wrapping_sub(self.last_beacon_ms) < DEMO_BEACON_INTERVAL_MS {
            return;
        }
        self.last_beacon_ms = now;
        for i in 0..self.friends {
            table.on_beacon(Self::BASE_ID + i as u32, DEMO_RSSI[i % DEMO_RSSI.len()], now);
        }
    }
}

pub fn run(peripherals: esp_hal::peripherals::Peripherals) -> ! {
    let delay = Delay::new();

    // The radio needs internal RAM; the overlay framebuffer goes to PSRAM.
    esp_alloc::heap_allocator!(#[esp_hal::ram(reclaimed)] size: 64 * 1024);
    esp_alloc::heap_allocator!(size: 36 * 1024);
    esp_alloc::psram_allocator!(peripherals.PSRAM, esp_hal::psram);

    let timg0 = esp_hal::timer::timg::TimerGroup::new(peripherals.TIMG0);
    esp_rtos::start(timg0.timer0, peripherals.FROM_CPU_INTR0);

    println!("amulett: booting (product firmware)");

    let i2c = I2c::new(
        peripherals.I2C0,
        I2cConfig::default().with_frequency(Rate::from_khz(400)),
    )
    .expect("invalid I2C config")
    .with_sda(peripherals.GPIO15)
    .with_scl(peripherals.GPIO14);
    let bus = RefCell::new(i2c);

    let mut touch_reset = Output::new(peripherals.GPIO40, Level::Low, OutputConfig::default());
    delay.delay_millis(10);
    touch_reset.set_high();
    delay.delay_millis(50);
    let touch_int = Input::new(peripherals.GPIO11, InputConfig::default().with_pull(Pull::Up));
    let mut touch = Cst9217::new(RefCellDevice::new(&bus));
    let mut expander = Tca9554::new(RefCellDevice::new(&bus));
    if let Err(e) = expander.init() {
        println!("TCA9554 init failed: {e:?}");
    }
    let boot = Input::new(peripherals.GPIO0, InputConfig::default().with_pull(Pull::Up));

    let dma_rx = dma_rx_buffer!(32).unwrap();
    let dma_tx = dma_tx_buffer!(display::CHUNK_BYTES).unwrap();
    let spi = Spi::new(
        peripherals.SPI2,
        SpiConfig::default()
            .with_frequency(Rate::from_mhz(40))
            .with_mode(Mode::_0),
    )
    .expect("invalid SPI config")
    .with_sck(peripherals.GPIO38)
    .with_cs(peripherals.GPIO12)
    .with_sio0(peripherals.GPIO4)
    .with_sio1(peripherals.GPIO5)
    .with_sio2(peripherals.GPIO6)
    .with_sio3(peripherals.GPIO7)
    .with_dma(peripherals.DMA_CH0)
    .with_buffers(dma_rx, dma_tx);
    let lcd_reset = Output::new(peripherals.GPIO39, Level::Low, OutputConfig::default());
    let mut panel = Co5300::new(spi, lcd_reset);
    match panel.init(&delay) {
        Ok(()) => println!("CO5300 display initialised"),
        Err(e) => println!("CO5300 init failed: {e:?}"),
    }

    let id = own_id();
    println!("device id 0x{id:08X}");
    let mut table = FriendTable::new(id);
    let mut radio = match wireless::Radio::start(peripherals.BT, id) {
        Ok(r) => {
            println!(
                "beacon every {} ms, scan {} of {} ms",
                params::ADV_INTERVAL_MS,
                params::SCAN_WINDOW_MS,
                params::SCAN_INTERVAL_MS
            );
            Some(r)
        }
        Err(e) => {
            println!("radio start failed: {e:?}");
            None
        }
    };

    let mut color = ColorController::default();
    let mut last_color: Option<u16> = None;
    let mut fb: Option<Framebuffer> = None;
    let mut overlay = false;
    let mut last_overlay_ms = 0u32;
    let mut demo = Demo { active: false, friends: 0, last_beacon_ms: 0 };
    let mut gesture = Gesture::default();
    let mut points = [touch::Point { id: 0, x: 0, y: 0 }; 2];
    let mut touch_pos: Option<(i32, i32)> = None;
    let (mut boot_was_down, mut pwr_was_down) = (false, false);
    let mut last_report_ms = 0u32;
    let (mut writes, mut beacons) = (0u32, 0u32);
    let mut previous_ms = now_ms();

    loop {
        let frame_start = Instant::now();
        let now = now_ms();
        let dt_ms = now.wrapping_sub(previous_ms);
        previous_ms = now;

        // Radio: drain events, update the friend table.
        if let Some(radio) = radio.as_mut() {
            radio.tick(now);
            radio.poll(|report| {
                if let Some(b) = Beacon::decode(report.data) {
                    beacons += 1;
                    table.on_beacon(b.id, report.rssi, now);
                }
            });
        }
        demo.feed(&mut table, now);
        table.expire(now);
        let measures = table.measures();
        color.step(&measures, dt_ms);

        // Buttons: BOOT toggles the debug overlay, PWR toggles the demo mode.
        let boot_down = boot.is_low();
        if boot_down && !boot_was_down {
            overlay = !overlay;
            last_color = None; // force a redraw when the overlay disappears
            println!("overlay {}", if overlay { "on" } else { "off" });
        }
        boot_was_down = boot_down;
        let pwr_down = expander.pwr_pressed().unwrap_or(false);
        if pwr_down && !pwr_was_down {
            demo.active = !demo.active;
            demo.friends = 0;
            println!("demo mode {}", if demo.active { "on" } else { "off" });
        }
        pwr_was_down = pwr_down;

        // Touch (demo mode only): tap adds a fake friend, long press removes all.
        if touch_int.is_low() || touch_pos.is_some() {
            touch_pos = match touch.read_points(&mut points) {
                Ok(n) if n > 0 => Some((points[0].x as i32, points[0].y as i32)),
                _ => None,
            };
        }
        match gesture.update(touch_pos, now) {
            Some(TouchEvent::Tap(..)) if demo.active => {
                demo.friends = (demo.friends + 1).min(DEMO_RSSI.len());
                println!("demo friends: {}", demo.friends);
            }
            Some(TouchEvent::LongPress) if demo.active => {
                demo.friends = 0;
                println!("demo friends: 0");
            }
            _ => {}
        }

        // Display: plain colour only when it changed, overlay at a low rate.
        let raw = color.rgb565(params::HUE_DEG);
        if overlay {
            if now.wrapping_sub(last_overlay_ms) >= OVERLAY_INTERVAL_MS {
                last_overlay_ms = now;
                let fb = fb.get_or_insert_with(Framebuffer::new);
                fb.fill_raw(raw);
                ui::overlay(fb, 110, Rgb565::WHITE, &overlay_lines(&table, &measures, &color, demo.active));
                if let Err(e) = fb.flush(&mut panel) {
                    println!("flush failed: {e:?}");
                }
                writes += 1;
            }
        } else if last_color != Some(raw) {
            match panel.fill(raw) {
                Ok(()) => {
                    last_color = Some(raw);
                    writes += 1;
                }
                Err(e) => println!("fill failed: {e:?}"),
            }
        }

        if now.wrapping_sub(last_report_ms) >= REPORT_INTERVAL_MS {
            last_report_ms = now;
            println!(
                "friends={} n_eff={:.2} c_max={:.2} S={:.2} V={:.2} beacons/s={} display_writes={}",
                measures.count, measures.n_eff, measures.c_max, color.saturation, color.brightness, beacons, writes
            );
            for f in table.iter() {
                println!("  id=0x{:08X} rssi={:.0} dBm rate={:.1}/s", f.id, f.rssi, f.rate_per_s);
            }
            beacons = 0;
        }

        let used = frame_start.elapsed().as_millis() as u32;
        if used < params::DISPLAY_INTERVAL_MS {
            delay.delay_millis(params::DISPLAY_INTERVAL_MS - used);
        }
    }
}

fn overlay_lines(table: &FriendTable, m: &Measures, c: &ColorController, demo: bool) -> Vec<String> {
    let mut lines = Vec::new();
    lines.push(format!("Freunde: {}{}", m.count, if demo { "  (DEMO)" } else { "" }));
    lines.push(format!("Anzahl {:.2}  Naehe {:.2}", m.n_eff, m.c_max));
    lines.push(format!(
        "S {:.2}->{:.2}  V {:.2}->{:.2}",
        c.saturation,
        target_saturation(m),
        c.brightness,
        target_brightness(m)
    ));
    for f in table.iter().take(6) {
        lines.push(format!("{:04X} {:.0}dBm {:.1}/s", f.id & 0xFFFF, f.rssi, f.rate_per_s));
    }
    lines
}
