use serde_json::Value;

use super::error::BundleError;
use super::types::Bundle;

impl Bundle {
    /// Serialize to pretty JSON (the on-disk `.qbzb` form — plain JSON).
    pub fn to_json_string(&self) -> Result<String, BundleError> {
        serde_json::to_string_pretty(self).map_err(|e| BundleError::Io(e.to_string()))
    }

    /// True when the document actually CARRIES a secret value: an `auth` token,
    /// or a non-blank scrobbler secret. The export-side §3 warning keys on this
    /// — not on the auth domain alone (a bundle whose only secrets are scrobbler
    /// tokens still needs the warning).
    pub fn contains_secrets(&self) -> bool {
        let auth_token = self
            .domains
            .get("auth")
            .and_then(Value::as_object)
            .and_then(|a| a.get("user_auth_token"))
            .and_then(Value::as_str)
            .map(|t| !t.is_empty())
            .unwrap_or(false);
        if auth_token {
            return true;
        }
        self.domains
            .get("integrations")
            .and_then(|i| i.get("scrobblers"))
            .and_then(Value::as_object)
            .map(|s| {
                ["lastfm_session_key", "listenbrainz_token"].iter().any(|k| {
                    s.get(*k)
                        .and_then(Value::as_str)
                        .map(|v| !v.is_empty())
                        .unwrap_or(false)
                })
            })
            .unwrap_or(false)
    }

    /// Parse a bundle from JSON text (import step 1). The version gate (step 2)
    /// runs in [`super::plan`]; here we only require a JSON object with an
    /// integer `schema_version`, so a malformed version is caught early.
    pub fn parse(text: &str) -> Result<Bundle, BundleError> {
        let value: Value =
            serde_json::from_str(text).map_err(|e| BundleError::Parse(e.to_string()))?;
        let mut obj = match value {
            Value::Object(m) => m,
            _ => return Err(BundleError::Parse("bundle root is not a JSON object".into())),
        };
        let schema_version = match obj.remove("schema_version") {
            Some(Value::Number(n)) if n.is_i64() || n.is_u64() => {
                n.as_i64().unwrap_or(i64::MAX)
            }
            _ => return Err(BundleError::VersionMalformed),
        };
        let created_at = obj
            .remove("created_at")
            .and_then(|v| v.as_str().map(str::to_string))
            .unwrap_or_default();
        let source = obj
            .remove("source")
            .and_then(|v| serde_json::from_value(v).ok())
            .unwrap_or_default();
        Ok(Bundle {
            schema_version,
            created_at,
            source,
            domains: obj,
        })
    }
}

