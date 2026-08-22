//! DoP framing integration test.

use super::fixtures::write_dsf;
use qbz_dsd::{open_dsd, DopStream};

#[test]
fn dop_stream_frames_and_markers() {
    let path = write_dsf("dop64.dsf", 2, 2_822_400, 1, None);
    let demux = open_dsd(&path).unwrap();
    let mut dop = DopStream::new(demux).unwrap();
    assert_eq!(dop.carrier_rate(), 176_400);
    assert_eq!(dop.total_frames(), 4096 * 8 / 16);
    let words: Vec<i32> = dop.by_ref().collect();
    assert_eq!(words.len() as u64, dop.total_frames() * 2);
    // DSF silence is 0x69 LSB-first → bit-reversed payload is 0x96, and the
    // markers alternate per frame across both channels.
    assert_eq!(words[0], ((0x05 << 16) | 0x9696) << 8);
    assert_eq!(words[1], ((0x05 << 16) | 0x9696) << 8);
    assert_eq!(words[2], ((0xFA << 16) | 0x9696) << 8);
}
