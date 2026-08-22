//! qbz-i18n — frontend-agnostic gettext-style translation catalog.
//!
//! Reads the same gettext `.po` files Slint bundles, keyed by
//! `msgid = English source string` (no `msgctxt`). Reusable by any frontend
//! (Slint / TUI / headless) — no slint or tauri dependencies (ADR-006).

pub mod plural;
pub mod po;

mod language;
mod translate;

#[cfg(test)]
mod tests;

pub use language::{current_language, resolve_auto, set_language};
pub use plural::PluralRule;
pub use po::Catalog;
pub use translate::{mark, t, t_args, tf, tn};

use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::OnceLock;

/// Supported language codes, indexed by the value stored in [`CURRENT`].
const LANGS: [&str; 8] = ["en", "es", "de", "fr", "pt", "ru", "ja", "nl"];

/// Embedded `.po` sources. Path is relative to this file
/// (`crates/qbz-i18n/src/lib.rs`): `../` = `qbz-i18n/`, `../../` = `crates/`.
const PO_EN: &str = include_str!("../../qbz-ui/translations/en/LC_MESSAGES/qbz-ui.po");
const PO_ES: &str = include_str!("../../qbz-ui/translations/es/LC_MESSAGES/qbz-ui.po");
const PO_DE: &str = include_str!("../../qbz-ui/translations/de/LC_MESSAGES/qbz-ui.po");
const PO_FR: &str = include_str!("../../qbz-ui/translations/fr/LC_MESSAGES/qbz-ui.po");
const PO_PT: &str = include_str!("../../qbz-ui/translations/pt/LC_MESSAGES/qbz-ui.po");
const PO_RU: &str = include_str!("../../qbz-ui/translations/ru/LC_MESSAGES/qbz-ui.po");
const PO_JA: &str = include_str!("../../qbz-ui/translations/ja/LC_MESSAGES/qbz-ui.po");
const PO_NL: &str = include_str!("../../qbz-ui/translations/nl/LC_MESSAGES/qbz-ui.po");

/// Current language index (0=en, 1=es, 2=de, 3=fr, 4=pt, 5=ru, 6=ja, 7=nl). Defaults to en.
static CURRENT: AtomicU8 = AtomicU8::new(0);

/// Lazily-parsed catalogs, one slot per language.
static CATALOGS: [OnceLock<Catalog>; 8] = [
    OnceLock::new(),
    OnceLock::new(),
    OnceLock::new(),
    OnceLock::new(),
    OnceLock::new(),
    OnceLock::new(),
    OnceLock::new(),
    OnceLock::new(),
];

/// Map a language code to its index, if supported.
pub(crate) fn lang_index(lang: &str) -> Option<u8> {
    LANGS.iter().position(|&l| l == lang).map(|i| i as u8)
}

/// Get the parsed catalog for a language index, parsing on first use.
pub(crate) fn catalog(idx: u8) -> &'static Catalog {
    let idx = idx as usize;
    CATALOGS[idx].get_or_init(|| {
        let src = match idx {
            0 => PO_EN,
            1 => PO_ES,
            2 => PO_DE,
            3 => PO_FR,
            4 => PO_PT,
            5 => PO_RU,
            6 => PO_JA,
            7 => PO_NL,
            _ => PO_EN,
        };
        Catalog::parse(LANGS[idx], src)
    })
}

/// The catalog for the current language.
pub(crate) fn current_catalog() -> &'static Catalog {
    catalog(CURRENT.load(Ordering::Relaxed))
}
