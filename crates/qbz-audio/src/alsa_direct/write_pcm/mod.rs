//! `AlsaDirectStream::write` (i16 source) and `write_f32` (f32 source) — the
//! two large per-format match blocks (FloatLE/S32LE/S16LE/S243LE/S24LE
//! branches each doing conversion + `io.writei` + recovery), split by source
//! sample type since each is already near the 130-line budget on its own.

mod from_f32;
mod from_i16;
mod helpers;
