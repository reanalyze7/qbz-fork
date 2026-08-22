//! Active-language state mutation and POSIX locale-env resolution.

use crate::{lang_index, CURRENT, LANGS};
use std::sync::atomic::Ordering;

/// Set the active language. Accepts `"en"|"es"|"de"|"fr"|"pt"|"ru"|"ja"|"nl"`.
/// Unknown codes leave the current language unchanged.
pub fn set_language(lang: &str) {
    if let Some(idx) = lang_index(lang) {
        CURRENT.store(idx, Ordering::Relaxed);
    }
}

/// The currently active language code.
pub fn current_language() -> &'static str {
    LANGS[CURRENT.load(Ordering::Relaxed) as usize]
}

/// Resolve the desired language from the environment using POSIX precedence:
/// `$LC_ALL` > `$LC_MESSAGES` > `$LANG`. The 2-letter prefix is mapped to a
/// supported language; otherwise `"en"`. This does NOT mutate [`CURRENT`] —
/// the caller decides.
pub fn resolve_auto() -> &'static str {
    let raw = std::env::var("LC_ALL")
        .ok()
        .filter(|s| !s.is_empty())
        .or_else(|| std::env::var("LC_MESSAGES").ok().filter(|s| !s.is_empty()))
        .or_else(|| std::env::var("LANG").ok())
        .unwrap_or_default();

    let prefix: String = raw
        .chars()
        .take_while(|c| c.is_ascii_alphabetic())
        .collect::<String>()
        .to_ascii_lowercase();

    match lang_index(&prefix) {
        Some(idx) => LANGS[idx as usize],
        None => "en",
    }
}
