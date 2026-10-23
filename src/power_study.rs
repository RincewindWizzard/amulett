//! Power study firmware (cargo feature `power`): runs one operating state at a
//! time so that its current draw can be measured, see `docs/schlafmodi.md`.
//!
//! Every scenario prints a banner on the serial console, so that measurements
//! can be matched with the state. Pick a scenario at start with BOOT (next)
//! and PWR (start). Deep-sleep scenarios survive their own resets: the chosen
//! scenario and a cycle counter live in RTC memory.

use alloc::{format, string::String};
use core::cell::RefCell;

use embedded_graphics::{pixelcolor::Rgb565, prelude::*};
use embedded_hal_bus::i2c::RefCellDevice;
use esp_hal::{
    delay::Delay,
    dma_rx_buffer, dma_tx_buffer,
    gpio::{Event, Input, InputConfig, Level, Output, OutputConfig, Pull, WakeupConfig},
    i2c::master::{Config as I2cConfig, I2c},
    rtc_cntl::{
        sleep::{LowPower, RtcSleepConfig},
        wakeup_cause,
    },
    spi::{
        master::{Config as SpiConfig, Spi},
        Mode,
    },
    time::{Duration, Instant, Rate},
};
use esp_println::println;

use crate::{
    display::{self, Co5300, Framebuffer},
    expander::Tca9554,
    input::Buttons,
    pmic::{Axp2101, Rail},
    ui, wireless,
};

/// Marker that the RTC variables below hold a valid scenario.
const MAGIC: u32 = 0xA3_17_50_1E;
const DEEP_SLEEP_SECONDS: u64 = 10;
const LIGHT_SLEEP_SECONDS: u64 = 10;
/// Cycle of the "deep sleep + BLE" scenario.
const CYCLE_SECONDS: u64 = 60;
const CYCLE_ADVERTISE_MS: u32 = 1_000;
const RAIL_STEP_MS: u32 = 8_000;

// --8<-- [start:rtc]
#[esp_hal::ram(unstable(rtc_fast, persistent))]
static mut RTC_MAGIC: u32 = 0;
#[esp_hal::ram(unstable(rtc_fast, persistent))]
static mut RTC_SCENARIO: u32 = 0;
#[esp_hal::ram(unstable(rtc_fast, persistent))]
static mut RTC_CYCLES: u32 = 0;
// --8<-- [end:rtc]

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Scenario {
    /// Display on (white, bright), CPU running.
    Active,
    /// Display on, dimmed.
    DisplayDim,
    /// Display in sleep, CPU running (reference without display).
    CpuOnly,
    /// BLE advertising with the given interval, display in sleep, CPU running.
    Advertise(u32),
    /// Product radio (beacon + scan), display in sleep.
    BeaconScan,
    /// Display in sleep, CPU in light sleep, wake by timer or BOOT.
    LightSleep,
    /// Display in sleep, chip in deep sleep, wake by timer or BOOT.
    DeepSleep,
    /// Deep sleep; every `CYCLE_SECONDS` wake up, advertise for a second, sleep again.
    DeepSleepBle,
    /// Switches the AXP2101 rails off one after the other (to find the display rail).
    RailTest,
}

const SCENARIOS: [Scenario; 11] = [
    Scenario::Active,
    Scenario::DisplayDim,
    Scenario::CpuOnly,
    Scenario::Advertise(100),
    Scenario::Advertise(1000),
    Scenario::Advertise(5000),
    Scenario::BeaconScan,
    Scenario::LightSleep,
    Scenario::DeepSleep,
    Scenario::DeepSleepBle,
    Scenario::RailTest,
];

impl Scenario {
    fn name(self) -> String {
        match self {
            Scenario::Active => "Display an, hell".into(),
            Scenario::DisplayDim => "Display gedimmt".into(),
            Scenario::CpuOnly => "Display aus, CPU laeuft".into(),
            Scenario::Advertise(ms) => format!("BLE Advertising {ms} ms"),
            Scenario::BeaconScan => "Beacon + Scan".into(),
            Scenario::LightSleep => "Light Sleep".into(),
            Scenario::DeepSleep => "Deep Sleep".into(),
            Scenario::DeepSleepBle => "Deep Sleep + BLE".into(),
            Scenario::RailTest => "AXP2101-Schienen".into(),
        }
    }

    fn from_index(i: u32) -> Option<Scenario> {
        SCENARIOS.get(i as usize).copied()
    }
}

fn now_ms() -> u32 {
    Instant::now().duration_since_epoch().as_millis() as u32
}

fn own_id() -> u32 {
    let mac = esp_hal::efuse::base_mac_address();
    let b = mac.as_bytes();
    u32::from_le_bytes([b[2], b[3], b[4], b[5]])
}

// --8<-- [start:wake]
/// Wake sources: timer after `seconds` and a low level on BOOT (GPIO0).
fn arm_wakeup(lp: &mut LowPower<'_>, boot: &mut Input<'_>, seconds: u64) {
    lp.set_wakeup_deadline(Instant::now() + Duration::from_secs(seconds));
    boot.clear_interrupt();
    boot.listen(Event::LowLevel);
    let _ = boot.apply_wakeup_config(&WakeupConfig::default().with_low_power_path(true));
}
// --8<-- [end:wake]

fn print_wake_cause() {
    println!("wake cause: {:?}", wakeup_cause());
}

pub fn run(peripherals: esp_hal::peripherals::Peripherals) -> ! {
    let delay = Delay::new();
    esp_alloc::heap_allocator!(#[esp_hal::ram(reclaimed)] size: 64 * 1024);
    esp_alloc::heap_allocator!(size: 36 * 1024);
    esp_alloc::psram_allocator!(peripherals.PSRAM, esp_hal::psram);
    let timg0 = esp_hal::timer::timg::TimerGroup::new(peripherals.TIMG0);
    esp_rtos::start(timg0.timer0, peripherals.FROM_CPU_INTR0);

    let mut lp = LowPower::new(peripherals.LPWR);
    // SAFETY: GPIO0 is only read by the menu buttons; the wake configuration belongs to `boot`.
    let menu_pin = unsafe { peripherals.GPIO0.clone_unchecked() };
    let mut boot = Input::new(peripherals.GPIO0, InputConfig::default().with_pull(Pull::Up));

    // --8<-- [start:resume]
    // Resume a deep-sleep scenario after its own reset.
    let woke = !wakeup_cause().is_empty();
    // SAFETY: single core, RTC statics are only touched here.
    let (magic, stored) = unsafe { (core::ptr::read_volatile(&raw const RTC_MAGIC), core::ptr::read_volatile(&raw const RTC_SCENARIO)) };
    if woke && magic == MAGIC {
        if let Some(s @ (Scenario::DeepSleep | Scenario::DeepSleepBle)) = Scenario::from_index(stored) {
            let cycles = unsafe {
                let c = core::ptr::read_volatile(&raw const RTC_CYCLES) + 1;
                core::ptr::write_volatile(&raw mut RTC_CYCLES, c);
                c
            };
            println!("== SCENARIO {} (resumed, cycle {cycles}) ==", s.name());
            print_wake_cause();
            match s {
                Scenario::DeepSleepBle => {
                    // Send a beacon for a moment, then sleep again.
                    match wireless::Advertiser::start(peripherals.BT, "amulett", 100) {
                        Ok(a) => {
                            delay.delay_millis(CYCLE_ADVERTISE_MS);
                            a.stop();
                        }
                        Err(e) => println!("advertising failed: {e:?}"),
                    }
                    arm_wakeup(&mut lp, &mut boot, CYCLE_SECONDS);
                }
                _ => arm_wakeup(&mut lp, &mut boot, DEEP_SLEEP_SECONDS),
            }
            println!("sleeping ...");
            lp.sleep_deep(RtcSleepConfig::deep());
        }
    }
    // --8<-- [end:resume]

    println!("amulett: power study");
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
    let mut expander = Tca9554::new(RefCellDevice::new(&bus));
    let _ = expander.init();
    let mut pmic = Axp2101::new(RefCellDevice::new(&bus));
    let _ = pmic.enable_measurements();

    let dma_rx = dma_rx_buffer!(32).unwrap();
    let dma_tx = dma_tx_buffer!(display::CHUNK_BYTES).unwrap();
    let spi = Spi::new(
        peripherals.SPI2,
        SpiConfig::default().with_frequency(Rate::from_mhz(40)).with_mode(Mode::_0),
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
    if let Err(e) = panel.init(&delay) {
        println!("CO5300 init failed: {e:?}");
    }
    let mut fb = Framebuffer::new();

    // Menu: BOOT = next scenario, PWR (short) = start. GPIO0 is shared with `boot`.
    let mut buttons = Buttons::new(Input::new(menu_pin, InputConfig::default().with_pull(Pull::Up)));
    let mut selected = 0usize;
    let mut changed = true;
    let scenario = loop {
        let (next, start) = buttons.poll(&mut expander);
        if next {
            selected = (selected + 1) % SCENARIOS.len();
            changed = true;
        }
        if start {
            break SCENARIOS[selected];
        }
        if changed {
            changed = false;
            let _ = fb.clear(Rgb565::BLACK);
            ui::centered(&mut fb, 70, Rgb565::CYAN, "Stromstudie");
            for (i, s) in SCENARIOS.iter().enumerate() {
                let mark = if i == selected { ">" } else { " " };
                let color = if i == selected { Rgb565::WHITE } else { Rgb565::new(16, 32, 16) };
                ui::centered(&mut fb, 100 + i as i32 * 24, color, &format!("{mark} {}", s.name()));
            }
            ui::centered(&mut fb, 400, Rgb565::WHITE, "BOOT: weiter  PWR: Start");
            let _ = fb.flush(&mut panel);
        }
        delay.delay_millis(30);
    };

    let index = SCENARIOS.iter().position(|s| *s == scenario).unwrap_or(0) as u32;
    unsafe {
        core::ptr::write_volatile(&raw mut RTC_MAGIC, MAGIC);
        core::ptr::write_volatile(&raw mut RTC_SCENARIO, index);
        core::ptr::write_volatile(&raw mut RTC_CYCLES, 0);
    }
    println!("== SCENARIO {} ==", scenario.name());
    drop(fb);

    match scenario {
        Scenario::Active | Scenario::DisplayDim => {
            let _ = panel.fill(0xFFFF);
            let _ = panel.set_brightness(if scenario == Scenario::Active { 0xFF } else { 0x20 });
            idle_loop(&delay, "display on");
        }
        Scenario::CpuOnly => {
            let _ = panel.sleep(&delay);
            idle_loop(&delay, "cpu running, display asleep");
        }
        Scenario::Advertise(ms) => {
            let _ = panel.sleep(&delay);
            match wireless::Advertiser::start(peripherals.BT, "amulett", ms) {
                Ok(_adv) => idle_loop(&delay, "advertising"),
                Err(e) => println!("advertising failed: {e:?}"),
            }
            idle_loop(&delay, "advertising failed");
        }
        Scenario::BeaconScan => {
            let _ = panel.sleep(&delay);
            match wireless::Radio::start(peripherals.BT, own_id(), 0) {
                Ok(mut radio) => loop {
                    radio.tick(now_ms());
                    let mut n = 0;
                    radio.poll(|_| n += 1);
                    delay.delay_millis(50);
                },
                Err(e) => println!("radio failed: {e:?}"),
            }
            idle_loop(&delay, "radio failed");
        }
        // --8<-- [start:light]
        Scenario::LightSleep => {
            let _ = panel.sleep(&delay);
            loop {
                arm_wakeup(&mut lp, &mut boot, LIGHT_SLEEP_SECONDS);
                println!("light sleep for {LIGHT_SLEEP_SECONDS} s ...");
                lp.sleep_light(RtcSleepConfig::default());
                boot.unlisten();
                boot.clear_interrupt();
                print_wake_cause();
            }
        }
        // --8<-- [end:light]
        Scenario::DeepSleep | Scenario::DeepSleepBle => {
            let _ = panel.sleep(&delay);
            let seconds = if scenario == Scenario::DeepSleep { DEEP_SLEEP_SECONDS } else { CYCLE_SECONDS };
            arm_wakeup(&mut lp, &mut boot, seconds);
            println!("deep sleep for {seconds} s ...");
            lp.sleep_deep(RtcSleepConfig::deep());
        }
        Scenario::RailTest => {
            println!("WARNING: switching rails off may blank the display or reset the chip");
            let _ = panel.fill(0x07E0);
            loop {
                for rail in Rail::ALL {
                    let was = pmic.rail_enabled(rail).unwrap_or(false);
                    if !was {
                        println!("rail {rail:?} is off by default, skipped");
                        continue;
                    }
                    println!("rail {rail:?} OFF for {} s", RAIL_STEP_MS / 1000);
                    let _ = pmic.set_rail(rail, false);
                    delay.delay_millis(RAIL_STEP_MS);
                    let _ = pmic.set_rail(rail, true);
                    println!("rail {rail:?} ON again");
                    delay.delay_millis(RAIL_STEP_MS / 2);
                }
            }
        }
    }
}

fn idle_loop(delay: &Delay, what: &str) -> ! {
    loop {
        println!("{what}, t = {} s", now_ms() / 1000);
        delay.delay_millis(5000);
    }
}
