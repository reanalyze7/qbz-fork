use super::*;

#[test]
fn defaults_match_spec() {
    // 01-architecture.md §10.1 (FB6: default bind widened to 0.0.0.0 for
    // LAN-first control — restrict via [server] bind or token to opt back
    // into loopback-only).
    let (c, warns) = QbzdConfig::from_str("").unwrap();
    assert_eq!(c.config_version, 1);
    assert_eq!(c.server.bind, "0.0.0.0");
    assert_eq!(c.server.port, 8182);
    assert_eq!(c.log.level, "info");
    assert!(c.mpris.enabled);
    assert!(warns.is_empty());
}
#[test]
fn unknown_keys_warn_never_error() {
    // D14 / operator §5.4 (J5 silent-revert guard)
    let (_c, warns) = QbzdConfig::from_str("[server]\nbindd = \"0.0.0.0\"\n").unwrap();
    assert_eq!(warns, vec!["[server].bindd".to_string()]);
}
#[test]
fn server_token_defaults_none_and_parses_when_set() {
    // 02-cli-and-api.md §3.1.2: `[server] token` is opt-in — absent = None
    // (open control plane); present = the shared secret, no warning.
    let (open, warns) = QbzdConfig::from_str("").unwrap();
    assert_eq!(open.server.token, None);
    assert!(warns.is_empty());

    let (secured, warns) =
        QbzdConfig::from_str("[server]\ntoken = \"s3cret\"\n").unwrap();
    assert_eq!(secured.server.token.as_deref(), Some("s3cret"));
    assert!(warns.is_empty(), "known key must not warn: {warns:?}");
}
#[test]
fn server_token_empty_string_parses_as_present_but_filtering_gates_it() {
    // Empty or whitespace-only tokens in the config file parse successfully,
    // but are filtered to None by daemon.rs and client.rs to prevent
    // enabling auth with an empty secret.
    let (cfg, warns) = QbzdConfig::from_str("[server]\ntoken = \"\"\n").unwrap();
    assert_eq!(cfg.server.token, Some("".to_string()));
    assert!(warns.is_empty());

    let (cfg_ws, warns) =
        QbzdConfig::from_str("[server]\ntoken = \"   \"\n").unwrap();
    assert_eq!(cfg_ws.server.token, Some("   ".to_string()));
    assert!(warns.is_empty());
}
