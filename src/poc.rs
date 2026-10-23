use alloc::{format, string::String, vec::Vec};
use core::cell::RefCell;

use embedded_graphics::{
    pixelcolor::Rgb565,
    prelude::*,
    primitives::{Circle, PrimitiveStyle, Rectangle},
};
use embedded_hal::i2c::I2c as _;
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
    audio, display, expander,
    input::{Gesture, Nav, TouchEvent},
    imu, pmic, rtc, touch, ui, wireless,
};

const PAGE_TITLES: [&str; 6] = [
    "Testbild + Touch",
    "Energie (AXP2101)",
    "IMU (QMI8658)",
    "Audio (ES8311/ES7210)",
    "Funk (WLAN + BLE)",
    "Info + I2C",
];
const PAGE_AUDIO: usize = 3;
const PAGE_RADIO: usize = 4;
const BLE_NAME: &str = "amulett";
const BLE_INTERVAL_MS: u32 = 1000;
const FRAME_MS: u32 = 30;
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

/// Runs the hardware proof-of-concept pages (cargo feature `poc`).
pub fn run(peripherals: esp_hal::peripherals::Peripherals) -> ! {
    let delay = Delay::new();

    // The radio needs internal RAM; the large framebuffer goes to PSRAM.
    esp_alloc::heap_allocator!(#[esp_hal::ram(reclaimed)] size: 64 * 1024);
    esp_alloc::heap_allocator!(size: 36 * 1024);
    esp_alloc::psram_allocator!(peripherals.PSRAM, esp_hal::psram);

    let timg0 = esp_hal::timer::timg::TimerGroup::new(peripherals.TIMG0);
    esp_rtos::start(timg0.timer0, peripherals.FROM_CPU_INTR0);
    let wifi_peripheral = peripherals.WIFI;
    let mut bt_peripheral = peripherals.BT;

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

    // Audio: configure both codecs over I2C, then run I2S with MCLK on GPIO42.
    let mut codec = audio::es8311::Es8311::new(RefCellDevice::new(&bus));
    match codec.chip_id() {
        Ok(id) => println!("ES8311 chip id {id:02X?} (expected {:02X?})", audio::es8311::CHIP_ID),
        Err(e) => println!("ES8311 not reachable: {e:?}"),
    }
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
    let i2s_tx = i2s
        .i2s_tx
        .with_bclk(peripherals.GPIO9)
        .with_ws(peripherals.GPIO45)
        .with_dout(peripherals.GPIO8)
        .build();
    let i2s_rx = i2s.i2s_rx.with_din(peripherals.GPIO10).build();
    // MCLK is running now; the codecs can be configured.
    delay.delay_millis(10);
    if let Err(e) = codec.init(&delay).and_then(|_| codec.set_volume(70)) {
        println!("ES8311 init failed: {e:?}");
    }
    if let Err(e) = mic_adc.init() {
        println!("ES7210 init failed: {e:?}");
    }
    let amplifier = Output::new(peripherals.GPIO46, Level::Low, OutputConfig::default());
    let mut audio_io = audio::AudioIo::new(
        i2s_tx,
        dma_loop_buffer!(3840),
        i2s_rx,
        dma_rx_stream_buffer!(4 * 4092, 1024),
        amplifier,
    );
    let mut mic_levels = (0.0f32, 0.0f32);
    let mut advertiser: Option<wireless::Advertiser<'_>> = None;
    let mut radio_status = String::from("BLE aus");
    let mut wifi_list: Vec<wireless::AccessPoint> = Vec::new();
    let mut wifi_note = String::from("Tippen: WLAN-Scan");

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
        match gesture.update(touch_pos, Instant::now().duration_since_epoch().as_millis() as u32) {
            Some(TouchEvent::Down(x, y)) => println!("touch down at ({x}, {y})"),
            Some(TouchEvent::Move) | Some(TouchEvent::LongPress) | None => {}
            Some(TouchEvent::Tap(x, y)) => {
                println!("tap at ({x}, {y})");
                if page == PAGE_RADIO {
                    // Radio and scan share the antenna: stop BLE while scanning.
                    if let Some(a) = advertiser.take() {
                        a.stop();
                    }
                    wifi_note = String::from("Scanne ...");
                    draw_radio_page(&mut fb, &radio_status, &wifi_note, &wifi_list);
                    let _ = fb.flush(&mut panel);
                    // SAFETY: the previous Wi-Fi controller has been dropped.
                    let wifi = unsafe { wifi_peripheral.clone_unchecked() };
                    match wireless::scan_wifi(wifi, 6) {
                        Ok(list) => {
                            println!("wifi scan: {} networks", list.len());
                            for ap in &list {
                                println!("  {} ({} dBm, ch {})", ap.ssid, ap.rssi, ap.channel);
                            }
                            wifi_note = format!("{} Netze gefunden", list.len());
                            wifi_list = list;
                        }
                        Err(e) => {
                            println!("wifi scan failed: {e:?}");
                            wifi_note = String::from("Scan fehlgeschlagen");
                        }
                    }
                    radio_status = start_ble(&mut advertiser, &mut bt_peripheral);
                } else if page == PAGE_AUDIO {
                    if audio_io.tone_active() {
                        audio_io.stop_tone();
                    } else if !audio_io.start_tone() {
                        println!("tone could not be started");
                    }
                    println!("tone {}", if audio_io.tone_active() { "on" } else { "off" });
                } else if page == 0 {
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
            let previous = page;
            page = match n {
                Nav::Next => (page + 1) % PAGE_TITLES.len(),
                Nav::Prev => (page + PAGE_TITLES.len() - 1) % PAGE_TITLES.len(),
            };
            if previous == PAGE_AUDIO {
                audio_io.stop_tone();
                audio_io.stop_capture();
            }
            if previous == PAGE_RADIO {
                if let Some(a) = advertiser.take() {
                    a.stop();
                }
                radio_status = String::from("BLE aus");
            }
            if page == PAGE_RADIO {
                radio_status = start_ble(&mut advertiser, &mut bt_peripheral);
            }
            if page == PAGE_AUDIO && !audio_io.start_capture() {
                println!("microphone capture could not be started");
            }
        }

        // Slow sensors are polled every 250 ms, the IMU every frame.
        if page == 2 {
            if let Ok(Some(s)) = imu.read_sample() {
                imu_sample = s;
            }
        }
        if page == PAGE_AUDIO {
            if let Some((l, r)) = audio_io.read_levels() {
                // Slow decay so short peaks stay visible.
                mic_levels = (l.max(mic_levels.0 * 0.85), r.max(mic_levels.1 * 0.85));
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
            PAGE_AUDIO => draw_audio_page(&mut fb, mic_levels, audio_io.tone_active()),
            PAGE_RADIO => draw_radio_page(&mut fb, &radio_status, &wifi_note, &wifi_list),
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

/// Starts BLE advertising and returns a status line for the display.
fn start_ble<'d>(
    advertiser: &mut Option<wireless::Advertiser<'d>>,
    bt: &mut esp_hal::peripherals::BT<'d>,
) -> String {
    // SAFETY: only one advertiser exists at a time; the previous one was dropped.
    let bt = unsafe { bt.clone_unchecked() };
    match wireless::Advertiser::start(bt, BLE_NAME, BLE_INTERVAL_MS) {
        Ok(a) => {
            *advertiser = Some(a);
            println!("BLE advertising as '{BLE_NAME}' every {BLE_INTERVAL_MS} ms");
            format!("BLE: '{BLE_NAME}' alle {BLE_INTERVAL_MS} ms")
        }
        Err(e) => {
            println!("BLE start failed: {e:?}");
            String::from("BLE Start fehlgeschlagen")
        }
    }
}

fn draw_radio_page(
    fb: &mut display::Framebuffer,
    ble: &str,
    wifi_note: &str,
    list: &[wireless::AccessPoint],
) {
    let mut lines: Vec<String> = Vec::new();
    lines.push(String::from(ble));
    lines.push(String::from(wifi_note));
    for ap in list.iter().take(5) {
        let mut name = ap.ssid.clone();
        name.truncate(16);
        lines.push(format!("{name} {}dBm", ap.rssi));
    }
    ui::page(fb, PAGE_TITLES[PAGE_RADIO], PAGE_RADIO, PAGE_TITLES.len(), &lines);
}

fn draw_audio_page(fb: &mut display::Framebuffer, levels: (f32, f32), tone: bool) {
    let lines = [
        format!("Ton: {} (Tippen)", if tone { "an" } else { "aus" }),
        String::from("Mikrofon links / rechts"),
    ];
    ui::page(fb, PAGE_TITLES[PAGE_AUDIO], PAGE_AUDIO, PAGE_TITLES.len(), &lines);
    ui::bar(fb, 230, 240, levels.0 * 4.0, Rgb565::GREEN);
    ui::bar(fb, 255, 240, levels.1 * 4.0, Rgb565::CYAN);
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
    ui::page(fb, PAGE_TITLES[5], 5, PAGE_TITLES.len(), &lines);
}
