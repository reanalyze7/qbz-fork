//! Table creation and forward-only column migrations for the
//! `audio_settings` SQLite table.
//!
//! COUPLING: the column list here (the `ALTER TABLE ... ADD COLUMN` order)
//! is positionally coupled to the `SELECT` column list and `row.get(N)`
//! indices in `store_get.rs::get_settings()`. Do not reorder either list
//! independently of the other.

use rusqlite::Connection;

/// Create the `audio_settings` table if it doesn't exist yet, then apply
/// any migrations (new columns) needed for existing databases.
pub(crate) fn create_and_migrate_schema(conn: &Connection) -> Result<(), String> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS audio_settings (
            id INTEGER PRIMARY KEY CHECK (id = 1),
            output_device TEXT,
            exclusive_mode INTEGER NOT NULL DEFAULT 0,
            dac_passthrough INTEGER NOT NULL DEFAULT 0,
            preferred_sample_rate INTEGER,
            backend_type TEXT,
            alsa_plugin TEXT,
            alsa_hardware_volume INTEGER NOT NULL DEFAULT 0,
            stream_first_track INTEGER NOT NULL DEFAULT 1,
            stream_buffer_seconds INTEGER NOT NULL DEFAULT 2
        );",
    )
    .map_err(|e| format!("Failed to create audio settings table: {}", e))?;

    // Migration: Add new columns if they don't exist (for existing databases)
    let _ = conn.execute(
        "ALTER TABLE audio_settings ADD COLUMN backend_type TEXT",
        [],
    );
    let _ = conn.execute("ALTER TABLE audio_settings ADD COLUMN alsa_plugin TEXT", []);
    let _ = conn.execute(
        "ALTER TABLE audio_settings ADD COLUMN alsa_hardware_volume INTEGER DEFAULT 0",
        [],
    );
    let _ = conn.execute(
        "ALTER TABLE audio_settings ADD COLUMN stream_first_track INTEGER DEFAULT 1",
        [],
    );
    let _ = conn.execute(
        "ALTER TABLE audio_settings ADD COLUMN stream_buffer_seconds INTEGER DEFAULT 2",
        [],
    );
    let _ = conn.execute(
        "ALTER TABLE audio_settings ADD COLUMN streaming_only INTEGER DEFAULT 0",
        [],
    );
    let _ = conn.execute(
        "ALTER TABLE audio_settings ADD COLUMN limit_quality_to_device INTEGER DEFAULT 0",
        [],
    );
    let _ = conn.execute(
        "ALTER TABLE audio_settings ADD COLUMN device_max_sample_rate INTEGER",
        [],
    );
    let _ = conn.execute(
        "ALTER TABLE audio_settings ADD COLUMN normalization_enabled INTEGER DEFAULT 0",
        [],
    );
    let _ = conn.execute(
        "ALTER TABLE audio_settings ADD COLUMN normalization_target_lufs REAL DEFAULT -14.0",
        [],
    );
    let _ = conn.execute(
        "ALTER TABLE audio_settings ADD COLUMN gapless_enabled INTEGER DEFAULT 0",
        [],
    );
    let _ = conn.execute(
        "ALTER TABLE audio_settings ADD COLUMN device_sample_rate_limits TEXT",
        [],
    );
    let _ = conn.execute(
        "ALTER TABLE audio_settings ADD COLUMN pw_force_bitperfect INTEGER DEFAULT 0",
        [],
    );
    let _ = conn.execute(
        "ALTER TABLE audio_settings ADD COLUMN sync_audio_on_startup INTEGER DEFAULT 0",
        [],
    );
    let _ = conn.execute(
        "ALTER TABLE audio_settings ADD COLUMN quality_fallback_behavior TEXT DEFAULT 'ask'",
        [],
    );
    let _ = conn.execute(
        "ALTER TABLE audio_settings ADD COLUMN skip_sink_switch INTEGER DEFAULT 0",
        [],
    );
    let _ = conn.execute(
        "ALTER TABLE audio_settings ADD COLUMN allow_quality_fallback INTEGER DEFAULT 0",
        [],
    );
    let _ = conn.execute(
        "ALTER TABLE audio_settings ADD COLUMN reserve_dac_while_running INTEGER DEFAULT 0",
        [],
    );
    let _ = conn.execute(
        "ALTER TABLE audio_settings ADD COLUMN dsd_mode TEXT DEFAULT 'convert'",
        [],
    );
    let _ = conn.execute(
        "ALTER TABLE audio_settings ADD COLUMN crossfade_seconds REAL DEFAULT 0",
        [],
    );

    Ok(())
}
