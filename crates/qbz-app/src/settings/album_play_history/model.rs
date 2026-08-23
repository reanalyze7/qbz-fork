/// Album metadata captured at play time (refreshed on every play, so renames
/// and artwork refreshes converge).
pub struct AlbumPlayMeta<'a> {
    pub album_id: &'a str,
    pub title: &'a str,
    pub artist: &'a str,
    pub artist_id: &'a str,
    pub artwork_url: &'a str,
    pub quality_tier: &'a str,
    pub quality_label: &'a str,
    pub year: &'a str,
    pub source: &'a str,
}

/// One ranked album for the "Most Played Albums" rail / View-all page.
#[derive(Clone, Default, Debug, PartialEq)]
pub struct AlbumPlayRow {
    pub album_id: String,
    pub title: String,
    pub artist: String,
    pub artist_id: String,
    pub artwork_url: String,
    pub quality_tier: String,
    pub quality_label: String,
    pub year: String,
    pub source: String,
    pub plays: u32,
}
