//! UUID / long-hex redaction for exported diagnostics text.

/// Redact UUID + long-hex substrings (ported from DiagnosticsPanel.svelte's
/// `redactIdLike`, JS regex `/[0-9a-f]{8}-…/` + `/\b[0-9a-f]{32,}\b/`). Keeps
/// pasted diagnostics free of anything a secret scanner might flag. Operates on
/// chars so it is UTF-8 safe.
pub(super) fn redact_id_like(value: &str) -> String {
    let chars: Vec<char> = value.chars().collect();
    let n = chars.len();
    let is_word = |c: char| c.is_ascii_alphanumeric() || c == '_';
    let mut out = String::with_capacity(value.len());
    let mut i = 0;
    while i < n {
        // UUID shape (8-4-4-4-12 hex), word-boundary delimited.
        if uuid_at(&chars, i)
            && (i == 0 || !is_word(chars[i - 1]))
            && (i + 36 >= n || !is_word(chars[i + 36]))
        {
            out.push_str("<uuid>");
            i += 36;
            continue;
        }
        // A maximal word token that is entirely hex and >= 32 chars long.
        if chars[i].is_ascii_hexdigit() && (i == 0 || !is_word(chars[i - 1])) {
            let mut j = i;
            while j < n && is_word(chars[j]) {
                j += 1;
            }
            if j - i >= 32 && chars[i..j].iter().all(|c| c.is_ascii_hexdigit()) {
                out.push_str("<hex>");
                i = j;
                continue;
            }
        }
        out.push(chars[i]);
        i += 1;
    }
    out
}

/// Whether a 36-char `8-4-4-4-12` hex UUID starts at `i`.
fn uuid_at(chars: &[char], i: usize) -> bool {
    let groups = [8usize, 4, 4, 4, 12];
    let mut p = i;
    for (gi, &len) in groups.iter().enumerate() {
        for _ in 0..len {
            if p >= chars.len() || !chars[p].is_ascii_hexdigit() {
                return false;
            }
            p += 1;
        }
        if gi < 4 {
            if p >= chars.len() || chars[p] != '-' {
                return false;
            }
            p += 1;
        }
    }
    true
}
