/// Album-level fields written into every file's embedded tags. A `None`
/// (or blank) field REMOVES that tag (direct write is destructive, unlike the
/// override-only sidecar).
pub struct AlbumTagWrite {
    pub album_title: String,
    pub album_artist: String, // "" => remove the AlbumArtist tag
    pub year: Option<u32>,    // None => remove the date
    pub genre: Option<String>,
    pub catalog_number: Option<String>,
}

/// One file's per-track fields.
pub struct TrackTagWrite {
    pub file_path: String,
    pub title: String,
    pub track_number: Option<u32>,
    pub disc_number: Option<u32>,
}
