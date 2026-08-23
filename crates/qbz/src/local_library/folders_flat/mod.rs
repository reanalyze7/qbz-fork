//! Folders tab (flat mode) — the album grid grouped by directory rather than
//! by metadata.

mod derive;
mod load;

pub use derive::derive_folders;
pub use load::ensure_folders_loaded;
