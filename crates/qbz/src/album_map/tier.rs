//! Quality-tier classification shared by `map_album` and the TYPE column
//! heuristic.

/// Quality tier from a bit depth alone — `hires` above 16-bit, else `cd`.
pub fn tier(bit_depth: Option<u32>) -> &'static str {
    match bit_depth {
        Some(b) if b > 16 => "hires",
        Some(_) => "cd",
        None => "",
    }
}

/// Quality tier from a resolved bit depth, with a `hires` boolean fallback
/// for payloads that omit the bit depth but still flag the release hi-res.
pub fn tier_hires(bit_depth: Option<u32>, hires: bool) -> &'static str {
    match bit_depth {
        Some(b) if b > 16 => "hires",
        Some(_) => "cd",
        None if hires => "hires",
        None => "",
    }
}

/// Classify the list-row TYPE column from the album's track count, for
/// payloads (favorites, /label/getAlbums) that carry no explicit
/// release_type. Mirrors home.rs's Discover heuristic
/// (<=3 = Single, <=6 = EP, else Album).
pub fn classify_release_type(track_count: Option<u32>) -> &'static str {
    // Marked at the definition so the extractor sees the English literals; the
    // call sites (`album_map`/`home`) translate the marked value with `t(...)`.
    match track_count {
        Some(n) if n <= 3 => qbz_i18n::mark("Single"),
        Some(n) if n <= 6 => qbz_i18n::mark("EP"),
        _ => qbz_i18n::mark("Album"),
    }
}
