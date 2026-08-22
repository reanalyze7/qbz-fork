//! Serialize/parse helpers for the enum columns stored as TEXT in sqlite.

use qbz_models::mixtape::{
    AlbumSource, CollectionKind, CollectionPlayMode, CollectionSourceType, ItemType,
};

pub fn serialize_kind(k: CollectionKind) -> &'static str {
    match k {
        CollectionKind::Mixtape => "mixtape",
        CollectionKind::Collection => "collection",
        CollectionKind::ArtistCollection => "artist_collection",
    }
}
pub fn parse_kind(s: &str) -> CollectionKind {
    match s {
        "mixtape" => CollectionKind::Mixtape,
        "artist_collection" => CollectionKind::ArtistCollection,
        _ => CollectionKind::Collection,
    }
}
pub fn serialize_source_type(t: CollectionSourceType) -> &'static str {
    match t {
        CollectionSourceType::Manual => "manual",
        CollectionSourceType::ArtistDiscography => "artist_discography",
    }
}
pub fn parse_source_type(s: &str) -> CollectionSourceType {
    match s {
        "artist_discography" => CollectionSourceType::ArtistDiscography,
        _ => CollectionSourceType::Manual,
    }
}
pub fn serialize_play_mode(m: CollectionPlayMode) -> &'static str {
    match m {
        CollectionPlayMode::InOrder => "in_order",
        CollectionPlayMode::AlbumShuffle => "album_shuffle",
    }
}
pub fn parse_play_mode(s: &str) -> CollectionPlayMode {
    match s {
        "album_shuffle" => CollectionPlayMode::AlbumShuffle,
        _ => CollectionPlayMode::InOrder,
    }
}
pub fn serialize_item_type(t: ItemType) -> &'static str {
    match t {
        ItemType::Album => "album",
        ItemType::Track => "track",
        ItemType::Playlist => "playlist",
    }
}
pub fn parse_item_type(s: &str) -> ItemType {
    match s {
        "track" => ItemType::Track,
        "playlist" => ItemType::Playlist,
        _ => ItemType::Album,
    }
}
pub fn serialize_source(s: AlbumSource) -> &'static str {
    match s {
        AlbumSource::Qobuz => "qobuz",
        AlbumSource::Local => "local",
    }
}
pub fn parse_source(s: &str) -> AlbumSource {
    match s {
        "local" => AlbumSource::Local,
        _ => AlbumSource::Qobuz,
    }
}
