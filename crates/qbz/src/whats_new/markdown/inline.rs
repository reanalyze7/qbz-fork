//! Inline markdown handling: bold/code stripping and `[text](url)` links.

/// Strip inline `**bold**` / `` `code` `` markers and reduce inline markdown
/// links `[text](url)` to just their `text` (a single Slint Text block can't
/// carry clickable inline spans — a WHOLE-line link becomes a `KIND_LINK` block
/// instead, see `parse_standalone_link`). Keeps the inner text otherwise.
pub(super) fn strip_inline(text: &str) -> String {
    strip_markdown_links(text).replace("**", "").replace('`', "")
}

/// If `s[start..]` begins with a markdown link `[label](url)`, return
/// `(label, url, byte-index just past the ')')`. The `[](` `)` delimiters are
/// ASCII, so all returned slices sit on char boundaries. No nested brackets.
fn parse_link_at(s: &str, start: usize) -> Option<(&str, &str, usize)> {
    let rest = &s[start..];
    if !rest.starts_with('[') {
        return None;
    }
    let close_br = rest.find(']')?;
    if rest.as_bytes().get(close_br + 1) != Some(&b'(') {
        return None;
    }
    let open_paren = close_br + 1;
    let close_paren = open_paren + rest[open_paren..].find(')')?;
    let label = &rest[1..close_br];
    let url = &rest[open_paren + 1..close_paren];
    if url.is_empty() {
        return None;
    }
    Some((label, url, start + close_paren + 1))
}

/// Replace every inline `[text](url)` in a string with just its `text`.
fn strip_markdown_links(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut i = 0;
    while i < text.len() {
        if text.as_bytes()[i] == b'[' {
            if let Some((label, _url, end)) = parse_link_at(text, i) {
                out.push_str(label);
                i = end;
                continue;
            }
        }
        let ch = text[i..].chars().next().unwrap();
        out.push(ch);
        i += ch.len_utf8();
    }
    out
}

/// If the whole trimmed line is exactly one markdown link, return `(text, url)`.
pub(super) fn parse_standalone_link(s: &str) -> Option<(&str, &str)> {
    let t = s.trim();
    let (label, url, end) = parse_link_at(t, 0)?;
    (end == t.len()).then_some((label, url))
}
