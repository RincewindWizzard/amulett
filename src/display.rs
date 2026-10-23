//! CO5300 AMOLED panel (466x466) on SPI2 in QSPI mode.
//!
//! Wire protocol (from the vendor Arduino_GFX `Arduino_ESP32QSPI` bus): every
//! register write is `0x02`, a 24-bit address `00 <reg> 00`, then data on one
//! line. Pixel data is sent as `0x32`, address `00 3C 00`, with the payload on
//! four lines. Pixels are big-endian RGB565.

use embedded_graphics::{
    pixelcolor::{raw::RawU16, Rgb565},
    prelude::*,
};
use esp_hal::{
    delay::Delay,
    gpio::Output,
    spi::{
        master::{Address, Command, DataMode, SpiDma},
        Error as SpiError,
    },
    Blocking,
};

pub const WIDTH: u16 = 466;
pub const HEIGHT: u16 = 466;
/// The panel's visible area starts at column 6 of the controller RAM.
const COLUMN_OFFSET: u16 = 6;
/// Largest payload per pixel transaction; must fit the DMA TX buffer.
pub const CHUNK_BYTES: usize = 4096;

const CMD_SLPIN: u8 = 0x10;
const CMD_SLPOUT: u8 = 0x11;
const CMD_DISPOFF: u8 = 0x28;
const CMD_DISPON: u8 = 0x29;
const CMD_CASET: u8 = 0x2A;
const CMD_PASET: u8 = 0x2B;
const CMD_RAMWR: u8 = 0x2C;
const CMD_PIXFMT: u8 = 0x3A;
const CMD_BRIGHTNESS: u8 = 0x51;
const CMD_CTRL_DISPLAY1: u8 = 0x53;
const CMD_CE: u8 = 0x58;
const CMD_BRIGHTNESS_HBM: u8 = 0x63;
const CMD_SPI_MODE: u8 = 0xC4;

pub struct Co5300<'d> {
    spi: SpiDma<'d, Blocking>,
    reset: Output<'d>,
}

impl<'d> Co5300<'d> {
    pub fn new(spi: SpiDma<'d, Blocking>, reset: Output<'d>) -> Self {
        Self { spi, reset }
    }

    fn write_reg(&mut self, reg: u8, data: &[u8]) -> Result<(), SpiError> {
        self.spi.half_duplex_write(
            DataMode::Single,
            Command::_8Bit(0x02, DataMode::Single),
            Address::_24Bit((reg as u32) << 8, DataMode::Single),
            0,
            data,
        )
    }

    /// Hardware reset followed by the vendor init sequence.
    pub fn init(&mut self, delay: &Delay) -> Result<(), SpiError> {
        self.reset.set_high();
        delay.delay_millis(10);
        self.reset.set_low();
        delay.delay_millis(200);
        self.reset.set_high();
        delay.delay_millis(200);

        self.write_reg(CMD_SLPOUT, &[])?;
        delay.delay_millis(120);
        self.write_reg(0xFE, &[0x00])?;
        self.write_reg(CMD_SPI_MODE, &[0x80])?;
        self.write_reg(CMD_PIXFMT, &[0x55])?; // 16 bit per pixel
        self.write_reg(CMD_CTRL_DISPLAY1, &[0x20])?;
        self.write_reg(CMD_BRIGHTNESS_HBM, &[0xFF])?;
        self.write_reg(CMD_DISPON, &[])?;
        self.write_reg(CMD_BRIGHTNESS, &[0xD0])?;
        self.write_reg(CMD_CE, &[0x00])?; // contrast enhancement off
        delay.delay_millis(10);
        Ok(())
    }

    /// Sets panel brightness (0 = darkest, 255 = brightest).
    pub fn set_brightness(&mut self, value: u8) -> Result<(), SpiError> {
        self.write_reg(CMD_BRIGHTNESS, &[value])
    }

    /// Display off + sleep in (lowest panel power without cutting its supply).
    pub fn sleep(&mut self, delay: &Delay) -> Result<(), SpiError> {
        self.write_reg(CMD_DISPOFF, &[])?;
        delay.delay_millis(120);
        self.write_reg(CMD_SLPIN, &[])?;
        delay.delay_millis(120);
        Ok(())
    }

    pub fn wake(&mut self, delay: &Delay) -> Result<(), SpiError> {
        self.write_reg(CMD_SLPOUT, &[])?;
        delay.delay_millis(120);
        self.write_reg(CMD_DISPON, &[])
    }

    fn set_window(&mut self, x0: u16, y0: u16, x1: u16, y1: u16) -> Result<(), SpiError> {
        let (x0, x1) = (x0 + COLUMN_OFFSET, x1 + COLUMN_OFFSET);
        self.write_reg(
            CMD_CASET,
            &[(x0 >> 8) as u8, x0 as u8, (x1 >> 8) as u8, x1 as u8],
        )?;
        self.write_reg(
            CMD_PASET,
            &[(y0 >> 8) as u8, y0 as u8, (y1 >> 8) as u8, y1 as u8],
        )?;
        self.write_reg(CMD_RAMWR, &[])
    }

    /// Writes big-endian RGB565 pixel bytes into the given window.
    pub fn blit(&mut self, x: u16, y: u16, w: u16, h: u16, bytes: &[u8]) -> Result<(), SpiError> {
        self.set_window(x, y, x + w - 1, y + h - 1)?;
        for chunk in bytes.chunks(CHUNK_BYTES) {
            self.spi.half_duplex_write(
                DataMode::Quad,
                Command::_8Bit(0x32, DataMode::Single),
                Address::_24Bit(0x003C00, DataMode::Single),
                0,
                chunk,
            )?;
        }
        Ok(())
    }
}

/// Full-screen RGB565 framebuffer that implements `embedded-graphics`.
///
/// Pixels are stored big-endian so the buffer can be sent to the panel as is.
pub struct Framebuffer {
    pixels: alloc::boxed::Box<[u16]>,
}

impl Framebuffer {
    /// Allocates 466x466x2 bytes (about 424 KiB) from the global allocator,
    /// i.e. PSRAM once it has been registered.
    pub fn new() -> Self {
        let n = WIDTH as usize * HEIGHT as usize;
        Self {
            pixels: alloc::vec![0u16; n].into_boxed_slice(),
        }
    }

    pub fn flush(&self, panel: &mut Co5300<'_>) -> Result<(), SpiError> {
        // SAFETY: u16 -> u8 reinterpretation of an initialized slice.
        let bytes = unsafe {
            core::slice::from_raw_parts(self.pixels.as_ptr() as *const u8, self.pixels.len() * 2)
        };
        panel.blit(0, 0, WIDTH, HEIGHT, bytes)
    }
}

impl OriginDimensions for Framebuffer {
    fn size(&self) -> Size {
        Size::new(WIDTH as u32, HEIGHT as u32)
    }
}

impl DrawTarget for Framebuffer {
    type Color = Rgb565;
    type Error = core::convert::Infallible;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        for Pixel(p, c) in pixels {
            if p.x >= 0 && p.y >= 0 && (p.x as u16) < WIDTH && (p.y as u16) < HEIGHT {
                let raw: u16 = RawU16::from(c).into_inner();
                self.pixels[p.y as usize * WIDTH as usize + p.x as usize] = raw.to_be();
            }
        }
        Ok(())
    }
}
