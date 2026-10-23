//! Colour area: maps the friend measures to saturation and brightness.

use crate::friends::Measures;
use crate::params::*;

/// Target saturation (0..1) from the weighted friend count.
pub fn target_saturation(m: &Measures) -> f32 {
    (m.n_eff / FULL_SATURATION_COUNT).clamp(0.0, 1.0)
}

/// Target brightness (0..1) from the closeness of the nearest friend.
pub fn target_brightness(m: &Measures) -> f32 {
    V_MIN + (V_MAX - V_MIN) * m.c_max.clamp(0.0, 1.0)
}

/// Moves saturation and brightness towards their targets with a limited rate.
#[derive(Clone, Copy, Debug)]
pub struct ColorController {
    pub saturation: f32,
    pub brightness: f32,
}

impl Default for ColorController {
    fn default() -> Self {
        Self { saturation: 0.0, brightness: V_MIN }
    }
}

fn approach(current: f32, target: f32, dt_s: f32) -> f32 {
    if target > current {
        (current + RISE_PER_S * dt_s).min(target)
    } else {
        (current - FALL_PER_S * dt_s).max(target)
    }
}

impl ColorController {
    pub fn step(&mut self, m: &Measures, dt_ms: u32) {
        let dt = dt_ms as f32 / 1000.0;
        self.saturation = approach(self.saturation, target_saturation(m), dt);
        self.brightness = approach(self.brightness, target_brightness(m), dt);
    }

    /// Current colour as RGB565 for the given hue in degrees.
    pub fn rgb565(&self, hue_deg: u16) -> u16 {
        hsv_to_rgb565(hue_deg, self.saturation, self.brightness)
    }
}

/// HSV (hue in degrees, saturation and value 0..1) to RGB565.
pub fn hsv_to_rgb565(hue_deg: u16, s: f32, v: f32) -> u16 {
    let s = (s.clamp(0.0, 1.0) * 255.0 + 0.5) as u32;
    let v = (v.clamp(0.0, 1.0) * 255.0 + 0.5) as u32;
    let h = (hue_deg % 360) as u32;
    let region = h / 60;
    let rem = (h % 60) * 255 / 60; // 0..255 inside the region
    let p = v * (255 - s) / 255;
    let q = v * (255 - s * rem / 255) / 255;
    let t = v * (255 - s * (255 - rem) / 255) / 255;
    let (r, g, b) = match region {
        0 => (v, t, p),
        1 => (q, v, p),
        2 => (p, v, t),
        3 => (p, q, v),
        4 => (t, p, v),
        _ => (v, p, q),
    };
    (((r >> 3) << 11) | ((g >> 2) << 5) | (b >> 3)) as u16
}

#[cfg(test)]
mod tests {
    use super::*;

    fn unpack(c: u16) -> (u32, u32, u32) {
        (((c >> 11) & 0x1F) as u32 * 255 / 31, ((c >> 5) & 0x3F) as u32 * 255 / 63, (c & 0x1F) as u32 * 255 / 31)
    }

    fn m(n_eff: f32, c_max: f32) -> Measures {
        Measures { n_eff, c_max, count: 0 }
    }

    #[test]
    fn zero_saturation_is_grey() {
        for hue in [0, 90, 210, 300] {
            let (r, g, b) = unpack(hsv_to_rgb565(hue, 0.0, 0.5));
            assert!(r.abs_diff(g) <= 4 && g.abs_diff(b) <= 4, "{r} {g} {b}");
        }
    }

    #[test]
    fn primary_colours() {
        assert_eq!(hsv_to_rgb565(0, 1.0, 1.0), 0xF800);
        assert_eq!(hsv_to_rgb565(120, 1.0, 1.0), 0x07E0);
        assert_eq!(hsv_to_rgb565(240, 1.0, 1.0), 0x001F);
        assert_eq!(hsv_to_rgb565(210, 1.0, 0.0), 0x0000);
    }

    #[test]
    fn brightness_is_monotone() {
        let mut last = 0;
        for i in 0..=10 {
            let (r, g, b) = unpack(hsv_to_rgb565(210, 1.0, i as f32 / 10.0));
            let sum = r + g + b;
            assert!(sum >= last);
            last = sum;
        }
    }

    #[test]
    fn saturation_is_monotone() {
        // More saturation moves the colour away from grey: spread grows.
        let mut last = 0;
        for i in 0..=10 {
            let (r, g, b) = unpack(hsv_to_rgb565(210, i as f32 / 10.0, 1.0));
            let spread = r.max(g).max(b) - r.min(g).min(b);
            assert!(spread >= last);
            last = spread;
        }
    }

    #[test]
    fn targets_follow_measures() {
        assert_eq!(target_saturation(&m(0.0, 0.0)), 0.0);
        assert_eq!(target_saturation(&m(FULL_SATURATION_COUNT, 1.0)), 1.0);
        assert_eq!(target_saturation(&m(100.0, 1.0)), 1.0);
        assert!(target_saturation(&m(2.0, 0.5)) > target_saturation(&m(1.0, 0.5)));
        assert_eq!(target_brightness(&m(0.0, 0.0)), V_MIN);
        assert_eq!(target_brightness(&m(1.0, 1.0)), V_MAX);
    }

    #[test]
    fn no_friend_stays_grey_and_dark() {
        let mut c = ColorController::default();
        for _ in 0..100 {
            c.step(&m(0.0, 0.0), DISPLAY_INTERVAL_MS);
        }
        assert_eq!(c.saturation, 0.0);
        assert_eq!(c.brightness, V_MIN);
    }

    #[test]
    fn sudden_friend_is_not_a_jump() {
        let mut c = ColorController::default();
        c.step(&m(4.0, 1.0), DISPLAY_INTERVAL_MS);
        assert!(c.saturation < 0.1, "{}", c.saturation);
        assert!(c.brightness < 0.2, "{}", c.brightness);
        // ... but reaches the target after enough time.
        for _ in 0..100 {
            c.step(&m(4.0, 1.0), DISPLAY_INTERVAL_MS);
        }
        assert_eq!(c.saturation, 1.0);
        assert_eq!(c.brightness, V_MAX);
    }

    #[test]
    fn fades_back_to_grey_slowly() {
        let mut c = ColorController { saturation: 1.0, brightness: 1.0 };
        c.step(&m(0.0, 0.0), 1000);
        assert!((c.saturation - (1.0 - FALL_PER_S)).abs() < 1e-6);
        for _ in 0..200 {
            c.step(&m(0.0, 0.0), DISPLAY_INTERVAL_MS);
        }
        assert_eq!(c.saturation, 0.0);
        assert_eq!(c.brightness, V_MIN);
    }

    #[test]
    fn steady_target_gives_steady_colour() {
        let mut c = ColorController::default();
        for _ in 0..200 {
            c.step(&m(2.0, 0.5), DISPLAY_INTERVAL_MS);
        }
        let first = c.rgb565(HUE_DEG);
        c.step(&m(2.0, 0.5), DISPLAY_INTERVAL_MS);
        assert_eq!(first, c.rgb565(HUE_DEG));
    }
}
