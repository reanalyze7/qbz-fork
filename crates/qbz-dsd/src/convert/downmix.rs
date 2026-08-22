//! Multichannel → stereo downmix used by [`super::converter`]'s per-frame
//! fold, once decimation has produced one f32 sample per channel per frame.
//!
//! DSF/DFF positional channel order:
//!   3ch = FL FR C · 4ch = FL FR BL BR · 5ch = FL FR C BL BR ·
//!   6ch = FL FR C LFE BL BR.
//! ITU-R BS.775 downmix (center/surrounds at −3 dB, LFE discarded),
//! normalized by the worst-case coefficient sum against clipping.

const K: f32 = std::f32::consts::FRAC_1_SQRT_2;

/// Fold frame `frame` of `per_ch` (one `Vec<f32>` per input channel) down to
/// a single (left, right) stereo pair.
pub(super) fn fold_to_stereo(channels: usize, per_ch: &[Vec<f32>], frame: usize) -> (f32, f32) {
    let (ci, sli, sri) = match channels {
        3 => (Some(2), None, None),
        4 => (None, Some(2), Some(3)),
        5 => (Some(2), Some(3), Some(4)),
        6 => (Some(2), Some(4), Some(5)),
        _ => (None, None, None),
    };
    let norm =
        1.0 / (1.0 + if ci.is_some() { K } else { 0.0 } + if sli.is_some() { K } else { 0.0 });
    match channels {
        1 => (per_ch[0][frame], per_ch[0][frame]),
        2 => (per_ch[0][frame], per_ch[1][frame]),
        _ => {
            let mut l = per_ch[0][frame];
            let mut r = per_ch[1][frame];
            if let Some(i) = ci {
                l += K * per_ch[i][frame];
                r += K * per_ch[i][frame];
            }
            if let Some(i) = sli {
                l += K * per_ch[i][frame];
            }
            if let Some(i) = sri {
                r += K * per_ch[i][frame];
            }
            (l * norm, r * norm)
        }
    }
}
