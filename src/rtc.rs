//! PCF85063 real-time clock (I2C 0x51).

use embedded_hal::i2c::I2c;

pub const ADDR: u8 = 0x51;

const REG_SECONDS: u8 = 0x04;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DateTime {
    pub year: u16,
    pub month: u8,
    pub day: u8,
    pub hour: u8,
    pub minute: u8,
    pub second: u8,
    /// `true` when the oscillator-stop flag is set (time is not trustworthy).
    pub integrity_lost: bool,
}

fn bcd_to_bin(v: u8) -> u8 {
    (v >> 4) * 10 + (v & 0x0F)
}

fn bin_to_bcd(v: u8) -> u8 {
    ((v / 10) << 4) | (v % 10)
}

pub struct Pcf85063<I2C> {
    i2c: I2C,
}

impl<I2C: I2c> Pcf85063<I2C> {
    pub fn new(i2c: I2C) -> Self {
        Self { i2c }
    }

    pub fn read(&mut self) -> Result<DateTime, I2C::Error> {
        let mut r = [0u8; 7];
        self.i2c.write_read(ADDR, &[REG_SECONDS], &mut r)?;
        Ok(DateTime {
            integrity_lost: r[0] & 0x80 != 0,
            second: bcd_to_bin(r[0] & 0x7F),
            minute: bcd_to_bin(r[1] & 0x7F),
            hour: bcd_to_bin(r[2] & 0x3F),
            day: bcd_to_bin(r[3] & 0x3F),
            month: bcd_to_bin(r[5] & 0x1F),
            year: 2000 + bcd_to_bin(r[6]) as u16,
        })
    }

    /// Sets the time (24-hour mode) and clears the oscillator-stop flag.
    pub fn set(&mut self, t: &DateTime) -> Result<(), I2C::Error> {
        self.i2c.write(
            ADDR,
            &[
                REG_SECONDS,
                bin_to_bcd(t.second),
                bin_to_bcd(t.minute),
                bin_to_bcd(t.hour),
                bin_to_bcd(t.day),
                0, // weekday, unused
                bin_to_bcd(t.month),
                bin_to_bcd((t.year.saturating_sub(2000)) as u8),
            ],
        )
    }
}
