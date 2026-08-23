//! Artist-name normalization for merge/match.

/// Fold a common Latin accented char to its ASCII base (best-effort, no
/// `unicode-normalization` dep). Covers Spanish/European music metadata; the
/// uncovered tail just won't merge across diacritics. Mirrors the intent of
/// Tauri's NFKD + combining-mark strip in `normalizeArtistName`.
fn fold_diacritic(c: char) -> char {
    match c {
        'á' | 'à' | 'â' | 'ä' | 'ã' | 'å' | 'ā' => 'a',
        'é' | 'è' | 'ê' | 'ë' | 'ē' | 'ė' => 'e',
        'í' | 'ì' | 'î' | 'ï' | 'ī' => 'i',
        'ó' | 'ò' | 'ô' | 'ö' | 'õ' | 'ō' | 'ø' => 'o',
        'ú' | 'ù' | 'û' | 'ü' | 'ū' => 'u',
        'ñ' => 'n',
        'ç' => 'c',
        'ý' | 'ÿ' => 'y',
        'ß' => 's',
        _ => c,
    }
}

/// Normalize an artist name for merge/match: lowercase, fold diacritics,
/// collapse every run of non-alphanumerics to a single space, trim. So
/// "Alice In Chains" and "alice  in chains" both -> "alice in chains".
pub fn normalize_artist(name: &str) -> String {
    let mut out = String::with_capacity(name.len());
    let mut prev_space = false;
    for ch in name.to_lowercase().chars() {
        let c = fold_diacritic(ch);
        if c.is_ascii_alphanumeric() {
            out.push(c);
            prev_space = false;
        } else if !prev_space {
            out.push(' ');
            prev_space = true;
        }
    }
    out.trim().to_string()
}

/// Split a credit string into individual artist names on the usual
/// separators (comma already handled by the caller for `all_artists`).
pub(crate) fn split_credit(s: &str) -> Vec<String> {
    s.split([',', '&', '/', ';'])
        .flat_map(|p| {
            p.split(" feat ")
                .flat_map(|q| q.split(" ft "))
                .flat_map(|q| q.split(" featuring "))
                .flat_map(|q| q.split(" with "))
                .map(|q| q.to_string())
                .collect::<Vec<_>>()
        })
        .collect()
}
