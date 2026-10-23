//! Payload of the amulett beacon (BLE advertising data).
//!
//! One AD structure of type `0xFF` (manufacturer specific data):
//!
//! | offset | size | content                        |
//! |-------:|-----:|--------------------------------|
//! | 0      | 1    | length (12)                    |
//! | 1      | 1    | AD type `0xFF`                 |
//! | 2      | 2    | company id `0xFFFF` (LE)       |
//! | 4      | 2    | magic `"AM"`                   |
//! | 6      | 1    | format version                 |
//! | 7      | 4    | device id (LE)                 |
//! | 11     | 1    | flags: bits 0-3 faction id     |
//! | 12     | 1    | counter                        |

pub const COMPANY_ID: u16 = 0xFFFF;
pub const MAGIC: [u8; 2] = *b"AM";
pub const VERSION: u8 = 2;
/// Size of the encoded AD structure.
pub const ENCODED_LEN: usize = 13;
/// Mask of the faction id in the flags byte.
const FACTION_MASK: u8 = 0x0F;

const AD_TYPE_MANUFACTURER: u8 = 0xFF;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Beacon {
    pub id: u32,
    pub counter: u8,
    /// Raw faction id (0..=15); see [`crate::faction::Faction`].
    pub faction: u8,
}

impl Beacon {
    pub fn encode(&self) -> [u8; ENCODED_LEN] {
        let mut out = [0u8; ENCODED_LEN];
        out[0] = (ENCODED_LEN - 1) as u8;
        out[1] = AD_TYPE_MANUFACTURER;
        out[2..4].copy_from_slice(&COMPANY_ID.to_le_bytes());
        out[4..6].copy_from_slice(&MAGIC);
        out[6] = VERSION;
        out[7..11].copy_from_slice(&self.id.to_le_bytes());
        out[11] = self.faction & FACTION_MASK;
        out[12] = self.counter;
        out
    }

    /// Looks through all AD structures of an advertising payload and returns
    /// the amulett beacon if there is one. Other devices yield `None`.
    // --8<-- [start:decode]
    pub fn decode(adv_data: &[u8]) -> Option<Beacon> {
        let mut rest = adv_data;
        while let Some((&len, tail)) = rest.split_first() {
            let len = len as usize;
            if len == 0 || len > tail.len() {
                return None;
            }
            let (ad, next) = tail.split_at(len);
            if let Some(b) = Self::decode_structure(ad) {
                return Some(b);
            }
            rest = next;
        }
        None
    }
    // --8<-- [end:decode]

    /// `ad` is the AD structure without its length byte (type + data).
    fn decode_structure(ad: &[u8]) -> Option<Beacon> {
        if ad.len() != ENCODED_LEN - 1 || ad[0] != AD_TYPE_MANUFACTURER {
            return None;
        }
        if u16::from_le_bytes([ad[1], ad[2]]) != COMPANY_ID || ad[3..5] != MAGIC || ad[5] != VERSION
        {
            return None;
        }
        Some(Beacon {
            id: u32::from_le_bytes([ad[6], ad[7], ad[8], ad[9]]),
            faction: ad[10] & FACTION_MASK,
            counter: ad[11],
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip() {
        let b = Beacon { id: 0xDEADBEEF, counter: 7, faction: 1 };
        assert_eq!(Beacon::decode(&b.encode()), Some(b));
    }

    #[test]
    fn finds_beacon_after_other_structures() {
        let b = Beacon { id: 1, counter: 2, faction: 0 };
        let mut data = [0u8; 3 + ENCODED_LEN];
        data[..3].copy_from_slice(&[0x02, 0x01, 0x06]); // flags
        data[3..].copy_from_slice(&b.encode());
        assert_eq!(Beacon::decode(&data), Some(b));
    }

    #[test]
    fn rejects_foreign_devices() {
        // A typical iBeacon-like payload with a different company id.
        let foreign = [0x02, 0x01, 0x06, 0x05, 0xFF, 0x4C, 0x00, 0x01, 0x02];
        assert_eq!(Beacon::decode(&foreign), None);
        assert_eq!(Beacon::decode(&[]), None);
    }

    #[test]
    fn rejects_wrong_magic_and_version() {
        let mut data = Beacon { id: 5, counter: 0, faction: 0 }.encode();
        data[4] = b'X';
        assert_eq!(Beacon::decode(&data), None);
        let mut data = Beacon { id: 5, counter: 0, faction: 0 }.encode();
        data[6] = VERSION - 1; // old format (version 1) is rejected
        assert_eq!(Beacon::decode(&data), None);
    }

    #[test]
    fn faction_is_limited_to_four_bits() {
        let b = Beacon { id: 9, counter: 1, faction: 0xF3 };
        assert_eq!(Beacon::decode(&b.encode()).unwrap().faction, 0x03);
    }

    #[test]
    fn rejects_truncated_data() {
        let data = Beacon { id: 5, counter: 0, faction: 0 }.encode();
        assert_eq!(Beacon::decode(&data[..data.len() - 1]), None);
    }
}
