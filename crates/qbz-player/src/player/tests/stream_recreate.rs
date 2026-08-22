use super::*;

#[test]
fn no_stream_always_needs_new() {
    // Without an existing stream there is nothing to reuse — every
    // other flag is irrelevant.
    assert!(compute_needs_new_stream(
        false, false, false, false, false, false
    ));
}

#[test]
fn unchanged_format_on_default_backend_reuses_stream() {
    // Default rodio backend resamples internally, so an unchanged
    // decoded format on an existing stream needs no rebuild.
    assert!(!compute_needs_new_stream(
        true, false, false, false, false, false
    ));
}

#[test]
fn format_change_on_default_backend_rebuilds_for_native_rate() {
    // #449 regression guard: a decoded sample-rate/channel change must
    // rebuild the output stream on EVERY backend, not just the bit-perfect
    // ones, so the device follows the track's native rate.
    assert!(compute_needs_new_stream(
        true, true, false, false, false, false
    ));
}

#[test]
fn format_change_with_dac_passthrough_rebuilds() {
    assert!(compute_needs_new_stream(
        true, true, true, false, false, false
    ));
}

#[test]
fn format_change_with_alsa_direct_rebuilds() {
    assert!(compute_needs_new_stream(
        true, true, false, true, false, false
    ));
}

#[test]
fn format_change_with_coreaudio_exclusive_rebuilds() {
    assert!(compute_needs_new_stream(
        true, true, false, false, true, false
    ));
}

#[test]
fn bit_perfect_backends_without_format_change_reuse_stream() {
    // Bit-perfect flags only force a rebuild *together with* a format
    // change. On their own they should not.
    assert!(!compute_needs_new_stream(
        true, false, true, false, false, false
    ));
    assert!(!compute_needs_new_stream(
        true, false, false, true, false, false
    ));
    assert!(!compute_needs_new_stream(
        true, false, false, false, true, false
    ));
}

#[test]
fn coreaudio_shared_rate_mismatch_rebuilds_regardless_of_format_change() {
    // The CoreAudio shared-mode rate-drift case has nothing to do with
    // track format; it must rebuild whenever detected.
    assert!(compute_needs_new_stream(
        true, false, false, false, false, true
    ));
}
