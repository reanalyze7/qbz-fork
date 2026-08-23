// crates/qbzd/src/cli/settings/codec_value.rs — value parse/render for the
// free-text / numeric audio keys.

/// `audio.device`: empty / "system" / "default" clears to `None` (system
/// default); anything else is the device id verbatim (`hw:CARD=D30,DEV=0`,
/// a PipeWire node name, ...) — free text, not validated against a live
/// enumeration here (that is the TUI's job, T13; a headless `settings set`
/// must work with no device attached to check against).
pub(super) fn parse_output_device(v: &str) -> Option<String> {
    let trimmed = v.trim();
    if trimmed.is_empty() || trimmed.eq_ignore_ascii_case("system") || trimmed.eq_ignore_ascii_case("default")
    {
        None
    } else {
        Some(trimmed.to_string())
    }
}
pub(super) fn render_opt_string(v: &Option<String>) -> String {
    v.clone().unwrap_or_else(|| "system".to_string())
}

/// `audio.device_max_sample_rate`: "none"/"" clears the limit (Hz, matching
/// the stored unit directly — e.g. `192000`, not `192` kHz).
pub(super) fn parse_opt_u32(v: &str) -> Result<Option<u32>, String> {
    let trimmed = v.trim();
    if trimmed.is_empty() || trimmed.eq_ignore_ascii_case("none") {
        return Ok(None);
    }
    trimmed
        .parse::<u32>()
        .map(Some)
        .map_err(|_| format!("invalid sample rate '{trimmed}' — expected a Hz integer (e.g. 192000) or none"))
}
pub(super) fn render_opt_u32(v: Option<u32>) -> String {
    v.map(|r| r.to_string()).unwrap_or_else(|| "none".to_string())
}

pub(super) fn parse_dsd_mode(v: &str) -> Result<String, String> {
    match v.to_ascii_lowercase().as_str() {
        "convert" | "dop" | "native" => Ok(v.to_ascii_lowercase()),
        other => Err(format!(
            "invalid DSD mode '{other}' — expected one of: convert, dop, native"
        )),
    }
}

/// The daemon has no one to ask (03-setup-tui.md §3.3.2) — `settings set`
/// never writes `"ask"`, even though a legacy/imported store may still hold
/// it (readable via `settings show`, just not settable back to it).
pub(super) fn parse_quality_fallback_behavior(v: &str) -> Result<String, String> {
    match v.to_ascii_lowercase().as_str() {
        "always_fallback" | "always_skip" => Ok(v.to_ascii_lowercase()),
        "ask" => Err(
            "'ask' needs a UI the daemon doesn't have — use always_fallback or always_skip"
                .to_string(),
        ),
        other => Err(format!(
            "invalid value '{other}' — expected one of: always_fallback, always_skip"
        )),
    }
}

pub(super) fn parse_f32(v: &str) -> Result<f32, String> {
    v.trim()
        .parse::<f32>()
        .map_err(|_| format!("invalid number '{v}'"))
}

pub(super) fn parse_stream_buffer_seconds(v: &str) -> Result<u8, String> {
    let n: u8 = v
        .trim()
        .parse()
        .map_err(|_| format!("invalid buffer size '{v}' — expected 1-10"))?;
    if !(1..=10).contains(&n) {
        return Err(format!("invalid buffer size '{n}' — expected 1-10"));
    }
    Ok(n)
}
