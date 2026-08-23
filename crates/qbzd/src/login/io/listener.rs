use std::io::{Read, Write};
use std::net::TcpListener;
use std::time::{Duration, Instant};

use crate::login::error::LoginError;
use crate::login::parsing::parse_callback;

/// FB6: bind the one-shot login listener WIDE by default, on an EPHEMERAL
/// port — the LAN-first posture applies to the login listener too, so the
/// OAuth redirect lands regardless of which address on the box is actually
/// reachable. This is independent of `redirect_host` (the resolved
/// `--callback-host` > `SSH_CONNECTION` > `127.0.0.1` value embedded in the
/// URL, unchanged): the listener no longer binds that specific address.
///
///   - Family-aware only: an explicit IPv6 `redirect_host` binds the IPv6
///     wildcard `::` (so a bracketed IPv6 redirect URL still lands — a v4
///     wildcard under a v6 URL would guarantee a 300 s timeout).
///   - Everything else — loopback, a v4 LAN IP, or non-IP input (a hostname
///     in `--callback-host`, never handed to `TcpListener::bind` to avoid a
///     blocking DNS lookup) — binds the IPv4 wildcard `0.0.0.0`.
pub(crate) fn bind_login_listener(redirect_host: &str) -> Result<TcpListener, LoginError> {
    use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
    let fail = |e: std::io::Error| {
        LoginError::Failed(format!("could not bind the login listener: {e}"))
    };
    let wildcard: IpAddr = match redirect_host.parse::<IpAddr>() {
        Ok(IpAddr::V6(_)) => Ipv6Addr::UNSPECIFIED.into(),
        _ => Ipv4Addr::UNSPECIFIED.into(),
    };
    TcpListener::bind((wildcard, 0)).map_err(fail)
}

const SUCCESS_HTML: &str = "<html><body style=\"font-family:system-ui;text-align:center;padding:64px;background:#0f0f0f;color:#fff\">\
<h2>Login successful</h2><p>You can close this tab and return to your terminal.</p></body></html>";
const WAITING_HTML: &str = "<html><body style=\"font-family:system-ui;text-align:center;padding:64px;background:#0f0f0f;color:#fff\">\
<h2>Waiting for Qobuz…</h2></body></html>";

/// Accept connections until one carries a nonce-valid authorization code, then
/// return it and stop (exactly one accepted). Browser noise and nonce-mismatched
/// requests are answered with a neutral page and skipped. Non-blocking with a
/// 100 ms poll so the deadline is honored without a background thread leak — this
/// runs inside `spawn_blocking` and self-terminates at `deadline`.
pub(crate) fn capture_callback(
    listener: TcpListener,
    expected_nonce: &str,
    deadline: Instant,
) -> std::io::Result<Option<String>> {
    listener.set_nonblocking(true)?;
    loop {
        if Instant::now() >= deadline {
            return Ok(None);
        }
        match listener.accept() {
            Ok((mut stream, _)) => {
                stream.set_nonblocking(false).ok();
                stream.set_read_timeout(Some(Duration::from_secs(2))).ok();
                let mut buf = [0u8; 8192];
                let n = stream.read(&mut buf).unwrap_or(0);
                let request = String::from_utf8_lossy(&buf[..n]);
                let request_line = request.lines().next().unwrap_or("");
                let code = parse_callback(request_line, expected_nonce);

                let body = if code.is_some() { SUCCESS_HTML } else { WAITING_HTML };
                let response = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\n\
                     Content-Length: {}\r\nConnection: close\r\n\r\n{}",
                    body.len(),
                    body,
                );
                let _ = stream.write_all(response.as_bytes());
                let _ = stream.flush();

                if code.is_some() {
                    return Ok(code);
                }
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                std::thread::sleep(Duration::from_millis(100));
            }
            Err(e) => return Err(e),
        }
    }
}
