use clap::Subcommand;

use super::sub::{FavCmd, PlaylistCmd, QueueCmd, RecoCmd};
use super::scrobble_sub::{ConfigCmd, ScrobbleCmd};
use super::settings_sub::SettingsCmd;

#[derive(Subcommand)]
pub enum Cmd {
    /// Run the daemon in the foreground (systemd ExecStart)
    Run,
    /// Log in to Qobuz (one-shot browser listener; --paste; --token)
    Login {
        #[arg(long)] callback_host: Option<String>,
        #[arg(long)] paste: bool,
        #[arg(long)] token: Option<String>,
    },
    Logout,
    /// Interactive configurator (six screens)
    Setup,
    /// Composite daemon diagnostic
    Status { #[arg(long)] json: bool },
    Ping   { #[arg(long)] json: bool },
    /// One-line now-playing
    Now    { #[arg(long)] json: bool },
    /// Stream live daemon events (SSE); default = newline-delimited JSON
    Watch  { #[arg(long)] raw: bool },
    /// Search Qobuz — top hits with ids (--ids pipes into `queue add -`)
    Search {
        query: String,
        #[arg(long = "type", default_value = "all")] kind: String,
        #[arg(long, default_value_t = 20)] limit: u32,
        #[arg(long, default_value_t = 0)] offset: u32,
        #[arg(long)] ids: bool,
        #[arg(long)] json: bool,
    },
    /// Album page — tracklist with ids
    Album {
        id: String,
        #[arg(long)] suggest: bool,
        #[arg(long)] ids: bool,
        #[arg(long)] json: bool,
    },
    /// Artist page (default), --top tracks, or --albums grid
    Artist {
        id: u64,
        #[arg(long)] top: bool,
        #[arg(long)] albums: bool,
        #[arg(long, default_value_t = 20)] limit: u32,
        #[arg(long)] ids: bool,
        #[arg(long)] json: bool,
    },
    /// Similar artists or albums: artist:ID | album:ID
    Similar {
        selector: String,
        #[arg(long, default_value_t = 20)] limit: u32,
        #[arg(long)] ids: bool,
        #[arg(long)] json: bool,
    },
    /// For-You suggestions — seeds from the queue, or --seed <ID,ID>|-
    Suggest {
        #[arg(long)] seed: Option<String>,
        #[arg(long, default_value_t = 20)] limit: u32,
        #[arg(long)] ids: bool,
        #[arg(long)] json: bool,
    },
    /// Discover rails: index | most-streamed | new-releases | press-awards |
    /// qobuzissims | album-of-the-week | ideal-discography | playlists | tags |
    /// release-watch (replicate Discover; no recommendations)
    Discover {
        section: Option<String>,
        #[arg(long)] genre: Option<String>,
        #[arg(long)] tag: Option<String>,
        #[arg(long = "release-type")] release_type: Option<String>,
        #[arg(long = "type")] kind: Option<String>,
        #[arg(long, default_value_t = 20)] limit: u32,
        #[arg(long)] ids: bool,
        #[arg(long)] json: bool,
    },
    /// Recommendations: playlist <ID> (Suggested Songs — no history needed)
    Reco { #[command(subcommand)] cmd: RecoCmd },
    /// Favorites: list | add | remove
    Fav { #[command(subcommand)] cmd: FavCmd },
    /// Playlists: list | show
    Playlist { #[command(subcommand)] cmd: PlaylistCmd },
    /// Shuffle: on | off | toggle (bare = toggle)
    Shuffle { mode: Option<String> },
    /// Repeat: off | all | one
    Repeat  { mode: String },
    /// Current-track cover art — prints the URL, or --save PATH downloads it
    Art { #[arg(long)] save: Option<String> },
    /// Resolve a Qobuz URL to a kind:ID token (pure, no daemon)
    Resolve { url: String },
    /// Resume (bare) or play content: album:ID | track:ID | artist:ID | playlist:ID | URL
    Play   { content: Option<String> },
    Pause, Toggle, Stop, Next, Prev,
    /// Absolute secs, +N/-N, or mm:ss
    Seek   { position: String },
    /// Bare = read; 0-100, +N, -N
    Volume { value: Option<String>, #[arg(long)] json: bool },
    /// Bare = toggle
    Mute   { state: Option<String> },
    Queue    { #[command(subcommand)] cmd: QueueCmd },
    Settings { #[command(subcommand)] cmd: SettingsCmd },
    /// Scrobbling: login (Last.fm / ListenBrainz) · status · enable · disable
    Scrobble { #[command(subcommand)] cmd: ScrobbleCmd },
    Config   { #[command(subcommand)] cmd: ConfigCmd },
    Version  { #[arg(long)] json: bool },
    /// Generate an init service file (systemd/openrc/runit); prints to stdout
    Service {
        /// systemd | openrc | runit (auto-detected from the running init if omitted)
        init: Option<String>,
        /// User the service runs as (default: current user)
        #[arg(long)] user: Option<String>,
        /// Path to the qbzd binary (default: this executable, else /usr/bin/qbzd)
        #[arg(long)] bin: Option<String>,
        /// systemd: emit a SYSTEM unit (runs as --user) instead of a user unit
        #[arg(long)] system: bool,
    },
    /// Shell completions (hidden; packaged by T14)
    #[command(hide = true)]
    Completions { shell: clap_complete::Shell },
}
