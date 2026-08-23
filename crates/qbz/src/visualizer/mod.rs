//! ImmersiveView audio-visualizer glue.
//!
//! Spawns the frontend-agnostic FFT producer (`qbz_audio::visualizer`) against
//! the runtime's [`VisualizerTap`], latches each [`VizFrame`] into a single-slot
//! cell, and drains the latest frames into the `VisualizerState` Slint global on
//! a ~30 fps UI-thread timer. Persistent `VecModel`s are mutated in place
//! (`set_row_data`) so the Slint side keeps the same model identity — no
//! per-frame re-instantiation of the bound views.
//!
//! The tap starts disabled: nothing is captured and the FFT loop idles until the
//! immersive view calls `VisualizerState::set-enabled(true)` on open. There is no
//! Tauri command here — `set-enabled` drives `tap.set_enabled` directly, the same
//! pattern `playback.rs` uses for the rest of the runtime controls.
//!
//! Protected-audio note: this lives entirely downstream of the read-only ring
//! buffer. It touches none of the device/stream init (see CLAUDE.md "Audio
//! Backend System").

mod cells;
mod drain;
mod install;
mod shader;

pub use install::install;
