pub(super) fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        let mut out: String = s.chars().take(max.saturating_sub(1)).collect();
        out.push('…');
        out
    }
}

/// Pad `s` with spaces to `n` columns, or truncate it (with `…`) if it is longer.
pub(super) fn pad_to(s: &str, n: usize) -> String {
    let len = s.chars().count();
    if len >= n {
        truncate(s, n)
    } else {
        format!("{s}{}", " ".repeat(n - len))
    }
}
