//! Plain (`Send`) row types shared by the search results page and the
//! cortinilla (live dropdown).

mod cort;
mod result;

pub use cort::{CortRow, CortSection, CortinillaData};
pub(crate) use cort::{CORTINILLA_CAP_ALBUMS, CORTINILLA_CAP_ARTISTS, CORTINILLA_CAP_PLAYLISTS, CORTINILLA_CAP_TRACKS};
pub use result::{AlbumRow, ArtistRow, MostPopularRow, PlaylistRow, SearchData, TrackRowData};
