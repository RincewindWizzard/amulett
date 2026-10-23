//! QMI8658 6-axis IMU (I2C 0x6B).

use embedded_hal::i2c::I2c;

pub const ADDR: u8 = 0x6B;
pub const WHO_AM_I: u8 = 0x05;

const REG_WHO_AM_I: u8 = 0x00;
const REG_CTRL1: u8 = 0x02;
const REG_CTRL2: u8 = 0x03;
const REG_CTRL3: u8 = 0x04;
const REG_CTRL7: u8 = 0x08;
const REG_STATUS0: u8 = 0x2E;
const REG_AX_L: u8 = 0x35;

/// Accelerometer ±4 g, 125 Hz.
const CTRL2_ACC: u8 = 0b0_001_0110;
/// Gyroscope ±512 °/s, 125 Hz.
const CTRL3_GYR: u8 = 0b0_101_0110;
const ACC_LSB_PER_G: f32 = 8192.0;
const GYR_LSB_PER_DPS: f32 = 64.0;

#[derive(Clone, Copy, Debug, Default)]
pub struct Sample {
    /// Acceleration in g.
    pub accel: [f32; 3],
    /// Angular rate in °/s.
    pub gyro: [f32; 3],
}

pub struct Qmi8658<I2C> {
    i2c: I2C,
}

impl<I2C: I2c> Qmi8658<I2C> {
    pub fn new(i2c: I2C) -> Self {
        Self { i2c }
    }

    fn read(&mut self, reg: u8) -> Result<u8, I2C::Error> {
        let mut b = [0u8];
        self.i2c.write_read(ADDR, &[reg], &mut b)?;
        Ok(b[0])
    }

    pub fn who_am_i(&mut self) -> Result<u8, I2C::Error> {
        self.read(REG_WHO_AM_I)
    }

    /// Enables accelerometer and gyroscope with register auto-increment.
    pub fn init(&mut self) -> Result<(), I2C::Error> {
        self.i2c.write(ADDR, &[REG_CTRL1, 0x40])?;
        self.i2c.write(ADDR, &[REG_CTRL2, CTRL2_ACC])?;
        self.i2c.write(ADDR, &[REG_CTRL3, CTRL3_GYR])?;
        self.i2c.write(ADDR, &[REG_CTRL7, 0b0000_0011])
    }

    /// Returns `None` until both sensors have a fresh sample.
    pub fn read_sample(&mut self) -> Result<Option<Sample>, I2C::Error> {
        if self.read(REG_STATUS0)? & 0b11 != 0b11 {
            return Ok(None);
        }
        let mut raw = [0u8; 12];
        self.i2c.write_read(ADDR, &[REG_AX_L], &mut raw)?;
        let word = |i: usize| i16::from_le_bytes([raw[2 * i], raw[2 * i + 1]]) as f32;
        Ok(Some(Sample {
            accel: [
                word(0) / ACC_LSB_PER_G,
                word(1) / ACC_LSB_PER_G,
                word(2) / ACC_LSB_PER_G,
            ],
            gyro: [
                word(3) / GYR_LSB_PER_DPS,
                word(4) / GYR_LSB_PER_DPS,
                word(5) / GYR_LSB_PER_DPS,
            ],
        }))
    }
}
