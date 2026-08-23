use super::super::*;

#[test]
fn build_oauth_url_embeds_nonce_in_the_redirect_path() {
    // Step 1(a), amended: the URL embeds
    // redirect_url=http://<host>:<port>/<nonce> — the nonce rides the PATH,
    // never a state param the provider would have to echo.
    let url = build_oauth_url("app123", "127.0.0.1", 39114, "NONCEabc");
    assert!(url.starts_with("https://www.qobuz.com/signin/oauth?"), "{url}");
    assert!(url.contains("ext_app_id=app123"), "{url}");
    let decoded = urlencoding::decode(&url).unwrap();
    assert!(
        decoded.contains("redirect_url=http://127.0.0.1:39114/NONCEabc"),
        "{decoded}"
    );
}

#[test]
fn callback_host_is_embedded_in_the_redirect() {
    // Step 1(c): --callback-host embeds that host in the redirect URL.
    let url = build_oauth_url("app123", "192.168.0.40", 40000, "n");
    let decoded = urlencoding::decode(&url).unwrap();
    assert!(
        decoded.contains("redirect_url=http://192.168.0.40:40000/n"),
        "{decoded}"
    );
}

#[test]
fn build_oauth_url_brackets_an_ipv6_host_and_keeps_the_nonce_in_the_path() {
    // FB1: a non-loopback IPv6 host (e.g. from SSH_CONNECTION) must be
    // bracketed in the URL authority; the nonce still rides the path.
    let url = build_oauth_url("app123", "2001:db8::2", 40000, "nn");
    let decoded = urlencoding::decode(&url).unwrap();
    assert!(
        decoded.contains("redirect_url=http://[2001:db8::2]:40000/nn"),
        "{decoded}"
    );
}

// ---------------------- resolve_callback_host (FB1) ----------------------

#[test]
fn resolve_callback_host_cli_flag_wins_over_everything() {
    let (host, auto) =
        resolve_callback_host(Some("10.0.0.9"), Some("192.168.1.5 22 192.168.1.1 22"));
    assert_eq!(host, "10.0.0.9");
    assert!(!auto, "an explicit --callback-host is never auto-detected");
}

#[test]
fn resolve_callback_host_reads_the_server_ip_field_from_ssh_connection() {
    // SSH_CONNECTION = "client_ip client_port server_ip server_port" —
    // the SERVER ip (3rd field) is exactly what the operator's other
    // machine used to reach this box.
    let (host, auto) = resolve_callback_host(None, Some("203.0.113.4 51820 192.168.1.50 22"));
    assert_eq!(host, "192.168.1.50");
    assert!(auto);
}

#[test]
fn resolve_callback_host_handles_ipv6_ssh_connection() {
    let (host, auto) =
        resolve_callback_host(None, Some("2001:db8::1 51820 2001:db8::2 22"));
    assert_eq!(host, "2001:db8::2");
    assert!(auto);
}

#[test]
fn resolve_callback_host_falls_through_on_malformed_ssh_connection() {
    // Too few fields — no server-ip field to read.
    let (host, auto) = resolve_callback_host(None, Some("only-two fields"));
    assert_eq!(host, "127.0.0.1");
    assert!(!auto);
}

#[test]
fn resolve_callback_host_falls_through_on_non_ip_server_field() {
    // 3rd field present but not a parseable IP — reject, don't guess.
    let (host, auto) =
        resolve_callback_host(None, Some("203.0.113.4 51820 not-an-ip 22"));
    assert_eq!(host, "127.0.0.1");
    assert!(!auto);
}

#[test]
fn resolve_callback_host_falls_through_on_empty_ssh_connection() {
    // SSH_CONNECTION set but empty — no fields at all.
    let (host, auto) = resolve_callback_host(None, Some(""));
    assert_eq!(host, "127.0.0.1");
    assert!(!auto);
}

#[test]
fn resolve_callback_host_falls_through_when_ssh_connection_absent() {
    // The local-laptop case: no flag, no SSH session — unchanged default.
    let (host, auto) = resolve_callback_host(None, None);
    assert_eq!(host, "127.0.0.1");
    assert!(!auto);
}
