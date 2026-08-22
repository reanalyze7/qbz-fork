use super::*;

#[test]
fn strips_inline_formatting() {
    let html = "<p>One <strong>bold</strong> and <em>italic</em>.</p>";
    let plain = strip_html(html);
    assert_eq!(plain, "One bold and italic.");
}

#[test]
fn converts_br_to_newline() {
    let html = "Line 1<br>Line 2<br />Line 3";
    assert_eq!(strip_html(html), "Line 1\nLine 2\nLine 3");
}

#[test]
fn converts_paragraphs() {
    let html = "<p>First.</p><p>Second.</p>";
    assert_eq!(strip_html(html), "First.\n\nSecond.");
}

#[test]
fn decodes_common_entities() {
    let html = "Rock &amp; Roll &mdash; &ldquo;the rest&rdquo;.";
    assert_eq!(strip_html(html), "Rock & Roll \u{2014} \u{201C}the rest\u{201D}.");
}

#[test]
fn preserves_multibyte_characters() {
    // Mexican Spanish with accented chars, ñ, ó — the previous
    // byte-walking implementation would have shredded these into
    // their UTF-8 bytes (à+³ instead of ó).
    let html = "<p>La cantautora se estableció en Madrid, España.</p>";
    let plain = strip_html(html);
    assert_eq!(plain, "La cantautora se estableció en Madrid, España.");
}

#[test]
fn collapses_excess_newlines() {
    let html = "<p>A</p><p>B</p><p>C</p>";
    let out = strip_html(html);
    assert_eq!(out, "A\n\nB\n\nC");
}

// ---------- decode_html_entities ----------

#[test]
fn decodes_numeric_references() {
    assert_eq!(decode_html_entities("caf&#233;"), "café");
    assert_eq!(decode_html_entities("caf&#xE9;"), "café");
    assert_eq!(decode_html_entities("caf&#XE9;"), "café");
    assert_eq!(decode_html_entities("&#169; 2026"), "\u{00A9} 2026");
}

#[test]
fn decodes_accented_named_entities() {
    assert_eq!(
        decode_html_entities("Beyonc&eacute; &amp; M&ouml;tley Cr&uuml;e"),
        "Beyoncé & Mötley Crüe"
    );
    assert_eq!(decode_html_entities("&Eacute;douard"), "Édouard");
}

#[test]
fn decodes_malformed_bare_copy() {
    // The real TiVo credit line: no semicolon, double space
    // (qbz-nix-docs/qobuz-api/page-artist-response.json).
    assert_eq!(
        decode_html_entities("&copy  Mariano Prunes /TiVo"),
        "\u{00A9}  Mariano Prunes /TiVo"
    );
    assert_eq!(
        decode_html_entities("&copy John Book /TiVo"),
        "\u{00A9} John Book /TiVo"
    );
    // Bare form at end of text.
    assert_eq!(decode_html_entities("text &copy"), "text \u{00A9}");
    // Bare &amp / &nbsp / &reg at a boundary.
    assert_eq!(decode_html_entities("Tom &amp Jerry"), "Tom & Jerry");
    assert_eq!(decode_html_entities("QBZ&reg!"), "QBZ\u{00AE}!");
}

#[test]
fn no_false_positives_on_plain_ampersands() {
    // Names not in the table never decode without a semicolon.
    assert_eq!(decode_html_entities("AC&DC"), "AC&DC");
    assert_eq!(decode_html_entities("R&B and Rhythm&Blues"), "R&B and Rhythm&Blues");
    // Allowlisted prefix but no word boundary → untouched.
    assert_eq!(decode_html_entities("&copyright2020"), "&copyright2020");
    assert_eq!(decode_html_entities("&amplifier"), "&amplifier");
    // Unknown entity with semicolon stays literal.
    assert_eq!(decode_html_entities("&unknown;"), "&unknown;");
    // Numeric without semicolon stays literal (documented tradeoff).
    assert_eq!(decode_html_entities("&#169 2026"), "&#169 2026");
    // Trailing lone ampersand.
    assert_eq!(decode_html_entities("fish & chips &"), "fish & chips &");
}

#[test]
fn clean_text_is_untouched_and_decode_is_stable() {
    let clean = "Ya quedó: café, señor — “quotes” & nothing else… ©";
    assert_eq!(decode_html_entities(clean), clean);
    // Idempotence on typical decoded prose (no re-encoding artifacts).
    let once = decode_html_entities("Beyonc&eacute; &copy John &#8212; ok");
    assert_eq!(decode_html_entities(&once), once);
}

#[test]
fn decodes_windows_1252_quirk_range() {
    assert_eq!(decode_html_entities("It&#146;s here"), "It\u{2019}s here");
    assert_eq!(decode_html_entities("&#147;quoted&#148;"), "\u{201C}quoted\u{201D}");
}

#[test]
fn full_pipeline_on_real_bio_tail() {
    // Verbatim shape of the Metallica bio tail in the API sample —
    // the literal \n plus <br /> yield a blank line before the credit.
    let html = "apareció en 2023.\n<br />&copy  Mariano Prunes /TiVo";
    assert_eq!(
        strip_html(html),
        "apareció en 2023.\n\n\u{00A9}  Mariano Prunes /TiVo"
    );
}
