// crates/qbzd/src/api/sse/ — the `GET /api/events` Server-Sent Events stream
// (CONSOLE ext). A live push feed of CoreEvents so a client (a plasmoid, a bar
// applet, `qbzd watch`) reacts to playback/queue/library changes without
// polling.
//
// Concurrency: the control-plane serve loop is single-threaded, so an open SSE
// stream would starve every other request. `serve()` therefore moves this onto
// its OWN thread (Request is Send); `stream` blocks there until the client
// disconnects (respond() errors) or the bus closes. The rusqlite-free bus is a
// tokio broadcast, drained here with `blocking_recv()` from the plain thread.
//
// Wire format: one SSE frame per emitted event —
//   event: <CoreEvent type>\n
//   data: {"type":"…","data":{…}}\n\n
// The `data` line is the CoreEvent's own `#[serde(tag="type",content="data")]`
// JSON (single-line). A leading `: …` comment primes the stream; a dropped-lag
// notice is sent as a comment rather than silently losing ordering.
mod format;
mod reader;

use tiny_http::{Header, Request, Response, StatusCode};
use tokio::sync::broadcast;

use qbz_models::CoreEvent;

use reader::SseReader;

/// Stream CoreEvents to one client until it disconnects. Runs on a dedicated
/// thread; `req.respond` blocks here, writing chunked as `SseReader` yields.
pub fn stream(req: Request, rx: broadcast::Receiver<CoreEvent>) {
    let headers = vec![
        header("Content-Type", "text/event-stream"),
        header("Cache-Control", "no-cache"),
        header("Connection", "keep-alive"),
        // Defeat proxy/response buffering so frames flush immediately.
        header("X-Accel-Buffering", "no"),
    ];
    let response = Response::new(StatusCode(200), headers, SseReader::new(rx), None, None);
    let _ = req.respond(response);
}

fn header(name: &str, value: &str) -> Header {
    // Both are static ASCII — construction cannot fail.
    Header::from_bytes(name.as_bytes(), value.as_bytes()).expect("static header")
}
