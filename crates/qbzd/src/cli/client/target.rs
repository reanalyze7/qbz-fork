use crate::config::QbzdConfig;
use crate::paths::ProfileRoots;

/// A resolved target host + whether it is the local daemon (governs whether the
/// local `qbzd.toml` token and the linger check apply).
pub struct Target {
    pub addr: String,
    pub is_local: bool,
}

/// §1.5 target discovery: `--host` > `QBZD_HOST` > local `127.0.0.1:8182`. An
/// explicit override (flag or env) is treated as remote — only the local
/// default reads the local token / runs the linger check.
pub fn resolve_host(flag: Option<String>) -> Target {
    if let Some(h) = flag.filter(|h| !h.is_empty()) {
        return Target { addr: normalize_hostport(&h), is_local: false };
    }
    if let Ok(h) = std::env::var("QBZD_HOST") {
        if !h.is_empty() {
            return Target { addr: normalize_hostport(&h), is_local: false };
        }
    }
    Target { addr: "127.0.0.1:8182".into(), is_local: true }
}

/// Append the default port when the operator gave a bare host.
fn normalize_hostport(h: &str) -> String {
    if h.contains(':') {
        h.to_string()
    } else {
        format!("{h}:8182")
    }
}

/// §1.5 token discovery: `QBZD_TOKEN` (remote or local), else — only when
/// targeting the local daemon — the local `qbzd.toml` `[server] token`.
pub(crate) fn resolve_token(target: &Target, roots: &ProfileRoots) -> Option<String> {
    if let Ok(t) = std::env::var("QBZD_TOKEN") {
        if !t.is_empty() {
            return Some(t);
        }
    }
    if target.is_local {
        if let Ok((cfg, _)) = QbzdConfig::load(&roots.config.join("qbzd.toml")) {
            return cfg.server.token.filter(|t| !t.trim().is_empty());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn host_discovery_appends_default_port_and_flags_local() {
        assert_eq!(normalize_hostport("192.168.0.40"), "192.168.0.40:8182");
        assert_eq!(normalize_hostport("192.168.0.40:9000"), "192.168.0.40:9000");
        // An explicit flag is remote; the bare default is local.
        assert!(!resolve_host(Some("192.168.0.40".into())).is_local);
        // (Env-free path.) The default target is local.
        let t = resolve_host(None);
        if std::env::var("QBZD_HOST").is_err() {
            assert!(t.is_local);
            assert_eq!(t.addr, "127.0.0.1:8182");
        }
    }
}
