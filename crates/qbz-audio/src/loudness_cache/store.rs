//! Lecture / ecriture des mesures. Une mesure invraisemblable n'entre pas.

use rusqlite::params;

use super::{CachedLoudness, LoudnessCache};
use crate::loudness::gain::is_plausible_lufs;

impl LoudnessCache {
    /// Mesure cachee pour une piste, si elle existe.
    pub fn get(&self, track_id: u64) -> Option<CachedLoudness> {
        let conn = self.conn.lock().ok()?;
        conn.query_row(
            "SELECT measured_lufs, peak, source FROM track_loudness_v2 WHERE track_id = ?1",
            params![track_id as i64],
            |row| {
                Ok(CachedLoudness {
                    measured_lufs: row.get::<_, f64>(0)? as f32,
                    peak: row.get::<_, f64>(1)? as f32,
                    source: row.get(2)?,
                })
            },
        )
        .ok()
        .filter(|c| is_plausible_lufs(c.measured_lufs))
    }

    /// Enregistre (ou remplace) la mesure d'une piste.
    ///
    /// Une valeur hors plage est refusee plutot que stockee : c'est la
    /// derniere barriere avant qu'une mesure de silence ne devienne un
    /// reglage permanent pour ce morceau.
    pub fn set(&self, track_id: u64, measured_lufs: f32, peak: f32, source: &str) {
        if !is_plausible_lufs(measured_lufs) {
            log::warn!(
                "[LoudnessCache] Refus de cacher {:.1} LUFS pour la piste {} (hors plage, source {})",
                measured_lufs,
                track_id,
                source
            );
            return;
        }
        if let Ok(conn) = self.conn.lock() {
            let result = conn.execute(
                "INSERT OR REPLACE INTO track_loudness_v2
                    (track_id, measured_lufs, peak, source, created_at)
                 VALUES (?1, ?2, ?3, ?4, strftime('%s', 'now'))",
                params![track_id as i64, measured_lufs as f64, peak as f64, source],
            );
            if let Err(e) = result {
                log::warn!(
                    "[LoudnessCache] Failed to store loudness for track {}: {}",
                    track_id,
                    e
                );
            }
        }
    }

    /// Vrai si la piste a deja une mesure exploitable.
    pub fn has(&self, track_id: u64) -> bool {
        self.get(track_id).is_some()
    }
}
