//! DFF-format integration tests, plus the general "not a DSD file" case.

use super::fixtures::{tmp, write_dff};
use qbz_dsd::{open_dsd, DsdError};

#[test]
fn dff_parses_stereo() {
    let path = write_dff("plain.dff", 2, 2_822_400, 8192, b"DSD ");
    let demux = open_dsd(&path).unwrap();
    let info = demux.info();
    assert_eq!(info.dsd_rate, 2_822_400);
    assert_eq!(info.channels, 2);
    assert_eq!(info.sample_count, 8192 / 2 * 8);
    assert!(!info.lsb_first);
}

#[test]
fn dff_dst_rejected() {
    let path = write_dff("dst.dff", 2, 2_822_400, 128, b"DST ");
    match open_dsd(&path) {
        Err(DsdError::UnsupportedDst) => {}
        Err(other) => panic!("expected UnsupportedDst, got {other:?}"),
        Ok(_) => panic!("expected UnsupportedDst, got Ok"),
    }
}

#[test]
fn garbage_rejected() {
    let path = tmp("garbage.bin");
    std::fs::write(&path, b"definitely not dsd").unwrap();
    assert!(matches!(open_dsd(&path), Err(DsdError::Corrupt(_))));
}
