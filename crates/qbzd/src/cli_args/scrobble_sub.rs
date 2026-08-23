use clap::Subcommand;

#[derive(Subcommand)]
pub enum ScrobbleCmd {
    /// Connect a provider
    Login { #[command(subcommand)] cmd: ScrobbleLoginCmd },
    /// Connection + enabled state
    Status,
    /// Stop scrobbling to a provider (keeps credentials)
    Disable { provider: String },
    /// Resume scrobbling to a provider
    Enable { provider: String },
}

#[derive(Subcommand)]
pub enum ScrobbleLoginCmd {
    /// Last.fm web auth (prints a URL to approve)
    Lastfm,
    /// ListenBrainz user token (from listenbrainz.org/settings)
    Listenbrainz { #[arg(long)] token: String },
}

// The tokenless default has no rotation verb (02 §3.1.2): `config` is just
// path|show. Rotating the opt-in [server] token = edit qbzd.toml + restart.
#[derive(Subcommand)]
pub enum ConfigCmd { Path, Show { #[arg(long)] json: bool } }
