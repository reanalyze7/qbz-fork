use serde_json::Value;

use super::fmt::str_at;

/// 4 needs_auth · 5 configured device not present · else 0 (§1.3). Auth gates
/// before device: a login is the more common fix.
pub(super) fn exit_from_state(p: &Value) -> i32 {
    let auth = str_at(p, &["auth", "state"]);
    if auth == "needs_auth" {
        return 4;
    }
    let configured = p
        .pointer("/audio/configured_device")
        .map(|v| !v.is_null())
        .unwrap_or(false);
    let present = p
        .pointer("/audio/device_present")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);
    if configured && !present {
        return 5;
    }
    0
}
