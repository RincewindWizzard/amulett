//! AXP2101 power management (I2C 0x34).
//!
//! Register map taken from the XPowersLib shipped with the Waveshare examples.
//! On this board the supply rails are already enabled by the chip's power-on
//! defaults; the vendor examples only enable the ADC channels.

use embedded_hal::i2c::I2c;

pub const ADDR: u8 = 0x34;
pub const CHIP_ID: u8 = 0x4A;

const REG_STATUS1: u8 = 0x00;
const REG_STATUS2: u8 = 0x01;
const REG_IC_TYPE: u8 = 0x03;
const REG_ADC_CTRL: u8 = 0x30;
const REG_VBAT_H: u8 = 0x34;
const REG_VBUS_H: u8 = 0x38;
const REG_VSYS_H: u8 = 0x3A;
const REG_DIE_TEMP_H: u8 = 0x3C;
const REG_BAT_DET: u8 = 0x68;
const REG_DC_ONOFF: u8 = 0x80;
const REG_LDO_ONOFF0: u8 = 0x90;
const REG_LDO_ONOFF1: u8 = 0x91;
const REG_BAT_PERCENT: u8 = 0xA4;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChargeState {
    Standby,
    Charging,
    Discharging,
}

/// Snapshot of everything the PoC shows on the power page.
#[derive(Clone, Copy, Debug)]
pub struct Status {
    pub battery_present: bool,
    pub vbus_good: bool,
    pub charge: ChargeState,
    pub vbat_mv: u16,
    pub vbus_mv: u16,
    pub vsys_mv: u16,
    /// `None` when no battery is connected.
    pub percent: Option<u8>,
    pub die_temp_c: f32,
}

pub struct Axp2101<I2C> {
    i2c: I2C,
}

impl<I2C: I2c> Axp2101<I2C> {
    pub fn new(i2c: I2C) -> Self {
        Self { i2c }
    }

    fn read(&mut self, reg: u8) -> Result<u8, I2C::Error> {
        let mut b = [0u8];
        self.i2c.write_read(ADDR, &[reg], &mut b)?;
        Ok(b[0])
    }

    fn write(&mut self, reg: u8, val: u8) -> Result<(), I2C::Error> {
        self.i2c.write(ADDR, &[reg, val])
    }

    fn set_bits(&mut self, reg: u8, mask: u8) -> Result<(), I2C::Error> {
        let v = self.read(reg)?;
        self.write(reg, v | mask)
    }

    fn read_pair(&mut self, reg_h: u8, mask_h: u8) -> Result<u16, I2C::Error> {
        let h = self.read(reg_h)? & mask_h;
        let l = self.read(reg_h + 1)?;
        Ok(((h as u16) << 8) | l as u16)
    }

    pub fn chip_id(&mut self) -> Result<u8, I2C::Error> {
        self.read(REG_IC_TYPE)
    }

    // --8<-- [start:adc]
    /// Enables battery detection and the ADC channels used by [`Self::status`].
    pub fn enable_measurements(&mut self) -> Result<(), I2C::Error> {
        self.set_bits(REG_BAT_DET, 1 << 0)?;
        // bit0 VBAT, bit2 VBUS, bit3 VSYS, bit4 die temperature
        self.set_bits(REG_ADC_CTRL, 0b0001_1101)
    // --8<-- [end:adc]
    }

    /// Raw enable registers: DC-DC 1..5, ALDO1..4 + BLDO1..2, CPUSLDO + DLDO1..2.
    pub fn rails(&mut self) -> Result<[u8; 3], I2C::Error> {
        Ok([
            self.read(REG_DC_ONOFF)?,
            self.read(REG_LDO_ONOFF0)?,
            self.read(REG_LDO_ONOFF1)?,
        ])
    }

    // --8<-- [start:status]
    pub fn status(&mut self) -> Result<Status, I2C::Error> {
        let s1 = self.read(REG_STATUS1)?;
        let s2 = self.read(REG_STATUS2)?;
        let battery_present = s1 & (1 << 3) != 0;
        let vbus_good = s1 & (1 << 5) != 0;
        let charge = match s2 >> 5 {
            1 => ChargeState::Charging,
            2 => ChargeState::Discharging,
            _ => ChargeState::Standby,
        };
        let vbat_mv = if battery_present {
            self.read_pair(REG_VBAT_H, 0x1F)?
        } else {
            0
        };
        let vbus_mv = if vbus_good {
            self.read_pair(REG_VBUS_H, 0x3F)?
        } else {
            0
        };
        let vsys_mv = self.read_pair(REG_VSYS_H, 0x3F)?;
        let raw_temp = self.read_pair(REG_DIE_TEMP_H, 0x3F)?;
        let percent = if battery_present {
            Some(self.read(REG_BAT_PERCENT)?)
        } else {
            None
        };
        Ok(Status {
            battery_present,
            vbus_good,
            charge,
            vbat_mv,
            vbus_mv,
            vsys_mv,
            percent,
            die_temp_c: 22.0 + (7274.0 - raw_temp as f32) / 20.0,
    // --8<-- [end:status]
        })
    }
}
