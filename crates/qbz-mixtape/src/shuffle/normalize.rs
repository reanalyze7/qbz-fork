//! Title/artist normalization helpers used before similarity comparison.

/// Lowercase, strip diacritics, drop bracketed parentheticals, drop ` - `
/// suffixes, drop `feat. X` patterns, drop punctuation, collapse whitespace.
pub fn normalize_title(s: &str) -> String {
    let lower = s.to_lowercase();
    let unaccented = strip_diacritics(&lower);
    let unbracketed = remove_brackets(&unaccented);
    let untrailed = strip_dash_suffix(&unbracketed);
    let unfeatured = strip_feat(&untrailed);
    let unpunct = strip_punctuation(&unfeatured);
    collapse_whitespace(&unpunct)
}

/// Lowercase + strip diacritics + trim. Parens are preserved (e.g. `Foo (band)`
/// must not collapse to `Foo`).
pub fn normalize_artist(s: &str) -> String {
    let lower = s.to_lowercase();
    let unaccented = strip_diacritics(&lower);
    collapse_whitespace(&unaccented)
}

pub(super) fn strip_diacritics(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            'á' | 'à' | 'â' | 'ã' | 'ä' | 'å' | 'ā' | 'ą' => 'a',
            'é' | 'è' | 'ê' | 'ë' | 'ē' | 'ę' => 'e',
            'í' | 'ì' | 'î' | 'ï' | 'ī' => 'i',
            'ó' | 'ò' | 'ô' | 'õ' | 'ö' | 'ø' | 'ō' => 'o',
            'ú' | 'ù' | 'û' | 'ü' | 'ū' => 'u',
            'ñ' => 'n',
            'ç' => 'c',
            'ÿ' | 'ý' => 'y',
            'ß' => 's',
            other => other,
        })
        .collect()
}

pub(super) fn remove_brackets(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut depth = 0i32;
    for c in s.chars() {
        match c {
            '(' | '[' | '{' => depth += 1,
            ')' | ']' | '}' => {
                if depth > 0 {
                    depth -= 1;
                }
            }
            _ => {
                if depth == 0 {
                    out.push(c);
                }
            }
        }
    }
    out
}

pub(super) fn strip_dash_suffix(s: &str) -> String {
    match s.find(" - ") {
        Some(idx) => s[..idx].to_string(),
        None => s.to_string(),
    }
}

pub(super) fn strip_feat(s: &str) -> String {
    // Order matters: longest patterns first so " feat. " wins over " feat ".
    const PATTERNS: &[&str] = &[" featuring ", " feat. ", " feat ", " ft. ", " ft "];
    for p in PATTERNS {
        if let Some(idx) = s.find(p) {
            return s[..idx].to_string();
        }
    }
    s.to_string()
}

pub(super) fn strip_punctuation(s: &str) -> String {
    const PUNCT: &[char] = &[
        ',', '.', '!', '?', '¿', '¡', '"', '\'', ';', ':', '/', '\\',
    ];
    s.chars().filter(|c| !PUNCT.contains(c)).collect()
}

pub(super) fn collapse_whitespace(s: &str) -> String {
    s.split_whitespace().collect::<Vec<&str>>().join(" ")
}
