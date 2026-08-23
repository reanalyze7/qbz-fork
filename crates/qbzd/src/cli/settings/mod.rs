// crates/qbzd/src/cli/settings/mod.rs — `qbzd settings show|set`,
// (02-cli-and-api.md §2.2). ALL ⬇ daemon-down capable (§2.4): every verb here
// reads/writes the daemon's REAL stores directly at the daemon roots
// (`AudioSettingsStore::new_at`, `PlaybackPreferencesStore::new_at`,
// `daemon_prefs`) and best-effort nudges a
// running daemon via `POST /api/settings/reload` afterwards
// (`login::nudge_reload` — the same ping-then-reload pattern `login`/`logout`
// already use), never the other way around (§1.1: the CLI holds no daemon
// state of its own). Export/import land in T12.
//
// The canonical dotted-key table (`keys.rs`) is this CLI's own copy of the
// desktop Apply ladder (`qbz/src/settings.rs:87-94`, per-key classification
// `:877-967,1134-1290`; 03-setup-tui.md §4.3's 9-field Reinit list). Per
// 03-setup-tui.md §4.3 (normative): the server's reload response carries NO
// reinit/reload narrative of its own — that classification is composed
// CLIENT-side, which is exactly what [`ApplyClass`] is for.
//
// Value encoding (deliberate, P0 scope): every value round-trips as a plain
// string — `settings show --json`'s values are exactly what `settings set`
// accepts back, key for key. No per-type JSON (bool/number) encoding; that
// would need a second parser on top of the one `set` already has, for a
// convenience no shipped P0/P1 script needs (the CLI is the machine
// interface here, not `/api/settings/reload`'s response body).
//
// Split by responsibility: key table/classification (`keys.rs`), value
// codecs (`codec_*.rs`), store IO (`store.rs`), the validated writer
// (`write*.rs`), the local-nudge helpers (`nudge.rs`), the show/set/config
// verbs (`verbs.rs`), and the T12 export/import flow
// (`export.rs`/`import*.rs`/`reload.rs`/`summary.rs`).

mod codec_bool;
mod codec_playback;
mod codec_value;
mod export;
mod import;
mod import_apply;
mod keys;
mod nudge;
mod reload;
mod store;
mod summary;
mod verbs;
mod verbs_config;
mod write;
mod write_audio_reinit;
mod write_audio_reload;

pub(crate) use keys::ApplyClass;
pub(crate) use write::write_one;

pub use export::export;
pub use import::import;
pub use verbs::{set, show};
pub use verbs_config::{config_path, config_show};

#[cfg(test)]
mod tests_codec;
#[cfg(test)]
mod tests_import;
#[cfg(test)]
mod tests_key_table;
#[cfg(test)]
mod tests_support;
