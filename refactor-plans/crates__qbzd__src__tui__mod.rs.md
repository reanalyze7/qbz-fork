# crates/qbzd/src/tui/mod.rs (154 lines)

## Summary
`qbzd setup` TUI entry point: submodule declarations, the non-tty guard +
`run`/`run_sync` bootstrap (ratatui init/restore + spawn_blocking onto a
dedicated thread), the main `event_loop` (draw/poll/dispatch keys), and two
"suspend the alt-screen, run a blocking login flow on the plain terminal,
resume" helpers (`run_browser_login`, `run_scrobble_login` + its
`ScrobbleProvider` enum).

## Proposed split
By responsibility (bootstrap/lifecycle vs the event loop vs the
suspend-for-login helpers):

- `tui/mod.rs` (~50 lines) — module doc, `pub mod`/`mod` declarations, `use`
  block, the `TICK` constant, and the public `run` entry point (lines 37–49)
  + `run_sync` (lines 51–57) — the minimal bootstrap surface, re-exporting
  `event_loop` from the new submodule below.
- `tui/event_loop.rs` (~35 lines) — `event_loop` (lines 59–90): the
  draw/drain-worker/poll/dispatch-key loop. Small enough it could stay in
  `mod.rs`, but separating it isolates the hot loop from the one-shot login
  helpers below, which is the natural seam here.
- `tui/login_flows.rs` (~75 lines) — `run_browser_login` (lines 96–106),
  `ScrobbleProvider` enum (lines 109–112), `run_scrobble_login` (lines
  120–154): the two "suspend alt-screen, block_on an async login, resume"
  helpers, which share the same restore/init-terminal pattern and are only
  called from `event_loop`'s `LoopCmd` match arms.

## Re-export surface
`tui/mod.rs` stays the module callers use (`crate::tui::run(roots).await` from
wherever `qbzd setup` is dispatched, likely `main.rs` or a CLI command
module) — its public surface (`pub async fn run`) is unchanged; `event_loop`/
`run_browser_login`/`run_scrobble_login`/`ScrobbleProvider` all become
private-to-crate items in the new submodules, referenced from `mod.rs` via
`use event_loop::event_loop;` / `use login_flows::{run_browser_login,
run_scrobble_login};` (or kept `pub(crate)` if `app.rs`/`screens` need them
too — check for that before assuming module-private is safe).
Also re-exports `pub mod app`, `pub mod clipboard`, `pub mod strings`, `pub
mod theme`, `pub mod widgets`, `pub mod wizard_core`, and `mod screens` —
these are untouched by this split.

## Coupling / watch out
- `run_sync` calls `event_loop(&mut terminal, &mut app, &handle)` sandwiched
  between `ratatui::init()` and `ratatui::restore()` — this pairing is
  load-bearing (panic-hook installation + terminal restore-on-exit per the
  file's own header comment); don't let the split obscure that `run_sync`
  MUST bracket every call to `event_loop`.
- `event_loop`'s `LoopCmd::BrowserLogin`/`ScrobbleLastfm`/`ScrobbleListenbrainz`
  match arms call straight into `run_browser_login`/`run_scrobble_login` from
  inside the loop body — these need `use crate::tui::login_flows::{...};` (or
  equivalent) once split; the `terminal`/`app`/`handle` references passed
  through are already `&mut`/`&` params, so no ownership issues, just import
  wiring.
- Both login helpers do the same "restore → run blocking login → re-init
  terminal → update app state" dance with slightly different internals
  (browser opens a URL and waits; scrobble prompts differently per provider)
  — keep them together in `login_flows.rs` since they're conceptually one
  category ("suspend TUI for a blocking auth flow") even though not much code
  is literally shared.
- `strings::NON_TTY_ERROR`, `strings::ACCOUNT_BROWSER_HANDOFF`,
  `strings::SCROBBLE_*_HANDOFF`/`SCROBBLE_RETURN_HINT` are read from the
  sibling `strings` module by both `mod.rs` (non-tty guard) and
  `login_flows.rs` — no change needed, `strings` is already its own module.

## Verify after split
- `cargo build -p qbzd`.
- `cargo test -p qbzd` if there are TUI-related tests (check `tui/app.rs` and
  friends for existing coverage).
- Manual smoke test: run `qbzd setup` in an actual terminal, verify normal
  key handling still works, trigger a browser-login flow and a scrobble
  (Last.fm + ListenBrainz) connect flow to confirm the suspend/resume
  round-trip still restores the TUI correctly afterward.
