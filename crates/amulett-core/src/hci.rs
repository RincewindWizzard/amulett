//! Parsing of HCI events received from the BLE controller.

/// One entry of an LE Advertising Report.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AdvReport<'a> {
    pub addr: [u8; 6],
    pub data: &'a [u8],
    pub rssi: i8,
}

const PACKET_EVENT: u8 = 0x04;
const EVENT_LE_META: u8 = 0x3E;
const SUBEVENT_ADV_REPORT: u8 = 0x02;

/// Calls `f` for every report of an LE Advertising Report event packet
/// (including the leading HCI packet type byte). Other packets are ignored.
///
/// The reports are laid out per field, not per report: all event types, then
/// all address types, then all addresses, all data lengths, all data blocks and
/// finally all RSSI values.
pub fn for_each_adv_report<'a>(packet: &'a [u8], mut f: impl FnMut(AdvReport<'a>)) {
    if packet.len() < 5 || packet[0] != PACKET_EVENT || packet[1] != EVENT_LE_META {
        return;
    }
    let param_len = packet[2] as usize;
    let Some(params) = packet.get(3..3 + param_len) else {
        return;
    };
    if params[0] != SUBEVENT_ADV_REPORT {
        return;
    }
    let n = params[1] as usize;
    let body = &params[2..];
    // event types (n) + address types (n) + addresses (6n) + data lengths (n)
    let lens_at = 8 * n;
    if body.len() < 9 * n {
        return;
    }
    let lens = &body[lens_at..lens_at + n];
    let data_total: usize = lens.iter().map(|&l| l as usize).sum();
    if body.len() < 9 * n + data_total + n {
        return;
    }
    let mut data_at = 9 * n;
    let rssi_at = data_at + data_total;
    for i in 0..n {
        let len = lens[i] as usize;
        let mut addr = [0u8; 6];
        addr.copy_from_slice(&body[2 * n + 6 * i..2 * n + 6 * i + 6]);
        f(AdvReport {
            addr,
            data: &body[data_at..data_at + len],
            rssi: body[rssi_at + i] as i8,
        });
        data_at += len;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Event packet in a fixed buffer (the tests avoid `alloc`/`std`, see docs/entwicklung.md).
    struct Packet {
        buf: [u8; 96],
        len: usize,
    }

    impl Packet {
        fn push(&mut self, b: u8) {
            self.buf[self.len] = b;
            self.len += 1;
        }
        fn bytes(&self) -> &[u8] {
            &self.buf[..self.len]
        }
    }

    /// Builds an LE Advertising Report event with the given (data, rssi) reports.
    fn event(reports: &[(&[u8], i8)]) -> Packet {
        let n = reports.len();
        let mut p = Packet { buf: [0; 96], len: 0 };
        for b in [0x04, 0x3E, 0 /* length, patched below */, 0x02, n as u8] {
            p.push(b);
        }
        for _ in 0..2 * n {
            p.push(0x00); // event types, address types
        }
        for i in 0..n {
            for b in [i as u8 + 1, 2, 3, 4, 5, 6] {
                p.push(b);
            }
        }
        for (d, _) in reports {
            p.push(d.len() as u8);
        }
        for (d, _) in reports {
            for &b in *d {
                p.push(b);
            }
        }
        for (_, r) in reports {
            p.push(*r as u8);
        }
        p.buf[2] = (p.len - 3) as u8;
        p
    }

    /// (first data byte, data length, rssi, first address byte) of up to four reports.
    fn summary(packet: &[u8]) -> ([(u8, usize, i8, u8); 4], usize) {
        let mut out = [(0, 0, 0, 0); 4];
        let mut n = 0;
        for_each_adv_report(packet, |r| {
            out[n] = (r.data.first().copied().unwrap_or(0), r.data.len(), r.rssi, r.addr[0]);
            n += 1;
        });
        (out, n)
    }

    #[test]
    fn single_report() {
        let p = event(&[(&[1, 2, 3], -60)]);
        let (got, n) = summary(p.bytes());
        assert_eq!(n, 1);
        assert_eq!(got[0], (1, 3, -60, 1));
    }

    #[test]
    fn two_reports_with_different_lengths() {
        let p = event(&[(&[9, 8], -70), (&[7, 6, 5, 4], -45)]);
        let (got, n) = summary(p.bytes());
        assert_eq!(n, 2);
        assert_eq!(got[0], (9, 2, -70, 1));
        assert_eq!(got[1], (7, 4, -45, 2));
    }

    #[test]
    fn ignores_other_packets() {
        assert_eq!(summary(&[0x04, 0x0E, 0x04, 0x01, 0x03, 0x0C, 0x00]).1, 0);
        assert_eq!(summary(&[0x02, 0x00]).1, 0);
        assert_eq!(summary(&[]).1, 0);
        // LE meta event with another subevent (connection complete)
        assert_eq!(summary(&[0x04, 0x3E, 0x02, 0x01, 0x00]).1, 0);
    }

    #[test]
    fn truncated_packet_is_ignored() {
        let p = event(&[(&[1, 2, 3], -60)]);
        assert_eq!(summary(&p.bytes()[..p.len - 2]).1, 0);
    }
}
