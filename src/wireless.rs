//! WLAN scan and BLE advertising through `esp-radio`.
//!
//! The radio is only brought up on demand: the Wi-Fi controller exists just
//! for the duration of a scan, the BLE controller just for the time the
//! device advertises.

use alloc::{string::String, vec::Vec};

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
// --8<-- [end:wifi_scan]
}

/// Opcodes of the HCI commands needed for plain advertising (OGF 0x08 / 0x03).
const OP_RESET: u16 = 0x0C03;
const OP_SET_ADV_PARAMS: u16 = 0x2006;
const OP_SET_ADV_DATA: u16 = 0x2008;
const OP_SET_ADV_ENABLE: u16 = 0x200A;

#[derive(Debug)]
#[allow(dead_code)] // fields are only used through the Debug output
pub enum BleError {
    Init,
    Hci(u16, u8),
    Timeout(u16),
}

/// A BLE controller that advertises a connectable name with a given interval.
pub struct Advertiser<'d> {
    ble: BleConnector<'d>,
}

impl<'d> Advertiser<'d> {
    // --8<-- [start:ble_start]
    pub fn start(bt: BT<'d>, name: &str, interval_ms: u32) -> Result<Self, BleError> {
        let ble = BleConnector::new(bt, Default::default()).map_err(|_| BleError::Init)?;
        let mut this = Self { ble };
        this.command(OP_RESET, &[])?;

        // Interval unit is 0.625 ms; min = max = requested interval.
        let units = (interval_ms * 1000 / 625).clamp(0x20, 0x4000) as u16;
        let [lo, hi] = units.to_le_bytes();
        this.command(
            OP_SET_ADV_PARAMS,
            &[
                lo, hi, lo, hi, // min / max interval
                0x00, // ADV_IND (connectable, undirected)
                0x00, // own address type: public
                0x00, // peer address type
                0, 0, 0, 0, 0, 0, // peer address
                0x07, // all three advertising channels
                0x00, // no filter
            ],
        )?;

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
        this.command(OP_SET_ADV_DATA, &data)?;
        this.command(OP_SET_ADV_ENABLE, &[0x01])?;
        Ok(this)
    // --8<-- [end:ble_start]
    }

    /// Sends one HCI command and waits for its Command Complete event.
    fn command(&mut self, opcode: u16, params: &[u8]) -> Result<(), BleError> {
        let mut packet = [0u8; 4 + 40];
        packet[0] = 0x01; // HCI command packet
        packet[1..3].copy_from_slice(&opcode.to_le_bytes());
        packet[3] = params.len() as u8;
        packet[4..4 + params.len()].copy_from_slice(params);
        self.ble
            .write(&packet[..4 + params.len()])
            .map_err(|_| BleError::Init)?;

        let delay = esp_hal::delay::Delay::new();
        let mut buf = [0u8; 64];
        for _ in 0..200 {
            let n = self.ble.next(&mut buf).map_err(|_| BleError::Init)?;
            // [0x04, 0x0E, len, num_cmds, opcode_lo, opcode_hi, status, ...]
            if n >= 7 && buf[0] == 0x04 && buf[1] == 0x0E {
                let got = u16::from_le_bytes([buf[4], buf[5]]);
                if got == opcode {
                    return match buf[6] {
                        0 => Ok(()),
                        status => Err(BleError::Hci(opcode, status)),
                    };
                }
            }
            delay.delay_millis(5);
        }
        Err(BleError::Timeout(opcode))
    }

    pub fn stop(mut self) {
        let _ = self.command(OP_SET_ADV_ENABLE, &[0x00]);
    }
}
