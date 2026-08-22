//! Minimal gettext `.po` parser producing a [`Catalog`].
//!
//! Keyed by `msgid` (the English source string); we do NOT use `msgctxt`.
//! Handles: the header entry (`msgid ""`), `msgid` / `msgid_plural` /
//! `msgstr` / `msgstr[N]`, multi-line string continuations (adjacent quoted
//! lines concatenate), `#` comment lines, and `\n` / `\t` / `\"` / `\\` escapes.
//! An empty `msgstr` means "no translation" → lookups return `None` so callers
//! fall back to the English source.

mod catalog;
mod parser;

#[cfg(test)]
mod tests;

pub use catalog::Catalog;
