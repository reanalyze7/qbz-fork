// crates/qbzd/src/cli/scrobble/ — the `qbzd scrobble …` verbs (CONSOLE ext).
// Connect Last.fm / ListenBrainz and manage scrobbling, using the SAME
// methodology as `qbzd login`: Last.fm prints an authorize URL and exchanges
// the token after the user approves; ListenBrainz takes a pasted user token
// (like `login --token`).
//
// Credentials land in the CANONICAL scrobbler store
// (`qbz_app::settings::scrobblers::ScrobblerSettingsStore`, SQLite at the daemon
// data root) — the SAME store the desktop uses and the one the settings bundle
// export/import already carries. A running daemon is nudged to reload so the
// scrobble-on-play driver picks up the new credentials. These are LOCAL,
// daemon-down-capable operations, like `login`/`settings set`.
mod connect;
mod status;
mod support;

pub use connect::{login_lastfm, login_listenbrainz};
pub use status::{set_enabled, status};
