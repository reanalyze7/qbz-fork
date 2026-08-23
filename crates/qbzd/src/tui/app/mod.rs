// crates/qbzd/src/tui/app/mod.rs — the qbzd setup-TUI's top-level state
// machine.
//
// Owns the eight screens (the D7 six-screen cap was lifted for FB4's Wizard and
// the CONSOLE ext's Scrobbler, both owner-sanctioned), the route,
// the dirty-save model (§4), the App-level overlays (help / result panel /
// dirty-leave modal), and the worker plumbing (§5.5: NO I/O on keystrokes — disk
// and HTTP happen only at screen entry, `r`, save, and the immediate actions, on
// a worker with a spinner). Persistence is reused wholesale: saves go through
// T11's `write_one`, import/export through the T12 bundle engine, auth through
// the T5 login engine. The TUI adds no persistence of its own (03 §6).
//
// Split across sibling files (all `App` fields are `pub(super)` so every file
// here can reach them): `nav*.rs` (the pure navigation-intent layer),
// `messages*.rs` (the shared App/screen vocabulary), `state*.rs` (the `App`
// struct + its non-render methods, as further `impl App` blocks), `keys*.rs`
// (key handling), `save.rs`/`actions.rs`/`wizard_workers.rs` (worker spawns),
// `worker_results*.rs` (applying worker results), `draw*.rs` (rendering),
// `worker_fns*.rs`/`worker_import*.rs` (the free worker-thread functions),
// `misc.rs` (the plain-terminal-flow callbacks), and `tests_*.rs`.

mod actions;
mod draw;
mod draw_chrome;
mod draw_sidebar;
mod keys;
mod keys_dispatch;
mod messages;
mod messages_worker;
mod misc;
mod nav;
mod nav_classify;
mod save;
mod state;
mod state_nav;
mod state_query;
mod state_status;
mod wizard_workers;
mod worker_fns;
mod worker_fns_ext;
mod worker_import;
mod worker_import_plan;
mod worker_results;
mod worker_results_bundle;

pub use messages::{DrawCtx, LoopCmd, Screen, ScreenAction, SCREENS};
pub use nav::Focus;
pub use state::App;

#[cfg(test)]
mod tests_focus;
#[cfg(test)]
mod tests_footer;
#[cfg(test)]
mod tests_layout;
#[cfg(test)]
mod tests_support;
