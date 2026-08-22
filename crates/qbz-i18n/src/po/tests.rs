use super::Catalog;

const SAMPLE: &str = r#"
msgid ""
msgstr ""
"Language: es\n"
"Plural-Forms: nplurals=2; plural=(n != 1);\n"

# a simple singular
msgid "Play"
msgstr "Reproducir"

# an empty translation -> no translation
msgid "Pause"
msgstr ""

# a plural entry
msgid "{} track"
msgid_plural "{} tracks"
msgstr[0] "{} pista"
msgstr[1] "{} pistas"
"#;

#[test]
fn parses_singular() {
    let cat = Catalog::parse("es", SAMPLE);
    assert_eq!(cat.get("Play"), Some("Reproducir"));
}

#[test]
fn empty_msgstr_is_none() {
    let cat = Catalog::parse("es", SAMPLE);
    assert_eq!(cat.get("Pause"), None);
}

#[test]
fn missing_msgid_is_none() {
    let cat = Catalog::parse("es", SAMPLE);
    assert_eq!(cat.get("Stop"), None);
}

#[test]
fn parses_plural_forms() {
    let cat = Catalog::parse("es", SAMPLE);
    assert_eq!(cat.get_plural("{} track", 0), Some("{} pista"));
    assert_eq!(cat.get_plural("{} track", 1), Some("{} pistas"));
    assert_eq!(cat.get_plural("{} track", 2), None);
}

#[test]
fn reads_nplurals_from_header() {
    let cat = Catalog::parse("es", SAMPLE);
    assert_eq!(cat.nplurals(), 2);
    // (n != 1): 1 -> form 0, else form 1.
    assert_eq!(cat.plural_rule().index(1), 0);
    assert_eq!(cat.plural_rule().index(3), 1);
}

#[test]
fn handles_multiline_continuation_and_escapes() {
    let po = r#"
msgid "greeting"
msgstr "Hello "
"world\nLine\ttab \"q\" back\\slash"
"#;
    let cat = Catalog::parse("en", po);
    assert_eq!(
        cat.get("greeting"),
        Some("Hello world\nLine\ttab \"q\" back\\slash")
    );
}
