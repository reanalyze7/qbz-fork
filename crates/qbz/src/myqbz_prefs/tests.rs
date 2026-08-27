use super::actions::coerce_label;
use super::store::Branding;

#[test]
fn coerce_blank_label_yields_default() {
    assert_eq!(coerce_label(""), "My Qoqobuz");
    assert_eq!(coerce_label("   "), "My Qoqobuz");
    assert_eq!(coerce_label("  Tapes  "), "Tapes");
    assert_eq!(coerce_label("Tapes"), "Tapes");
}

#[test]
fn branding_defaults() {
    let b = Branding::default();
    assert_eq!(b.label, "My Qoqobuz");
    assert!(b.icon_path.is_empty());
}

#[test]
fn legacy_json_without_fields_deserializes() {
    let b: Branding = serde_json::from_str("{}").expect("empty object deserializes");
    assert_eq!(b.label, "My Qoqobuz");
    assert!(b.icon_path.is_empty());
}

#[test]
fn missing_icon_path_field_keeps_label() {
    let b: Branding =
        serde_json::from_str(r#"{"label":"Tapes"}"#).expect("partial object deserializes");
    assert_eq!(b.label, "Tapes");
    assert!(b.icon_path.is_empty());
}
