//! Shared keyboard-driven multi-select core (Excel-style Shift+Click range).
//!
//! Port of the Tauri selection model (`src/lib/utils/multiSelect.ts`):
//! - Shift+Click extends an **additive** range from an anchor (the last
//!   explicitly-clicked row) to the clicked row — it only ever ADDS, never
//!   deselects (`applyShiftRange`).
//! - Plain click / Ctrl+Click stay a single per-row toggle (Tauri reads only
//!   `shiftKey` at click time; Ctrl/Cmd never branch there).
//!
//! This module owns ONE per-surface anchor (a thread-local, UI thread only) and
//! the generic span-fill. The modifier state itself comes from
//! [`crate::keybindings::mods`] (fed by winit `ModifiersChanged`); the caller
//! reads `mods().2` (shift) and routes here. The helper is generic over the row
//! type via a setter closure so it works for both `TrackItem` and (later) the
//! album row struct — they are distinct Slint-generated types with no shared
//! trait.

mod anchor;
mod range;

pub use anchor::{
    anchor_for, clear_anchor, resolve_anchor, set_anchor, SURFACE_ALBUM, SURFACE_ARTIST,
    SURFACE_FAVORITES, SURFACE_LABEL, SURFACE_LOCAL_ALBUMS, SURFACE_LOCAL_TRACKS, SURFACE_MIX,
    SURFACE_OFFLINE, SURFACE_PLAYLIST,
};
pub use range::{apply_shift_range, select_all};
