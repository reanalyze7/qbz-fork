//! Unit tests for the language-state, translation-lookup, and locale
//! resolution logic split across `lib.rs`/`language.rs`/`translate.rs`.

use super::*;
use crate::translate::substitute;
use std::sync::Mutex;

// Language state is global; serialize tests that mutate it.
static LOCK: Mutex<()> = Mutex::new(());

#[test]
fn embedded_catalogs_parse() {
    let _g = LOCK.lock().unwrap();
    // Header-only catalogs still parse and expose nplurals.
    assert_eq!(catalog(0).nplurals(), 2); // en
    assert_eq!(catalog(3).nplurals(), 2); // fr
    // fr uses (n > 1): 1 -> form 0, 2 -> form 1.
    assert_eq!(catalog(3).plural_rule().index(1), 0);
    assert_eq!(catalog(3).plural_rule().index(2), 1);
}

#[test]
fn language_switch_changes_current() {
    let _g = LOCK.lock().unwrap();
    set_language("en");
    assert_eq!(current_language(), "en");
    set_language("fr");
    assert_eq!(current_language(), "fr");
    // Unknown code leaves it unchanged.
    set_language("zz");
    assert_eq!(current_language(), "fr");
    set_language("en");
}

#[test]
fn mark_returns_literal_unchanged() {
    // `mark` is a no-op at runtime; it only exists for static extraction.
    assert_eq!(mark("Album"), "Album");
    assert_eq!(t(mark("Album")), t("Album"));
}

#[test]
fn t_falls_back_to_msgid() {
    let _g = LOCK.lock().unwrap();
    set_language("en");
    // Bundled en catalog has no message entries yet → identity fallback.
    assert_eq!(t("Play"), "Play");
    assert_eq!(t("Some Untranslated String"), "Some Untranslated String");
}

#[test]
fn tn_english_fallback_by_count() {
    let _g = LOCK.lock().unwrap();
    set_language("en");
    assert_eq!(tn("{} track", "{} tracks", 1), "{} track");
    assert_eq!(tn("{} track", "{} tracks", 0), "{} tracks");
    assert_eq!(tn("{} track", "{} tracks", 3), "{} tracks");
}

#[test]
fn t_args_substitutes_placeholders() {
    let _g = LOCK.lock().unwrap();
    set_language("en");
    assert_eq!(t_args("Hi {}", &["x"]), "Hi x");
    assert_eq!(t_args("{} of {}", &["3", "10"]), "3 of 10");
}

#[test]
fn tf_substitutes_after_plural() {
    let _g = LOCK.lock().unwrap();
    set_language("en");
    assert_eq!(tf("{} track", "{} tracks", 1, &["1"]), "1 track");
    assert_eq!(tf("{} track", "{} tracks", 3, &["3"]), "3 tracks");
}

#[test]
fn substitute_handles_missing_and_extra_args() {
    // No language state touched, but keep ordering deterministic anyway.
    let _g = LOCK.lock().unwrap();
    assert_eq!(substitute("a {} b {}", &["1"]), "a 1 b {}");
    assert_eq!(substitute("only {}", &["1", "2"]), "only 1");
}

#[test]
fn resolve_auto_maps_prefix() {
    let _g = LOCK.lock().unwrap();
    std::env::remove_var("LC_ALL");
    std::env::remove_var("LC_MESSAGES");
    std::env::set_var("LANG", "fr_FR.UTF-8");
    assert_eq!(resolve_auto(), "fr");
    std::env::set_var("LANG", "xx_XX");
    assert_eq!(resolve_auto(), "en");
    std::env::set_var("LANG", "nl_NL.UTF-8");
    assert_eq!(resolve_auto(), "nl");
    std::env::set_var("LC_MESSAGES", "de_DE.UTF-8");
    assert_eq!(resolve_auto(), "de");
    std::env::remove_var("LC_MESSAGES");
    std::env::remove_var("LANG");
}

#[test]
fn resolve_auto_honors_lc_all_precedence() {
    let _g = LOCK.lock().unwrap();
    // LC_ALL wins over LC_MESSAGES and LANG (POSIX precedence).
    std::env::set_var("LANG", "fr_FR.UTF-8");
    std::env::set_var("LC_MESSAGES", "de_DE.UTF-8");
    std::env::set_var("LC_ALL", "es_ES.UTF-8");
    assert_eq!(resolve_auto(), "es");
    // Empty LC_ALL is skipped, falling through to LC_MESSAGES.
    std::env::set_var("LC_ALL", "");
    assert_eq!(resolve_auto(), "de");
    // No LC_ALL/LC_MESSAGES → LANG is used.
    std::env::remove_var("LC_ALL");
    std::env::remove_var("LC_MESSAGES");
    assert_eq!(resolve_auto(), "fr");
    std::env::remove_var("LANG");
}
