// crates/qbzd/src/cli/playlist/ — the `qbzd playlist …` verbs (02 §2.3).
mod internal;
mod read;
mod write;

pub use read::{list, show};
pub use write::{add, create, edit, remove, rm};
