//! Compiled redaction patterns and the cheap pre-check that guards them.

use std::sync::OnceLock;

use regex::Regex;

/// Compiled redaction patterns. Group 1 captures the labeled-key prefix that is kept;
/// the trailing value is what gets replaced. Patterns are case-insensitive.
pub(super) fn patterns() -> &'static Vec<Regex> {
    static PATTERNS: OnceLock<Vec<Regex>> = OnceLock::new();
    PATTERNS.get_or_init(|| {
        [
            // Qobuz user auth token (labeled key, JSON/query/assignment forms)
            r#"(?i)(user_auth_token["':=\s]+)[A-Za-z0-9._\-]+"#,
            // …and the request header form
            r#"(?i)(x-user-auth-token:\s*)\S+"#,
            // request_sig MD5 hex (labeled key form, hex >= 8)
            r#"(?i)(request_sig["':=\s]+)[a-f0-9]{8,}"#,
            // request_sig as a bare URL query param
            r#"(?i)(request_sig=)[a-f0-9]+"#,
            // app secret (app_secret / appsecret)
            r#"(?i)(app_?secret["':=\s]+)[A-Za-z0-9]+"#,
            // password
            r#"(?i)(password["':=\s]+)[^\s"',&]+"#,
            // authorization: Bearer <token>
            r#"(?i)(authorization:\s*bearer\s+)\S+"#,
            // bare bearer token
            r#"(?i)(bearer\s+)[A-Za-z0-9._\-]+"#,
            // OAuth access/refresh tokens (labeled key form)
            r#"(?i)((access|refresh)_token["':=\s]+)[A-Za-z0-9._\-]+"#,
            // generic URL token param
            r#"(?i)(token=)[^&\s"']+"#,
        ]
        .iter()
        .filter_map(|p| Regex::new(p).ok())
        .collect()
    })
}

/// Cheap pre-check: does the line contain any substring that one of the regexes could
/// match? Avoids running the whole pattern set on the overwhelming majority of lines.
pub(super) fn has_redaction_candidate(lower: &str) -> bool {
    const NEEDLES: [&str; 6] = ["token", "secret", "password", "bearer", "auth", "sig"];
    NEEDLES.iter().any(|n| lower.contains(n))
}
