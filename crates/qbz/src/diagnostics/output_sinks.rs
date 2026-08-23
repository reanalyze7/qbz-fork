//! Live-audio-device probing: CPAL sink enumeration + best-effort `pactl`
//! sample-format detection. The one piece that shells out — must run inside
//! `spawn_blocking`, never on the async path.

/// Query the live output sinks (BLOCKING — CPAL enumeration). Must be called
/// inside a `spawn_blocking`. Returns `(active_output, available_outputs)`:
/// `active_output` is the description (fallback name) of the default sink (the
/// ACTIVE output), and `available_outputs` is the description of every sink.
/// On `Err`, both are empty (treated as no sinks).
pub(super) fn collect_output_sinks() -> (Option<String>, Vec<String>, Option<(String, String)>) {
    let label = |s: &qbz_audio::output_sinks::OutputSinkInfo| -> String {
        if s.description.is_empty() {
            s.name.clone()
        } else {
            s.description.clone()
        }
    };
    let fmt = active_sink_format();
    match qbz_audio::output_sinks::list_output_sinks() {
        Ok(sinks) => {
            let active = sinks.iter().find(|s| s.is_default).map(&label);
            let available = sinks.iter().map(&label).collect();
            (active, available, fmt)
        }
        Err(e) => {
            log::warn!("[qbz-slint] diagnostics: list_output_sinks failed: {e}");
            (None, Vec::new(), fmt)
        }
    }
}

/// Best-effort LIVE sample format of the active (default) output sink, parsed
/// from `pactl list sinks short`. Returns `(rate, format)` like
/// `("44100 Hz", "s32le · 2ch")` — the rate is what the device is ACTUALLY
/// running at right now (vs the saved "Preferred Sample Rate"). Linux/PipeWire/
/// Pulse only (via pactl); `None` if pactl is unavailable or the sink can't be
/// determined. READ-ONLY — never touches the protected audio backend.
fn active_sink_format() -> Option<(String, String)> {
    use std::process::Command;
    let default = Command::new("pactl")
        .arg("get-default-sink")
        .output()
        .ok()
        .filter(|o| o.status.success())
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string());

    let out = Command::new("pactl")
        .args(["list", "sinks", "short"])
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let text = String::from_utf8(out.stdout).ok()?;

    // sample-spec token like "s32le 2ch 44100Hz" -> ("44100 Hz", "s32le · 2ch").
    let parse_spec = |spec: &str| -> (String, String) {
        let (mut rate, mut chans, mut fmt) = (String::new(), String::new(), String::new());
        for tok in spec.split_whitespace() {
            if let Some(hz) = tok.strip_suffix("Hz") {
                rate = format!("{hz} Hz");
            } else if tok.ends_with("ch") {
                chans = tok.to_string();
            } else {
                fmt = tok.to_string();
            }
        }
        let format = match (fmt.is_empty(), chans.is_empty()) {
            (false, false) => format!("{fmt} · {chans}"),
            (false, true) => fmt,
            (true, false) => chans,
            (true, true) => spec.trim().to_string(),
        };
        (rate, format)
    };

    // Prefer the default sink; fall back to the first RUNNING sink.
    let mut running: Option<(String, String)> = None;
    for line in text.lines() {
        let cols: Vec<&str> = line.split('\t').collect();
        if cols.len() < 5 {
            continue;
        }
        let (name, spec, state) = (cols[1], cols[3], cols[4]);
        if let Some(d) = &default {
            if name == d {
                return Some(parse_spec(spec));
            }
        }
        if state.eq_ignore_ascii_case("RUNNING") && running.is_none() {
            running = Some(parse_spec(spec));
        }
    }
    running
}
