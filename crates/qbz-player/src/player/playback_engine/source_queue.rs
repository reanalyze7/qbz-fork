//! Thread-safe source queue for gapless playback, shared by all three
//! thread-based backends (ALSA Direct, JACK, DoP) with different item types
//! (`BoxedSampleIter` for ALSA/JACK, `BoxedDopIter` for DoP).

use std::collections::VecDeque;
use std::sync::{Condvar, Mutex};
use std::time::Duration;

/// The writer thread consumes sources; append() pushes new ones.
pub(crate) struct SourceQueue<S> {
    queue: Mutex<VecDeque<S>>,
    /// Notifies the writer thread that a new source is available
    notify: Condvar,
}

impl<S> SourceQueue<S> {
    pub(super) fn new() -> Self {
        Self {
            queue: Mutex::new(VecDeque::new()),
            notify: Condvar::new(),
        }
    }

    /// Push a new source to the back of the queue
    pub(super) fn push(&self, source: S) {
        let mut q = self.queue.lock().unwrap();
        q.push_back(source);
        self.notify.notify_one();
    }

    /// Try to pop the next source (non-blocking)
    pub(super) fn try_pop(&self) -> Option<S> {
        let mut q = self.queue.lock().unwrap();
        q.pop_front()
    }

    /// Wait for a source to become available (with timeout)
    /// Returns None on timeout (used to check stop/pause flags)
    pub(super) fn wait_for_source(&self, timeout: Duration) -> Option<S> {
        let mut q = self.queue.lock().unwrap();
        if q.is_empty() {
            let (guard, _) = self.notify.wait_timeout(q, timeout).unwrap();
            q = guard;
        }
        q.pop_front()
    }

    pub(super) fn is_empty(&self) -> bool {
        self.queue.lock().unwrap().is_empty()
    }
}
