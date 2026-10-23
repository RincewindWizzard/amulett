//! All tuning parameters in one place.
//!
//! Start values; they are meant to be adjusted after field tests, see
//! `docs/farbanzeige.md`.

// --- Radio -----------------------------------------------------------------

/// Advertising interval of the amulett beacon.
pub const ADV_INTERVAL_MS: u32 = 300;
/// Scan interval and window (duty cycle = window / interval).
pub const SCAN_INTERVAL_MS: u32 = 100;
pub const SCAN_WINDOW_MS: u32 = 30;

// --- Friend table ------------------------------------------------------------

/// A friend is forgotten when no beacon arrived for this long.
pub const EXPIRE_MS: u32 = 10_000;
/// Smoothing factor of the RSSI exponential moving average (0..1, larger = faster).
pub const RSSI_EMA_ALPHA: f32 = 0.25;
/// RSSI (dBm) at which a friend counts as "next to me" (closeness 1).
pub const RSSI_NEAR_DBM: f32 = -50.0;
/// RSSI (dBm) at which a friend counts as "far away" (closeness 0).
pub const RSSI_FAR_DBM: f32 = -90.0;
/// Length of the window over which the beacon rate is measured.
pub const RATE_WINDOW_MS: u32 = 5_000;
/// Capacity of the friend table.
pub const MAX_FRIENDS: usize = 16;

// --- Colour display ------------------------------------------------------------

/// Hue of the colour area per faction (degrees): A blue, B orange. Red is reserved for danger.
pub const HUE_A_DEG: u16 = 210;
pub const HUE_B_DEG: u16 = 30;
/// Weighted friend count at which the colour reaches full saturation.
pub const FULL_SATURATION_COUNT: f32 = 4.0;
/// Brightness without friends and with the nearest friend right next to us (0..1).
pub const V_MIN: f32 = 0.08;
pub const V_MAX: f32 = 1.0;
/// Maximum change per second of saturation / brightness.
pub const RISE_PER_S: f32 = 0.5;
pub const FALL_PER_S: f32 = 0.25;
/// Interval of the display update loop.
pub const DISPLAY_INTERVAL_MS: u32 = 50;

// --- Factions and danger -----------------------------------------------------------

/// Two beacons of an enemy within this time confirm it (a single beacon is ignored).
pub const CONFIRM_WINDOW_MS: u32 = 3_000;
/// Danger starts when the closeness of the nearest confirmed enemy reaches this value ...
pub const DANGER_ON: f32 = 0.45;
/// ... and ends when it falls below this value (hysteresis, must be smaller than `DANGER_ON`).
pub const DANGER_OFF: f32 = 0.30;
/// Pulse period of the danger display at `DANGER_ON` and at closeness 1.
pub const PULSE_SLOW_MS: u32 = 1_200;
pub const PULSE_FAST_MS: u32 = 300;
/// Lowest brightness of the pulse (0..1).
pub const PULSE_MIN: f32 = 0.35;
/// Play a warning tone while in danger (needs the audio hardware to be initialised).
pub const WARN_TONE: bool = false;
