//! One-shot local HTTP listener that captures the OAuth redirect code.

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

/// Accept connections until one carries the OAuth code, replying with a
/// minimal success page. Browser noise (favicon requests, etc.) is answered
/// and skipped.
pub(super) async fn capture_oauth_code(listener: TcpListener) -> Option<String> {
    loop {
        let (mut stream, _) = listener.accept().await.ok()?;
        let mut buf = [0u8; 8192];
        let n = stream.read(&mut buf).await.ok()?;
        let request = String::from_utf8_lossy(&buf[..n]);
        let target = request
            .lines()
            .next()
            .and_then(|line| line.split_whitespace().nth(1))
            .unwrap_or("");
        let code = query_param(target, "code_autorisation")
            .or_else(|| query_param(target, "code"));

        let body = if code.is_some() {
            "<html><body style=\"font-family:system-ui;text-align:center;padding:64px;background:#0f0f0f;color:#fff\">\
             <h2>Login successful</h2><p>You can close this tab and return to QBZ.</p></body></html>"
        } else {
            "<html><body style=\"font-family:system-ui;text-align:center;padding:64px;background:#0f0f0f;color:#fff\">\
             <h2>Waiting for Qobuz...</h2></body></html>"
        };
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            body.len(),
            body,
        );
        let _ = stream.write_all(response.as_bytes()).await;
        let _ = stream.flush().await;

        if code.is_some() {
            return code;
        }
    }
}

/// Extract and percent-decode a query parameter from an HTTP request target.
fn query_param(target: &str, key: &str) -> Option<String> {
    let query = target.split_once('?')?.1;
    for pair in query.split('&') {
        let Some((k, v)) = pair.split_once('=') else {
            continue;
        };
        if k == key {
            return urlencoding::decode(v).ok().map(|s| s.into_owned());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::query_param;

    #[test]
    fn query_param_extracts_and_decodes() {
        assert_eq!(
            query_param("/?code_autorisation=abc123", "code_autorisation"),
            Some("abc123".to_string())
        );
        assert_eq!(
            query_param("/?a=1&code=x%2Fy", "code"),
            Some("x/y".to_string())
        );
        assert_eq!(query_param("/favicon.ico", "code"), None);
        assert_eq!(query_param("/?other=1", "code"), None);
    }
}
