use std::path::PathBuf;

use super::*;

fn scratch_dir(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "qbzd-paths-test-{name}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ))
}

#[test]
fn config_override_uses_parent_dir_and_creates_it_0700() {
    let dir = scratch_dir("config-override");
    let _ = std::fs::remove_dir_all(&dir);
    let config_file = dir.join("nested").join("qbzd.toml");

    let roots = ProfileRoots::resolve(Some(&config_file), None);

    assert_eq!(roots.config, dir.join("nested"));
    let meta = std::fs::metadata(&roots.config).expect("config dir created");
    assert!(meta.is_dir());
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(meta.permissions().mode() & 0o777, 0o700);
    }
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn data_root_override_places_cache_under_it_not_beside_it() {
    let data_dir = scratch_dir("data-override");

    let roots = ProfileRoots::resolve(None, Some(&data_dir));

    assert_eq!(roots.data, data_dir);
    assert_eq!(roots.cache, data_dir.join("cache"));
    // Never derived as a sibling of data_root.
    assert_ne!(roots.cache, data_dir.parent().unwrap().join("qbzd-cache"));
}

#[test]
fn defaults_resolve_under_xdg_roots_without_touching_real_home() {
    // SAFETY: single-threaded within this test; original values restored
    // before returning so no other test observes the override. This test
    // must never touch the real developer $HOME/.config etc.
    let base = scratch_dir("xdg-defaults");
    let xdg_config = base.join("config");
    let xdg_data = base.join("data");
    let xdg_cache = base.join("cache");

    let saved = [
        ("XDG_CONFIG_HOME", std::env::var("XDG_CONFIG_HOME").ok()),
        ("XDG_DATA_HOME", std::env::var("XDG_DATA_HOME").ok()),
        ("XDG_CACHE_HOME", std::env::var("XDG_CACHE_HOME").ok()),
    ];
    std::env::set_var("XDG_CONFIG_HOME", &xdg_config);
    std::env::set_var("XDG_DATA_HOME", &xdg_data);
    std::env::set_var("XDG_CACHE_HOME", &xdg_cache);

    let roots = ProfileRoots::resolve(None, None);

    for (key, prev) in saved {
        match prev {
            Some(v) => std::env::set_var(key, v),
            None => std::env::remove_var(key),
        }
    }

    assert_eq!(roots.config, xdg_config.join("qbzd"));
    assert_eq!(roots.data, xdg_data.join("qbzd"));
    assert_eq!(roots.cache, xdg_cache.join("qbzd"));
    let _ = std::fs::remove_dir_all(&base);
}
