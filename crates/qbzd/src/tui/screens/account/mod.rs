// crates/qbzd/src/tui/screens/account/ — the Account screen (03 §3.1).
//
// Status + inline auth. All auth work reuses the T5 engine (login.rs) — the TUI
// adds zero auth logic. Token paste is fully inline (masked input →
// login_with_token_arg, which validates via validate_token BEFORE persisting).
// Browser login is a suspend-and-run handoff to the engine (see the task report):
// the TUI leaves the alternate screen, the engine prints the URL + 300 s wait +
// the SSH-forward hint on failure, then the TUI resumes. The Status row NEVER
// fabricates a name offline — it shows only "credential file present".
mod draw;
mod state;

pub use state::{AccountState, AuthSnapshot};
