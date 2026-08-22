//! Write-time secret redaction — the single most important safety layer in this crate.
//!
//! Two layers, applied in this order by [`redact`]:
//!   1. **Literal live-secret layer:** exact-string replacement of values registered via
//!      [`register_secret`] right after login (the real `user_auth_token`, `app_secret`, …).
//!      Catches tokens logged without a labeled key, which the regexes can't anticipate.
//!   2. **Regex layer:** a fixed set of labeled-key patterns (auth tokens, request_sig,
//!      app_secret, password, bearer/authorization, access/refresh tokens, URL `token=`).
//!      A cheap `.contains` pre-check short-circuits lines with no candidate substring.
//!
//! Every match collapses the secret VALUE to `***REDACTED***` while preserving the
//! labeled key prefix (capture group 1) so the line stays debuggable.

mod patterns;
mod registry;

use patterns::{has_redaction_candidate, patterns};
use registry::secrets;

const REPLACEMENT: &str = "***REDACTED***";
/// Live secret values shorter than this are ignored (too generic to scrub safely).
const MIN_SECRET_LEN: usize = 6;

/// Register a live secret value so the literal layer scrubs it everywhere, even when
/// logged without a labeled key. Empty / very short values (< [`MIN_SECRET_LEN`]) are
/// ignored, and duplicates are skipped.
pub fn register_secret(value: String) {
    if value.len() < MIN_SECRET_LEN {
        return;
    }
    if let Ok(mut guard) = secrets().write() {
        if !guard.iter().any(|s| s == &value) {
            guard.push(value);
        }
    }
}

/// Redact secrets from a single log line. Literal live-secret layer first, then regex.
pub fn redact(line: &str) -> String {
    let mut out = line.to_string();

    // Layer 1 — literal live secrets.
    if let Ok(guard) = secrets().read() {
        for secret in guard.iter() {
            if out.contains(secret.as_str()) {
                out = out.replace(secret.as_str(), REPLACEMENT);
            }
        }
    }

    // Layer 2 — labeled-key regexes (guarded by a cheap substring pre-check).
    let lower = out.to_ascii_lowercase();
    if has_redaction_candidate(&lower) {
        for re in patterns() {
            if re.is_match(&out) {
                out = re
                    .replace_all(&out, format!("${{1}}{REPLACEMENT}").as_str())
                    .into_owned();
            }
        }
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn redacts_all_known_shapes() {
        for s in [
            "GET /track?request_sig=ab12cd34ef56 user_auth_token=SEKRET_TOKEN_123",
            "X-User-Auth-Token: SEKRET_TOKEN_123",
            r#"{"app_secret":"abc123def","password":"hunter2"}"#,
            "authorization: Bearer eyJ.aaa.bbb",
        ] {
            let r = redact(s);
            assert!(!r.contains("SEKRET_TOKEN_123"), "leaked auth token: {r}");
            assert!(!r.contains("ab12cd34ef56"), "leaked request_sig: {r}");
            assert!(!r.contains("abc123def"), "leaked app_secret: {r}");
            assert!(!r.contains("hunter2"), "leaked password: {r}");
            assert!(!r.contains("eyJ.aaa.bbb"), "leaked bearer token: {r}");
        }
    }

    #[test]
    fn literal_registry_scrubs_unlabeled_value() {
        register_secret("LIVE_TOKEN_xyz".into());
        let r = redact("blah LIVE_TOKEN_xyz blah");
        assert!(!r.contains("LIVE_TOKEN_xyz"), "literal secret survived: {r}");
        assert!(r.contains(REPLACEMENT), "no redaction marker: {r}");
    }

    #[test]
    fn short_secret_is_ignored() {
        register_secret("abc".into()); // < MIN_SECRET_LEN -> not registered
        let r = redact("value abc here");
        assert!(r.contains("abc"), "short value should not be scrubbed: {r}");
    }

    #[test]
    fn has_redaction_candidate_gates_regex_layer() {
        // A line with no candidate substring at all should never hit the regex layer,
        // and a labeled-key line with a legacy/cached-style secret shape must still be
        // recognized as a candidate and redacted (guards against a future refactor of
        // the NEEDLES list silently starving the regex layer).
        assert!(!has_redaction_candidate("just a normal log line, nothing to see"));
        assert!(has_redaction_candidate(
            "cached user_auth_token=abcdef123456 from legacy session file"
        ));
        let r = redact("cached user_auth_token=abcdef123456 from legacy session file");
        assert!(!r.contains("abcdef123456"), "legacy cached token leaked: {r}");
    }
}
