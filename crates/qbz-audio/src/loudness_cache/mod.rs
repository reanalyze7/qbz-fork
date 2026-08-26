//! Cache de loudness — persiste les mesures EBU R128 dans SQLite.
//!
//! Suit le pattern `AudioSettingsStore` : la base vit dans
//! `dirs::data_dir()/qbz/loudness_cache.db`. Thread-safe via `Mutex<Connection>`.
//!
//! # Schema v2 : on stocke le LUFS MESURE, pas un ecart en dB
//!
//! La v1 stockait `gain_db = cible - mesure`, sans memoriser la cible. Changer
//! le LUFS cible dans les reglages rendait donc toutes les entrees fausses du
//! delta, silencieusement. En stockant la mesure, le gain est recalcule a la
//! lecture pour la cible courante. La table v1 est supprimee a l'ouverture :
//! ses valeurs sont irrecuperables (cible inconnue) et beaucoup etaient de
//! toute facon fausses — mesurees sur la fin du morceau precedent.

mod store;

#[cfg(test)]
mod tests;

use rusqlite::Connection;
use std::sync::Mutex;

use crate::loudness::gain::gain_db_for;

/// Mesure de loudness d'une piste, telle que persistee.
#[derive(Debug, Clone)]
pub struct CachedLoudness {
    /// Loudness integree EBU R128 du morceau, en LUFS.
    pub measured_lufs: f32,
    /// Pic echantillon (0.0-1.0+), 0.0 si inconnu.
    pub peak: f32,
    /// Origine de la mesure : "ebur128", "ebur128-offline" ou "replaygain".
    pub source: String,
}

impl CachedLoudness {
    /// Ecart en dB a appliquer pour atteindre `target_lufs` (borne).
    pub fn gain_db(&self, target_lufs: f32) -> f32 {
        gain_db_for(self.measured_lufs, target_lufs)
    }
}

pub struct LoudnessCache {
    conn: Mutex<Connection>,
}

impl LoudnessCache {
    pub fn new() -> Result<Self, String> {
        let data_dir = dirs::data_dir()
            .ok_or_else(|| "Could not determine data directory".to_string())?
            .join("qbz");

        std::fs::create_dir_all(&data_dir)
            .map_err(|e| format!("Failed to create data directory: {}", e))?;

        let db_path = data_dir.join("loudness_cache.db");
        let conn = Connection::open(&db_path)
            .map_err(|e| format!("Failed to open loudness cache database: {}", e))?;

        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL;")
            .map_err(|e| format!("Failed to enable WAL for loudness cache database: {}", e))?;

        Self::init_schema(&conn)?;

        log::info!("[LoudnessCache] Opened at {}", db_path.display());

        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    /// Cache en memoire, pour les tests.
    #[cfg(test)]
    pub fn in_memory() -> Result<Self, String> {
        let conn = Connection::open_in_memory().map_err(|e| e.to_string())?;
        Self::init_schema(&conn)?;
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    fn init_schema(conn: &Connection) -> Result<(), String> {
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS track_loudness_v2 (
                track_id INTEGER PRIMARY KEY,
                measured_lufs REAL NOT NULL,
                peak REAL NOT NULL DEFAULT 0.0,
                source TEXT NOT NULL DEFAULT 'ebur128',
                created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
            );
            DROP TABLE IF EXISTS track_loudness;",
        )
        .map_err(|e| format!("Failed to create loudness table: {}", e))
    }
}
