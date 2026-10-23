//! Table of nearby amulets (friends and enemies) and the measures derived from it.

use crate::faction::{relation, Faction, Relation};
use crate::params::*;

/// Closeness (0..1) from a smoothed RSSI in dBm.
pub fn closeness(rssi_dbm: f32) -> f32 {
    let c = (rssi_dbm - RSSI_FAR_DBM) / (RSSI_NEAR_DBM - RSSI_FAR_DBM);
    c.clamp(0.0, 1.0)
}

#[derive(Clone, Copy, Debug)]
pub struct Entry {
    pub id: u32,
    /// `Friend` or `Enemy`; unknown factions are never stored.
    pub relation: Relation,
    /// Exponentially smoothed RSSI in dBm.
    pub rssi: f32,
    pub last_seen_ms: u32,
    /// Two beacons arrived within `CONFIRM_WINDOW_MS` (a lone beacon never counts as enemy).
    pub confirmed: bool,
    /// Beacons per second over the last full rate window.
    pub rate_per_s: f32,
    window_start_ms: u32,
    window_count: u16,
}

impl Entry {
    pub fn closeness(&self) -> f32 {
        closeness(self.rssi)
    }
}

/// The measures shown by the display.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Measures {
    /// Number of friends, each weighted with its closeness.
    pub n_eff: f32,
    /// Closeness of the nearest friend.
    pub c_max: f32,
    /// Number of friends currently in the table.
    pub count: usize,
    /// Closeness of the nearest confirmed enemy (0 without enemies).
    pub danger: f32,
    /// Number of confirmed enemies.
    pub enemies: usize,
}

pub struct FriendTable {
    own_id: u32,
    own_faction: Faction,
    entries: [Option<Entry>; MAX_FRIENDS],
}

impl FriendTable {
    pub const fn new(own_id: u32, own_faction: Faction) -> Self {
        Self { own_id, own_faction, entries: [None; MAX_FRIENDS] }
    }

    pub fn own_faction(&self) -> Faction {
        self.own_faction
    }

    /// Registers a received beacon. Beacons with the own id or an unknown
    /// faction are ignored.
    pub fn on_beacon(&mut self, id: u32, faction_id: u8, rssi_dbm: i8, now_ms: u32) {
        if id == self.own_id {
            return;
        }
        let rel = relation(self.own_faction, faction_id);
        if rel == Relation::Ignore {
            return;
        }
        let rssi = rssi_dbm as f32;
        // Known device: update.
        if let Some(e) = self.entries.iter_mut().flatten().find(|e| e.id == id) {
            e.relation = rel;
            e.rssi += RSSI_EMA_ALPHA * (rssi - e.rssi);
            if now_ms.wrapping_sub(e.last_seen_ms) <= CONFIRM_WINDOW_MS {
                e.confirmed = true;
            }
            e.last_seen_ms = now_ms;
            if now_ms.wrapping_sub(e.window_start_ms) >= RATE_WINDOW_MS {
                e.rate_per_s = e.window_count as f32 * 1000.0
                    / now_ms.wrapping_sub(e.window_start_ms) as f32;
                e.window_start_ms = now_ms;
                e.window_count = 0;
            }
            e.window_count = e.window_count.saturating_add(1);
            return;
        }
        // New device: take a free slot.
        let new = Entry {
            id,
            relation: rel,
            rssi,
            last_seen_ms: now_ms,
            confirmed: false,
            rate_per_s: 0.0,
            window_start_ms: now_ms,
            window_count: 1,
        };
        if let Some(slot) = self.entries.iter_mut().find(|e| e.is_none()) {
            *slot = Some(new);
        }
    }

    /// Removes devices that have not been heard for `EXPIRE_MS`.
    pub fn expire(&mut self, now_ms: u32) {
        for e in self.entries.iter_mut() {
            if matches!(e, Some(f) if now_ms.wrapping_sub(f.last_seen_ms) > EXPIRE_MS) {
                *e = None;
            }
        }
        // A device that was quiet for longer than the confirm window needs two new beacons.
        for e in self.entries.iter_mut().flatten() {
            if now_ms.wrapping_sub(e.last_seen_ms) > CONFIRM_WINDOW_MS {
                e.confirmed = false;
            }
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &Entry> {
        self.entries.iter().flatten()
    }

    pub fn measures(&self) -> Measures {
        let mut m = Measures::default();
        for e in self.iter() {
            let c = e.closeness();
            match e.relation {
                Relation::Friend => {
                    m.n_eff += c;
                    if c > m.c_max {
                        m.c_max = c;
                    }
                    m.count += 1;
                }
                Relation::Enemy if e.confirmed => {
                    if c > m.danger {
                        m.danger = c;
                    }
                    m.enemies += 1;
                }
                _ => {}
            }
        }
        m
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const OWN: u32 = 1;
    const A: u8 = 0;
    const B: u8 = 1;

    fn table() -> FriendTable {
        FriendTable::new(OWN, Faction::A)
    }

    #[test]
    fn closeness_is_clamped_and_monotone() {
        assert_eq!(closeness(-120.0), 0.0);
        assert_eq!(closeness(-20.0), 1.0);
        assert!(closeness(-60.0) > closeness(-70.0));
        assert!((closeness(-70.0) - 0.5).abs() < 1e-6);
    }

    #[test]
    fn empty_table_has_zero_measures() {
        assert_eq!(table().measures(), Measures::default());
    }

    #[test]
    fn own_beacon_is_ignored() {
        let mut t = table();
        t.on_beacon(OWN, A, -40, 0);
        t.on_beacon(OWN, B, -40, 10);
        t.on_beacon(OWN, B, -40, 20);
        assert_eq!(t.measures(), Measures::default());
    }

    #[test]
    fn more_friends_raise_weighted_count() {
        let mut t = table();
        t.on_beacon(2, A, -60, 0);
        let one = t.measures().n_eff;
        t.on_beacon(3, A, -60, 0);
        assert!(t.measures().n_eff > one);
        assert_eq!(t.measures().count, 2);
    }

    #[test]
    fn nearer_friend_raises_c_max() {
        let mut t = table();
        t.on_beacon(2, A, -80, 0);
        let far = t.measures().c_max;
        for i in 1..20 {
            t.on_beacon(2, A, -45, i * 100);
        }
        assert!(t.measures().c_max > far);
    }

    #[test]
    fn friend_measures_stay_in_range() {
        let mut t = table();
        for id in 2..(2 + MAX_FRIENDS as u32 + 4) {
            t.on_beacon(id, A, -30, 0);
        }
        let m = t.measures();
        assert!(m.c_max <= 1.0);
        assert_eq!(m.count, MAX_FRIENDS);
        assert!(m.n_eff <= MAX_FRIENDS as f32);
    }

    #[test]
    fn single_outlier_changes_little() {
        let mut t = table();
        for i in 0..20 {
            t.on_beacon(2, A, -50, i * 100);
        }
        let before = t.measures().c_max;
        t.on_beacon(2, A, -75, 2_100); // outlier, 25 dB below the usual level
        assert!(before - t.measures().c_max < 0.2);
    }

    #[test]
    fn friends_expire() {
        let mut t = table();
        t.on_beacon(2, A, -50, 1_000);
        t.expire(1_000 + EXPIRE_MS);
        assert_eq!(t.measures().count, 1);
        t.expire(1_000 + EXPIRE_MS + 1);
        assert_eq!(t.measures(), Measures::default());
    }

    #[test]
    fn expire_handles_clock_wrap() {
        let mut t = table();
        t.on_beacon(2, A, -50, u32::MAX - 100);
        t.expire(50); // 151 ms later, across the wrap
        assert_eq!(t.measures().count, 1);
    }

    #[test]
    fn beacon_rate_is_measured() {
        let mut t = table();
        for i in 0..=60 {
            t.on_beacon(2, A, -50, i * 100);
        }
        let rate = t.iter().next().unwrap().rate_per_s;
        assert!((rate - 10.0).abs() < 1.0, "rate {rate}");
    }

    // --- factions ---

    #[test]
    fn enemies_do_not_count_as_friends() {
        let mut t = table();
        for i in 0..10 {
            t.on_beacon(2, B, -45, i * 100);
        }
        let m = t.measures();
        assert_eq!((m.n_eff, m.c_max, m.count), (0.0, 0.0, 0));
        assert_eq!(m.enemies, 1);
        assert!(m.danger > 0.9);
    }

    #[test]
    fn friends_do_not_create_danger() {
        let mut t = table();
        for i in 0..10 {
            t.on_beacon(2, A, -45, i * 100);
        }
        assert_eq!(t.measures().danger, 0.0);
        assert_eq!(t.measures().enemies, 0);
    }

    #[test]
    fn unknown_faction_is_not_stored() {
        let mut t = table();
        t.on_beacon(2, 7, -40, 0);
        t.on_beacon(2, 7, -40, 100);
        assert_eq!(t.iter().count(), 0);
    }

    #[test]
    fn lone_enemy_beacon_is_not_confirmed() {
        let mut t = table();
        t.on_beacon(2, B, -40, 0);
        assert_eq!(t.measures().danger, 0.0);
        t.on_beacon(2, B, -40, CONFIRM_WINDOW_MS + 1); // too late to confirm
        assert_eq!(t.measures().danger, 0.0);
        t.on_beacon(2, B, -40, CONFIRM_WINDOW_MS + 500);
        assert!(t.measures().danger > 0.9);
    }

    #[test]
    fn nearest_enemy_defines_danger() {
        let mut t = table();
        for i in 0..10 {
            t.on_beacon(2, B, -85, i * 100);
            t.on_beacon(3, B, -55, i * 100);
        }
        let m = t.measures();
        assert_eq!(m.enemies, 2);
        assert!((m.danger - closeness(-55.0)).abs() < 0.02, "{}", m.danger);
    }

    #[test]
    fn enemy_confirmation_lapses_when_quiet() {
        let mut t = table();
        for i in 0..5 {
            t.on_beacon(2, B, -45, i * 100);
        }
        assert!(t.measures().danger > 0.0);
        t.expire(400 + CONFIRM_WINDOW_MS + 1);
        assert_eq!(t.measures().danger, 0.0);
        t.expire(400 + EXPIRE_MS + 1);
        assert_eq!(t.iter().count(), 0);
    }
}
