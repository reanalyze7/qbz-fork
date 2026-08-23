use std::net::SocketAddr;
use std::sync::Arc;

use tiny_http::Method;

use super::gate::access_gate;
use super::router::route;
use super::routes_table::{P0_ROUTES, P1_ROUTES};
use super::state::{ApiHandle, ApiState, BindError, BoundServer};
use super::sse;

/// Boot step 5 (01 §8.1): bind only — stateless, so the foreign-occupant
/// diagnosis (in `daemon.rs`) runs BEFORE stores (6) and runtime composition (7).
pub fn bind(addr: SocketAddr) -> Result<BoundServer, BindError> {
    match tiny_http::Server::http(addr) {
        Ok(server) => Ok(BoundServer {
            server: Arc::new(server),
        }),
        Err(e) => Err(classify_bind_error(e, addr)),
    }
}

fn classify_bind_error(
    e: Box<dyn std::error::Error + Send + Sync + 'static>,
    addr: SocketAddr,
) -> BindError {
    if let Some(io) = e.downcast_ref::<std::io::Error>() {
        if io.kind() == std::io::ErrorKind::AddrInUse {
            return BindError::AddrInUse(addr);
        }
    }
    BindError::Other(e.to_string())
}

/// Best-effort occupant probe for the step-5 diagnosis: `GET /api/ping` and
/// check the response identifies as qbzd (`"app":"qbzd"`). Loopback, short
/// timeout, no dependency on reqwest (the CLI's async client is not built here).
pub fn probe_is_qbzd(addr: SocketAddr) -> bool {
    use std::io::{Read, Write};
    use std::net::TcpStream;
    use std::time::Duration;
    let mut stream = match TcpStream::connect_timeout(&addr, Duration::from_millis(500)) {
        Ok(s) => s,
        Err(_) => return false,
    };
    let _ = stream.set_read_timeout(Some(Duration::from_secs(1)));
    let _ = stream.set_write_timeout(Some(Duration::from_secs(1)));
    let req = format!(
        "GET /api/ping HTTP/1.1\r\nHost: {addr}\r\nConnection: close\r\n\r\n"
    );
    if stream.write_all(req.as_bytes()).is_err() {
        return false;
    }
    let _ = stream.flush();
    let mut buf = Vec::new();
    let _ = stream.take(4096).read_to_end(&mut buf);
    let text = String::from_utf8_lossy(&buf);
    text.contains("\"app\":\"qbzd\"")
}

/// Boot step 11 (01 §8.1): start serving on the already-bound socket. Requests
/// are handled inline on one thread (serialized). `unblock` (from the handle)
/// ends `incoming_requests` for graceful shutdown.
pub fn serve(server: BoundServer, state: ApiState) -> ApiHandle {
    // The counted P0 surface — logged at boot so an operator sees exactly how
    // many routes this build exposes (and it anchors the const to production).
    log::info!(
        "control plane serving {} route(s) ({} P0 + {} P1)",
        P0_ROUTES.len() + P1_ROUTES.len(),
        P0_ROUTES.len(),
        P1_ROUTES.len()
    );
    let srv = server.server;
    let srv_handle = srv.clone();
    let thread = std::thread::Builder::new()
        .name("qbzd-api".into())
        .spawn(move || {
            for mut req in srv.incoming_requests() {
                // `/api/events` is a long-lived SSE stream: it would block this
                // single serving thread forever. Move it onto its OWN thread
                // (Request is Send) so the control plane keeps answering. The
                // origin/token gate is applied first, identically to `route`.
                let is_events = *req.method() == Method::Get
                    && req.url().split('?').next() == Some("/api/events");
                if is_events {
                    let has_origin = req.headers().iter().any(|h| h.field.equiv("Origin"));
                    let auth = req
                        .headers()
                        .iter()
                        .find(|h| h.field.equiv("Authorization"))
                        .map(|h| h.value.as_str().to_owned());
                    if let Some(reject) =
                        access_gate(has_origin, "GET", "/api/events", auth.as_deref(), state.token.as_deref())
                    {
                        let _ = req.respond(reject.response());
                        continue;
                    }
                    let rx = state.bus.subscribe();
                    std::thread::Builder::new()
                        .name("qbzd-sse".into())
                        .spawn(move || sse::stream(req, rx))
                        .ok();
                    continue;
                }
                let resp = route(&state, &mut req);
                let _ = req.respond(resp);
            }
        })
        .expect("failed to spawn qbzd api thread");
    ApiHandle {
        server: srv_handle,
        thread: Some(thread),
    }
}
