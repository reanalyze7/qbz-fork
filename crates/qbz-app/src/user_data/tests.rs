use super::UserDataPaths;

#[test]
fn starts_without_active_user() {
    let paths = UserDataPaths::new();

    assert_eq!(paths.current_user_id(), None);
    assert!(paths.user_data_dir().is_err());
    assert!(paths.user_cache_dir().is_err());
}

#[test]
fn set_and_clear_user_updates_current_user() {
    let paths = UserDataPaths::new();

    paths.set_user(10385965);
    assert_eq!(paths.current_user_id(), Some(10385965));

    paths.clear_user();
    assert_eq!(paths.current_user_id(), None);
}

#[test]
fn user_dirs_are_scoped_by_user_id() {
    let paths = UserDataPaths::new();
    paths.set_user(42);

    let data_dir = paths.user_data_dir().expect("data dir");
    let cache_dir = paths.user_cache_dir().expect("cache dir");

    assert!(data_dir.ends_with("qbz/users/42"));
    assert!(cache_dir.ends_with("qbz/users/42"));
}

#[test]
fn global_dirs_are_scoped_to_qbz() {
    let data_dir = UserDataPaths::global_data_dir().expect("global data dir");
    let cache_dir = UserDataPaths::global_cache_dir().expect("global cache dir");

    assert!(data_dir.ends_with("qbz"));
    assert!(cache_dir.ends_with("qbz"));
}
