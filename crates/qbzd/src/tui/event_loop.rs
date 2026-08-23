use std::time::Duration;

use ratatui::crossterm::event::{self, Event, KeyEventKind};
use ratatui::DefaultTerminal;
use tokio::runtime::Handle;

use super::app::{App, LoopCmd};
use super::login_flows::{run_browser_login, run_scrobble_login, ScrobbleProvider};

/// Poll cadence — also the spinner tick while a worker runs (§5.5). No I/O here;
/// the loop only redraws and drains the worker channel.
const TICK: Duration = Duration::from_millis(120);

pub(super) fn event_loop(terminal: &mut DefaultTerminal, app: &mut App, handle: &Handle) -> i32 {
    loop {
        if terminal.draw(|f| app.draw(f)).is_err() {
            return 1;
        }
        app.drain_worker();
        if app.busy() {
            app.busy_tick = app.busy_tick.wrapping_add(1);
        }

        if event::poll(TICK).unwrap_or(false) {
            match event::read() {
                Ok(Event::Key(key)) if key.kind == KeyEventKind::Press => match app.on_key(key) {
                    LoopCmd::None => {}
                    LoopCmd::BrowserLogin => run_browser_login(terminal, app, handle),
                    LoopCmd::ScrobbleLastfm => {
                        run_scrobble_login(terminal, app, handle, ScrobbleProvider::Lastfm)
                    }
                    LoopCmd::ScrobbleListenbrainz => {
                        run_scrobble_login(terminal, app, handle, ScrobbleProvider::Listenbrainz)
                    }
                },
                Ok(_) => {} // resize/mouse/etc. — the next draw re-lays-out (§5.4)
                Err(_) => return 1,
            }
        }

        if app.should_quit() {
            return 0;
        }
    }
}
