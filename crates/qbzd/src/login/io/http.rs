use std::io::{Read, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::time::Duration;

pub(crate) fn http_request_2xx(host: &str, method: &str, path: &str, token: Option<&str>) -> bool {
    let addr = match host.to_socket_addrs().ok().and_then(|mut a| a.next()) {
        Some(a) => a,
        None => return false,
    };
    let mut stream = match TcpStream::connect_timeout(&addr, Duration::from_millis(600)) {
        Ok(s) => s,
        Err(_) => return false,
    };
    let _ = stream.set_read_timeout(Some(Duration::from_secs(2)));
    let _ = stream.set_write_timeout(Some(Duration::from_secs(2)));
    let auth = token
        .map(|t| format!("Authorization: Bearer {t}\r\n"))
        .unwrap_or_default();
    let req = format!(
        "{method} {path} HTTP/1.1\r\nHost: {host}\r\n{auth}Content-Length: 0\r\nConnection: close\r\n\r\n"
    );
    if stream.write_all(req.as_bytes()).is_err() {
        return false;
    }
    let _ = stream.flush();
    let mut buf = [0u8; 128];
    let n = stream.read(&mut buf).unwrap_or(0);
    let status = String::from_utf8_lossy(&buf[..n]);
    matches!(
        status.lines().next().and_then(|l| l.split_whitespace().nth(1)),
        Some(code) if code.starts_with('2')
    )
}
