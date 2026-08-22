//! [`super::NativeDsdStream::refill`]: pull DSD bytes from the demuxer,
//! bit-reverse for LSB-first containers, and pack into 4-byte U32 words.

use super::NativeDsdStream;
use crate::dsd2pcm::bit_reverse;

impl NativeDsdStream {
    pub(super) fn refill(&mut self) -> bool {
        // Carry bytes are ALREADY bit-normalized (reversed on the refill that
        // produced them) — only the newly-read span gets reversed below.
        let mut planar: Vec<Vec<u8>> = vec![
            std::mem::take(&mut self.carry[0]),
            std::mem::take(&mut self.carry[1]),
        ];
        let pre = [planar[0].len(), planar[1].len()];
        let got = match self
            .demux
            .read_planar(&mut planar, super::REFILL_BYTES_PER_CH)
        {
            Ok(n) => n,
            Err(e) => {
                log::error!("[DSD/native] demux I/O error (not clean EOF): {e}");
                self.io_error = Some(e.to_string());
                self.done = true;
                return false;
            }
        };
        if self.lsb_first {
            for (i, chan) in planar.iter_mut().enumerate() {
                for b in chan[pre[i]..].iter_mut() {
                    *b = bit_reverse(*b);
                }
            }
        }
        if got == 0 {
            // EOF: pad the final partial word (if any) with DSD silence
            // (0x69 — planar is MSB-first-normalized at this point).
            let leftover = planar[0].len().min(planar[1].len());
            if leftover == 0 {
                self.done = true;
                return false;
            }
            for chan in planar.iter_mut() {
                while chan.len() % 4 != 0 {
                    chan.push(0x69);
                }
            }
            self.done = true;
        }
        let words = planar.iter().map(|c| c.len()).min().unwrap_or(0) / 4;
        self.buf.clear();
        self.idx = 0;
        self.buf.reserve(words * 2);
        for w in 0..words {
            for chan in planar.iter() {
                let b = [
                    chan[w * 4],
                    chan[w * 4 + 1],
                    chan[w * 4 + 2],
                    chan[w * 4 + 3],
                ];
                self.buf.push(self.pack_word(b));
            }
        }
        // Keep whatever didn't fill a whole word for the next refill.
        if !self.done {
            for (i, chan) in planar.iter().enumerate() {
                self.carry[i] = chan[words * 4..].to_vec();
            }
        }
        !self.buf.is_empty()
    }
}
