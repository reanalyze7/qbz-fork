//! First-run row seeding and one-time legacy-default backfills for the
//! `audio_settings` table. Runs once per `open_at()` call, after the schema
//! migrations in `schema.rs`.

use crate::AudioBackendType;
use rusqlite::{params, Connection};

/// Seed the single settings row on first run with the OOTB default backend
/// ("System"). INSERT OR IGNORE is a one-time seed: it only fires when the
/// row does not exist yet, so existing installs are never rewritten on
/// later launches (settings are only reset by the explicit Reset action,
/// never just by restarting).
///
/// There is deliberately NO backfill of existing NULL backend_type rows: a
/// NULL backend_type means "Auto" and is preserved as-is. (An earlier #375
/// workaround backfilled NULL -> PipeWire on every open, which hard-required
/// `pactl` and froze OOTB playback without it, #470.)
pub(crate) fn seed_default_row(conn: &Connection) -> Result<(), String> {
    let default_backend_json = serde_json::to_string(&AudioBackendType::default())
        .map_err(|e| format!("Failed to serialize default backend: {}", e))?;
    conn.execute(
        "INSERT OR IGNORE INTO audio_settings (id, exclusive_mode, dac_passthrough, backend_type) VALUES (1, 0, 0, ?1)",
        params![default_backend_json],
    )
    .map_err(|e| format!("Failed to seed audio settings row: {}", e))?;
    Ok(())
}

/// One-time backfill (#638 Phase C / F10): installs that first ran a
/// pre-#45 build had `limit_quality_to_device` backfilled to 1 by the
/// original DEFAULT-1 migration and still read `true` today. That was
/// inert while nothing consumed the flag; now that the local device cap
/// (fix 3) consumes it, those installs would silently gain a cap nobody
/// asked for on upgrade — a silently-appearing cap is the exact bug
/// class this work removes. Reset the flag to the modern default ONCE,
/// gated on `user_version`, so a user who deliberately re-enables it
/// afterwards is never clobbered again. The stamp is only written after
/// a successful UPDATE so a failed backfill retries on the next open.
pub(crate) fn backfill_legacy_defaults(conn: &Connection) -> Result<(), String> {
    let user_version: i64 = conn
        .query_row("PRAGMA user_version", [], |row| row.get(0))
        .unwrap_or(0);
    if user_version < 1 {
        match conn.execute(
            "UPDATE audio_settings SET limit_quality_to_device = 0 WHERE id = 1",
            [],
        ) {
            Ok(_) => {
                if let Err(e) = conn.pragma_update(None, "user_version", 1) {
                    log::warn!("audio settings: user_version stamp failed: {e}");
                }
            }
            Err(e) => {
                log::warn!("audio settings: limit_quality_to_device backfill failed: {e}")
            }
        }
    }
    Ok(())
}
