//! WLAN scan and BLE through `esp-radio`.
//!
//! The radio is only brought up on demand: the Wi-Fi controller exists just
//! for the duration of a scan; BLE runs either as the connectable PoC
//! [`Advertiser`] or as the product [`Radio`] (beacon + scanner).

use alloc::{string::String, vec::Vec};

use amulett_core::{beacon::Beacon, hci, params};
use embassy_futures::block_on;
use esp_hal::peripherals::{BT, WIFI};
use esp_radio::{
    ble::controller::BleConnector,
    wifi::{scan::ScanConfig, WifiController},
};

#[derive(Clone, Debug)]
pub struct AccessPoint {
    pub ssid: String,
    pub rssi: i8,
    pub channel: u8,
}

// --8<-- [start:wifi_scan]
/// Runs one blocking Wi-Fi scan and returns up to `max` access points,
/// strongest signal first.
pub fn scan_wifi(wifi: WIFI<'_>, max: usize) -> Result<Vec<AccessPoint>, esp_radio::wifi::WifiError> {
    let mut controller = WifiController::new(wifi, Default::default())?;
    let found = block_on(controller.scan_async(&ScanConfig::default().with_max(max)))?;
    let mut list: Vec<AccessPoint> = found
        .into_iter()
        .map(|ap| AccessPoint {
            ssid: String::from(ap.ssid.as_str()),
            rssi: ap.signal_strength,
            channel: ap.channel,
        })
        .collect();
    list.sort_by(|a, b| b.rssi.cmp(&a.rssi));
    Ok(list)
}
// --8<-- [end:wifi_scan]

/// Opcodes of the HCI commands used here (OGF 0x03 / 0x08).
const OP_RESET: u16 = 0x0C03;
const OP_SET_EVENT_MASK: u16 = 0x0C01;
const OP_SET_ADV_PARAMS: u16 = 0x2006;
const OP_SET_ADV_DATA: u16 = 0x2008;
const OP_SET_ADV_ENABLE: u16 = 0x200A;
const OP_SET_SCAN_PARAMS: u16 = 0x200B;
const OP_SET_SCAN_ENABLE: u16 = 0x200C;

/// Advertising PDU types.
const ADV_IND: u8 = 0x00;
const ADV_NONCONN_IND: u8 = 0x03;

#[derive(Debug)]
#[allow(dead_code)] // fields are only used through the Debug output
pub enum BleError {
    Init,
    Hci(u16, u8),
    Timeout(u16),
}

/// Sends one HCI command and waits for its Command Complete event. Other
/// packets that arrive meanwhile are dropped.
fn hci_command(ble: &mut BleConnector<'_>, opcode: u16, params: &[u8]) -> Result<(), BleError> {
    let mut packet = [0u8; 4 + 40];
    packet[0] = 0x01; // HCI command packet
    packet[1..3].copy_from_slice(&opcode.to_le_bytes());
    packet[3] = params.len() as u8;
    packet[4..4 + params.len()].copy_from_slice(params);
    ble.write(&packet[..4 + params.len()]).map_err(|_| BleError::Init)?;

    let delay = esp_hal::delay::Delay::new();
    let mut buf = [0u8; 270];
    for _ in 0..200 {
        let n = ble.next(&mut buf).map_err(|_| BleError::Init)?;
        // [0x04, 0x0E, len, num_cmds, opcode_lo, opcode_hi, status, ...]
        if n >= 7 && buf[0] == 0x04 && buf[1] == 0x0E {
            let got = u16::from_le_bytes([buf[4], buf[5]]);
            if got == opcode {
                return match buf[6] {
                    0 => Ok(()),
                    status => Err(BleError::Hci(opcode, status)),
                };
            }
        } else if n == 0 {
            delay.delay_millis(5);
        }
    }
    Err(BleError::Timeout(opcode))
}

/// Advertising parameters: `[min, max, type, own addr, peer addr type, peer addr, channels, filter]`.
fn adv_params(interval_ms: u32, pdu_type: u8) -> [u8; 15] {
    // Interval unit is 0.625 ms; min = max = requested interval.
    let units = (interval_ms * 1000 / 625).clamp(0x20, 0x4000) as u16;
    let [lo, hi] = units.to_le_bytes();
    [lo, hi, lo, hi, pdu_type, 0x00, 0x00, 0, 0, 0, 0, 0, 0, 0x07, 0x00]
}

// --8<-- [start:ble_start]
/// Connectable advertiser with a local name (hardware PoC page "Funk").
pub struct Advertiser<'d> {
    ble: BleConnector<'d>,
}

impl<'d> Advertiser<'d> {
    pub fn start(bt: BT<'d>, name: &str, interval_ms: u32) -> Result<Self, BleError> {
        let mut ble = BleConnector::new(bt, Default::default()).map_err(|_| BleError::Init)?;
        hci_command(&mut ble, OP_RESET, &[])?;
        hci_command(&mut ble, OP_SET_ADV_PARAMS, &adv_params(interval_ms, ADV_IND))?;

        let mut data = [0u8; 32];
        let mut n = 0;
        // Flags: LE general discoverable, BR/EDR not supported.
        data[1..4].copy_from_slice(&[0x02, 0x01, 0x06]);
        n += 3;
        let name = name.as_bytes();
        let name = &name[..name.len().min(31 - n - 2)];
        data[1 + n] = name.len() as u8 + 1;
        data[2 + n] = 0x09; // complete local name
        data[3 + n..3 + n + name.len()].copy_from_slice(name);
        n += 2 + name.len();
        data[0] = n as u8;
        hci_command(&mut ble, OP_SET_ADV_DATA, &data)?;
        hci_command(&mut ble, OP_SET_ADV_ENABLE, &[0x01])?;
        Ok(Self { ble })
    }

    pub fn stop(mut self) {
        let _ = hci_command(&mut self.ble, OP_SET_ADV_ENABLE, &[0x00]);
    }
}
// --8<-- [end:ble_start]

/// Product radio: sends the amulett beacon (not connectable) and scans for
/// the beacons of other amulets at the same time.
pub struct Radio<'d> {
    ble: BleConnector<'d>,
    beacon: Beacon,
    last_counter_ms: u32,
}

impl<'d> Radio<'d> {
    /// Advertising data: flags + beacon, padded to the 32 byte HCI format.
    fn adv_data(beacon: &Beacon) -> [u8; 32] {
        let mut data = [0u8; 32];
        data[1..4].copy_from_slice(&[0x02, 0x01, 0x06]);
        data[4..4 + amulett_core::beacon::ENCODED_LEN].copy_from_slice(&beacon.encode());
        data[0] = (3 + amulett_core::beacon::ENCODED_LEN) as u8;
        data
    }

    // --8<-- [start:radio_start]
    pub fn start(bt: BT<'d>, own_id: u32) -> Result<Self, BleError> {
        let mut ble = BleConnector::new(bt, Default::default()).map_err(|_| BleError::Init)?;
        let beacon = Beacon { id: own_id, counter: 0 };
        hci_command(&mut ble, OP_RESET, &[])?;
        // Enable the LE Meta event (bit 61), which carries the advertising reports.
        hci_command(&mut ble, OP_SET_EVENT_MASK, &[0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x1F, 0x00, 0x20])?;

        // Beacon: not connectable.
        hci_command(
            &mut ble,
            OP_SET_ADV_PARAMS,
            &adv_params(params::ADV_INTERVAL_MS, ADV_NONCONN_IND),
        )?;
        hci_command(&mut ble, OP_SET_ADV_DATA, &Self::adv_data(&beacon))?;
        hci_command(&mut ble, OP_SET_ADV_ENABLE, &[0x01])?;

        // Scanner: passive, report every packet (no duplicate filter).
        let interval = (params::SCAN_INTERVAL_MS * 1000 / 625) as u16;
        let window = (params::SCAN_WINDOW_MS * 1000 / 625) as u16;
        let [il, ih] = interval.to_le_bytes();
        let [wl, wh] = window.to_le_bytes();
        hci_command(&mut ble, OP_SET_SCAN_PARAMS, &[0x00, il, ih, wl, wh, 0x00, 0x00])?;
        hci_command(&mut ble, OP_SET_SCAN_ENABLE, &[0x01, 0x00])?;
        Ok(Self { ble, beacon, last_counter_ms: 0 })
    }
    // --8<-- [end:radio_start]

    /// Increments the beacon counter once per second.
    pub fn tick(&mut self, now_ms: u32) {
        if now_ms.wrapping_sub(self.last_counter_ms) >= 1000 {
            self.last_counter_ms = now_ms;
            self.beacon.counter = self.beacon.counter.wrapping_add(1);
            let _ = hci_command(&mut self.ble, OP_SET_ADV_DATA, &Self::adv_data(&self.beacon));
        }
    }

    /// Drains pending HCI events and calls `f` for every advertising report.
    /// Returns the number of packets handled.
    pub fn poll(&mut self, mut f: impl FnMut(hci::AdvReport<'_>)) -> usize {
        let mut buf = [0u8; 270];
        let mut handled = 0;
        while handled < 64 {
            match self.ble.next(&mut buf) {
                Ok(n) if n > 0 => {
                    hci::for_each_adv_report(&buf[..n], &mut f);
                    handled += 1;
                }
                _ => break,
            }
        }
        handled
    }
}
