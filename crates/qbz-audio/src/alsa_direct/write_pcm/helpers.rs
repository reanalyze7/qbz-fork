//! Shared "check written == frames, else recover" tail used by every format
//! arm of `write`/`write_f32`. Factored out purely to keep each format's
//! per-arm code short — behavior (log text, recovery call) is unchanged from
//! the original inlined match arms.

use crate::alsa_direct::recovery::recover_write_error;
use alsa::pcm::PCM;

/// Finish a `writei()` call: warn on a partial write, then hand any error to
/// `recover_write_error`. `suffix` is appended to the partial-write log line
/// and passed straight through as the recovery-log suffix (e.g. `"(S24_3LE)"`
/// or `""`), matching the original per-format inline arms exactly.
pub(super) fn handle_write(
    io_result: Result<usize, alsa::Error>,
    frames: usize,
    pcm: &PCM,
    suffix: &str,
) -> Result<(), String> {
    match io_result {
        Ok(written) => {
            if written != frames {
                if suffix.is_empty() {
                    log::warn!("[ALSA Direct] Partial write: {} / {} frames", written, frames);
                } else {
                    log::warn!(
                        "[ALSA Direct] Partial write: {} / {} frames {}",
                        written,
                        frames,
                        suffix
                    );
                }
            }
            Ok(())
        }
        Err(e) => {
            if let Err(msg) = recover_write_error(pcm, e.errno() as i32, suffix) {
                Err(msg)
            } else {
                Ok(())
            }
        }
    }
}
