//! Blocking `Read`/`Seek`/`MediaSource` trait impls for `BufferedMediaSource`
//! — the highest-risk, most-subtle code in this module (condvar-blocked
//! reads and seeks against a growing buffer).

use std::io::{Error as IoError, ErrorKind, Read, Result as IoResult, Seek, SeekFrom};

use super::buffer::BufferedMediaSource;

impl Read for BufferedMediaSource {
    fn read(&mut self, buf: &mut [u8]) -> IoResult<usize> {
        use std::sync::atomic::Ordering;

        let (lock, cvar) = &*self.state;
        let mut state = lock
            .lock()
            .map_err(|_| IoError::new(ErrorKind::Other, "Failed to acquire buffer lock"))?;

        let read_pos = self.read_pos.load(Ordering::SeqCst) as usize;

        // Wait for data if we're ahead of buffer
        while read_pos >= state.data.len()
            && !state.download_complete
            && state.download_error.is_none()
        {
            state = cvar
                .wait(state)
                .map_err(|_| IoError::new(ErrorKind::Other, "Condition variable wait failed"))?;
        }

        // Check for errors
        if let Some(ref err) = state.download_error {
            return Err(IoError::new(ErrorKind::Other, err.clone()));
        }

        // EOF if at end and download complete
        if read_pos >= state.data.len() && state.download_complete {
            return Ok(0);
        }

        // Read available data
        let available = state.data.len() - read_pos;
        let to_read = buf.len().min(available);
        buf[..to_read].copy_from_slice(&state.data[read_pos..read_pos + to_read]);
        self.read_pos
            .store((read_pos + to_read) as u64, Ordering::SeqCst);

        Ok(to_read)
    }
}

impl Seek for BufferedMediaSource {
    fn seek(&mut self, pos: SeekFrom) -> IoResult<u64> {
        use std::sync::atomic::Ordering;

        let (lock, cvar) = &*self.state;
        let mut state = lock
            .lock()
            .map_err(|_| IoError::new(ErrorKind::Other, "Failed to acquire buffer lock"))?;

        let current_pos = self.read_pos.load(Ordering::SeqCst) as i64;

        let new_pos = match pos {
            SeekFrom::Start(offset) => offset as i64,
            SeekFrom::Current(offset) => current_pos + offset,
            SeekFrom::End(offset) => {
                // For End seeks, we need to know total size or have complete download
                if let Some(total) = state.total_size {
                    total as i64 + offset
                } else if state.download_complete {
                    state.data.len() as i64 + offset
                } else {
                    // Can't seek from end without knowing size
                    return Err(IoError::new(
                        ErrorKind::Unsupported,
                        "Cannot seek from end while streaming without known size",
                    ));
                }
            }
        };

        if new_pos < 0 {
            return Err(IoError::new(
                ErrorKind::InvalidInput,
                "Seek position before start of stream",
            ));
        }

        let new_pos_usize = new_pos as usize;

        // If seeking forward beyond buffer, wait for data
        while new_pos_usize > state.data.len()
            && !state.download_complete
            && state.download_error.is_none()
        {
            state = cvar
                .wait(state)
                .map_err(|_| IoError::new(ErrorKind::Other, "Condition variable wait failed"))?;
        }

        if let Some(ref err) = state.download_error {
            return Err(IoError::new(ErrorKind::Other, err.clone()));
        }

        // After download complete, check bounds
        if state.download_complete && new_pos_usize > state.data.len() {
            return Err(IoError::new(
                ErrorKind::InvalidInput,
                "Seek position beyond end of stream",
            ));
        }

        self.read_pos.store(new_pos as u64, Ordering::SeqCst);
        Ok(new_pos as u64)
    }
}

