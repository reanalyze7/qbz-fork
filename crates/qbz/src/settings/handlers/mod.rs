//! Settings-panel callback handlers, one file per Slint callback kind.

mod bool;
mod bool_keys;
mod offline;
mod reset;
mod select;
mod select_backend;
mod select_device;
mod slider;

pub use bool::handle_bool;
pub use reset::{handle_release_device, handle_reset};
pub use select::{handle_select, handle_string};
pub use slider::handle_slider;
