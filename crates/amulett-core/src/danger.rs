//! Danger state with hysteresis and the pulsing alarm colour.

use crate::params::*;

/// On/off state of the danger display.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Danger {
    pub active: bool,
}

impl Danger {
    // --8<-- [start:update]
    /// `closeness` is the closeness of the nearest confirmed enemy (0 without enemies).
    /// Returns `true` when the state changed.
    pub fn update(&mut self, closeness: f32) -> bool {
        let next = if self.active { closeness >= DANGER_OFF } else { closeness >= DANGER_ON };
        let changed = next != self.active;
        self.active = next;
        changed
    }
    // --8<-- [end:update]
}

/// Pulse period in milliseconds: slow at `DANGER_ON`, fast at closeness 1.
pub fn pulse_period_ms(closeness: f32) -> u32 {
    let t = ((closeness - DANGER_ON) / (1.0 - DANGER_ON)).clamp(0.0, 1.0);
    let slow = PULSE_SLOW_MS as f32;
    let fast = PULSE_FAST_MS as f32;
    (slow + (fast - slow) * t) as u32
}

/// Brightness (`PULSE_MIN..=1`) of a triangle wave with the given period.
pub fn pulse_level(time_ms: u32, period_ms: u32) -> f32 {
    let period = period_ms.max(1);
    let phase = (time_ms % period) as f32 / period as f32; // 0..1
    let tri = if phase < 0.5 { phase * 2.0 } else { (1.0 - phase) * 2.0 };
    PULSE_MIN + (1.0 - PULSE_MIN) * tri
}

/// Alarm colour (pure red) as RGB565 for the given time and enemy closeness.
pub fn alarm_rgb565(time_ms: u32, closeness: f32) -> u16 {
    crate::color::hsv_to_rgb565(0, 1.0, pulse_level(time_ms, pulse_period_ms(closeness)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_inactive() {
        assert!(!Danger::default().active);
    }

    #[test]
    fn switches_on_at_threshold() {
        let mut d = Danger::default();
        assert!(!d.update(DANGER_ON - 0.01));
        assert!(!d.active);
        assert!(d.update(DANGER_ON));
        assert!(d.active);
    }

    #[test]
    fn hysteresis_prevents_flicker() {
        let mut d = Danger::default();
        d.update(0.6);
        assert!(d.active);
        // Wobbling between the two thresholds keeps the alarm on.
        for c in [0.40, 0.46, 0.35, 0.44, 0.32] {
            d.update(c);
            assert!(d.active, "off at {c}");
        }
        d.update(DANGER_OFF - 0.01);
        assert!(!d.active);
        // ... and does not come back below the on-threshold.
        d.update(0.40);
        assert!(!d.active);
    }

    #[test]
    fn ends_when_no_enemy_is_left() {
        let mut d = Danger::default();
        d.update(0.9);
        d.update(0.0);
        assert!(!d.active);
    }

    #[test]
    fn thresholds_form_a_hysteresis() {
        assert!(DANGER_OFF < DANGER_ON);
    }

    #[test]
    fn pulse_gets_faster_when_enemy_is_closer() {
        assert_eq!(pulse_period_ms(DANGER_ON), PULSE_SLOW_MS);
        assert_eq!(pulse_period_ms(1.0), PULSE_FAST_MS);
        assert!(pulse_period_ms(0.8) < pulse_period_ms(0.5));
        assert_eq!(pulse_period_ms(0.0), PULSE_SLOW_MS);
    }

    #[test]
    fn pulse_level_stays_in_range_and_peaks_mid_period() {
        for t in 0..2000 {
            let l = pulse_level(t, 700);
            assert!((PULSE_MIN..=1.0).contains(&l), "{l}");
        }
        assert!((pulse_level(0, 1000) - PULSE_MIN).abs() < 1e-6);
        assert!((pulse_level(500, 1000) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn alarm_colour_is_red() {
        for t in (0..1500).step_by(50) {
            let c = alarm_rgb565(t, 0.7);
            let (r, g, b) = ((c >> 11) & 0x1F, (c >> 5) & 0x3F, c & 0x1F);
            assert!(r > 0 && g == 0 && b == 0, "{c:04X}");
        }
    }
}
