//! Native JACK output backend (#263 Tier 3).
//!
//! QBZ appears as a first-class JACK client (`qbz`) with stable output ports
//! `qbz:out_FL` / `qbz:out_FR`, patchable in qjackctl / qpwgraph / Reaper. The
//! client + ports are created ONCE and live for the whole session, so routing
//! survives track changes. On activation the ports are auto-connected to the
//! system's physical playback (so it "just works" without a patchbay); a user
//! may re-patch them freely.
//!
//! **NOT bit-perfect.** A JACK graph runs at ONE fixed rate, so audio is
//! resampled to the graph rate (by the player's feeder) before it reaches us.
//! Opt-in routing-freedom trade; the bit-perfect ALSA-exclusive / DAC-passthrough
//! paths are untouched.
//!
//! Architecture: a lock-free SPSC ring of **f32 samples** (`ringbuf`) sits
//! between the player's feeder thread (push, via [`JackStream::write_f32`]) and
//! the JACK `process` callback (pop, JACK's RT thread). f32 elements mean no
//! byte/alignment handling; writes/reads are kept to whole stereo frames.

mod process;
mod stream;

pub use stream::JackStream;

/// Ring capacity in stereo frames (~1.5 s at 44.1 kHz). Generous so the feeder
/// never blocks audio decode under normal scheduling.
const RING_CAPACITY_FRAMES: usize = 1 << 16; // 65536
/// Max stereo frames a single `process` cycle requests; the reusable scratch is
/// pre-sized to this so the RT callback never allocates.
const MAX_NFRAMES: usize = 16384;
