#![no_std]
#![no_main]
// The hardware PoC drivers are only used by the `poc` feature.
#![cfg_attr(not(feature = "poc"), allow(dead_code))]
#![allow(unreachable_code)]

extern crate alloc;

mod audio;
#[allow(dead_code)]
mod display;
#[allow(dead_code)]
mod expander;
mod imu;
#[allow(dead_code)]
mod input;
#[allow(dead_code)]
mod pmic;
#[cfg(feature = "poc")]
mod poc;
#[cfg(feature = "power")]
mod power_study;
#[cfg(not(any(feature = "poc", feature = "power")))]
mod product;
#[allow(dead_code)]
mod rtc;
mod touch;
#[allow(dead_code)]
mod ui;
#[allow(dead_code)]
mod wireless;

use esp_backtrace as _;
use esp_hal::{clock::CpuClock, main};

esp_bootloader_esp_idf::esp_app_desc!();

#[main]
fn main() -> ! {
    let peripherals = esp_hal::init(esp_hal::Config::default().with_cpu_clock(CpuClock::max()));

    #[cfg(feature = "poc")]
    poc::run(peripherals);
    #[cfg(feature = "power")]
    power_study::run(peripherals);
    #[cfg(not(any(feature = "poc", feature = "power")))]
    product::run(peripherals)
}
