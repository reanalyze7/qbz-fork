use std::io::{Cursor, Read};

use tokio::sync::broadcast;

use qbz_models::CoreEvent;

use super::format::format_event;

/// A blocking `Read` over the CoreEvent bus: each `read` serves the current
/// frame's bytes, and when a frame is exhausted it blocks for the next event.
/// `Ok(0)` (EOF) only when the bus closes (daemon shutting down).
pub(super) struct SseReader {
    rx: broadcast::Receiver<CoreEvent>,
    frame: Cursor<Vec<u8>>,
    primed: bool,
}

impl SseReader {
    pub(super) fn new(rx: broadcast::Receiver<CoreEvent>) -> Self {
        SseReader { rx, frame: Cursor::new(Vec::new()), primed: false }
    }

    /// The next frame to send, blocking for a bus event. `None` = bus closed.
    fn next_frame(&mut self) -> Option<Vec<u8>> {
        if !self.primed {
            self.primed = true;
            return Some(b": qbzd event stream\n\n".to_vec());
        }
        loop {
            match self.rx.blocking_recv() {
                Ok(ev) => {
                    if let Some(frame) = format_event(&ev) {
                        return Some(frame.into_bytes());
                    }
                    // Not an emitted event — keep waiting.
                }
                Err(broadcast::error::RecvError::Lagged(n)) => {
                    return Some(format!(": lagged {n} event(s)\n\n").into_bytes());
                }
                Err(broadcast::error::RecvError::Closed) => return None,
            }
        }
    }
}

impl Read for SseReader {
    fn read(&mut self, out: &mut [u8]) -> std::io::Result<usize> {
        loop {
            let n = self.frame.read(out)?;
            if n > 0 {
                return Ok(n);
            }
            match self.next_frame() {
                Some(bytes) => self.frame = Cursor::new(bytes),
                None => return Ok(0), // bus closed → end the stream
            }
        }
    }
}
