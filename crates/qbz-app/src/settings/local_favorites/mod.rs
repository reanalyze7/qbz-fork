//! Headless local-favorites service.
//!
//! Frontend-agnostic store for favoriting LOCAL library items (genuine local
//! files — never the Qobuz offline cache). Mirrors `pinned_items.rs`
//! (same pragmas, error style, in-memory `(kind, id)` set) per ADR-006; the
//! per-user lifecycle lives in the `qbz` crate wrapper (`crate::local_favorites`).
//!
//! Rows carry a display snapshot (title/subtitle/artwork) taken at favorite
//! time plus a denormalized `artist` (for per-artist counts) and `source`
//! (`local`). The `CHECK` on `source` refuses `qobuz_download` at
//! write time, so the mixed-library feed built from this store is inherently
//! free of Qobuz-offline duplicates.

mod model;
mod queries;
mod service;
#[cfg(test)]
mod tests;

pub use model::{LocalFavItem, DB_FILE_NAME};
pub use service::LocalFavoritesService;
