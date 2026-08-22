//! HiFi Wizard (DAC setup) controller.
//!
//! Slice 6 (check step): runs the frontend-agnostic audio-stack probes
//! (`qbz_audio::health`) on open, maps them to per-distro copy-paste
//! remediations the check step renders, and recomputes them when the user
//! overrides the distro. Read-only — nothing here writes a system file or
//! opens a stream.

mod check;
mod review;
mod select;
mod test;

pub use check::{apply_health, open_immediate, set_distro, set_init};
pub use review::{apply_configs, checked_dacs, gen_configs_blocking, toggle_config, DacConfigData};
pub use select::{
    apply_candidates, begin_detect, detect_blocking, toggle_dac, validate_manual, DacCandidateData,
};
pub use test::{
    apply_poll, begin_test, end_test, queue_empty_notice, stash_test_tracks, test_tracks,
    track_matches_seed, TestSeed, TEST_SEEDS,
};
