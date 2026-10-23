#![no_std]
#![no_main]

extern crate alloc;

#[allow(dead_code)]
mod display;
#[allow(dead_code)]
mod expander;
mod imu;
mod pmic;
#[allow(dead_code)]
mod rtc;
mod touch;
mod ui;

use alloc::{format, string::String, vec::Vec};
use core::cell::RefCell;

use embedded_graphics::{
    pixelcolor::Rgb565,
    prelude::*,
    primitives::{Circle, PrimitiveStyle, Rectangle},
};
use embedded_hal::i2c::I2c as _;
use embedded_hal_bus::i2c::RefCellDevice;
use esp_backtrace as _;
use esp_hal::{
    clock::CpuClock,
    delay::Delay,
    dma_rx_buffer, dma_tx_buffer,
    gpio::{Input, InputConfig, Level, Output, OutputConfig, Pull},
    i2c::master::{Config as I2cConfig, I2c},
    main,
    spi::{
        master::{Config as SpiConfig, Spi},
        Mode,
    },
    time::{Instant, Rate},
};
use esp_println::println;

esp_bootloader_esp_idf::esp_app_desc!();

const PAGE_TITLES: [&str; 4] = ["Testbild + Touch", "Energie (AXP2101)", "IMU (QMI8658)", "Info + I2C"];
const FRAME_MS: u32 = 30;
const SWIPE_MIN_DX: i32 = 100;
const SWIPE_MAX_DY: i32 = 120;
const BRIGHTNESS_LEVELS: [u8; 3] = [0x20, 0x80, 0xFF];

/// Known devices on the shared I2C bus, see `docs/hardware.md`.
const KNOWN_I2C: [(u8, &str); 7] = [
    (0x18, "ES8311"),
    (0x20, "TCA9554"),
    (0x34, "AXP2101"),
    (0x40, "ES7210"),
    (0x51, "PCF85063"),
    (0x5A, "CST9217"),
    (0x6B, "QMI8658"),
];

#[derive(Clone, Copy, PartialEq, Eq)]
enum Nav {
    Next,
    Prev,
}

/// Turns raw touch points into tap and swipe events.
#[derive(Default)]
struct Gesture {
    start: Option<(i32, i32)>,
    last: (i32, i32),
}

enum TouchEvent {
    Down(i32, i32),
    Move,
    Tap(i32, i32),
    Swipe(Nav),
}

impl Gesture {
    fn update(&mut self, point: Option<(i32, i32)>) -> Option<TouchEvent> {
        match (point, self.start) {
            (Some(p), None) => {
                self.start = Some(p);
                self.last = p;
                Some(TouchEvent::Down(p.0, p.1))
            }
            (Some(p), Some(_)) => {
                self.last = p;
                Some(TouchEvent::Move)
            }
            (None, Some(s)) => {
                self.start = None;
                let (dx, dy) = (self.last.0 - s.0, self.last.1 - s.1);
                if dx.abs() >= SWIPE_MIN_DX && dy.abs() <= SWIPE_MAX_DY {
                    Some(TouchEvent::Swipe(if dx < 0 { Nav::Next } else { Nav::Prev }))
                } else if dx.abs() < 20 && dy.abs() < 20 {
                    Some(TouchEvent::Tap(s.0, s.1))
                } else {
                    None
                }
            }
            (None, None) => None,
        }
    }
}

#[main]
fn main() -> ! {
    let peripherals = esp_hal::init(esp_hal::Config::default().with_cpu_clock(CpuClock::max()));
    let delay = Delay::new();

    esp_alloc::psram_allocator!(peripherals.PSRAM, esp_hal::psram);

    println!("amulett: booting");

    // Shared I2C bus: PMIC, expander, touch, IMU, RTC, audio codecs.
    let i2c = I2c::new(
        peripherals.I2C0,
        I2cConfig::default().with_frequency(Rate::from_khz(400)),
    )
    .expect("invalid I2C config")
    .with_sda(peripherals.GPIO15)
    .with_scl(peripherals.GPIO14);
    let bus = RefCell::new(i2c);

    // Touch controller reset must be released before it answers on the bus.
    let mut touch_reset = Output::new(peripherals.GPIO40, Level::Low, OutputConfig::default());
    delay.delay_millis(10);
    touch_reset.set_high();
    delay.delay_millis(50);
    let touch_int = Input::new(peripherals.GPIO11, InputConfig::default().with_pull(Pull::Up));

    // I2C scan: show which known devices answer.
    let mut found: Vec<(u8, &'static str)> = Vec::new();
    {
        let mut probe = RefCellDevice::new(&bus);
        for addr in 0x08u8..0x78 {
            if probe.write(addr, &[]).is_ok() {
                let name = KNOWN_I2C
                    .iter()
                    .find(|(a, _)| *a == addr)
                    .map(|(_, n)| *n)
                    .unwrap_or("unbekannt");
                println!("I2C 0x{addr:02X}: {name}");
                found.push((addr, name));
            }
        }
    }

    let mut pmic = pmic::Axp2101::new(RefCellDevice::new(&bus));
    match pmic.chip_id() {
        Ok(id) => println!("AXP2101 chip id 0x{id:02X} (expected 0x{:02X})", pmic::CHIP_ID),
        Err(e) => println!("AXP2101 not reachable: {e:?}"),
    }
    if let Err(e) = pmic.enable_measurements() {
        println!("AXP2101 ADC enable failed: {e:?}");
    }
    match pmic.rails() {
        Ok([dc, ldo0, ldo1]) => {
            println!("AXP2101 rails: DC=0b{dc:08b} LDO0=0b{ldo0:08b} LDO1=0b{ldo1:08b}")
        }
        Err(e) => println!("AXP2101 rails unreadable: {e:?}"),
    }

    let mut expander = expander::Tca9554::new(RefCellDevice::new(&bus));
    if let Err(e) = expander.init() {
        println!("TCA9554 init failed: {e:?}");
    }

    let mut imu = imu::Qmi8658::new(RefCellDevice::new(&bus));
    match imu.who_am_i() {
        Ok(id) => println!("QMI8658 who-am-i 0x{id:02X} (expected 0x{:02X})", imu::WHO_AM_I),
        Err(e) => println!("QMI8658 not reachable: {e:?}"),
    }
    if let Err(e) = imu.init() {
        println!("QMI8658 init failed: {e:?}");
    }

    let mut touch = touch::Cst9217::new(RefCellDevice::new(&bus));
    let mut rtc = rtc::Pcf85063::new(RefCellDevice::new(&bus));

    // Display on SPI2 in quad mode.
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
    let mut panel = display::Co5300::new(spi, lcd_reset);
    match panel.init(&delay) {
        Ok(()) => println!("CO5300 display initialised"),
        Err(e) => println!("CO5300 init failed: {e:?}"),
    }

    let mut fb = display::Framebuffer::new();
    let boot = Input::new(peripherals.GPIO0, InputConfig::default().with_pull(Pull::Up));

    let mut page = 0usize;
    let mut brightness = BRIGHTNESS_LEVELS.len() - 1;
    let mut boot_was_down = false;
    let mut pwr_was_down = false;
    let mut gesture = Gesture::default();
    let mut points = [touch::Point { id: 0, x: 0, y: 0 }; 2];
    let mut touch_pos: Option<(i32, i32)> = None;
    let mut imu_sample = imu::Sample::default();
    let mut power_status: Option<pmic::Status> = None;
    let mut last_slow_read = Instant::now();
    let mut nav: Option<Nav> = None;

    loop {
        let frame_start = Instant::now();

        // Buttons: BOOT = next page, PWR = previous page.
        let boot_down = boot.is_low();
        if boot_down && !boot_was_down {
            println!("BOOT pressed");
            nav = Some(Nav::Next);
        }
        boot_was_down = boot_down;
        let pwr_down = expander.pwr_pressed().unwrap_or(false);
        if pwr_down && !pwr_was_down {
            println!("PWR pressed");
            nav = Some(Nav::Prev);
        }
        pwr_was_down = pwr_down;

        // Touch: only talk to the controller when it signals data or a finger is down.
        if touch_int.is_low() || touch_pos.is_some() {
            touch_pos = match touch.read_points(&mut points) {
                Ok(n) if n > 0 => Some((points[0].x as i32, points[0].y as i32)),
                _ => None,
            };
        }
        match gesture.update(touch_pos) {
            Some(TouchEvent::Down(x, y)) => println!("touch down at ({x}, {y})"),
            Some(TouchEvent::Move) | None => {}
            Some(TouchEvent::Tap(x, y)) => {
                println!("tap at ({x}, {y})");
                if page == 0 {
                    brightness = (brightness + 1) % BRIGHTNESS_LEVELS.len();
                    let _ = panel.set_brightness(BRIGHTNESS_LEVELS[brightness]);
                    println!("brightness 0x{:02X}", BRIGHTNESS_LEVELS[brightness]);
                }
            }
            Some(TouchEvent::Swipe(n)) => {
                println!("swipe {}", if n == Nav::Next { "left" } else { "right" });
                nav = Some(n);
            }
        }
        if let Some(n) = nav.take() {
            page = match n {
                Nav::Next => (page + 1) % PAGE_TITLES.len(),
                Nav::Prev => (page + PAGE_TITLES.len() - 1) % PAGE_TITLES.len(),
            };
        }

        // Slow sensors are polled every 250 ms, the IMU every frame.
        if page == 2 {
            if let Ok(Some(s)) = imu.read_sample() {
                imu_sample = s;
            }
        }
        if last_slow_read.elapsed().as_millis() >= 250 {
            last_slow_read = Instant::now();
            if page == 1 {
                power_status = pmic.status().ok();
            }
        }

        match page {
            0 => draw_test_page(&mut fb, touch_pos, brightness),
            1 => draw_power_page(&mut fb, &power_status),
            2 => draw_imu_page(&mut fb, &imu_sample),
            _ => draw_info_page(&mut fb, &found, &mut rtc),
        }
        if let Err(e) = fb.flush(&mut panel) {
            println!("flush failed: {e:?}");
        }

        let used = frame_start.elapsed().as_millis() as u32;
        if used < FRAME_MS {
            delay.delay_millis(FRAME_MS - used);
        }
    }
}

fn draw_test_page(fb: &mut display::Framebuffer, touch: Option<(i32, i32)>, brightness: usize) {
    let _ = fb.clear(Rgb565::BLACK);
    for (i, color) in [Rgb565::RED, Rgb565::GREEN, Rgb565::BLUE].into_iter().enumerate() {
        let _ = Rectangle::new(Point::new(i as i32 * 155, 0), Size::new(156, 466))
            .into_styled(PrimitiveStyle::with_fill(color))
            .draw(fb);
    }
    let _ = Circle::new(Point::new(0, 0), 466)
        .into_styled(PrimitiveStyle::with_stroke(Rgb565::WHITE, 4))
        .draw(fb);
    ui::centered(fb, 210, Rgb565::WHITE, "amulett");
    ui::centered(fb, 240, Rgb565::WHITE, PAGE_TITLES[0]);
    ui::centered(
        fb,
        270,
        Rgb565::WHITE,
        &format!("Helligkeit {}/{} (Tippen)", brightness + 1, BRIGHTNESS_LEVELS.len()),
    );
    ui::centered(fb, 330, Rgb565::WHITE, "BOOT / Wischen links: weiter");
    ui::centered(fb, 355, Rgb565::WHITE, "PWR / Wischen rechts: zurueck");
    if let Some((x, y)) = touch {
        let _ = Circle::with_center(Point::new(x, y), 40)
            .into_styled(PrimitiveStyle::with_fill(Rgb565::YELLOW))
            .draw(fb);
        ui::centered(fb, 400, Rgb565::YELLOW, &format!("Touch x={x} y={y}"));
    }
}

fn draw_power_page(fb: &mut display::Framebuffer, status: &Option<pmic::Status>) {
    let mut lines: Vec<String> = Vec::new();
    match status {
        None => lines.push(String::from("keine Daten")),
        Some(s) => {
            lines.push(format!("USB (VBUS): {}", if s.vbus_good { "ja" } else { "nein" }));
            lines.push(format!("VBUS {} mV  SYS {} mV", s.vbus_mv, s.vsys_mv));
            if s.battery_present {
                lines.push(format!("Akku {} mV", s.vbat_mv));
                lines.push(format!("Ladung {}%", s.percent.unwrap_or(0)));
            } else {
                lines.push(String::from("kein Akku"));
            }
            lines.push(format!(
                "Status: {}",
                match s.charge {
                    pmic::ChargeState::Charging => "laedt",
                    pmic::ChargeState::Discharging => "entlaedt",
                    pmic::ChargeState::Standby => "Standby",
                }
            ));
            lines.push(format!("Chip {:.1} C", s.die_temp_c));
        }
    }
    ui::page(fb, PAGE_TITLES[1], 1, PAGE_TITLES.len(), &lines);
}

fn draw_imu_page(fb: &mut display::Framebuffer, s: &imu::Sample) {
    let lines = [
        String::from("Beschleunigung (g)"),
        format!("x {:+.2}  y {:+.2}  z {:+.2}", s.accel[0], s.accel[1], s.accel[2]),
        String::from("Drehrate (Grad/s)"),
        format!("x {:+.0}  y {:+.0}  z {:+.0}", s.gyro[0], s.gyro[1], s.gyro[2]),
    ];
    ui::page(fb, PAGE_TITLES[2], 2, PAGE_TITLES.len(), &lines);
    // Horizontal tilt bars for x and y acceleration.
    ui::bar(fb, 290, 240, (s.accel[0] + 1.0) / 2.0, Rgb565::GREEN);
    ui::bar(fb, 315, 240, (s.accel[1] + 1.0) / 2.0, Rgb565::CYAN);
}

fn draw_info_page(
    fb: &mut display::Framebuffer,
    found: &[(u8, &'static str)],
    rtc: &mut rtc::Pcf85063<RefCellDevice<'_, I2c<'_, esp_hal::Blocking>>>,
) {
    let mut lines: Vec<String> = Vec::new();
    match rtc.read() {
        Ok(t) => lines.push(format!(
            "{:04}-{:02}-{:02} {:02}:{:02}:{:02}{}",
            t.year,
            t.month,
            t.day,
            t.hour,
            t.minute,
            t.second,
            if t.integrity_lost { " ?" } else { "" }
        )),
        Err(_) => lines.push(String::from("RTC nicht erreichbar")),
    }
    lines.push(format!("{} I2C-Geraete:", found.len()));
    for (addr, name) in found {
        lines.push(format!("0x{addr:02X} {name}"));
    }
    ui::page(fb, PAGE_TITLES[3], 3, PAGE_TITLES.len(), &lines);
}
