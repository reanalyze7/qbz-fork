use rusqlite::Connection;

/// Additive schema migrations for `queue_tracks`/`player_state`, run every
/// time the store opens. ORDER-DEPENDENT only in the sense that each guard
/// checks for its own column; they don't depend on each other, but keep them
/// in this order since it matches the historical rollout sequence.
pub(super) fn run_migrations(conn: &Connection) {
    let has_hires: bool = conn
        .query_row(
            "SELECT COUNT(*) FROM pragma_table_info('queue_tracks') WHERE name = 'hires'",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0)
        > 0;

    if !has_hires {
        let _ = conn.execute_batch(
            "
            ALTER TABLE queue_tracks ADD COLUMN hires INTEGER NOT NULL DEFAULT 0;
            ALTER TABLE queue_tracks ADD COLUMN bit_depth INTEGER;
            ALTER TABLE queue_tracks ADD COLUMN sample_rate REAL;
            ",
        );
    }

    let has_is_local: bool = conn
        .query_row(
            "SELECT COUNT(*) FROM pragma_table_info('queue_tracks') WHERE name = 'is_local'",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0)
        > 0;

    if !has_is_local {
        let _ = conn.execute_batch(
            "
            ALTER TABLE queue_tracks ADD COLUMN is_local INTEGER NOT NULL DEFAULT 0;
            ALTER TABLE queue_tracks ADD COLUMN album_id TEXT;
            ALTER TABLE queue_tracks ADD COLUMN artist_id INTEGER;
            ",
        );
    }

    let has_source: bool = conn
        .query_row(
            "SELECT COUNT(*) FROM pragma_table_info('queue_tracks') WHERE name = 'source'",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0)
        > 0;

    if !has_source {
        let _ = conn.execute_batch(
            "
            ALTER TABLE queue_tracks ADD COLUMN source TEXT;
            ",
        );
    }

    let has_streamable: bool = conn
        .query_row(
            "SELECT COUNT(*) FROM pragma_table_info('queue_tracks') WHERE name = 'streamable'",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0)
        > 0;

    if !has_streamable {
        let _ = conn.execute_batch(
            "
            ALTER TABLE queue_tracks ADD COLUMN streamable INTEGER NOT NULL DEFAULT 1;
            ALTER TABLE queue_tracks ADD COLUMN parental_warning INTEGER NOT NULL DEFAULT 0;
            ALTER TABLE queue_tracks ADD COLUMN source_item_id_hint TEXT;
            ",
        );
    }

    let has_last_view: bool = conn
        .query_row(
            "SELECT COUNT(*) FROM pragma_table_info('player_state') WHERE name = 'last_view'",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0)
        > 0;

    if !has_last_view {
        let _ = conn.execute_batch(
            "
            ALTER TABLE player_state ADD COLUMN last_view TEXT NOT NULL DEFAULT 'home';
            ALTER TABLE player_state ADD COLUMN view_context_id TEXT;
            ALTER TABLE player_state ADD COLUMN view_context_type TEXT;
            ",
        );
    }
}
