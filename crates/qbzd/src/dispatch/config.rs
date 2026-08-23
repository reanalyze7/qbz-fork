use crate::cli_args::{ConfigCmd, ScrobbleCmd, ScrobbleLoginCmd, SettingsCmd};
use crate::roots::login_roots;
use crate::{cli, tui};

pub async fn settings(host: Option<String>, cmd: SettingsCmd) -> i32 {
    let roots = login_roots();
    match cmd {
        SettingsCmd::Show { json } => cli::settings::show(json, &roots),
        SettingsCmd::Set { key, value } => cli::settings::set(&roots, &key, &value),
        SettingsCmd::Export { file, from, include_auth } => {
            cli::settings::export(&roots, file, &from, include_auth)
        }
        SettingsCmd::Import { file, include_auth, trust_dsd, remap, dry_run } => {
            let _ = host;
            cli::settings::import(&roots, &file, include_auth, trust_dsd, &remap, dry_run).await
        }
    }
}

pub async fn scrobble(host: Option<String>, cmd: ScrobbleCmd) -> i32 {
    let roots = login_roots();
    match cmd {
        ScrobbleCmd::Login { cmd } => match cmd {
            ScrobbleLoginCmd::Lastfm => cli::scrobble::login_lastfm(host, &roots).await,
            ScrobbleLoginCmd::Listenbrainz { token } => {
                cli::scrobble::login_listenbrainz(host, token, &roots).await
            }
        },
        ScrobbleCmd::Status => cli::scrobble::status(&roots),
        ScrobbleCmd::Disable { provider } => {
            cli::scrobble::set_enabled(host, provider, false, &roots).await
        }
        ScrobbleCmd::Enable { provider } => {
            cli::scrobble::set_enabled(host, provider, true, &roots).await
        }
    }
}

pub fn config(cmd: ConfigCmd) -> i32 {
    let roots = login_roots();
    match cmd {
        ConfigCmd::Path => cli::settings::config_path(&roots),
        ConfigCmd::Show { json } => cli::settings::config_show(json, &roots),
    }
}

pub async fn setup() -> i32 {
    // The setup TUI edits the daemon's REAL stores at the daemon roots,
    // honoring a `qbzd.toml` `data_root` override exactly like `run`.
    let roots = login_roots();
    tui::run(roots).await
}
