//! Page navigation history — a browser-like back/forward stack.
//!
//! The shell records a [`NavEntry`] on every fresh navigation; the
//! `[<] [>]` button pair (and the mouse back/forward buttons) walk the
//! stack. UI thread only, hence `thread_local`.
//!
//! Scroll-position restoration: each entry remembers the viewport-y of the
//! scroll container that was showing it. The mounted view continuously
//! reports its live scroll via [`set_live_scroll`] (a NavState callback), so
//! every navigation — fresh [`record`] or [`go_back`]/[`go_forward`] — can
//! stamp the outgoing entry without touching the ~30 `record` call sites.
//! `go_back`/`go_forward` hand the restored scroll back to the shell, which
//! arms `NavState.restore-scope` + `scroll-restore`; the destination's scroll
//! container picks it up once its content has laid out.

mod entry;
mod history;
mod navigation;
mod stepping;

#[cfg(test)]
mod tests;

pub use entry::NavEntry;
pub use history::{current, set_live_scroll};
pub use navigation::{push_or_replace_search, record, reset_root};
pub use stepping::{can_back, can_forward, go_back, go_forward};
