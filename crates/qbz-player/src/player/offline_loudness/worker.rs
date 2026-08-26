use std::sync::atomic::Ordering;
use std::sync::mpsc::Receiver;
use std::sync::Arc;
use std::time::Instant;

use qbz_audio::loudness::gain::gain_factor_for;
use qbz_audio::{LoudnessCache, LoudnessMeter};
use rodio::Source;

use super::OfflineJob;
use crate::player::decode::decode_with_fallback;

/// Echantillons pousses d'un coup dans le meter (multiple du nb de canaux).
const CHUNK_FRAMES: usize = 4096;

pub(super) fn run(rx: Receiver<OfflineJob>, cache: Arc<LoudnessCache>) {
    while let Ok(job) = rx.recv() {
        if cache.has(job.track_id) {
            continue;
        }
        let started_at = Instant::now();
        let Some((lufs, peak)) = measure(&job.data) else {
            log::warn!(
                "[OfflineLoudness] Piste {} non mesurable hors-ligne",
                job.track_id
            );
            continue;
        };
        cache.set(job.track_id, lufs, peak, "ebur128-offline");

        let applied = job.should_apply();
        if applied {
            if let Some(ref atomic) = job.gain_atomic {
                let gain = gain_factor_for(lufs, job.target_lufs);
                atomic.store(gain.to_bits(), Ordering::Relaxed);
            }
        }

        log::info!(
            "[OfflineLoudness] Piste {} mesuree en {:.1}s : {:.1} LUFS, pic {:.3}{}",
            job.track_id,
            started_at.elapsed().as_secs_f32(),
            lufs,
            peak,
            if applied {
                " — gain pose avant la premiere note"
            } else {
                " — mis en cache"
            }
        );
    }
}

/// Decode le morceau entier et renvoie (LUFS integres, pic).
fn measure(data: &[u8]) -> Option<(f32, f32)> {
    let source = decode_with_fallback(data).ok()?;
    let sample_rate = source.sample_rate().get();
    let channels = source.channels().get();
    let mut meter = LoudnessMeter::new(sample_rate, channels)?;

    let chunk = CHUNK_FRAMES * channels.max(1) as usize;
    let mut buf: Vec<f32> = Vec::with_capacity(chunk);
    for sample in source {
        buf.push(sample);
        if buf.len() >= chunk {
            if !meter.feed(&buf) {
                return None;
            }
            buf.clear();
        }
    }
    if !buf.is_empty() {
        let keep = buf.len() - buf.len() % channels.max(1) as usize;
        meter.feed(&buf[..keep]);
    }

    Some((meter.integrated_lufs()?, meter.peak()))
}
