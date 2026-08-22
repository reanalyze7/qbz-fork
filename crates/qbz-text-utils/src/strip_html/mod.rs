//! Convert HTML-ish strings from Qobuz (biographies, album reviews)
//! into Slint-friendly plain text. Slint's `Text` is single-style, so
//! we cannot render inline strong/em formatting — those tags are
//! stripped but their content stays inline. Paragraph and line-break
//! structure IS preserved: `<br>` collapses to `\n`, `</p>` to a
//! blank line so Text renders the paragraphs separated visually.

mod breaks;
mod collapse;
mod entities;
mod entity_table;
mod tags;

#[cfg(test)]
mod tests;

use breaks::normalize_breaks;
use collapse::collapse_blank_lines;
use entities::decode_entities;
use tags::strip_remaining_tags;

/// Render an HTML-ish blurb into plain text with paragraph breaks.
pub fn strip_html(input: &str) -> String {
    let normalized = normalize_breaks(input);
    let stripped = strip_remaining_tags(&normalized);
    let decoded = decode_entities(&stripped);
    collapse_blank_lines(&decoded)
}

/// Decode HTML entities into plain text. Public because some API prose
/// fields carry entities without any markup (e.g. the artist biography
/// source credit) — those callers want entity decoding WITHOUT the
/// tag-strip / paragraph passes of [`strip_html`].
///
/// Handles:
/// - the named entities Qobuz/TiVo prose actually emits (see `entity_table`),
///   incl. the full Latin-1 accented set;
/// - numeric character references, decimal (`&#233;`) and hex (`&#xE9;`),
///   with the Windows-1252 quirk range (`&#146;` → ’) mapped like
///   browsers do;
/// - MALFORMED no-semicolon forms for a tiny allowlist (`&copy` `&reg`
///   `&amp` `&nbsp`) when followed by a word boundary — TiVo biography
///   credits really arrive as `&copy  John Book /TiVo` (no semicolon).
///   Tradeoff: prose that literally means the string "&copy " would be
///   rewritten; accepted, since that never appears in catalog prose,
///   while the malformed credit line appears on virtually every bio.
///   Names outside the allowlist ("AC&DC", "&copyright2020") are never
///   touched without their semicolon.
pub fn decode_html_entities(input: &str) -> String {
    decode_entities(input)
}
