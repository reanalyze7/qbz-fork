//! Integration tests over synthesized DSF/DFF files. Entry point Cargo
//! discovers (`tests/*.rs`); actual test code lives under
//! `demux_convert/` and is pulled in via `#[path]`.

#[path = "demux_convert/fixtures.rs"]
mod fixtures;
#[path = "demux_convert/dsf_tests.rs"]
mod dsf_tests;
#[path = "demux_convert/dop_tests.rs"]
mod dop_tests;
#[path = "demux_convert/dff_tests.rs"]
mod dff_tests;
