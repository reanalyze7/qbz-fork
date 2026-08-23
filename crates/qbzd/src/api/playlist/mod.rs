// crates/qbzd/src/api/playlist/ — playlists (02 §2.3, §3.4 row 24). GET
// /api/playlists (the user's collection) and GET /api/playlist?id= (one
// playlist with its COMPLETE track list — get_playlist auto-pages server-side),
// plus playlist CRUD (create/update/delete) and track mutation (add/remove).
// Auth-gated; typed serde shapes verbatim.
mod crud;
mod internal;
mod reads;
mod tracks;

pub use crud::{create, delete, update};
pub use reads::{list, show};
pub use tracks::{tracks_add, tracks_remove};
