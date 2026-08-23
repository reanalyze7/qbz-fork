// crates/qbzd/src/cli/settings/tests_key_table.rs — KEY_TABLE integrity,
// the Usage/Io exit-code split, and the show/set round-trip.

use super::keys::KEY_TABLE;
use super::store::read_all;
use super::tests_support::{cleanup, scratch_roots};
use super::write::{write_one, SetError};

#[test]
fn unknown_key_is_rejected_and_lists_every_valid_key() {
    let roots = scratch_roots("unknown-key");
    let err = write_one(&roots, "audio.bogus", "x").unwrap_err();
    // 02 §1.3: an unknown key is a USAGE mistake (exit 2), never Io.
    assert!(matches!(err, SetError::Usage(_)), "{err:?}");
    assert!(err.message().contains("unknown setting key 'audio.bogus'"), "{err}");
    for (k, _) in KEY_TABLE {
        assert!(err.message().contains(k), "missing '{k}' from the listed keys:\n{err}");
    }
    cleanup(&roots);
}

#[test]
fn invalid_value_for_a_known_key_is_a_usage_error_not_io() {
    // 02 §1.3: a bad value for a KNOWN key is still exit 2 (usage), same
    // class as an unknown key — never Io, which is reserved for a store
    // that failed to open/write after the value parsed fine.
    let roots = scratch_roots("bad-value");
    let err = write_one(&roots, "audio.exclusive_mode", "maybe").unwrap_err();
    assert!(matches!(err, SetError::Usage(_)), "{err:?}");
    cleanup(&roots);
}

#[test]
fn store_open_failure_is_an_io_error_not_usage() {
    // 02 §1.3: the key classifies and the value parses fine (exclusive_mode
    // is a plain bool) — a store that then fails to even OPEN (here: the
    // data root is blocked by a plain file, so `create_dir_all` fails) is
    // exit 1 (Io), never the usage exit 2.
    let roots = scratch_roots("store-open-blocked");
    std::fs::create_dir_all(roots.data.parent().unwrap()).unwrap();
    std::fs::write(&roots.data, b"not a directory").unwrap();
    let err = write_one(&roots, "audio.exclusive_mode", "true").unwrap_err();
    assert!(matches!(err, SetError::Io(_)), "{err:?}");
    cleanup(&roots);
}

#[test]
fn key_table_has_no_duplicate_keys() {
    let mut seen = std::collections::HashSet::new();
    for (k, _) in KEY_TABLE {
        assert!(seen.insert(*k), "duplicate canonical key: {k}");
    }
}

#[test]
fn show_json_round_trips_into_set_for_every_canonical_key() {
    // The brief's Step 1 property: `settings show --json` includes every
    // key `settings set` accepts, AND the value it reports for a key is
    // itself a valid `set` input for that same key (a real functional
    // round-trip against temp on-disk stores, not just "same key names").
    let roots = scratch_roots("roundtrip");
    let values = read_all(&roots).expect("read_all opens fresh stores with defaults");
    assert_eq!(values.len(), KEY_TABLE.len(), "read_all must cover every canonical key");
    for (key, value) in &values {
        // The one documented exception (03-setup-tui.md §3.3.2): a fresh
        // (or desktop-imported) store's `quality_fallback_behavior`
        // column defaults to `"ask"`, which `set` correctly REJECTS —
        // "the daemon has no one to ask... the TUI never writes ask".
        // `show` must still be able to READ it; the round-trip property
        // is deliberately one-way for this single value.
        if *key == "audio.quality_fallback_behavior" && value == "ask" {
            continue;
        }
        write_one(&roots, key, value)
            .unwrap_or_else(|e| panic!("show's own value for '{key}' ('{value}') was rejected by set: {e}"));
    }
    cleanup(&roots);
}

#[test]
fn set_then_show_persists_across_a_fresh_store_open() {
    let roots = scratch_roots("persist");
    write_one(&roots, "audio.backend", "alsa").expect("set backend");
    write_one(&roots, "audio.exclusive_mode", "true").expect("set exclusive");
    write_one(&roots, "playback.quality", "cd").expect("set quality");
    write_one(&roots, "playback.autoplay", "track_only").expect("set autoplay");

    let values: std::collections::HashMap<_, _> = read_all(&roots).unwrap().into_iter().collect();
    assert_eq!(values["audio.backend"], "alsa");
    assert_eq!(values["audio.exclusive_mode"], "true");
    assert_eq!(values["playback.quality"], "cd");
    assert_eq!(values["playback.autoplay"], "track_only");
    cleanup(&roots);
}
