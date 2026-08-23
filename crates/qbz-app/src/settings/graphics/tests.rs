use super::*;

fn unique_test_dir(name: &str) -> std::path::PathBuf {
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("qbz-app-{name}-{}-{nonce}", std::process::id()))
}

#[test]
fn graphics_settings_default_values_are_stable() {
    let settings = GraphicsSettings::default();

    assert!(settings.hardware_acceleration);
    assert!(!settings.force_x11);
    assert_eq!(settings.gdk_scale, None);
    assert_eq!(settings.gdk_dpi_scale, None);
    assert_eq!(settings.gsk_renderer, None);
    assert_eq!(settings.preferred_gpu, "auto");
    assert!(!settings.nvidia_compat_mode);
}

#[test]
fn graphics_settings_store_returns_defaults() {
    let dir = unique_test_dir("graphics-default");
    let store = GraphicsSettingsStore::new_at(&dir).expect("open store");

    let settings = store.get_settings().expect("get settings");

    assert!(settings.hardware_acceleration);
    assert!(!settings.force_x11);
    assert_eq!(settings.preferred_gpu, "auto");
    assert!(!settings.nvidia_compat_mode);
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn graphics_settings_persist_all_fields() {
    let dir = unique_test_dir("graphics-persist");
    {
        let store = GraphicsSettingsStore::new_at(&dir).expect("open store");
        store
            .set_hardware_acceleration(false)
            .expect("set hardware acceleration");
        store.set_force_x11(true).expect("set force x11");
        store
            .set_gdk_scale(Some("2".to_string()))
            .expect("set gdk scale");
        store
            .set_gdk_dpi_scale(Some("0.5".to_string()))
            .expect("set gdk dpi scale");
        store
            .set_gsk_renderer(Some("ngl".to_string()))
            .expect("set gsk renderer");
        store
            .set_preferred_gpu("discrete")
            .expect("set preferred gpu");
        store
            .set_nvidia_compat_mode(true)
            .expect("set nvidia compat mode");
    }

    let reopened = GraphicsSettingsStore::new_at(&dir).expect("reopen store");
    let settings = reopened.get_settings().expect("get settings");

    assert!(!settings.hardware_acceleration);
    assert!(settings.force_x11);
    assert_eq!(settings.gdk_scale.as_deref(), Some("2"));
    assert_eq!(settings.gdk_dpi_scale.as_deref(), Some("0.5"));
    assert_eq!(settings.gsk_renderer.as_deref(), Some("ngl"));
    assert_eq!(settings.preferred_gpu, "discrete");
    assert!(settings.nvidia_compat_mode);
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn graphics_settings_reopen_does_not_overwrite_existing_row() {
    let dir = unique_test_dir("graphics-no-overwrite");
    {
        let store = GraphicsSettingsStore::new_at(&dir).expect("open store");
        store
            .set_hardware_acceleration(false)
            .expect("set hardware acceleration");
        store
            .set_preferred_gpu("software")
            .expect("set preferred gpu");
    }

    let reopened = GraphicsSettingsStore::new_at(&dir).expect("reopen store");
    let settings = reopened.get_settings().expect("get settings");

    assert!(!settings.hardware_acceleration);
    assert_eq!(settings.preferred_gpu, "software");
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn graphics_settings_readonly_opens_existing_db() {
    let dir = unique_test_dir("graphics-readonly");
    let db_path = dir.join("graphics_settings.db");
    {
        let store = GraphicsSettingsStore::new_at(&dir).expect("open store");
        store
            .set_preferred_gpu("integrated")
            .expect("set preferred gpu");
    }

    let readonly =
        GraphicsSettingsStore::new_readonly_at_path(&db_path).expect("open readonly store");
    let settings = readonly.get_settings().expect("get readonly settings");

    assert_eq!(settings.preferred_gpu, "integrated");
    let _ = std::fs::remove_dir_all(dir);
}
