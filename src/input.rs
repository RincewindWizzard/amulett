//! Touch gestures shared by the PoC pages and the product screens.

const SWIPE_MIN_DX: i32 = 100;
const SWIPE_MAX_DY: i32 = 120;
const LONG_PRESS_MS: u32 = 800;
const MOVE_TOLERANCE: i32 = 20;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Nav {
    Next,
    Prev,
}

/// Turns raw touch points into tap and swipe events.
#[derive(Default)]
pub struct Gesture {
    pub start: Option<(i32, i32)>,
    pub last: (i32, i32),
    start_ms: u32,
    long_fired: bool,
}

pub enum TouchEvent {
    Down(i32, i32),
    Move,
    Tap(i32, i32),
    LongPress,
    Swipe(Nav),
}

impl Gesture {
    /// `point` is the current touch position, `now_ms` a monotonic clock.
    pub fn update(&mut self, point: Option<(i32, i32)>, now_ms: u32) -> Option<TouchEvent> {
        match (point, self.start) {
            (Some(p), None) => {
                self.start = Some(p);
                self.last = p;
                self.start_ms = now_ms;
                self.long_fired = false;
                Some(TouchEvent::Down(p.0, p.1))
            }
            (Some(p), Some(s)) => {
                self.last = p;
                let still = (p.0 - s.0).abs() < MOVE_TOLERANCE && (p.1 - s.1).abs() < MOVE_TOLERANCE;
                if still && !self.long_fired && now_ms.wrapping_sub(self.start_ms) >= LONG_PRESS_MS {
                    self.long_fired = true;
                    Some(TouchEvent::LongPress)
                } else {
                    Some(TouchEvent::Move)
                }
            }
            (None, Some(s)) => {
                self.start = None;
                let (dx, dy) = (self.last.0 - s.0, self.last.1 - s.1);
                if self.long_fired {
                    None
                } else if dx.abs() >= SWIPE_MIN_DX && dy.abs() <= SWIPE_MAX_DY {
                    Some(TouchEvent::Swipe(if dx < 0 { Nav::Next } else { Nav::Prev }))
                } else if dx.abs() < MOVE_TOLERANCE && dy.abs() < MOVE_TOLERANCE {
                    Some(TouchEvent::Tap(s.0, s.1))
                } else {
                    None
                }
            }
            (None, None) => None,
        }
    }
}
