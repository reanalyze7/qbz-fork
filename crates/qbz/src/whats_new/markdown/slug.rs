//! TOC anchor slugification + leading-indent counting.

/// Slugify a heading label into a TOC anchor id (port of the Tauri `slugify`).
pub(super) fn slugify(input: &str) -> String {
    let lowered = input.trim().to_lowercase();
    // Drop markdown emphasis chars, then keep [a-z0-9] + spaces/hyphens.
    let mut cleaned = String::with_capacity(lowered.len());
    for ch in lowered.chars() {
        if matches!(ch, '`' | '*' | '_' | '~') {
            continue;
        }
        if ch.is_ascii_alphanumeric() || ch == ' ' || ch == '-' {
            cleaned.push(ch);
        }
    }
    // Collapse whitespace runs to single hyphens, then collapse hyphen runs.
    let mut out = String::with_capacity(cleaned.len());
    let mut last_hyphen = false;
    for ch in cleaned.chars() {
        if ch == ' ' || ch == '-' {
            if !last_hyphen {
                out.push('-');
                last_hyphen = true;
            }
        } else {
            out.push(ch);
            last_hyphen = false;
        }
    }
    out.trim_matches('-').to_string()
}

/// Count leading-space indentation (a tab counts as 2), like the Tauri
/// `countLeadingSpaces`.
pub(super) fn count_leading_spaces(line: &str) -> usize {
    let mut count = 0;
    for ch in line.chars() {
        match ch {
            ' ' => count += 1,
            '\t' => count += 2,
            _ => break,
        }
    }
    count
}
