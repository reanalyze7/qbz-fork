//! Top-level open/apply/reset lifecycle for the detail controller.

mod navigate;
mod reset_apply;

pub use navigate::navigate;
pub use reset_apply::{apply, apply_not_found, get_collection, reset};
