//! Stable theme identity: the enum, its persisted slug, display name (proper-
//! noun data, NOT an i18n key — see ADR / i18n rule), category grouping, and a
//! luminance-derived light/dark flag.

mod category;
mod category_map;
mod display_name;
mod meta;
mod slug;
mod theme_id;
#[cfg(test)]
mod tests;

pub use category::ThemeCategory;
pub use meta::default_slug;
pub use theme_id::{ThemeId, ALL};
