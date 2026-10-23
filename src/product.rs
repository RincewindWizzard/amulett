//! Product firmware: choose a faction at start, then show friends as a colour
//! (saturation = how many, brightness = how close) and warn about enemies.

use alloc::{format, string::String, vec::Vec};
use core::cell::RefCell;

use amulett_core::{
    beacon::Beacon,
    color::{hsv_to_rgb565, target_brightness, target_saturation, ColorController},
    danger::{alarm_rgb565, Danger},
    faction::Faction,
    friends::{FriendTable, Measures},
    params,
};
use embedded_graphics::{
    pixelcolor::{raw::RawU16, Rgb565},
    prelude::*,
    primitives::{Circle, PrimitiveStyle},
};
use embedded_hal_bus::i2c::RefCellDevice;
use esp_hal::{
    delay::Delay,
    dma_loop_buffer, dma_rx_buffer, dma_rx_stream_buffer, dma_tx_buffer,
    gpio::{Input, InputConfig, Level, Output, OutputConfig, Pull},
    i2c::master::{Config as I2cConfig, I2c},
    i2s::master::{Channels, DataFormat, I2s, TdmConfig},
    spi::{
        master::{Config as SpiConfig, Spi},
        Mode,
    },
    time::{Instant, Rate},
};
use esp_println::println;

use crate::{
    audio::{self, AudioIo},
    display::{self, Co5300, Framebuffer},
    expander::Tca9554,
    input::{Gesture, TouchEvent},
    touch::{self, Cst9217},
    ui, wireless,
};

/// Fake neighbours of the demo mode: RSSI of the friends (near to far) and of the enemy levels.
const DEMO_FRIEND_RSSI: [i8; 6] = [-48, -58, -68, -78, -55, -65];
const DEMO_ENEMY_RSSI: [Option<i8>; 4] = [None, Some(-85), Some(-65), Some(-48)];
const DEMO_BEACON_INTERVAL_MS: u32 = 200;
const OVERLAY_INTERVAL_MS: u32 = 250;
const REPORT_INTERVAL_MS: u32 = 1000;
const BUTTON_POLL_MS: u32 = 30;

type Bus<'a> = RefCellDevice<'a, I2c<'a, esp_hal::Blocking>>;

fn now_ms() -> u32 {
    Instant::now().duration_since_epoch().as_millis() as u32
}

/// Device id: low four bytes of the factory MAC address.
fn own_id() -> u32 {
    let mac = esp_hal::efuse::base_mac_address();
    let b = mac.as_bytes();
    u32::from_le_bytes([b[2], b[3], b[4], b[5]])
}

fn rgb565(raw: u16) -> Rgb565 {
    Rgb565::from(RawU16::new(raw))
}

/// Edge detection for the two side buttons.
struct Buttons<'d> {
    boot: Input<'d>,
    boot_was_down: bool,
    pwr_was_down: bool,
}

impl<'d> Buttons<'d> {
    fn new(boot: Input<'d>) -> Self {
        let boot_was_down = boot.is_low();
        Self { boot, boot_was_down, pwr_was_down: true }
    }

    /// Returns (BOOT pressed, PWR pressed) since the last call.
    fn poll(&mut self, expander: &mut Tca9554<Bus<'_>>) -> (bool, bool) {
        let boot_down = self.boot.is_low();
        let boot = boot_down && !self.boot_was_down;
        self.boot_was_down = boot_down;
        let pwr_down = expander.pwr_pressed().unwrap_or(false);
        let pwr = pwr_down && !self.pwr_was_down;
        self.pwr_was_down = pwr_down;
        (boot, pwr)
    }
}

// --8<-- [start:select]
/// Start screen: BOOT switches between A and B, a short press on PWR confirms.
fn select_faction(
    panel: &mut Co5300<'_>,
    fb: &mut Framebuffer,
    buttons: &mut Buttons<'_>,
    expander: &mut Tca9554<Bus<'_>>,
    delay: &Delay,
) -> Faction {
    let mut selected = Faction::A;
    let mut changed = true;
    loop {
        let (boot, pwr) = buttons.poll(expander);
        if boot {
            selected = selected.other();
            changed = true;
        }
        if pwr {
            return selected;
        }
        if changed {
            changed = false;
            let _ = fb.clear(Rgb565::BLACK);
            ui::centered(fb, 90, Rgb565::WHITE, "Fraktion waehlen");
            for (faction, cx) in [(Faction::A, 140), (Faction::B, 326)] {
                let color = rgb565(hsv_to_rgb565(faction.hue(), 1.0, 1.0));
                let _ = Circle::with_center(Point::new(cx, 233), 120)
                    .into_styled(PrimitiveStyle::with_fill(color))
                    .draw(fb);
                if faction == selected {
                    let _ = Circle::with_center(Point::new(cx, 233), 140)
                        .into_styled(PrimitiveStyle::with_stroke(Rgb565::WHITE, 6))
                        .draw(fb);
                }
                let mut letter = String::new();
                letter.push(faction.letter());
                ui::centered_at(fb, cx, 240, Rgb565::WHITE, &letter);
            }
            ui::centered(fb, 360, Rgb565::WHITE, "BOOT: wechseln");
            ui::centered(fb, 385, Rgb565::WHITE, "PWR kurz: bestaetigen");
            if let Err(e) = fb.flush(panel) {
                println!("flush failed: {e:?}");
            }
        }
        delay.delay_millis(BUTTON_POLL_MS);
    }
}
// --8<-- [end:select]

struct Demo {
    active: bool,
    friends: usize,
    /// Index into `DEMO_ENEMY_RSSI`.
    enemy_level: usize,
    last_beacon_ms: u32,
}

impl Demo {
    const FRIEND_BASE_ID: u32 = 0xD000_0000;
    const ENEMY_ID: u32 = 0xE000_0000;

    fn feed(&mut self, table: &mut FriendTable, now: u32) {
        if !self.active || now.wrapping_sub(self.last_beacon_ms) < DEMO_BEACON_INTERVAL_MS {
            return;
        }
        self.last_beacon_ms = now;
        let own = table.own_faction();
        for i in 0..self.friends {
            let rssi = DEMO_FRIEND_RSSI[i % DEMO_FRIEND_RSSI.len()];
            table.on_beacon(Self::FRIEND_BASE_ID + i as u32, own.id(), rssi, now);
        }
        if let Some(rssi) = DEMO_ENEMY_RSSI[self.enemy_level] {
            table.on_beacon(Self::ENEMY_ID, own.other().id(), rssi, now);
        }
    }

    fn clear(&mut self) {
        self.friends = 0;
        self.enemy_level = 0;
    }
}

pub fn run(peripherals: esp_hal::peripherals::Peripherals) -> ! {
    let delay = Delay::new();

    // The radio needs internal RAM; the framebuffer goes to PSRAM.
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
    let mut buttons = Buttons::new(Input::new(
        peripherals.GPIO0,
        InputConfig::default().with_pull(Pull::Up),
    ));

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
    let mut fb = Framebuffer::new();

    // Optional warning tone: only set up the audio hardware if it is enabled.
    let mut audio_io: Option<AudioIo<'_>> = if params::WARN_TONE {
        let mut codec = audio::es8311::Es8311::new(RefCellDevice::new(&bus));
        let mut mic_adc = audio::es7210::Es7210::new(RefCellDevice::new(&bus));
        let i2s = I2s::new(
            peripherals.I2S0,
            peripherals.DMA_CH1,
            TdmConfig::new_tdm_philips()
                .with_sample_rate(Rate::from_hz(audio::SAMPLE_RATE))
                .with_data_format(DataFormat::Data16Channel16)
                .with_channels(Channels::STEREO)
                .with_signal_loopback(true),
        )
        .expect("invalid I2S config")
        .with_mclk(peripherals.GPIO42);
        let tx = i2s
            .i2s_tx
            .with_bclk(peripherals.GPIO9)
            .with_ws(peripherals.GPIO45)
            .with_dout(peripherals.GPIO8)
            .build();
        let rx = i2s.i2s_rx.with_din(peripherals.GPIO10).build();
        delay.delay_millis(10);
        if let Err(e) = codec.init(&delay).and_then(|_| codec.set_volume(70)) {
            println!("ES8311 init failed: {e:?}");
        }
        if let Err(e) = mic_adc.init() {
            println!("ES7210 init failed: {e:?}");
        }
        let amplifier = Output::new(peripherals.GPIO46, Level::Low, OutputConfig::default());
        Some(AudioIo::new(
            tx,
            dma_loop_buffer!(3840),
            rx,
            dma_rx_stream_buffer!(4 * 4092, 1024),
            amplifier,
        ))
    } else {
        None
    };

    // Start screen: no beacon is sent and none is evaluated until a faction is confirmed.
    let faction = select_faction(&mut panel, &mut fb, &mut buttons, &mut expander, &delay);
    println!("Fraktion {} bestaetigt", faction.letter());
    let hue = faction.hue();

    let id = own_id();
    println!("device id 0x{id:08X}");
    let mut table = FriendTable::new(id, faction);
    let mut radio = match wireless::Radio::start(peripherals.BT, id, faction.id()) {
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
    let mut danger = Danger::default();
    let mut last_color: Option<u16> = None;
    let mut overlay = false;
    let mut last_overlay_ms = 0u32;
    let mut demo = Demo { active: false, friends: 0, enemy_level: 0, last_beacon_ms: 0 };
    let mut gesture = Gesture::default();
    let mut points = [touch::Point { id: 0, x: 0, y: 0 }; 2];
    let mut touch_pos: Option<(i32, i32)> = None;
    let mut last_report_ms = 0u32;
    let (mut writes, mut beacons) = (0u32, 0u32);
    let mut previous_ms = now_ms();

    loop {
        let frame_start = Instant::now();
        let now = now_ms();
        let dt_ms = now.wrapping_sub(previous_ms);
        previous_ms = now;

        // Radio: drain events, update the table.
        if let Some(radio) = radio.as_mut() {
            radio.tick(now);
            radio.poll(|report| {
                if let Some(b) = Beacon::decode(report.data) {
                    beacons += 1;
                    table.on_beacon(b.id, b.faction, report.rssi, now);
                }
            });
        }
        demo.feed(&mut table, now);
        table.expire(now);
        let measures = table.measures();
        color.step(&measures, dt_ms);
        if danger.update(measures.danger) {
            last_color = None; // redraw after the mode change
            println!("danger {} (enemy closeness {:.2})", if danger.active { "on" } else { "off" }, measures.danger);
            if let Some(a) = audio_io.as_mut() {
                if danger.active {
                    a.start_tone();
                } else {
                    a.stop_tone();
                }
            }
        }

        // Buttons: BOOT toggles the debug overlay, PWR the demo mode.
        let (boot, pwr) = buttons.poll(&mut expander);
        if boot {
            overlay = !overlay;
            last_color = None;
            println!("overlay {}", if overlay { "on" } else { "off" });
        }
        if pwr {
            demo.active = !demo.active;
            demo.clear();
            println!("demo mode {}", if demo.active { "on" } else { "off" });
        }

        // Touch (demo mode only): upper half adds a friend, lower half cycles the enemy
        // distance (none, far, middle, near), a long press removes everything.
        if touch_int.is_low() || touch_pos.is_some() {
            touch_pos = match touch.read_points(&mut points) {
                Ok(n) if n > 0 => Some((points[0].x as i32, points[0].y as i32)),
                _ => None,
            };
        }
        match gesture.update(touch_pos, now) {
            Some(TouchEvent::Tap(_, y)) if demo.active => {
                if y < 233 {
                    demo.friends = (demo.friends + 1).min(DEMO_FRIEND_RSSI.len());
                } else {
                    demo.enemy_level = (demo.enemy_level + 1) % DEMO_ENEMY_RSSI.len();
                }
                println!("demo: {} friends, enemy level {}", demo.friends, demo.enemy_level);
            }
            Some(TouchEvent::LongPress) if demo.active => {
                demo.clear();
                println!("demo: cleared");
            }
            _ => {}
        }

        // Display.
        if danger.active {
            let raw = alarm_rgb565(now, measures.danger);
            fb.fill_raw(raw);
            ui::centered(&mut fb, 225, Rgb565::WHITE, "GEFAHR");
            ui::bar(&mut fb, 250, 200, measures.danger, Rgb565::WHITE);
            if let Err(e) = fb.flush(&mut panel) {
                println!("flush failed: {e:?}");
            }
            writes += 1;
        } else {
            let raw = color.rgb565(hue);
            let due = overlay && now.wrapping_sub(last_overlay_ms) >= OVERLAY_INTERVAL_MS;
            if due || (!overlay && last_color != Some(raw)) {
                last_overlay_ms = now;
                last_color = Some(raw);
                fb.fill_raw(raw);
                draw_faction_marker(&mut fb, faction);
                if overlay {
                    ui::overlay(&mut fb, 110, Rgb565::WHITE, &overlay_lines(&table, &measures, &color, demo.active));
                }
                if let Err(e) = fb.flush(&mut panel) {
                    println!("flush failed: {e:?}");
                }
                writes += 1;
            }
        }

        if now.wrapping_sub(last_report_ms) >= REPORT_INTERVAL_MS {
            last_report_ms = now;
            println!(
                "friends={} n_eff={:.2} c_max={:.2} enemies={} danger={:.2} S={:.2} V={:.2} beacons/s={} display_writes={}",
                measures.count, measures.n_eff, measures.c_max, measures.enemies, measures.danger,
                color.saturation, color.brightness, beacons, writes
            );
            for e in table.iter() {
                println!(
                    "  id=0x{:08X} {:?}{} rssi={:.0} dBm rate={:.1}/s",
                    e.id, e.relation, if e.confirmed { "" } else { " (unconfirmed)" }, e.rssi, e.rate_per_s
                );
            }
            beacons = 0;
        }

        let used = frame_start.elapsed().as_millis() as u32;
        if used < params::DISPLAY_INTERVAL_MS {
            delay.delay_millis(params::DISPLAY_INTERVAL_MS - used);
        }
    }
}

/// Faction letter in a dark disc at the top edge, so the faction is readable
/// even when the colour area is grey.
fn draw_faction_marker(fb: &mut Framebuffer, faction: Faction) {
    let _ = Circle::with_center(Point::new(ui::CENTER, 34), 44)
        .into_styled(PrimitiveStyle::with_fill(Rgb565::BLACK))
        .draw(fb);
    let mut letter = String::new();
    letter.push(faction.letter());
    ui::centered(fb, 41, Rgb565::WHITE, &letter);
}

fn overlay_lines(table: &FriendTable, m: &Measures, c: &ColorController, demo: bool) -> Vec<String> {
    let mut lines = Vec::new();
    lines.push(format!(
        "Fraktion {}  Freunde {}{}",
        table.own_faction().letter(),
        m.count,
        if demo { " (DEMO)" } else { "" }
    ));
    lines.push(format!("Anzahl {:.2}  Naehe {:.2}", m.n_eff, m.c_max));
    lines.push(format!(
        "S {:.2}->{:.2}  V {:.2}->{:.2}",
        c.saturation,
        target_saturation(m),
        c.brightness,
        target_brightness(m)
    ));
    lines.push(format!("Feinde {}  Gefahr {:.2}", m.enemies, m.danger));
    for e in table.iter().take(5) {
        let tag = match e.relation {
            amulett_core::faction::Relation::Friend => "F",
            _ => "X",
        };
        lines.push(format!("{tag} {:04X} {:.0}dBm {:.1}/s", e.id & 0xFFFF, e.rssi, e.rate_per_s));
    }
    lines
}
