//! Plain, `Send` data produced on the worker thread, applied to the Slint
//! globals on the event loop.

pub(super) struct CreditRowData {
    /// Already-localized, UPPER-CASED role label (display).
    pub(super) role: String,
    /// Original role string (for the musician-nav role hint, 1:1 with Tauri
    /// which passes the raw group role, not the display label).
    pub(super) role_raw: String,
    pub(super) names: Vec<String>,
}

pub struct TrackInfoData {
    pub(super) title: String,
    pub(super) album: String,
    pub(super) artist: String,
    /// "" -> render the artist as plain text (no link).
    pub(super) artist_id: String,
    pub(super) duration: String,
    pub(super) quality: String,
    pub(super) isrc: String,
    pub(super) label: String,
    pub(super) label_id: String,
    pub(super) copyright: String,
    pub(super) credits: Vec<CreditRowData>,
}

pub(super) struct PerformerData {
    pub(super) name: String,
    /// ", Role1, Role2" suffix (empty when the performer has no roles).
    pub(super) roles: String,
    /// First role (clean), or "Performer" — the musician-nav role hint,
    /// 1:1 with Tauri `handlePerformerClick` (roles[0] || 'Performer').
    pub(super) primary_role: String,
}

pub(super) struct AlbumTrackData {
    pub(super) id: String,
    pub(super) number: String,
    pub(super) title: String,
    pub(super) artist: String,
    pub(super) has_credits: bool,
    pub(super) performers: Vec<PerformerData>,
    pub(super) copyright: String,
}

pub struct AlbumCreditsData {
    pub(super) title: String,
    pub(super) artist: String,
    pub(super) label: String,
    pub(super) label_id: String,
    pub(super) release_date: String,
    pub(super) meta_line: String,
    pub(super) quality: String,
    pub(super) review: String,
    pub(super) has_review: bool,
    pub(super) tracks: Vec<AlbumTrackData>,
}
