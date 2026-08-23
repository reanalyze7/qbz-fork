//! Pure formatting helpers (mirror the Tauri modal helpers). Zero dependency
//! on Slint or the data structs.

use chrono::NaiveDate;

/// "Title (Version)" when a non-empty version exists, else "Title".
pub(super) fn format_title(title: &str, version: Option<&str>) -> String {
    let title = title.trim();
    match version.map(str::trim).filter(|v| !v.is_empty()) {
        Some(v) => format!("{title} ({v})"),
        None => title.to_string(),
    }
}

/// Track length as "M:SS" (zero-padded seconds), like Tauri `formatDuration`.
pub(super) fn track_duration(secs: u32) -> String {
    format!("{}:{:02}", secs / 60, secs % 60)
}

/// Album length as "1h 21m" / "45m" (no seconds), like `formatAlbumDuration`.
pub(super) fn album_duration(secs: u32) -> String {
    let hours = secs / 3600;
    let minutes = (secs % 3600) / 60;
    if hours > 0 {
        qbz_i18n::t_args("{} h {} min", &[&hours.to_string(), &minutes.to_string()])
    } else {
        qbz_i18n::t_args("{} min", &[&minutes.to_string()])
    }
}

/// Sample-rate value without a trailing ".0" (96.0 -> "96", 44.1 -> "44.1"),
/// matching JS number interpolation in the Tauri modals. NOT normalized
/// (Hz vs kHz) — the modals print the raw maximum_sampling_rate as Tauri does.
pub(super) fn fmt_rate(rate: f64) -> String {
    if rate.fract().abs() < f64::EPSILON {
        format!("{}", rate as i64)
    } else {
        format!("{rate}")
    }
}

/// Track Info quality — 1:1 with Tauri `formatQuality`: "24-bit / 96kHz" (NO
/// space before kHz), each field only when present; when both are absent fall
/// back to the Hi-Res / Lossless label (by `hires_streamable`).
pub(super) fn track_quality(bit: Option<u32>, rate: Option<f64>, hires_streamable: bool) -> String {
    let mut parts: Vec<String> = Vec::new();
    if let Some(b) = bit {
        parts.push(format!("{b}-bit"));
    }
    if let Some(r) = rate {
        parts.push(format!("{}kHz", fmt_rate(r)));
    }
    if parts.is_empty() {
        return if hires_streamable { "Hi-Res" } else { "Lossless" }.to_string();
    }
    parts.join(" / ")
}

/// Album Info quality — 1:1 with Tauri `formatQuality(bitDepth, samplingRate)`:
/// "24-Bit / 96 kHz" (capital Bit, space before kHz); the three present/absent
/// branches; empty string when both are absent (no fabricated defaults).
pub(super) fn album_quality(bit: Option<u32>, rate: Option<f64>) -> String {
    match (bit, rate) {
        (Some(b), Some(r)) => format!("{b}-Bit / {} kHz", fmt_rate(r)),
        (Some(b), None) => format!("{b}-Bit"),
        (None, Some(r)) => format!("{} kHz", fmt_rate(r)),
        (None, None) => String::new(),
    }
}

/// Localized "September 2, 2021" (full month). Empty when no/invalid date.
pub(super) fn full_release_date(raw: Option<&str>) -> String {
    let Some(raw) = raw else {
        return String::new();
    };
    let raw = raw.trim();
    if raw.is_empty() {
        return String::new();
    }
    let head = raw.get(0..10).unwrap_or(raw);
    if let Ok(parsed) = NaiveDate::parse_from_str(head, "%Y-%m-%d") {
        return parsed
            .format_localized("%B %-d, %Y", crate::dates::current_locale())
            .to_string();
    }
    String::new()
}

/// ", Role1, Role2" suffix for an Album-Credits performer row (raw roles,
/// 1:1 with Tauri which does NOT localize these), or "" when role-less.
pub(super) fn roles_suffix(roles: &[String]) -> String {
    if roles.is_empty() {
        String::new()
    } else {
        format!(", {}", roles.join(", "))
    }
}
