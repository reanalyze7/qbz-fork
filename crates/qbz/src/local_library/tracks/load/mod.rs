//! Server-paginated flat list load path for the Tracks tab — the perf path
//! that avoids the documented ~16K freeze: each page is a
//! `search_with_filter_page` query, appended on scroll.

mod apply;
mod more;
mod spawn;
mod state;

pub use more::*;
pub use spawn::*;
pub use state::tracks_current_snapshot;

pub(crate) use state::tracks_current;
