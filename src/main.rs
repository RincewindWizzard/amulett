#![no_std]
#![no_main]

use esp_backtrace as _;
use esp_hal::{
    clock::CpuClock,
    delay::Delay,
    i2c::master::{Config as I2cConfig, I2c},
    main,
    time::Rate,
};
use esp_println::println;

esp_bootloader_esp_idf::esp_app_desc!();

// Waveshare ESP32-S3-Touch-AMOLED-1.75: shared I2C bus carrying
// AXP2101 (PMIC), CST9217 (touch), QMI8658 (IMU) and the audio codec.
// Pins taken from the vendor schematic; verify against your board revision.
const I2C_SDA_GPIO: u8 = 15;
const I2C_SCL_GPIO: u8 = 14;

#[main]
fn main() -> ! {
    let peripherals = esp_hal::init(esp_hal::Config::default().with_cpu_clock(CpuClock::max()));
    let delay = Delay::new();

    let mut i2c = I2c::new(
        peripherals.I2C0,
        I2cConfig::default().with_frequency(Rate::from_khz(400)),
    )
    .expect("invalid I2C config")
    .with_sda(peripherals.GPIO15)
    .with_scl(peripherals.GPIO14);

    println!("amulett: booting (I2C SDA=GPIO{I2C_SDA_GPIO}, SCL=GPIO{I2C_SCL_GPIO})");

    loop {
        println!("scanning I2C bus...");
        for addr in 0x08u8..0x78 {
            if i2c.write(addr, &[]).is_ok() {
                println!("  device at 0x{addr:02X}");
            }
        }
        delay.delay_millis(5000);
    }
}
