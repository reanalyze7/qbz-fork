use super::fresh_origins_store;

#[test]
fn allowed_origins_store_creates_defaults() {
    let (dir, store) = fresh_origins_store("origins-default");

    let origins = store.get_origins().expect("get origins");
    let names = origins
        .iter()
        .map(|origin| origin.origin.as_str())
        .collect::<Vec<_>>();

    assert_eq!(
        names,
        vec![
            "control.qbz.lol",
            "vicrodh.github.io",
            "www.control.qbz.lol"
        ]
    );
    assert!(origins.iter().all(|origin| origin.is_default));
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn allowed_origins_add_normalizes_and_remove_works() {
    let (dir, store) = fresh_origins_store("origins-add-remove");

    let origin = store
        .add_origin("  EXAMPLE.com  ")
        .expect("add custom origin");
    assert_eq!(origin.origin, "example.com");
    assert!(!origin.is_default);
    assert!(store.is_origin_allowed("example.com"));

    store.remove_origin(origin.id).expect("remove origin");
    assert!(!store.is_origin_allowed("example.com"));
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn allowed_origins_rejects_empty_and_duplicate_origins() {
    let (dir, store) = fresh_origins_store("origins-invalid");

    assert!(store.add_origin("   ").is_err());
    store.add_origin("example.com").expect("add origin");
    let duplicate = store.add_origin("example.com").expect_err("duplicate");

    assert_eq!(duplicate, "Origin already exists");
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn allowed_origins_restore_defaults_adds_missing_defaults() {
    let (dir, store) = fresh_origins_store("origins-restore");
    let origin = store.add_origin("example.com").expect("add origin");
    let default_id = store
        .get_origins()
        .expect("get origins")
        .into_iter()
        .find(|entry| entry.origin == "control.qbz.lol")
        .expect("find default")
        .id;
    store.remove_origin(default_id).expect("remove default");

    store.restore_defaults().expect("restore defaults");
    let origins = store.get_origins().expect("get origins");

    assert!(origins.iter().any(|entry| entry.origin == "control.qbz.lol"));
    assert!(origins.iter().any(|entry| entry.id == origin.id));
    let _ = std::fs::remove_dir_all(dir);
}
