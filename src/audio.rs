//! Audio: ES8311 playback codec (I2C 0x18), ES7210 microphone ADC (I2C 0x40)
//! and the shared I2S bus (MCLK GPIO42, BCLK GPIO9, WS GPIO45, DOUT GPIO8,
//! DIN GPIO10, amplifier enable GPIO46).
//!
//! Register sequences are ported from Espressif's `esp_codec_dev` (ES7210)
//! and the Waveshare `es8311.c` example, both configured for 16 kHz, 16 bit,
//! slave mode, MCLK = 256 x fs.

use embedded_hal::i2c::I2c;

pub const SAMPLE_RATE: u32 = 16_000;

pub mod es8311 {
    use super::*;

    pub const ADDR: u8 = 0x18;
    /// Expected values of the chip-id registers 0xFD/0xFE.
    pub const CHIP_ID: [u8; 2] = [0x83, 0x11];

    pub struct Es8311<I2C> {
        i2c: I2C,
    }

    impl<I2C: I2c> Es8311<I2C> {
        pub fn new(i2c: I2C) -> Self {
            Self { i2c }
        }

        fn read(&mut self, reg: u8) -> Result<u8, I2C::Error> {
            let mut b = [0u8];
            self.i2c.write_read(ADDR, &[reg], &mut b)?;
            Ok(b[0])
        }

        fn write(&mut self, reg: u8, value: u8) -> Result<(), I2C::Error> {
            self.i2c.write(ADDR, &[reg, value])
        }

        fn update(&mut self, reg: u8, mask: u8, value: u8) -> Result<(), I2C::Error> {
            let v = self.read(reg)?;
            self.write(reg, (v & !mask) | (value & mask))
        }

        pub fn chip_id(&mut self) -> Result<[u8; 2], I2C::Error> {
            Ok([self.read(0xFD)?, self.read(0xFE)?])
        }

        // --8<-- [start:es8311_init]
        /// 16 kHz, 16 bit, I2S slave, MCLK from pin at 4.096 MHz (256 x fs).
        pub fn init(&mut self, delay: &esp_hal::delay::Delay) -> Result<(), I2C::Error> {
            // Reset, then power-on command.
            self.write(0x00, 0x1F)?;
            delay.delay_millis(20);
            self.write(0x00, 0x00)?;
            self.write(0x00, 0x80)?;

            // Clock: all clocks on, MCLK from pad, not inverted.
            self.write(0x01, 0x3F)?;
            self.update(0x06, 1 << 5, 0)?;
            // Divider coefficients for MCLK 4.096 MHz / 16 kHz.
            self.update(0x02, 0xF8, 0x00)?; // pre_div = 1, pre_multi = 1x
            self.write(0x03, 0x10)?; // single speed, ADC OSR
            self.write(0x04, 0x10)?; // DAC OSR
            self.write(0x05, 0x00)?; // ADC/DAC dividers = 1
            self.update(0x06, 0x1F, 0x03)?; // BCLK divider = 4
            self.update(0x07, 0x3F, 0x00)?; // LRCK divider high
            self.write(0x08, 0xFF)?; // LRCK divider low

            // Serial port: slave, I2S, 16 bit in/out.
            self.update(0x00, 1 << 6, 0)?;
            self.write(0x09, 0x0C)?;
            self.write(0x0A, 0x0C)?;

            self.write(0x0D, 0x01)?; // power up analog circuitry
            self.write(0x0E, 0x02)?; // enable PGA and ADC modulator
            self.write(0x12, 0x00)?; // power up DAC
            self.write(0x13, 0x10)?; // enable output to headphone drive
            self.write(0x1C, 0x6A)?; // ADC equalizer bypass, cancel DC offset
            self.write(0x37, 0x08)?; // bypass DAC equalizer
        // --8<-- [end:es8311_init]
            Ok(())
        }

        /// Volume 0..=100.
        pub fn set_volume(&mut self, volume: u8) -> Result<(), I2C::Error> {
            let volume = volume.min(100) as u16;
            let reg = if volume == 0 { 0 } else { volume * 256 / 100 - 1 };
            self.write(0x32, reg as u8)
        }

        #[allow(dead_code)]
        pub fn mute(&mut self, mute: bool) -> Result<(), I2C::Error> {
            self.update(0x31, (1 << 6) | (1 << 5), if mute { 0xFF } else { 0x00 })
        }
    }
}

pub mod es7210 {
    use super::*;

    pub const ADDR: u8 = 0x40;
    /// Gain register value for 30 dB (3 dB per step).
    const GAIN_30DB: u8 = 10;

    pub struct Es7210<I2C> {
        i2c: I2C,
    }

    impl<I2C: I2c> Es7210<I2C> {
        pub fn new(i2c: I2C) -> Self {
            Self { i2c }
        }

        fn read(&mut self, reg: u8) -> Result<u8, I2C::Error> {
            let mut b = [0u8];
            self.i2c.write_read(ADDR, &[reg], &mut b)?;
            Ok(b[0])
        }

        fn write(&mut self, reg: u8, value: u8) -> Result<(), I2C::Error> {
            self.i2c.write(ADDR, &[reg, value])
        }

        fn update(&mut self, reg: u8, mask: u8, value: u8) -> Result<(), I2C::Error> {
            let v = self.read(reg)?;
            self.write(reg, (v & !mask) | (value & mask))
        }

        /// Selects MIC1 and MIC2 (the two onboard microphones), 30 dB gain.
        fn select_mics(&mut self) -> Result<(), I2C::Error> {
            for i in 0..4 {
                self.update(0x43 + i, 0x10, 0x00)?;
            }
            self.write(0x4B, 0xFF)?;
            self.write(0x4C, 0xFF)?;
            // MIC1
            self.update(0x01, 0x0B, 0x00)?;
            self.write(0x4B, 0x00)?;
            self.update(0x43, 0x10, 0x10)?;
            self.update(0x43, 0x0F, GAIN_30DB)?;
            // MIC2
            self.update(0x01, 0x0B, 0x00)?;
            self.write(0x4B, 0x00)?;
            self.update(0x44, 0x10, 0x10)?;
            self.update(0x44, 0x0F, GAIN_30DB)?;
            // Two microphones: plain I2S, no TDM.
            self.write(0x12, 0x00)
        }

        // --8<-- [start:es7210_init]
        /// Open + configure (slave, 16 bit I2S) + start capturing.
        pub fn init(&mut self) -> Result<(), I2C::Error> {
            self.write(0x00, 0xFF)?;
            self.write(0x00, 0x41)?;
            self.write(0x01, 0x3F)?;
            self.write(0x09, 0x30)?;
            self.write(0x0A, 0x30)?;
            self.write(0x23, 0x2A)?;
            self.write(0x22, 0x0A)?;
            self.write(0x20, 0x0A)?;
            self.write(0x21, 0x2A)?;
            self.update(0x08, 0x01, 0x00)?; // slave mode
            self.write(0x40, 0x43)?;
            self.write(0x41, 0x70)?; // MIC bias 2.87 V
            self.write(0x42, 0x70)?;
            self.write(0x07, 0x20)?;
            self.write(0x02, 0xC1)?;
            self.select_mics()?;
            let off_reg = self.read(0x01)?;

            // Format: 16 bit, I2S normal.
            self.update(0x11, 0xE0, 0x60)?;
            self.update(0x11, 0x03, 0x00)?;

            // Start.
            self.write(0x01, off_reg)?;
            self.write(0x06, 0x00)?;
            self.write(0x40, 0x43)?;
            for reg in 0x47..=0x4A {
                self.write(reg, 0x08)?;
            }
            self.select_mics()?;
            self.write(0x40, 0x43)?;
            self.write(0x00, 0x71)?;
            self.write(0x00, 0x41)
        // --8<-- [end:es7210_init]
        }
    }
}

/// Peak amplitude (0.0..=1.0) of the left and right channel of 16-bit
/// interleaved little-endian stereo samples.
// --8<-- [start:levels]
pub fn peak_levels(bytes: &[u8]) -> (f32, f32) {
    let (mut l, mut r) = (0i32, 0i32);
    for frame in bytes.chunks_exact(4) {
        let left = i16::from_le_bytes([frame[0], frame[1]]) as i32;
        let right = i16::from_le_bytes([frame[2], frame[3]]) as i32;
        l = l.max(left.abs());
        r = r.max(right.abs());
    }
    (l as f32 / 32768.0, r as f32 / 32768.0)
// --8<-- [end:levels]
}

/// Fills `buf` with a 400 Hz sine (exactly 40 frames per period at 16 kHz),
/// stereo, 16 bit little-endian, at roughly 25 % of full scale.
pub fn fill_sine(buf: &mut [u8]) {
    const PERIOD: usize = 40;
    // One period of 8000 * sin(), precomputed to avoid libm.
    const TABLE: [i16; PERIOD] = [
        0, 1251, 2472, 3632, 4702, 5657, 6472, 7128, 7608, 7902, 8000, 7902, 7608, 7128, 6472,
        5657, 4702, 3632, 2472, 1251, 0, -1251, -2472, -3632, -4702, -5657, -6472, -7128, -7608,
        -7902, -8000, -7902, -7608, -7128, -6472, -5657, -4702, -3632, -2472, -1251,
    ];
    for (i, frame) in buf.chunks_exact_mut(4).enumerate() {
        let s = TABLE[i % PERIOD].to_le_bytes();
        frame[0] = s[0];
        frame[1] = s[1];
        frame[2] = s[0];
        frame[3] = s[1];
    }
}

use esp_hal::{
    dma::{DmaLoopBuf, DmaRxStreamBuf},
    gpio::Output,
    i2s::master::{I2sRx, I2sRxDmaTransfer, I2sTx, I2sTxDmaTransfer},
    Blocking,
};

/// Owns the I2S transmitter/receiver and the amplifier enable pin and moves
/// them in and out of running DMA transfers.
pub struct AudioIo<'d> {
    tx: Option<I2sTx<'d, Blocking>>,
    loop_buf: Option<DmaLoopBuf>,
    tone: Option<I2sTxDmaTransfer<'d, Blocking, DmaLoopBuf>>,
    rx: Option<I2sRx<'d, Blocking>>,
    rx_buf: Option<DmaRxStreamBuf>,
    capture: Option<I2sRxDmaTransfer<'d, Blocking, DmaRxStreamBuf>>,
    amplifier: Output<'d>,
}

impl<'d> AudioIo<'d> {
    pub fn new(
        tx: I2sTx<'d, Blocking>,
        mut loop_buf: DmaLoopBuf,
        rx: I2sRx<'d, Blocking>,
        rx_buf: DmaRxStreamBuf,
        amplifier: Output<'d>,
    ) -> Self {
        let n = loop_buf.len() / 160 * 160;
        fill_sine(&mut loop_buf[..n]);
        Self {
            tx: Some(tx),
            loop_buf: Some(loop_buf),
            tone: None,
            rx: Some(rx),
            rx_buf: Some(rx_buf),
            capture: None,
            amplifier,
        }
    }

    pub fn tone_active(&self) -> bool {
        self.tone.is_some()
    }

    /// Enables the amplifier and plays the looped 400 Hz tone.
    pub fn start_tone(&mut self) -> bool {
        let (Some(tx), Some(buf)) = (self.tx.take(), self.loop_buf.take()) else {
            return false;
        };
        self.amplifier.set_high();
        match tx.write(buf) {
            Ok(transfer) => {
                self.tone = Some(transfer);
                true
            }
            Err((_, tx, buf)) => {
                self.tx = Some(tx);
                self.loop_buf = Some(buf);
                self.amplifier.set_low();
                false
            }
        }
    }

    pub fn stop_tone(&mut self) {
        if let Some(transfer) = self.tone.take() {
            let (tx, buf) = transfer.stop();
            self.tx = Some(tx);
            self.loop_buf = Some(buf);
        }
        self.amplifier.set_low();
    }

    pub fn start_capture(&mut self) -> bool {
        let (Some(rx), Some(buf)) = (self.rx.take(), self.rx_buf.take()) else {
            return false;
        };
        match rx.read(buf) {
            Ok(transfer) => {
                self.capture = Some(transfer);
                true
            }
            Err((_, rx, buf)) => {
                self.rx = Some(rx);
                self.rx_buf = Some(buf);
                false
            }
        }
    }

    pub fn stop_capture(&mut self) {
        if let Some(transfer) = self.capture.take() {
            let (rx, buf) = transfer.stop();
            self.rx = Some(rx);
            self.rx_buf = Some(buf);
        }
    }

    /// Drains captured samples and returns the peak level of the left and
    /// right microphone since the last call, or `None` when not capturing.
    pub fn read_levels(&mut self) -> Option<(f32, f32)> {
        let transfer = self.capture.as_mut()?;
        let mut scratch = [0u8; 1024];
        let (mut left, mut right) = (0.0f32, 0.0f32);
        loop {
            let available = transfer.available_bytes().min(scratch.len()) / 4 * 4;
            if available == 0 {
                break;
            }
            let got = transfer.pop(&mut scratch[..available]);
            let (l, r) = peak_levels(&scratch[..got]);
            left = left.max(l);
            right = right.max(r);
        }
        Some((left, right))
    }
}
