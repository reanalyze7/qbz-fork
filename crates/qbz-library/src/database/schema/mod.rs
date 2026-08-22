//! Schema creation and migrations, split by chronological order.
//!
//! `init_schema` creates every table/index a fresh database needs.
//! `run_migrations` brings an *existing* database up to date by applying
//! each historical migration block in strict original order — the blocks
//! are split across `migrations_v1`..`migrations_v6` purely to stay under
//! the 130-line file limit; they must still run sequentially.
//! `migrations_v4` (the sample_rate INTEGER -> REAL migration) further
//! delegates to `sample_rate_rebuild` and `sample_rate_readd`.

mod init_core;
mod init_extra;
mod init_misc;
mod migrations_v1;
mod migrations_v2;
mod migrations_v3;
mod migrations_v4;
mod migrations_v5;
mod migrations_v6;
mod sample_rate_readd;
mod sample_rate_rebuild;

use crate::LibraryError;

use super::LibraryDatabase;

impl LibraryDatabase {
    pub(super) fn init_schema(&self) -> Result<(), LibraryError> {
        self.init_core()?;
        self.init_extra()?;
        self.init_misc()?;
        Ok(())
    }

    pub(super) fn run_migrations(&self) -> Result<(), LibraryError> {
        self.migrate_v1()?;
        self.migrate_v2()?;
        self.migrate_v3()?;
        self.migrate_v4()?;
        self.migrate_v5()?;
        self.migrate_v6()?;
        Ok(())
    }
}
