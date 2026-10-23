//! Table of nearby friends and the two measures derived from it.

use crate::params::*;

/// Closeness (0..1) from a smoothed RSSI in dBm.
pub fn closeness(rssi_dbm: f32) -> f32 {
    let c = (rssi_dbm - RSSI_FAR_DBM) / (RSSI_NEAR_DBM - RSSI_FAR_DBM);
    c.clamp(0.0, 1.0)
}

#[derive(Clone, Copy, Debug)]
pub struct Friend {
    pub id: u32,
    /// Exponentially smoothed RSSI in dBm.
    pub rssi: f32,
    pub last_seen_ms: u32,
    /// Beacons per second over the last full rate window.
    pub rate_per_s: f32,
    window_start_ms: u32,
    window_count: u16,
}

impl Friend {
    pub fn closeness(&self) -> f32 {
        closeness(self.rssi)
    }
}

/// The two measures shown by the colour display.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Measures {
    /// Number of friends, each weighted with its closeness.
    pub n_eff: f32,
    /// Closeness of the nearest friend.
    pub c_max: f32,
    /// Number of friends currently in the table.
    pub count: usize,
}

pub struct FriendTable {
    own_id: u32,
    entries: [Option<Friend>; MAX_FRIENDS],
}

impl FriendTable {
    pub const fn new(own_id: u32) -> Self {
        Self { own_id, entries: [None; MAX_FRIENDS] }
    }

    /// Registers a received beacon. Beacons with the own id are ignored.
    pub fn on_beacon(&mut self, id: u32, rssi_dbm: i8, now_ms: u32) {
        if id == self.own_id {
            return;
        }
        let rssi = rssi_dbm as f32;
        // Known friend: update.
        if let Some(f) = self.entries.iter_mut().flatten().find(|f| f.id == id) {
            f.rssi += RSSI_EMA_ALPHA * (rssi - f.rssi);
            f.last_seen_ms = now_ms;
            if now_ms.wrapping_sub(f.window_start_ms) >= RATE_WINDOW_MS {
                f.rate_per_s = f.window_count as f32 * 1000.0
                    / now_ms.wrapping_sub(f.window_start_ms) as f32;
                f.window_start_ms = now_ms;
                f.window_count = 0;
            }
            f.window_count = f.window_count.saturating_add(1);
            return;
        }
        // New friend: take a free slot.
        let new = Friend {
            id,
            rssi,
            last_seen_ms: now_ms,
            rate_per_s: 0.0,
            window_start_ms: now_ms,
            window_count: 1,
        };
        if let Some(slot) = self.entries.iter_mut().find(|e| e.is_none()) {
            *slot = Some(new);
        }
    }

    /// Removes friends that have not been heard for `EXPIRE_MS`.
    pub fn expire(&mut self, now_ms: u32) {
        for e in self.entries.iter_mut() {
            if matches!(e, Some(f) if now_ms.wrapping_sub(f.last_seen_ms) > EXPIRE_MS) {
                *e = None;
            }
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &Friend> {
        self.entries.iter().flatten()
    }

    pub fn measures(&self) -> Measures {
        let mut m = Measures::default();
        for f in self.iter() {
            let c = f.closeness();
            m.n_eff += c;
            if c > m.c_max {
                m.c_max = c;
            }
            m.count += 1;
        }
        m
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const OWN: u32 = 1;

    #[test]
    fn closeness_is_clamped_and_monotone() {
        assert_eq!(closeness(-120.0), 0.0);
        assert_eq!(closeness(-20.0), 1.0);
        assert!(closeness(-60.0) > closeness(-70.0));
        assert!((closeness(-70.0) - 0.5).abs() < 1e-6);
    }

    #[test]
    fn empty_table_has_zero_measures() {
        let t = FriendTable::new(OWN);
        assert_eq!(t.measures(), Measures::default());
    }

    #[test]
    fn own_beacon_is_ignored() {
        let mut t = FriendTable::new(OWN);
        t.on_beacon(OWN, -40, 0);
        assert_eq!(t.measures().count, 0);
    }

    #[test]
    fn more_friends_raise_weighted_count() {
        let mut t = FriendTable::new(OWN);
        t.on_beacon(2, -60, 0);
        let one = t.measures().n_eff;
        t.on_beacon(3, -60, 0);
        assert!(t.measures().n_eff > one);
        assert_eq!(t.measures().count, 2);
    }

    #[test]
    fn nearer_friend_raises_c_max() {
        let mut t = FriendTable::new(OWN);
        t.on_beacon(2, -80, 0);
        let far = t.measures().c_max;
        for i in 1..20 {
            t.on_beacon(2, -45, i * 100);
        }
        assert!(t.measures().c_max > far);
    }

    #[test]
    fn measures_stay_in_range() {
        let mut t = FriendTable::new(OWN);
        for id in 2..(2 + MAX_FRIENDS as u32 + 4) {
            t.on_beacon(id, -30, 0);
        }
        let m = t.measures();
        assert!(m.c_max <= 1.0);
        assert_eq!(m.count, MAX_FRIENDS);
        assert!(m.n_eff <= MAX_FRIENDS as f32);
    }

    #[test]
    fn single_outlier_changes_little() {
        let mut t = FriendTable::new(OWN);
        for i in 0..20 {
            t.on_beacon(2, -50, i * 100);
        }
        let before = t.measures().c_max;
        t.on_beacon(2, -75, 2_100); // outlier, 25 dB below the usual level
        let after = t.measures().c_max;
        assert!(before - after < 0.2, "{before} -> {after}");
    }

    #[test]
    fn friends_expire() {
        let mut t = FriendTable::new(OWN);
        t.on_beacon(2, -50, 1_000);
        t.expire(1_000 + EXPIRE_MS);
        assert_eq!(t.measures().count, 1);
        t.expire(1_000 + EXPIRE_MS + 1);
        assert_eq!(t.measures().count, 0);
        assert_eq!(t.measures(), Measures::default());
    }

    #[test]
    fn expire_handles_clock_wrap() {
        let mut t = FriendTable::new(OWN);
        t.on_beacon(2, -50, u32::MAX - 100);
        t.expire(50); // 151 ms later, across the wrap
        assert_eq!(t.measures().count, 1);
    }

    #[test]
    fn beacon_rate_is_measured() {
        let mut t = FriendTable::new(OWN);
        // 10 beacons per second for 6 seconds
        for i in 0..=60 {
            t.on_beacon(2, -50, i * 100);
        }
        let rate = t.iter().next().unwrap().rate_per_s;
        assert!((rate - 10.0).abs() < 1.0, "rate {rate}");
    }
}
