//! Small drawing helpers for the demo pages on the round 466x466 display.

use alloc::string::String;
use core::fmt::Write;

use embedded_graphics::{
    mono_font::{ascii::FONT_10X20, MonoTextStyle},
    pixelcolor::Rgb565,
    prelude::*,
    primitives::{Circle, PrimitiveStyle, Rectangle},
    text::{Alignment, Text},
};

use crate::display::Framebuffer;

pub const CENTER: i32 = 233;
pub const LINE_HEIGHT: i32 = 26;

pub fn text_style(color: Rgb565) -> MonoTextStyle<'static, Rgb565> {
    MonoTextStyle::new(&FONT_10X20, color)
}

/// Draws one centered line of text; `y` is the baseline.
pub fn centered(fb: &mut Framebuffer, y: i32, color: Rgb565, text: &str) {
    let _ = Text::with_alignment(text, Point::new(CENTER, y), text_style(color), Alignment::Center)
        .draw(fb);
}

/// Draws a page title and a list of centered lines below it.
pub fn page(fb: &mut Framebuffer, title: &str, index: usize, count: usize, lines: &[String]) {
    let _ = fb.clear(Rgb565::BLACK);
    let _ = Circle::new(Point::new(0, 0), 466)
        .into_styled(PrimitiveStyle::with_stroke(Rgb565::new(4, 8, 4), 3))
        .draw(fb);
    centered(fb, 70, Rgb565::CYAN, title);
    let mut dots = String::new();
    for i in 0..count {
        let _ = write!(dots, "{}", if i == index { "O " } else { ". " });
    }
    centered(fb, 100, Rgb565::new(16, 32, 16), dots.trim_end());
    let mut y = 150;
    for line in lines {
        centered(fb, y, Rgb565::WHITE, line);
        y += LINE_HEIGHT;
    }
}

/// Draws a horizontal bar graph (`value` in 0.0..=1.0) centered on the display.
pub fn bar(fb: &mut Framebuffer, y: i32, width: u32, value: f32, color: Rgb565) {
    let x = CENTER - width as i32 / 2;
    let _ = Rectangle::new(Point::new(x, y), Size::new(width, 14))
        .into_styled(PrimitiveStyle::with_stroke(Rgb565::WHITE, 1))
        .draw(fb);
    let filled = (width as f32 * value.clamp(0.0, 1.0)) as u32;
    if filled > 0 {
        let _ = Rectangle::new(Point::new(x, y), Size::new(filled, 14))
            .into_styled(PrimitiveStyle::with_fill(color))
            .draw(fb);
    }
}
