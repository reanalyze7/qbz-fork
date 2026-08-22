//! Migrations, part 4 of 6 (chronological order preserved): the
//! sample_rate INTEGER -> REAL migration. This single migration block was
//! ~150 lines in the monolithic `database.rs`, so its two sequential
//! sub-steps (table rebuild, then re-adding columns added by later
//! migrations) are split into `sample_rate_rebuild.rs` and
//! `sample_rate_readd.rs` to stay under the 130-line file limit — the
//! logic itself is unchanged and still runs as one atomic migration step
//! from `run_migrations`'s point of view.

use crate::LibraryError;

use super::super::LibraryDatabase;

impl LibraryDatabase {
    pub(super) fn migrate_v4(&self) -> Result<(), LibraryError> {
        // Migration: Change sample_rate from INTEGER to REAL for decimal precision (44.1kHz, 88.2kHz, etc.)
        // Check if sample_rate is currently INTEGER
        let sample_rate_type: String = self
            .conn
            .query_row(
                "SELECT type FROM pragma_table_info('local_tracks') WHERE name = 'sample_rate'",
                [],
                |row| row.get(0),
            )
            .unwrap_or_else(|_| "REAL".to_string());

        if sample_rate_type == "INTEGER" {
            log::info!(
                "Running migration: changing sample_rate from INTEGER to REAL for decimal precision"
            );

            self.rebuild_local_tracks_for_real_sample_rate()?;
            self.readd_columns_after_sample_rate_migration()?;

            log::info!("Migration completed: sample_rate is now REAL");
        }

        Ok(())
    }
}
