//! Ephemeral folder: open a folder OUTSIDE the indexed library, browse +
//! play it without writing to library.db. The scan/metadata logic is shared
//! (`qbz_library::ephemeral`); here we drive the picker, build the
//! album-grouped pane, and persist the path.

mod build;
mod open;
mod reset;

pub use open::{open_ephemeral, rehydrate_ephemeral};
pub use reset::clear_ephemeral;
