use serde_json::Value;

/// The frozen exit-code taxonomy (02 §1.3). `exit_code()` is the ONLY source of
/// a networked verb's process exit code; scripts encode these numbers forever.
#[derive(Debug)]
pub enum CliError {
    /// exit 3 — connect refused / timeout on the target (carries `host` for the
    /// §1.4 daemon-down copy).
    Unreachable(String),
    /// exit 4 — daemon in NeedsAuth; a Qobuz session is required.
    NeedsAuth,
    /// exit 5 — audio/device error (device unopenable, volume/seek fixed in DSD).
    Device(String),
    /// exit 6 — unknown id / index out of range.
    NotFound(String),
    /// exit 1 — breaking `api_version` skew: the verb refuses politely (§1.6).
    ApiSkew { daemon: u32, cli: u32 },
    /// exit 1 — any other runtime error (daemon said no / local failure).
    Runtime(String),
}

impl CliError {
    pub fn exit_code(&self) -> i32 {
        match self {
            CliError::Runtime(_) | CliError::ApiSkew { .. } => 1,
            CliError::Unreachable(_) => 3,
            CliError::NeedsAuth => 4,
            CliError::Device(_) => 5,
            CliError::NotFound(_) => 6,
        }
    }
}

impl std::fmt::Display for CliError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use crate::cli::copy;
        match self {
            CliError::Unreachable(host) => write!(f, "{}", copy::daemon_down(host)),
            CliError::NeedsAuth => write!(f, "{}", copy::daemon_up_needs_auth()),
            // `error_from_envelope` pre-builds the two DSD-specific codes into
            // the full verbatim §1.4 copy (already "error: ..." prefixed); a
            // plain device message from the server gets the generic prefix.
            CliError::Device(m) => {
                if m.starts_with("error:") {
                    write!(f, "{m}")
                } else {
                    write!(f, "error: {m}")
                }
            }
            CliError::NotFound(m) => write!(f, "error: {m}"),
            CliError::ApiSkew { daemon, cli } => write!(f, "{}", copy::api_version_skew(*daemon, *cli)),
            CliError::Runtime(m) => write!(f, "error: {m}"),
        }
    }
}
impl std::error::Error for CliError {}

/// Map an error envelope's `code` (02 §3.1.3) to the frozen exit taxonomy. The
/// CLI keys off `code`, never raw HTTP status. `origin_forbidden`/`invalid_token`
/// and anything unrecognized → exit 1.
pub(super) fn error_from_envelope(v: &Value) -> CliError {
    let err = v.get("error");
    let code = err
        .and_then(|e| e.get("code"))
        .and_then(|c| c.as_str())
        .unwrap_or("");
    let message = err
        .and_then(|e| e.get("message"))
        .and_then(|m| m.as_str())
        .unwrap_or("daemon returned an error")
        .to_string();
    match code {
        "needs_auth" => CliError::NeedsAuth,
        "not_found" => CliError::NotFound(message),
        // §1.4's verbatim DSD blocks are frozen client-side copy, not the
        // server's short envelope message — swap them in so `qbzd seek`/
        // `volume`/`mute` print the exact documented multi-line text.
        "volume_fixed_dsd" => CliError::Device(crate::cli::copy::volume_fixed_dsd()),
        "seek_unsupported_dsd" => CliError::Device(crate::cli::copy::seek_unsupported_dsd()),
        "audio_unavailable" | "device_error" => {
            CliError::Device(message)
        }
        _ => CliError::Runtime(message),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exit_codes_match_the_frozen_table() {
        // 02-cli-and-api.md §1.3.
        assert_eq!(CliError::Unreachable("x".into()).exit_code(), 3);
        assert_eq!(CliError::NeedsAuth.exit_code(), 4);
        assert_eq!(CliError::Device("x".into()).exit_code(), 5);
        assert_eq!(CliError::NotFound("x".into()).exit_code(), 6);
        assert_eq!(CliError::ApiSkew { daemon: 2, cli: 1 }.exit_code(), 1);
        assert_eq!(CliError::Runtime("x".into()).exit_code(), 1);
    }

    #[test]
    fn error_envelope_maps_code_to_exit() {
        let needs = serde_json::json!({"error": {"code": "needs_auth", "message": "no"}});
        assert_eq!(error_from_envelope(&needs).exit_code(), 4);
        let nf = serde_json::json!({"error": {"code": "not_found", "message": "no"}});
        assert_eq!(error_from_envelope(&nf).exit_code(), 6);
        let dev = serde_json::json!({"error": {"code": "volume_fixed_dsd", "message": "no"}});
        assert_eq!(error_from_envelope(&dev).exit_code(), 5);
        // origin_forbidden / invalid_token / unknown all fall to 1.
        let origin = serde_json::json!({"error": {"code": "origin_forbidden", "message": "no"}});
        assert_eq!(error_from_envelope(&origin).exit_code(), 1);
        let tok = serde_json::json!({"error": {"code": "invalid_token", "message": "no"}});
        assert_eq!(error_from_envelope(&tok).exit_code(), 1);
    }
}
