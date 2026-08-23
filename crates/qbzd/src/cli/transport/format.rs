// ============================ shared rendering ============================

pub(super) fn fmt_mmss(secs: u64) -> String {
    format!("{}:{:02}", secs / 60, secs % 60)
}

/// `96000` Hz -> `"96kHz"`; `44100` Hz -> `"44.1kHz"` — the `now` human line's
/// own Hz->kHz rendering choice (independent of the documented Hz-vs-kHz JSON
/// quirk between `playback.sample_rate` and `track.sample_rate`, which is
/// left as-is on the wire, 02 §2.2).
pub(super) fn fmt_khz(hz: u64) -> String {
    if hz % 1000 == 0 {
        format!("{}kHz", hz / 1000)
    } else {
        format!("{:.1}kHz", hz as f64 / 1000.0)
    }
}
