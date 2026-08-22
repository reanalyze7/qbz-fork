//! The [`Catalog`] data model: parsing a `.po` file's lines (via
//! `super::parser`'s helpers) into singular/plural lookup maps, and the
//! public lookup API.

use std::collections::HashMap;

use crate::plural::PluralRule;

use super::parser::{append_continuation, flush, parse_quoted, Entry, Field};

/// A parsed translation catalog for a single language.
#[derive(Debug, Clone)]
pub struct Catalog {
    lang: String,
    plural_rule: PluralRule,
    /// Singular: msgid -> msgstr (non-empty only).
    singular: HashMap<String, String>,
    /// Plural: msgid -> Vec of msgstr[N] (index = plural form).
    plural: HashMap<String, Vec<String>>,
}

impl Catalog {
    /// Parse `.po` text for the given language code.
    pub fn parse(lang: &str, po_text: &str) -> Catalog {
        let mut singular: HashMap<String, String> = HashMap::new();
        let mut plural: HashMap<String, Vec<String>> = HashMap::new();
        let mut plural_rule = PluralRule::default();

        let mut cur = Entry::default();

        // Track which field the current continuation lines belong to.
        for raw_line in po_text.lines() {
            let line = raw_line.trim();

            if line.is_empty() {
                flush(&mut cur, &mut singular, &mut plural, &mut plural_rule);
                continue;
            }
            if line.starts_with('#') {
                continue;
            }

            if let Some(rest) = line.strip_prefix("msgctxt ") {
                // We ignore msgctxt for keying but still consume its string so a
                // trailing continuation doesn't bleed into the wrong field.
                let _ = parse_quoted(rest);
                cur.last = Field::None;
            } else if let Some(rest) = line.strip_prefix("msgid_plural ") {
                cur.msgid_plural = Some(parse_quoted(rest).unwrap_or_default());
                cur.last = Field::MsgidPlural;
            } else if let Some(rest) = line.strip_prefix("msgid ") {
                // A new msgid begins a new entry; flush any pending one.
                flush(&mut cur, &mut singular, &mut plural, &mut plural_rule);
                cur.msgid = Some(parse_quoted(rest).unwrap_or_default());
                cur.last = Field::Msgid;
            } else if let Some(rest) = line.strip_prefix("msgstr[") {
                // msgstr[N] "..."
                if let Some(close) = rest.find(']') {
                    let n: usize = rest[..close].trim().parse().unwrap_or(0);
                    let after = rest[close + 1..].trim_start();
                    let val = parse_quoted(after).unwrap_or_default();
                    if cur.msgstr_plural.len() <= n {
                        cur.msgstr_plural.resize(n + 1, String::new());
                    }
                    cur.msgstr_plural[n] = val;
                    cur.last = Field::MsgstrPlural(n);
                }
            } else if let Some(rest) = line.strip_prefix("msgstr ") {
                cur.msgstr = Some(parse_quoted(rest).unwrap_or_default());
                cur.last = Field::Msgstr;
            } else if line.starts_with('"') {
                // Continuation line: append to the most recently seen field.
                let piece = parse_quoted(line).unwrap_or_default();
                append_continuation(&mut cur, piece);
            }
        }
        // Flush trailing entry (file may not end with a blank line).
        flush(&mut cur, &mut singular, &mut plural, &mut plural_rule);

        Catalog {
            lang: lang.to_string(),
            plural_rule,
            singular,
            plural,
        }
    }

    /// Language code this catalog was parsed for.
    pub fn lang(&self) -> &str {
        &self.lang
    }

    /// The catalog's plural rule (from the header `Plural-Forms`).
    pub fn plural_rule(&self) -> PluralRule {
        self.plural_rule
    }

    /// Number of plural forms for this catalog.
    pub fn nplurals(&self) -> usize {
        self.plural_rule.nplurals()
    }

    /// Look up the translated singular for `msgid`.
    /// Returns `None` when there is no (non-empty) translation.
    pub fn get(&self, msgid: &str) -> Option<&str> {
        self.singular.get(msgid).map(|s| s.as_str())
    }

    /// Look up the translated plural form `form_index` for `msgid`.
    /// Returns `None` when missing or empty.
    pub fn get_plural(&self, msgid: &str, form_index: usize) -> Option<&str> {
        let forms = self.plural.get(msgid)?;
        let val = forms.get(form_index)?;
        if val.is_empty() {
            None
        } else {
            Some(val.as_str())
        }
    }
}
