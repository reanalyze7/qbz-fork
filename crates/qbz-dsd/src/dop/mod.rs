//! DoP (DSD over PCM) framing per the DoP Open Standard v1.1 (dCS).
//!
//! Each 24-bit PCM sample carries 16 DSD bits (MSB-first, temporally
//! first byte in the high bits) under a marker byte that alternates
//! 0x05 / 0xFA on successive frames. Both channels of one frame carry the
//! SAME marker. A DoP-aware DAC detects the alternation and switches to
//! DSD mode; anything that alters even one sample breaks the sequence and
//! the DAC falls back to interpreting the stream as PCM (loud noise) —
//! which is why the packed words must travel a bit-exact integer path
//! (no f32, no gain, no resampling).
//!
//! Output words are FINAL S32 samples: the 24-bit DoP word left-justified
//! (`<< 8`), ready for `snd_pcm_writei` on an S32_LE stream.

mod packer;
mod stream;

pub use packer::DopPacker;
pub use stream::DopStream;

/// PCM carrier rate for a DSD bit rate: 16 DSD bits per frame per channel.
/// DSD64 → 176 400 Hz, DSD128 → 352 800 Hz.
pub const fn dop_carrier_rate(dsd_rate: u32) -> u32 {
    dsd_rate / 16
}
