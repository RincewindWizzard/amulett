//! CST9217 capacitive touch controller (I2C 0x5A).
//!
//! Protocol taken from the vendor SensorLib (`TouchDrvCST92xx`): write the
//! 16-bit command 0xD000, read a 15-byte report, then acknowledge with 0xAB.

use embedded_hal::i2c::I2c;

pub const ADDR: u8 = 0x5A;

const READ_COMMAND: [u8; 2] = [0xD0, 0x00];
const ACK: u8 = 0xAB;
const MAX_FINGERS: usize = 2;
const REPORT_LEN: usize = MAX_FINGERS * 5 + 5;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Point {
    pub id: u8,
    pub x: u16,
    pub y: u16,
}

pub struct Cst9217<I2C> {
    i2c: I2C,
}

impl<I2C: I2c> Cst9217<I2C> {
    pub fn new(i2c: I2C) -> Self {
        Self { i2c }
    }

    /// Reads the current report. Returns the number of valid points written
    /// to `points` (0 when the finger has been lifted).
    pub fn read_points(&mut self, points: &mut [Point; MAX_FINGERS]) -> Result<usize, I2C::Error> {
        let mut buf = [0u8; REPORT_LEN];
        self.i2c.write_read(ADDR, &READ_COMMAND, &mut buf)?;
        self.i2c
            .write(ADDR, &[READ_COMMAND[0], READ_COMMAND[1], ACK])?;

        if buf[6] != ACK {
            return Ok(0);
        }
        let count = (buf[5] & 0x7F) as usize;
        if count == 0 || count > MAX_FINGERS {
            return Ok(0);
        }
        let mut valid = 0;
        for i in 0..count {
            let d = &buf[i * 5 + if i == 0 { 0 } else { 2 }..];
            let id = d[0] >> 4;
            let pressed = d[0] & 0x0F;
            if pressed != 0x06 || id as usize >= MAX_FINGERS {
                continue;
            }
            points[valid] = Point {
                id,
                x: ((d[1] as u16) << 4) | (d[3] >> 4) as u16,
                y: ((d[2] as u16) << 4) | (d[3] & 0x0F) as u16,
            };
            valid += 1;
        }
        Ok(valid)
    }
}
