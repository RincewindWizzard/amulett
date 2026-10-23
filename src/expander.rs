//! TCA9554 8-bit I/O expander (I2C 0x20).
//!
//! The board routes several interrupt lines and the PWR button through it,
//! see `docs/hardware.md`.

use embedded_hal::i2c::I2c;

pub const ADDR: u8 = 0x20;

const REG_INPUT: u8 = 0x00;
const REG_OUTPUT: u8 = 0x01;
const REG_CONFIG: u8 = 0x03;

pub const PIN_RTC_INT: u8 = 3;
pub const PIN_SYS_OUT: u8 = 4;
pub const PIN_AXP_IRQ: u8 = 5;
pub const PIN_QMI_INT1: u8 = 6;
pub const PIN_GPS_RST: u8 = 7;

pub struct Tca9554<I2C> {
    i2c: I2C,
}

impl<I2C: I2c> Tca9554<I2C> {
    pub fn new(i2c: I2C) -> Self {
        Self { i2c }
    }

    /// Configures every pin as input except P7 (GPS reset, output, driven low).
    pub fn init(&mut self) -> Result<(), I2C::Error> {
        self.i2c.write(ADDR, &[REG_OUTPUT, 0x00])?;
        self.i2c.write(ADDR, &[REG_CONFIG, 0x7F])
    }

    pub fn read_inputs(&mut self) -> Result<u8, I2C::Error> {
        let mut b = [0u8];
        self.i2c.write_read(ADDR, &[REG_INPUT], &mut b)?;
        Ok(b[0])
    }

    // --8<-- [start:pwr]
    /// `true` while the PWR button is held (SYS_OUT is high when pressed).
    pub fn pwr_pressed(&mut self) -> Result<bool, I2C::Error> {
        Ok(self.read_inputs()? & (1 << PIN_SYS_OUT) != 0)
    // --8<-- [end:pwr]
    }
}
