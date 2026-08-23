// ============================ pure, unit-tested ============================

mod query;
use query::{code_from_query, url_path};

/// FB1 (owner feedback, post-smoke): resolve which host the OAuth redirect
/// targets. The common real-world case is configuring the daemon headless
/// over SSH from another machine on the LAN — the login URL must be openable
/// from ANY browser on the network by default, no flags.
///
/// Priority:
///   1. `cli_flag` (`--callback-host`) — explicit, unchanged, always wins.
///   2. `ssh_connection` (`$SSH_CONNECTION`)'s 3rd whitespace-separated field
///      — the SERVER ip, i.e. exactly the address the operator's other
///      machine used to reach this box. Malformed/short/non-IP values fall
///      through.
///   3. `127.0.0.1` — today's local-laptop behavior, unchanged.
///
/// Returns `(host, auto_detected)`; `auto_detected` is true only for case 2,
/// so callers can print the one extra explanatory line (§1.4 voice).
pub fn resolve_callback_host(
    cli_flag: Option<&str>,
    ssh_connection: Option<&str>,
) -> (String, bool) {
    if let Some(h) = cli_flag {
        return (h.to_string(), false);
    }
    if let Some(server_ip) = ssh_connection
        .and_then(|c| c.split_whitespace().nth(2))
        .filter(|candidate| candidate.parse::<std::net::IpAddr>().is_ok())
    {
        return (server_ip.to_string(), true);
    }
    ("127.0.0.1".to_string(), false)
}

/// Build the Qobuz browser authorize URL. Mirrors the desktop shape
/// (`crates/qbz/src/auth.rs:76-80`) except the redirect URL carries the CSPRNG
/// nonce as its path segment: `redirect_url=http://<host>:<port>/<nonce>`. The
/// binding rides the redirect URL itself (preserved verbatim by the OAuth
/// round-trip) instead of a `state` param, because the working desktop flow
/// sends no `state` and Qobuz is not proven to echo one.
pub fn build_oauth_url(ext_app_id: &str, host: &str, port: u16, nonce: &str) -> String {
    let redirect = format!("http://{}:{port}/{nonce}", bracket_ipv6_for_url(host));
    format!(
        "https://www.qobuz.com/signin/oauth?ext_app_id={}&redirect_url={}",
        ext_app_id,
        urlencoding::encode(&redirect),
    )
}

/// Bracket a bare IPv6 literal for use in a URL authority
/// (`2001:db8::2` → `[2001:db8::2]`); IPv4, hostnames, and already-bracketed
/// input pass through unchanged. FB1: `SSH_CONNECTION`'s server-IP field can
/// be IPv6.
fn bracket_ipv6_for_url(host: &str) -> String {
    if host.contains(':') && !host.starts_with('[') {
        format!("[{host}]")
    } else {
        host.to_string()
    }
}

/// Parse an HTTP request line from the one-shot listener. Returns the
/// authorization code ONLY when the request PATH carries the expected nonce
/// (`GET /<nonce>?...`) — a mismatched or absent path nonce is dropped (the D6
/// binding). No dependency on any `state` query param (Qobuz is not proven to
/// echo one; one present is simply ignored). `code_autorisation` wins over
/// `code`, matching the desktop.
pub fn parse_callback(request_line: &str, expected_nonce: &str) -> Option<String> {
    let target = request_line.split_whitespace().nth(1)?;
    let (path, query) = target.split_once('?')?;
    if path.trim_matches('/') != expected_nonce {
        return None; // wrong or absent path nonce → drop
    }
    code_from_query(query)
}

/// Parse pasted `--paste` input: either a full redirect URL or a bare
/// authorization code. A pasted URL carries the nonce in its PATH (that is how
/// the redirect was built); validation is lenient — an empty path is tolerated
/// (hand-pasted, possibly truncated), a present-but-wrong path nonce is
/// rejected.
pub fn code_from_paste(input: &str, expected_nonce: &str) -> Option<String> {
    let input = input.trim();
    if input.is_empty() {
        return None;
    }
    match input.split_once('?') {
        Some((prefix, query)) => {
            let seg = url_path(prefix).trim_matches('/');
            if !seg.is_empty() && seg != expected_nonce {
                return None; // present but mismatched → drop
            }
            code_from_query(query)
        }
        // No query string: a bare code is fine; a bare URL has nothing to extract.
        None if input.contains("://") => None,
        None => Some(input.to_string()),
    }
}

/// A 48-hex-char (24-byte) CSPRNG nonce, bound into the redirect-URL path.
pub(super) fn gen_nonce() -> String {
    use rand::RngExt;
    let mut bytes = [0u8; 24];
    rand::rng().fill(&mut bytes);
    let mut s = String::with_capacity(48);
    for b in bytes {
        use std::fmt::Write as _;
        let _ = write!(s, "{b:02x}");
    }
    s
}

#[cfg(test)]
mod tests;
