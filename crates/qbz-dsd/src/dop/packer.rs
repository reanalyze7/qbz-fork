//! [`DopPacker`]: stateful DoP frame packer (keeps marker phase across
//! calls).

/// Stateful DoP frame packer (keeps marker phase across calls).
pub struct DopPacker {
    marker_fa: bool,
}

impl DopPacker {
    pub fn new() -> Self {
        Self { marker_fa: false }
    }

    /// Pack planar MSB-first DSD bytes (2 bytes per channel per frame) into
    /// interleaved S32 DoP samples, appending to `out`. Consumes
    /// `min(len)/2` frames worth of every channel.
    pub fn pack(&mut self, planar: &[Vec<u8>], out: &mut Vec<i32>) {
        let ch = planar.len();
        let frames = planar.iter().map(|c| c.len()).min().unwrap_or(0) / 2;
        out.reserve(frames * ch);
        for f in 0..frames {
            let marker: i32 = if self.marker_fa { 0xFA } else { 0x05 };
            for c in planar.iter() {
                let b0 = c[f * 2] as i32;
                let b1 = c[f * 2 + 1] as i32;
                out.push(((marker << 16) | (b0 << 8) | b1) << 8);
            }
            self.marker_fa = !self.marker_fa;
        }
    }

    /// DSD silence (0x69 payload) with valid alternating markers — REQUIRED
    /// for pause/stop/tail padding: PCM zeros would break the marker
    /// sequence and pop the DAC out of DSD mode.
    pub fn silence(&mut self, n_frames: usize, channels: u16, out: &mut Vec<i32>) {
        out.reserve(n_frames * channels as usize);
        for _ in 0..n_frames {
            let marker: i32 = if self.marker_fa { 0xFA } else { 0x05 };
            for _ in 0..channels {
                out.push(((marker << 16) | 0x6969) << 8);
            }
            self.marker_fa = !self.marker_fa;
        }
    }
}

impl Default for DopPacker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn packer_layout_and_marker_alternation() {
        let mut p = DopPacker::new();
        let mut out = Vec::new();
        // One channel pair, two frames.
        let l = vec![0xAB, 0xCD, 0x12, 0x34];
        let r = vec![0x55, 0xAA, 0x9C, 0x63];
        p.pack(&[l, r], &mut out);
        assert_eq!(out.len(), 4);
        assert_eq!(out[0], ((0x05 << 16) | 0xABCD) << 8);
        assert_eq!(out[1], ((0x05 << 16) | 0x55AA) << 8);
        assert_eq!(out[2], ((0xFA << 16) | 0x1234) << 8);
        assert_eq!(out[3], ((0xFA << 16) | 0x9C63) << 8);
        // Phase continues across calls.
        let mut out2 = Vec::new();
        p.pack(&[vec![0, 0], vec![0, 0]], &mut out2);
        assert_eq!(out2[0], (0x05 << 16) << 8);
    }

    #[test]
    fn silence_is_0x69_payload_with_markers() {
        let mut p = DopPacker::new();
        let mut out = Vec::new();
        p.silence(2, 2, &mut out);
        assert_eq!(out[0], ((0x05 << 16) | 0x6969) << 8);
        assert_eq!(out[2], ((0xFA << 16) | 0x6969) << 8);
    }
}
