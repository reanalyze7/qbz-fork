//! `impl DiagController`: `refresh` (+ its async body, in `refresh.rs`) and
//! `export_clipboard` (`export.rs`) — the stateful orchestration core. Both
//! stay under this module since they share the cached `export` snapshot
//! field: `refresh_async` writes it, `export_clipboard` reads it.

mod export;
mod refresh;
