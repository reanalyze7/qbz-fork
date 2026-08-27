//! Per-user persistence for the "My Qoqobuz" navigation branding (custom label
//! + custom icon path).
//!
//! Mirrors the Tauri `myQbzNavStore` contract (spec 20 §0.1), re-homed so the
//! ENTRY POINT is Settings > Appearance (DQ3) rather than a sidebar context
//! menu. The persisted shape is `{ label, icon_path }`:
//!
//!  - `label`     : the custom label. A trimmed-empty value is coerced to the
//!                  default `"My Qoqobuz"` and that default string is what gets
//!                  persisted (matching `setMyQbzLabel`).
//!  - `icon_path` : an absolute filesystem path to a user-chosen image, or the
//!                  empty string for "default" (the branded `my-qbz.svg`). The
//!                  reset action stores the empty string — i.e. removes the
//!                  custom icon — rather than persisting a default path.
//!
//! Storage is per-user JSON, scoped the same way as the per-user tray
//! DBs so different Qobuz accounts keep independent branding:
//!
//!   <data_dir>/qbz/users/<user_id>/myqbz_branding.json
//!
//! The store is intentionally minimal: read-modify-write the whole tiny file
//! on every set. `init_for_user` binds it on shell entry; the Slint-facing
//! `seed` / `apply_*` helpers bridge it to `MyQbzBrandingState`.

mod actions;
mod store;
mod ui_bridge;

#[cfg(test)]
mod tests;

pub use actions::{reset_icon, set_label};
pub use store::init_for_user;
pub use ui_bridge::{pick_icon, seed};

/// The default "My Qoqobuz" label (spec 20 §0.1 `DEFAULT_LABEL`).
pub const DEFAULT_LABEL: &str = "My Qoqobuz";
