//! reqwest error-message diagnostics — self-contained, no dependency on the
//! rest of the module.

/// reqwest's `Display` hides the source chain — which is exactly where the
/// diagnosis lives (Akamai's >100-header small-object flood surfaces as hyper's
/// "message head is too large" two levels down). Walk `source()` and join the
/// chain so logs AND signature matching see the real cause.
pub fn describe_reqwest_error(err: &reqwest::Error) -> String {
    use std::error::Error as _;
    let mut out = err.to_string();
    let mut source = err.source();
    while let Some(cause) = source {
        out.push_str(": ");
        out.push_str(&cause.to_string());
        source = cause.source();
    }
    out
}

/// True when an error message (already chain-expanded by
/// [`describe_reqwest_error`]) shows hyper's hard-coded h1 100-header cap.
/// Akamai answers SMALL raw-url objects with ~106 headers (the `X-AK-GRN` /
/// `X-AK-FWD-ERROR: ERR_POC_FWD_OBJ_TOO_SMALL` flood), so EVERY reqwest fetch
/// of such an URL fails this way — streaming probe and full download alike.
// KEPT: this recognises a specific Qobuz CDN failure (the X-AK-FWD-ERROR
// header flood) that makes EVERY fetch of an affected URL fail the same way.
// The detector exists and nothing consults it, so that condition currently
// surfaces as a generic network error. Wiring it is the fix; deleting it loses
// the diagnosis.
#[allow(dead_code)]
pub fn is_header_flood_error(message: &str) -> bool {
    let haystack = message.to_ascii_lowercase();
    haystack.contains("message head is too large") || haystack.contains("too many headers")
}
