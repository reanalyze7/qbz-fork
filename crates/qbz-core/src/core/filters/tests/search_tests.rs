use qbz_models::Artist;

use super::super::{parse_page, parse_search_all};
use super::no_albums;
use super::super::super::{AlbumBlacklistFilter, BlacklistFilter};

#[test]
fn parse_search_all_filters_blacklisted_artist() {
    let json = serde_json::json!({
        "albums":  { "items": [], "total": 0, "offset": 0, "limit": 30 },
        "tracks":  { "items": [], "total": 0, "offset": 0, "limit": 30 },
        "artists": {
            "items": [
                { "id": 1, "name": "Keep" },
                { "id": 999, "name": "Blocked" }
            ],
            "total": 2, "offset": 0, "limit": 30
        },
        "playlists": { "items": [], "total": 0, "offset": 0, "limit": 30 }
    });
    let blocked: BlacklistFilter = [999].into_iter().collect();
    let out = parse_search_all(&json, &blocked, &no_albums());
    assert_eq!(out.artists.items.len(), 1);
    assert_eq!(out.artists.items[0].name, "Keep");
    assert_eq!(out.artists.total, 1);
    assert!(out.most_popular.is_none());
}

#[test]
fn parse_page_skips_poisoned_item_keeps_rest() {
    // One malformed entry (id is a string where Artist.id: u64) must NOT
    // blank the page — the old whole-page from_value did exactly that.
    let json = serde_json::json!({
        "artists": {
            "items": [
                { "id": 1, "name": "Keep" },
                { "id": "poisoned", "name": "Bad" },
                { "id": 2, "name": "AlsoKeep" }
            ],
            "total": 3, "offset": 0, "limit": 30
        }
    });
    let page = parse_page::<Artist>(&json, "artists");
    assert_eq!(page.items.len(), 2);
    assert_eq!(page.items[0].name, "Keep");
    assert_eq!(page.items[1].name, "AlsoKeep");
    assert_eq!(page.total, 3);
    assert_eq!(page.limit, 30);
}

#[test]
fn parse_search_all_filters_blocked_album() {
    let json = serde_json::json!({
        "albums": {
            "items": [
                { "id": "keep", "title": "Keep", "artist": { "id": 1, "name": "A" } },
                { "id": "blk",  "title": "Bogus", "artist": { "id": 1, "name": "A" } }
            ],
            "total": 2, "offset": 0, "limit": 30
        },
        "tracks":  { "items": [], "total": 0, "offset": 0, "limit": 30 },
        "artists": { "items": [], "total": 0, "offset": 0, "limit": 30 },
        "playlists": { "items": [], "total": 0, "offset": 0, "limit": 30 }
    });
    let abl: AlbumBlacklistFilter = ["blk".to_string()].into_iter().collect();
    let out = parse_search_all(&json, &BlacklistFilter::new(), &abl);
    assert_eq!(out.albums.items.len(), 1);
    assert_eq!(out.albums.items[0].id, "keep");
    assert_eq!(out.albums.total, 1);
}
