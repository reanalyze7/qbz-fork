use crate::config::QbzdConfig;

use super::state::NetworkState;
use super::toml_rewrite::rewrite_toml;

#[test]
fn rewrite_preserves_unknown_and_known_keys() {
    // A file with a released key (data_root), a schema key we don't edit
    // (log.level) and an unrecognized key must ALL survive a server edit.
    let existing = "config_version = 1\ndata_root = \"/srv/qbzd\"\n\n[server]\nbind = \"127.0.0.1\"\nport = 8182\n\n[log]\nlevel = \"debug\"\n\n[weird]\nkey = \"value\"\n";
    let out = rewrite_toml(existing, "0.0.0.0", 9000, Some("secret")).unwrap();
    let parsed: toml::Table = toml::from_str(&out).unwrap();
    assert_eq!(parsed["config_version"].as_integer(), Some(1));
    assert_eq!(parsed["data_root"].as_str(), Some("/srv/qbzd"));
    assert_eq!(parsed["log"]["level"].as_str(), Some("debug"));
    assert_eq!(parsed["weird"]["key"].as_str(), Some("value"));
    assert_eq!(parsed["server"]["bind"].as_str(), Some("0.0.0.0"));
    assert_eq!(parsed["server"]["port"].as_integer(), Some(9000));
    assert_eq!(parsed["server"]["token"].as_str(), Some("secret"));
}

#[test]
fn empty_token_clears_the_key() {
    let existing = "[server]\nport = 8182\ntoken = \"old\"\n";
    let out = rewrite_toml(existing, "127.0.0.1", 8182, None).unwrap();
    let parsed: toml::Table = toml::from_str(&out).unwrap();
    assert!(parsed["server"].get("token").is_none(), "empty token removes the key");
}

#[test]
fn bad_ip_and_port_are_rejected() {
    let cfg = QbzdConfig::default();
    let mut st = NetworkState::new(&cfg, Vec::new());
    st.staged.bind = "not-an-ip".to_string();
    assert!(st.validated().is_err());
    st.staged.bind = "127.0.0.1".to_string();
    st.staged.port = "70000".to_string();
    assert!(st.validated().is_err());
    st.staged.port = "8182".to_string();
    assert!(st.validated().is_ok());
}
