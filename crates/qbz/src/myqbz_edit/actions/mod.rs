//! Public entry points — the hero overflow menu + modal submit handlers.

mod bulk;
mod kind_mode;
mod rename;

pub use bulk::{delete, remove_selected};
pub use kind_mode::{convert_kind, toggle_play_mode};
pub use rename::{rename, set_description};
