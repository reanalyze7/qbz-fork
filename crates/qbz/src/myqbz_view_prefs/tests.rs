use super::Prefs;

#[test]
fn defaults_match_spec_18() {
    let p = Prefs::default();
    assert_eq!(p.view_mode, "list");
    assert_eq!(p.sort_by, "position");
    assert_eq!(p.sort_dir, "asc");
    assert_eq!(p.type_filter, "all");
    assert!(!p.src_qobuz && !p.src_local);
}

#[test]
fn legacy_json_without_fields_deserializes_to_defaults() {
    let p: Prefs = serde_json::from_str("{}").expect("empty object deserializes");
    assert_eq!(p, Prefs::default());
}

#[test]
fn partial_json_keeps_present_fields() {
    let p: Prefs = serde_json::from_str(r#"{"view_mode":"grid","src_local":true}"#)
        .expect("partial object deserializes");
    assert_eq!(p.view_mode, "grid");
    assert!(p.src_local);
    // Absent fields fall back to defaults.
    assert_eq!(p.sort_by, "position");
    assert_eq!(p.type_filter, "all");
    assert!(!p.src_qobuz);
}
