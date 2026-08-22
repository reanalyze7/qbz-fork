//! `AlsaDirectStream::write` (i16 source).

use super::helpers::handle_write;
use crate::alsa_direct::AlsaDirectStream;
use alsa::pcm::Format;

impl AlsaDirectStream {
    /// Write audio samples to ALSA (auto-converts from i16 based on detected format)
    pub fn write(&self, samples_i16: &[i16]) -> Result<(), String> {
        let pcm = self.pcm.lock().unwrap();
        let frames = samples_i16.len() / self.channels as usize;

        match self.format {
            Format::FloatLE => {
                // Convert i16 to f32
                let samples_f32: Vec<f32> =
                    samples_i16.iter().map(|&s| s as f32 / 32768.0).collect();
                let io = pcm
                    .io_f32()
                    .map_err(|e| format!("Failed to get PCM I/O: {}", e))?;
                handle_write(io.writei(&samples_f32), frames, &pcm, "")
            }
            Format::S32LE => {
                // Convert i16 to i32 (bit-perfect: shift left 16 bits)
                let samples_i32: Vec<i32> = samples_i16.iter().map(|&s| (s as i32) << 16).collect();
                let io = pcm
                    .io_i32()
                    .map_err(|e| format!("Failed to get PCM I/O: {}", e))?;
                handle_write(io.writei(&samples_i32), frames, &pcm, "")
            }
            Format::S16LE => {
                // Direct write (no conversion needed)
                let io = pcm
                    .io_i16()
                    .map_err(|e| format!("Failed to get PCM I/O: {}", e))?;
                handle_write(io.writei(samples_i16), frames, &pcm, "")
            }
            Format::S243LE => {
                // S24_3LE: 24-bit packed in 3 bytes, little-endian
                // Required by SMSL-class USB DACs (TAS1020B chip)
                // Convert i16 → i24: shift left 8 bits, then pack into 3 bytes
                let mut bytes: Vec<u8> = Vec::with_capacity(samples_i16.len() * 3);
                for &sample in samples_i16 {
                    // Convert i16 to i24 (lossless: zeros in lower 8 bits)
                    let s24 = (sample as i32) << 8;
                    // Pack as 3 bytes in little-endian order
                    bytes.push((s24 & 0xFF) as u8); // LSB
                    bytes.push(((s24 >> 8) & 0xFF) as u8); // Middle
                    bytes.push(((s24 >> 16) & 0xFF) as u8); // MSB (sign-extended)
                }
                // Use raw byte I/O for 3-byte packed format
                let io = pcm.io_bytes();
                handle_write(io.writei(&bytes), frames, &pcm, "(S24_3LE)")
            }
            Format::S24LE => {
                // S24LE: 24-bit in 32-bit container (padded)
                // Convert i16 → i32, shift left 16 bits (same as S32LE for i16 source)
                let samples_i32: Vec<i32> = samples_i16.iter().map(|&s| (s as i32) << 16).collect();
                let io = pcm
                    .io_i32()
                    .map_err(|e| format!("Failed to get PCM I/O: {}", e))?;
                handle_write(io.writei(&samples_i32), frames, &pcm, "")
            }
            _ => Err(format!("Unsupported format: {:?}", self.format)),
        }
    }
}
