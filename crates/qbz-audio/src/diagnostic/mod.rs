//! Audio bit-depth diagnostic
//!
//! Lock-free capture that accumulates an OR-mask of all sample values
//! converted to i32. The trailing zeros in the mask reveal the effective
//! bit depth of the source data — no sample storage needed.
//!
//! Works for both rodio (PipeWire/ALSA via CPAL) and ALSA Direct paths
//! via a transparent Source wrapper.

mod result;
mod source;
mod state;

pub use result::BitDepthResult;
pub use source::DiagnosticSource;
pub use state::AudioDiagnostic;
