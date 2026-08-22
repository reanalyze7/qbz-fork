//! `AlsaDirectStream::write_f32` (f32 source).

use crate::alsa_direct::recovery::recover_write_error;
use crate::alsa_direct::AlsaDirectStream;
use alsa::pcm::Format;

impl AlsaDirectStream {
    /// Write f32 audio samples to ALSA (converts to hardware format with full precision)
    ///
    /// f32 has 24 bits of significand, so 24-bit audio is preserved losslessly.
    /// This is the primary write path for the f32 pipeline.
    pub fn write_f32(&self, samples_f32: &[f32]) -> Result<(), String> {
        let pcm = self.pcm.lock().unwrap();
        let frames = samples_f32.len() / self.channels as usize;

        match self.format {
            Format::FloatLE => {
                // Direct write - no conversion needed
                let io = pcm
                    .io_f32()
                    .map_err(|e| format!("Failed to get PCM I/O: {}", e))?;

                match io.writei(samples_f32) {
                    Ok(written) => {
                        if written != frames {
                            log::warn!(
                                "[ALSA Direct] Partial write: {} / {} frames",
                                written,
                                frames
                            );
                        }
                        Ok(())
                    }
                    Err(e) => {
                        if let Err(msg) = recover_write_error(&pcm, e.errno() as i32, "") {
                            Err(msg)
                        } else {
                            Ok(())
                        }
                    }
                }
            }
            Format::S32LE => {
                // f32 [-1.0, 1.0] -> i32 full range
                let samples_i32: Vec<i32> = samples_f32
                    .iter()
                    .map(|&s| (s * 2_147_483_647.0) as i32)
                    .collect();

                let io = pcm
                    .io_i32()
                    .map_err(|e| format!("Failed to get PCM I/O: {}", e))?;

                match io.writei(&samples_i32) {
                    Ok(written) => {
                        if written != frames {
                            log::warn!(
                                "[ALSA Direct] Partial write: {} / {} frames",
                                written,
                                frames
                            );
                        }
                        Ok(())
                    }
                    Err(e) => {
                        if let Err(msg) = recover_write_error(&pcm, e.errno() as i32, "") {
                            Err(msg)
                        } else {
                            Ok(())
                        }
                    }
                }
            }
            Format::S24LE => {
                // f32 -> 24-bit in 32-bit container
                // Clamp to 24-bit range: [-8388608, 8388607]
                let samples_i32: Vec<i32> = samples_f32
                    .iter()
                    .map(|&s| {
                        let scaled = s * 8_388_607.0;
                        scaled.clamp(-8_388_608.0, 8_388_607.0) as i32
                    })
                    .collect();

                let io = pcm
                    .io_i32()
                    .map_err(|e| format!("Failed to get PCM I/O: {}", e))?;

                match io.writei(&samples_i32) {
                    Ok(written) => {
                        if written != frames {
                            log::warn!(
                                "[ALSA Direct] Partial write: {} / {} frames",
                                written,
                                frames
                            );
                        }
                        Ok(())
                    }
                    Err(e) => {
                        if let Err(msg) = recover_write_error(&pcm, e.errno() as i32, "") {
                            Err(msg)
                        } else {
                            Ok(())
                        }
                    }
                }
            }
            Format::S243LE => {
                // S24_3LE: 24-bit packed in 3 bytes, little-endian
                // f32 -> 24-bit integer, packed into 3 bytes
                let mut bytes: Vec<u8> = Vec::with_capacity(samples_f32.len() * 3);

                for &sample in samples_f32 {
                    let scaled = sample * 8_388_607.0;
                    let s24 = scaled.clamp(-8_388_608.0, 8_388_607.0) as i32;
                    // Pack as 3 bytes in little-endian order
                    bytes.push((s24 & 0xFF) as u8); // LSB
                    bytes.push(((s24 >> 8) & 0xFF) as u8); // Middle
                    bytes.push(((s24 >> 16) & 0xFF) as u8); // MSB (sign-extended)
                }

                let io = pcm.io_bytes();

                match io.writei(&bytes) {
                    Ok(written) => {
                        if written != frames {
                            log::warn!(
                                "[ALSA Direct] Partial write: {} / {} frames (S24_3LE)",
                                written,
                                frames
                            );
                        }
                        Ok(())
                    }
                    Err(e) => {
                        if let Err(msg) = recover_write_error(&pcm, e.errno() as i32, "(S24_3LE)") {
                            Err(msg)
                        } else {
                            Ok(())
                        }
                    }
                }
            }
            Format::S16LE => {
                // f32 -> i16
                let samples_i16: Vec<i16> =
                    samples_f32.iter().map(|&s| (s * 32_767.0) as i16).collect();

                let io = pcm
                    .io_i16()
                    .map_err(|e| format!("Failed to get PCM I/O: {}", e))?;

                match io.writei(&samples_i16) {
                    Ok(written) => {
                        if written != frames {
                            log::warn!(
                                "[ALSA Direct] Partial write: {} / {} frames",
                                written,
                                frames
                            );
                        }
                        Ok(())
                    }
                    Err(e) => {
                        if let Err(msg) = recover_write_error(&pcm, e.errno() as i32, "") {
                            Err(msg)
                        } else {
                            Ok(())
                        }
                    }
                }
            }
            _ => Err(format!("Unsupported format: {:?}", self.format)),
        }
    }
}
