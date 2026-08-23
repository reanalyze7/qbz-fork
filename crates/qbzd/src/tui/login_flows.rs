use std::io::Write;

use ratatui::DefaultTerminal;
use tokio::runtime::Handle;

use super::app::App;
use super::strings;

/// Suspend the alt-screen and run the T5 browser-login engine on the plain
/// terminal (it prints the OAuth URL, waits 300 s, and prints the SSH-forward
/// hint on failure), then resume the TUI. Deliberate divergence from the §3.1
/// in-panel countdown — see the task report.
pub(super) fn run_browser_login(terminal: &mut DefaultTerminal, app: &mut App, handle: &Handle) {
    ratatui::restore();
    println!("{}\n", strings::ACCOUNT_BROWSER_HANDOFF);
    let roots = app.roots().clone();
    let result = handle.block_on(async { crate::login::login_browser(&roots, None).await });
    *terminal = ratatui::init();
    let mapped = result
        .map(|session| (session.email, Some(session.subscription_label)))
        .map_err(|e| e.to_string());
    app.after_browser_login(mapped);
}

/// Which scrobbler provider a suspended connect flow targets.
pub(super) enum ScrobbleProvider {
    Lastfm,
    Listenbrainz,
}

/// Suspend the alt-screen and run the scrobbler connect flow on the plain
/// terminal — the SAME methodology as the browser login. Last.fm prints an
/// authorize URL and waits for Enter (the CLI verb owns that); ListenBrainz
/// prompts for a pasted user token here, then hands it to the CLI verb. Both
/// write the canonical ScrobblerSettingsStore, so the screen is reloaded on
/// resume.
pub(super) fn run_scrobble_login(
    terminal: &mut DefaultTerminal,
    app: &mut App,
    handle: &Handle,
    provider: ScrobbleProvider,
) {
    ratatui::restore();
    let roots = app.roots().clone();
    match provider {
        ScrobbleProvider::Lastfm => {
            println!("{}", strings::SCROBBLE_LASTFM_HANDOFF);
            let _ = handle.block_on(async { crate::cli::scrobble::login_lastfm(None, &roots).await });
        }
        ScrobbleProvider::Listenbrainz => {
            println!("{}", strings::SCROBBLE_LISTENBRAINZ_HANDOFF);
            print!("token: ");
            let _ = std::io::stdout().flush();
            let mut token = String::new();
            if std::io::stdin().read_line(&mut token).is_ok() {
                let token = token.trim().to_string();
                if token.is_empty() {
                    println!("no token entered — skipped.");
                } else {
                    let _ = handle
                        .block_on(async { crate::cli::scrobble::login_listenbrainz(None, token, &roots).await });
                }
            }
        }
    }
    println!("{}", strings::SCROBBLE_RETURN_HINT);
    let mut line = String::new();
    let _ = std::io::stdin().read_line(&mut line);
    *terminal = ratatui::init();
    app.refresh_scrobbler();
}
