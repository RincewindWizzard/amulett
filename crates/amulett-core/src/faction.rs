//! Factions and the relation between them.

use crate::params::{HUE_A_DEG, HUE_B_DEG};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Faction {
    A,
    B,
}

/// How a received amulet relates to the own one.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Relation {
    Friend,
    Enemy,
    /// Unknown faction: neither friend nor enemy.
    Ignore,
}

impl Faction {
    /// Raw id used in the beacon.
    pub const fn id(self) -> u8 {
        match self {
            Faction::A => 0,
            Faction::B => 1,
        }
    }

    pub const fn from_id(id: u8) -> Option<Faction> {
        match id {
            0 => Some(Faction::A),
            1 => Some(Faction::B),
            _ => None,
        }
    }

    pub const fn letter(self) -> char {
        match self {
            Faction::A => 'A',
            Faction::B => 'B',
        }
    }

    /// Hue of the colour area in degrees.
    pub const fn hue(self) -> u16 {
        match self {
            Faction::A => HUE_A_DEG,
            Faction::B => HUE_B_DEG,
        }
    }

    pub const fn other(self) -> Faction {
        match self {
            Faction::A => Faction::B,
            Faction::B => Faction::A,
        }
    }
}

// --8<-- [start:relation]
/// The single place that decides who is friend and who is enemy.
pub fn relation(own: Faction, other_id: u8) -> Relation {
    match Faction::from_id(other_id) {
        None => Relation::Ignore,
        Some(f) if f == own => Relation::Friend,
        Some(_) => Relation::Enemy,
    }
}
// --8<-- [end:relation]

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_faction_is_friend() {
        assert_eq!(relation(Faction::A, 0), Relation::Friend);
        assert_eq!(relation(Faction::B, 1), Relation::Friend);
    }

    #[test]
    fn other_faction_is_enemy() {
        assert_eq!(relation(Faction::A, 1), Relation::Enemy);
        assert_eq!(relation(Faction::B, 0), Relation::Enemy);
    }

    #[test]
    fn unknown_faction_is_ignored() {
        assert_eq!(relation(Faction::A, 2), Relation::Ignore);
        assert_eq!(relation(Faction::B, 15), Relation::Ignore);
    }

    #[test]
    fn ids_roundtrip_and_hues_differ() {
        for f in [Faction::A, Faction::B] {
            assert_eq!(Faction::from_id(f.id()), Some(f));
            assert_eq!(f.other().other(), f);
        }
        assert_ne!(Faction::A.hue(), Faction::B.hue());
    }
}
