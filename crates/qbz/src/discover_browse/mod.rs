//! Discover "View all" full-list controller.
//!
//! Opens a single album module (New Releases, Press Accolades, Ideal
//! Discography, Qobuzissimes, Albums of the Week, Most Streamed) as a
//! paginated full-grid (or list) page. The Carousel's "View all" link
//! fires `discover-view-all(endpoint, title)`; the shell records the
//! history entry, switches the view to ContentView::DiscoverBrowse, and
//! calls `navigate` here.
//!
//! Pagination is driven off the backend `has_more` flag (the discover
//! endpoints carry no `total`): each page advances `offset` by the
//! FETCHED item count and stops once `has_more` is false. Reuses the
//! Discover home mappers (`crate::home::map_album` / `card_to_item`) so the
//! cards carry the same genre + localized release date as the home carousels.
//!
//! Header tools (mirroring Tauri's DiscoverBrowseView): a client-side
//! search filter over the loaded albums (disables load-more while active),
//! the shared genre filter (re-fetches from offset 0 with the raw selected
//! genre ids — Qobuz facets sub-genre ids server-side, no client narrowing),
//! and a grid/list view toggle.

mod fetch;
mod filter;
mod load_more;
mod navigate;

pub use filter::apply_filter;
pub use load_more::load_more;
pub use navigate::navigate;

/// Page size — two carousel pages' worth, fetched per request.
const PAGE_SIZE: u32 = 50;
