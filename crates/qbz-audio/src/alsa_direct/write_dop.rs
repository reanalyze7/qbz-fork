//! `AlsaDirectStream::write_dop_i32`.

use super::recovery::recover_write_error;
use super::AlsaDirectStream;
use alsa::pcm::Format;

impl AlsaDirectStream {
    /// Write pre-packed 32-bit direct words (DoP frames in S32, or native
    /// DSD_U32 words) VERBATIM — no scaling, no conversion. Only valid on
    /// streams created with [`Self::new_dop`] / [`Self::new_native_dsd`].
    /// DSD_U32 formats fail alsa-rs's checked-format i32 IO, so they use the
    /// unchecked accessor — sound because both layouts are exactly 32 bits
    /// per channel per frame, same as S32.
    pub fn write_dop_i32(&self, samples: &[i32]) -> Result<(), String> {
        let pcm = self.pcm.lock().unwrap();
        let frames = samples.len() / self.channels as usize;
        let io = if self.format == Format::S32LE {
            pcm.io_i32()
                .map_err(|e| format!("Failed to get PCM I/O: {}", e))?
        } else {
            unsafe { pcm.io_unchecked::<i32>() }
        };
        match io.writei(samples) {
            Ok(written) => {
                if written != frames {
                    log::warn!("[ALSA Direct] Partial DoP write: {} / {} frames", written, frames);
                }
                Ok(())
            }
            Err(e) => {
                if let Err(msg) = recover_write_error(&pcm, e.errno() as i32, "(DoP)") {
                    Err(msg)
                } else {
                    Ok(())
                }
            }
        }
    }
}
