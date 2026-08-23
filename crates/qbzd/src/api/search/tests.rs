use super::params::{parse_query, Category, SearchType, DEFAULT_LIMIT, MAX_LIMIT};

#[test]
fn parse_query_defaults_type_all_limit_20_offset_0() {
    let p = parse_query("q=spain").unwrap();
    assert_eq!(p.q, "spain");
    assert_eq!(p.stype, SearchType::All);
    assert_eq!(p.limit, 20);
    assert_eq!(p.offset, 0);
}

#[test]
fn parse_query_percent_decodes_the_query() {
    // `light as a feather` → space-encoded both ways.
    let p = parse_query("q=light%20as%20a%20feather").unwrap();
    assert_eq!(p.q, "light as a feather");
    let plus = parse_query("q=kind+of+blue").unwrap();
    // urlencoding treats `+` literally (RFC 3986); a real client encodes
    // spaces as %20, so `+` stays a plus — documented, not a bug.
    assert_eq!(plus.q, "kind+of+blue");
}

#[test]
fn parse_query_reads_each_typed_category() {
    assert_eq!(
        parse_query("q=x&type=albums").unwrap().stype,
        SearchType::One(Category::Albums)
    );
    assert_eq!(
        parse_query("q=x&type=tracks").unwrap().stype,
        SearchType::One(Category::Tracks)
    );
    assert_eq!(
        parse_query("q=x&type=artists").unwrap().stype,
        SearchType::One(Category::Artists)
    );
    assert_eq!(
        parse_query("q=x&type=playlists").unwrap().stype,
        SearchType::One(Category::Playlists)
    );
    assert_eq!(parse_query("q=x&type=all").unwrap().stype, SearchType::All);
}

#[test]
fn parse_query_rejects_missing_or_blank_query() {
    assert!(parse_query("").is_err());
    assert!(parse_query("type=albums").is_err());
    assert!(parse_query("q=").is_err());
    assert!(parse_query("q=%20%20").is_err());
}

#[test]
fn parse_query_rejects_unknown_type() {
    let (message, _hint) = parse_query("q=x&type=songs").unwrap_err();
    assert_eq!(message, "unknown type 'songs'");
}

#[test]
fn parse_query_clamps_limit_and_ignores_bad_numbers() {
    assert_eq!(parse_query("q=x&limit=500").unwrap().limit, MAX_LIMIT);
    assert_eq!(parse_query("q=x&limit=0").unwrap().limit, 1);
    assert_eq!(parse_query("q=x&limit=nope").unwrap().limit, DEFAULT_LIMIT);
    assert_eq!(parse_query("q=x&offset=5").unwrap().offset, 5);
    assert_eq!(parse_query("q=x&offset=bad").unwrap().offset, 0);
}

#[test]
fn search_type_as_str_round_trips_the_flag_values() {
    assert_eq!(SearchType::All.as_str(), "all");
    assert_eq!(SearchType::One(Category::Albums).as_str(), "albums");
    assert_eq!(SearchType::One(Category::Tracks).as_str(), "tracks");
    assert_eq!(SearchType::One(Category::Artists).as_str(), "artists");
    assert_eq!(SearchType::One(Category::Playlists).as_str(), "playlists");
}
