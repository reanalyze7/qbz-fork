use super::*;

/// Surfaces a DSD stream's mid-file demux I/O error as a stream error when
/// the stream runs out. The DoP writer consumes these streams as boxed
/// `Iterator<Item = i32>`, which can only say "no more words", so without
/// this wrapper a read failure is indistinguishable from a clean end of
/// track and the user gets a silent early stop.
#[cfg_attr(not(target_os = "linux"), allow(dead_code))]
pub(crate) struct DsdErrorReport<S: qbz_dsd::DsdWordSource> {
    inner: S,
    // SharedState is a #[derive(Clone)] handle of Arc<Atomic..> fields, so a
    // clone already shares the same state — no extra Arc wrapper (that is also
    // the type `thread_state` is passed around as at the call sites).
    state: SharedState,
    reported: bool,
}

#[cfg_attr(not(target_os = "linux"), allow(dead_code))]
impl<S: qbz_dsd::DsdWordSource> DsdErrorReport<S> {
    pub(crate) fn new(inner: S, state: SharedState) -> Self {
        Self {
            inner,
            state,
            reported: false,
        }
    }
}

impl<S: qbz_dsd::DsdWordSource> Iterator for DsdErrorReport<S> {
    type Item = i32;
    fn next(&mut self) -> Option<i32> {
        let item = self.inner.next();
        if item.is_none() && !self.reported {
            self.reported = true;
            if let Some(e) = self.inner.io_error() {
                self.state
                    .record_stream_error(format!("DSD playback failed: {e}"));
            }
        }
        item
    }
}
