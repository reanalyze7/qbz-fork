//! `impl PlaybackEngine` blocks that `match self` over all backend variants,
//! split by concern. Rust allows multiple `impl Foo` blocks across files in
//! the same module tree, so each submodule just re-opens `impl PlaybackEngine`.

mod append;
mod crossfade;
mod play_pause;
mod position;
mod query;
mod transport;
mod volume;
