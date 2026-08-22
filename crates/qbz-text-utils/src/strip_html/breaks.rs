//! `<br>` / `</p>` → newline normalization pass.

/// Walk by char (not byte) so multi-byte UTF-8 sequences (ó, é, "—",
/// curly quotes) survive untouched. Skip recognized `<br>` and `</p>`
/// runs by replacing them with newlines; pass everything else through
/// so the second pass can strip the remaining tags.
pub(super) fn normalize_breaks(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut rest = input;
    while !rest.is_empty() {
        if let Some(stripped) = rest.strip_prefix('<') {
            if let Some((replacement, consumed)) = match_break_or_paragraph(stripped) {
                out.push_str(replacement);
                rest = &stripped[consumed..];
                continue;
            }
        }
        // Advance one char (not one byte) — pushes the full UTF-8
        // sequence intact.
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

/// Try to match `<br>` (any case, with optional spaces and self-
/// closing slash) or `</p>` (any case). `s` starts AFTER the opening
/// `<`. Returns the replacement string + bytes consumed (after the
/// closing `>`).
fn match_break_or_paragraph(s: &str) -> Option<(&'static str, usize)> {
    let bytes = s.as_bytes();
    // </p>
    if bytes.len() >= 3
        && bytes[0] == b'/'
        && (bytes[1] == b'p' || bytes[1] == b'P')
        && bytes[2] == b'>'
    {
        return Some(("\n\n", 3));
    }
    // <br>, <br/>, <br />, etc.
    if bytes.len() >= 3 && (bytes[0] == b'b' || bytes[0] == b'B')
        && (bytes[1] == b'r' || bytes[1] == b'R')
    {
        let mut j = 2usize;
        while j < bytes.len() && bytes[j] != b'>' {
            // Only allow whitespace and a single '/' between `br` and `>`.
            if !bytes[j].is_ascii_whitespace() && bytes[j] != b'/' {
                return None;
            }
            j += 1;
        }
        if j < bytes.len() && bytes[j] == b'>' {
            return Some(("\n", j + 1));
        }
    }
    None
}
