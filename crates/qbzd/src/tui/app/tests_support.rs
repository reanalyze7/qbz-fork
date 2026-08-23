// crates/qbzd/src/tui/app/tests_support.rs — shared test fixtures: a bare
// `App` constructed directly (no I/O, unlike `App::new`) and a TestBackend
// render helper, used by `tests_focus.rs` and `tests_layout.rs`.

use std::path::PathBuf;

use ratatui::backend::TestBackend;
use ratatui::Terminal;

use qbz_audio::settings::AudioSettings;

use crate::paths::ProfileRoots;
use crate::tui::screens::account::{AccountState, AuthSnapshot};
use crate::tui::screens::audio::AudioState;
use crate::tui::screens::bundle::BundleState;
use crate::tui::screens::network::NetworkState;
use crate::tui::screens::playback::PlaybackState;
use crate::tui::screens::scrobbler::ScrobblerState;
use crate::tui::screens::wizard::WizardState;

use super::messages::Screen;
use super::messages_worker::{Active, Overlay};
use super::nav::{section_index, Focus};
use super::state::App;

pub(super) fn bare_app(section: Screen, focus: Focus) -> App {
    let (tx, rx) = std::sync::mpsc::channel();
    let rt = tokio::runtime::Builder::new_current_thread().build().unwrap();
    let active = match section {
        Screen::Account => Active::Account(AccountState::new(AuthSnapshot::default())),
        Screen::Audio => Active::Audio(AudioState::new(&AudioSettings::default())),
        Screen::Playback => Active::Playback(PlaybackState::new(
            "hires_plus",
            true,
            &AudioSettings::default(),
            &qbz_app::settings::playback::PlaybackPreferences::default(),
        )),
        Screen::Network => Active::Network(NetworkState::new(&crate::config::QbzdConfig::default(), Vec::new())),
        Screen::Bundle => Active::Bundle(BundleState::new(false)),
        Screen::Wizard => Active::Wizard(WizardState::new()),
        Screen::Scrobbler => Active::Scrobbler(ScrobblerState::new(&ProfileRoots {
            config: PathBuf::from("/nonexistent"),
            data: PathBuf::from("/nonexistent"),
            cache: PathBuf::from("/nonexistent"),
        })),
    };
    App {
        roots: ProfileRoots {
            config: PathBuf::from("/nonexistent"),
            data: PathBuf::from("/nonexistent"),
            cache: PathBuf::from("/nonexistent"),
        },
        handle: rt.handle().clone(),
        tx,
        rx,
        active,
        active_section: section,
        nav_cursor: section_index(section),
        focus,
        status: None,
        reachable: false,
        auth: AuthSnapshot::default(),
        overlay: Overlay::None,
        busy: None,
        busy_tick: 0,
        should_quit: false,
        leave_after_save: None,
    }
}

pub(super) fn render(app: &App, w: u16, h: u16) -> String {
    let mut terminal = Terminal::new(TestBackend::new(w, h)).unwrap();
    terminal.draw(|f| app.draw(f)).unwrap();
    let buf = terminal.backend().buffer().clone();
    let mut out = String::new();
    for y in 0..h {
        for x in 0..w {
            out.push_str(buf[(x, y)].symbol());
        }
        out.push('\n');
    }
    out
}
