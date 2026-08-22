//! Entity-decoding logic (named + numeric), separated from the data
//! table in `entity_table.rs`.

use super::entity_table::{BARE_NAMES, NAMED};

pub(super) fn decode_entities(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut rest = input;
    while !rest.is_empty() {
        if rest.as_bytes()[0] == b'&' {
            if let Some((decoded, consumed)) = match_entity(&rest[1..]) {
                out.push(decoded);
                rest = &rest[1 + consumed..];
                continue;
            }
        }
        let mut chars = rest.chars();
        if let Some(ch) = chars.next() {
            out.push(ch);
            rest = chars.as_str();
        } else {
            break;
        }
    }
    out
}

/// Match one entity at `s`, which starts AFTER the `&`. Returns the
/// decoded char + bytes consumed (after the `&`).
fn match_entity(s: &str) -> Option<(char, usize)> {
    let bytes = s.as_bytes();
    if bytes.first() == Some(&b'#') {
        return match_numeric(&s[1..]).map(|(ch, used)| (ch, used + 1));
    }
    // Read the maximal alphanumeric name run (entity names are ASCII).
    let name_len = bytes
        .iter()
        .take_while(|b| b.is_ascii_alphanumeric())
        .count();
    if name_len == 0 || name_len > 8 {
        return None;
    }
    let name = &s[..name_len];
    let decoded = NAMED
        .iter()
        .find(|(n, _)| *n == name)
        .map(|(_, ch)| *ch)?;
    if bytes.get(name_len) == Some(&b';') {
        return Some((decoded, name_len + 1));
    }
    // Malformed no-semicolon tolerance: allowlisted names only, and only
    // when the next char is a word boundary (end of text, whitespace or
    // punctuation) — so "&copyright" or "AC&DCfan" never decode.
    let at_boundary = match s[name_len..].chars().next() {
        None => true,
        Some(next) => !next.is_alphanumeric(),
    };
    if at_boundary && BARE_NAMES.contains(&name) {
        return Some((decoded, name_len));
    }
    None
}

/// Numeric character reference. `s` starts AFTER `&#`. The semicolon is
/// REQUIRED here — a bare `&#169 ` is left literal (unlike the named
/// allowlist, malformed numeric forms haven't been observed in the wild
/// and digits-then-space appears in ordinary prose too easily).
fn match_numeric(s: &str) -> Option<(char, usize)> {
    let bytes = s.as_bytes();
    let (radix, digits_start) = match bytes.first() {
        Some(b'x') | Some(b'X') => (16u32, 1usize),
        _ => (10u32, 0usize),
    };
    let mut value: u32 = 0;
    let mut i = digits_start;
    while i < bytes.len() {
        let Some(d) = (bytes[i] as char).to_digit(radix) else { break };
        value = value.checked_mul(radix)?.checked_add(d)?;
        if value > 0x10FFFF {
            return None;
        }
        i += 1;
    }
    if i == digits_start || bytes.get(i) != Some(&b';') {
        return None;
    }
    // Browsers map the C1 control range through Windows-1252 (CMS-sourced
    // text really contains `&#146;` for ’); mirror the common cases.
    let value = match value {
        0x82 => 0x201A, // ‚
        0x84 => 0x201E, // „
        0x85 => 0x2026, // …
        0x91 => 0x2018, // '
        0x92 => 0x2019, // '
        0x93 => 0x201C, // "
        0x94 => 0x201D, // "
        0x95 => 0x2022, // •
        0x96 => 0x2013, // –
        0x97 => 0x2014, // —
        0x99 => 0x2122, // ™
        v => v,
    };
    // Reject other control chars — decoding them into UI text helps nobody.
    let ch = char::from_u32(value)?;
    if ch.is_control() && ch != '\n' && ch != '\t' {
        return None;
    }
    Some((ch, i + 1))
}
