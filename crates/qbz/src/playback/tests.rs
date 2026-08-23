//! Unit tests for the pure formatters in `quality.rs`.

use super::quality::{fmt_elapsed, fmt_remaining};

#[test]
fn elapsed_pads_seconds() {
    assert_eq!(fmt_elapsed(0), "0:00");
    assert_eq!(fmt_elapsed(9), "0:09");
    assert_eq!(fmt_elapsed(65), "1:05");
    assert_eq!(fmt_elapsed(605), "10:05");
}

#[test]
fn remaining_counts_down_and_pads() {
    assert_eq!(fmt_remaining(0, 200), "-3:20");
    assert_eq!(fmt_remaining(195, 200), "-0:05");
    assert_eq!(fmt_remaining(200, 200), "-0:00");
    // Position past duration must not underflow.
    assert_eq!(fmt_remaining(250, 200), "-0:00");
}
