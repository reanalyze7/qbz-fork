/// Plain, `Send` payload mapping `qbz_integrations::ArtistMetadata` into
/// the shape the Origin section of the sidebar renders.
pub struct MbMetadata {
    pub mbid: String,
    pub origin: MbOrigin,
}

#[derive(Default)]
pub struct MbOrigin {
    pub is_person: bool,
    pub begin_date: String,
    pub end_date: String,
    pub location_display: String,
    pub location_clickable: bool,
}
