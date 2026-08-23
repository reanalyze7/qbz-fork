use std::io::Cursor;

use tiny_http::Response;

use crate::api::err_json;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum FavType {
    Track,
    Album,
    Artist,
}

impl FavType {
    /// Accepts singular or plural, case-insensitive (lenient input).
    pub(super) fn parse(s: &str) -> Option<FavType> {
        match s.trim().to_ascii_lowercase().as_str() {
            "track" | "tracks" => Some(FavType::Track),
            "album" | "albums" => Some(FavType::Album),
            "artist" | "artists" => Some(FavType::Artist),
            _ => None,
        }
    }
    /// Singular form for add_favorite/remove_favorite (core.rs:1088/1099).
    pub(super) fn singular(self) -> &'static str {
        match self {
            FavType::Track => "track",
            FavType::Album => "album",
            FavType::Artist => "artist",
        }
    }
    /// Plural form for get_favorites (core.rs:1072).
    pub(super) fn plural(self) -> &'static str {
        match self {
            FavType::Track => "tracks",
            FavType::Album => "albums",
            FavType::Artist => "artists",
        }
    }
}

pub(super) fn bad_type(got: &str) -> Response<Cursor<Vec<u8>>> {
    err_json(400, "bad_request", &format!("unknown fav type '{got}'"), "type: track | album | artist")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fav_type_parses_singular_and_plural_and_maps_both_forms() {
        for s in ["track", "tracks", "TRACK"] {
            assert_eq!(FavType::parse(s), Some(FavType::Track));
        }
        assert_eq!(FavType::parse("albums"), Some(FavType::Album));
        assert_eq!(FavType::parse("artist"), Some(FavType::Artist));
        assert_eq!(FavType::parse("songs"), None);
        // the trap, hidden: read uses plural, write uses singular.
        assert_eq!(FavType::Track.plural(), "tracks");
        assert_eq!(FavType::Track.singular(), "track");
    }
}
