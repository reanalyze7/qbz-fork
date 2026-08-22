//! Line-parsing internals for [`super::Catalog::parse`]: the per-entry state
//! machine (`Field`/`Entry`), flushing a completed entry into the catalog
//! maps, and quoted-string unescaping.

use std::collections::HashMap;

use crate::plural::PluralRule;

/// Which field the parser last touched (for continuation lines).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Field {
    None,
    Msgid,
    MsgidPlural,
    Msgstr,
    MsgstrPlural(usize),
}

impl Default for Field {
    fn default() -> Self {
        Field::None
    }
}

#[derive(Debug, Default)]
pub(super) struct Entry {
    pub(super) msgid: Option<String>,
    pub(super) msgid_plural: Option<String>,
    pub(super) msgstr: Option<String>,
    pub(super) msgstr_plural: Vec<String>,
    pub(super) last: Field,
}

pub(super) fn flush(
    cur: &mut Entry,
    singular: &mut HashMap<String, String>,
    plural: &mut HashMap<String, Vec<String>>,
    plural_rule: &mut PluralRule,
) {
    let entry = std::mem::take(cur);
    let msgid = match entry.msgid {
        Some(m) => m,
        None => return,
    };

    // Header entry: empty msgid carries metadata in its msgstr.
    if msgid.is_empty() {
        if let Some(header) = entry.msgstr {
            for line in header.split('\n') {
                if let Some(value) = line.strip_prefix("Plural-Forms:") {
                    *plural_rule = PluralRule::parse(value.trim());
                }
            }
        }
        return;
    }

    if entry.msgid_plural.is_some() || !entry.msgstr_plural.is_empty() {
        plural.insert(msgid, entry.msgstr_plural);
    } else if let Some(s) = entry.msgstr {
        if !s.is_empty() {
            singular.insert(msgid, s);
        }
    }
}

/// Append a parsed continuation-line `piece` to whichever field `cur.last`
/// currently points at.
pub(super) fn append_continuation(cur: &mut Entry, piece: String) {
    match cur.last {
        Field::Msgid => cur.msgid.get_or_insert_with(String::new).push_str(&piece),
        Field::MsgidPlural => cur
            .msgid_plural
            .get_or_insert_with(String::new)
            .push_str(&piece),
        Field::Msgstr => cur.msgstr.get_or_insert_with(String::new).push_str(&piece),
        Field::MsgstrPlural(n) => {
            if n < cur.msgstr_plural.len() {
                cur.msgstr_plural[n].push_str(&piece);
            }
        }
        Field::None => {}
    }
}

/// Extract and unescape the contents of a leading `"..."` segment.
pub(super) fn parse_quoted(s: &str) -> Option<String> {
    let s = s.trim();
    let bytes = s.as_bytes();
    if bytes.first() != Some(&b'"') {
        return None;
    }
    let mut out = String::new();
    let mut chars = s[1..].chars();
    while let Some(c) = chars.next() {
        match c {
            '"' => break, // closing quote
            '\\' => match chars.next() {
                Some('n') => out.push('\n'),
                Some('t') => out.push('\t'),
                Some('r') => out.push('\r'),
                Some('"') => out.push('"'),
                Some('\\') => out.push('\\'),
                Some(other) => out.push(other),
                None => break,
            },
            other => out.push(other),
        }
    }
    Some(out)
}
