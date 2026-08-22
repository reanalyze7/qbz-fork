//! Progress-emitting library scan (frontend-agnostic).
//!
//! Lifts the scan-orchestration loop that lived in the Tauri command
//! `v2_library_scan` / `v2_library_scan_folder` into the core crate so any
//! frontend (Slint, TUI) can drive it. The Tauri side polled an
//! `Arc<Mutex<ScanProgress>>` over the IPC boundary; in-process callers get
//! the same information pushed through `on_event` and check `cancel` at every
//! file boundary. The per-file logic (CUE-first, sidecar override, embedded →
//! folder artwork, insert, missing-file cleanup) is replicated exactly.

mod audio_file;
mod audio_loop;
mod cleanup;
mod cue;
mod cue_loop;
mod event;
mod folder_loop;
mod helpers;
mod orchestrate;
mod outcome;
mod targets;

pub use event::ScanEvent;
pub use orchestrate::scan_with_progress;
