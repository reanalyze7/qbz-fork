//! Shared row-builder primitives (1:1 with DiagnosticsPanel.svelte's small
//! formatting helpers).

use crate::DiagRow;

/// One diagnostics row. `status`: 0 info | 1 match | 2 mismatch.
pub(in crate::diagnostics) fn row(label: &str, saved: &str, runtime: &str, status: i32) -> DiagRow {
    DiagRow {
        label: label.into(),
        saved: saved.into(),
        runtime: runtime.into(),
        status,
    }
}

/// `ON`/`OFF`, mirroring the Tauri `bool()` helper.
pub(in crate::diagnostics) fn yn(value: bool) -> &'static str {
    if value {
        "ON"
    } else {
        "OFF"
    }
}

/// `Some -> value`, `None -> "—"`, mirroring the Tauri `str()` helper.
pub(in crate::diagnostics) fn opt(value: &Option<String>) -> String {
    value.clone().unwrap_or_else(|| "—".to_string())
}

/// Match status for the saved-vs-runtime comparison (Audio/Graphics).
pub(in crate::diagnostics) fn match_status(saved: &str, runtime: &str) -> i32 {
    if saved == "—" || runtime == "—" {
        0
    } else if saved == runtime {
        1
    } else {
        2
    }
}

/// Format a kHz value without a trailing ".0" (96.0 -> "96", 44.1 -> "44.1").
pub(in crate::diagnostics) fn trim_khz(khz: f64) -> String {
    if khz.fract().abs() < f64::EPSILON {
        format!("{}", khz as i64)
    } else {
        format!("{khz:.1}")
    }
}
