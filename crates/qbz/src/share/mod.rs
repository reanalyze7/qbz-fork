//! Share-link helpers — Qobuz track URL + Song.link (Odesli) resolution
//! + clipboard copy. Used by the track context menu's Share actions.

mod clipboard;
mod songlink;
mod urls;

pub use clipboard::copy_to_clipboard;
pub use songlink::{albumlink_for_album, songlink_for_track};
pub use urls::*;
