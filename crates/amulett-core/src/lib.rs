//! Hardware-independent logic of the amulett firmware.
//!
//! Everything here is `no_std`, allocation free and unit-testable on the
//! development machine, see `docs/entwicklung.md`.

#![cfg_attr(not(test), no_std)]

pub mod beacon;
pub mod color;
pub mod friends;
pub mod hci;
pub mod params;
