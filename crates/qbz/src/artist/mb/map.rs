use super::date::format_mb_date_short;
use super::types::MbOrigin;

pub(crate) fn map_origin(meta: &qbz_integrations::musicbrainz::ArtistMetadata) -> MbOrigin {
    use qbz_integrations::musicbrainz::{ArtistType, LocationPrecision};

    let is_person = matches!(meta.artist_type, ArtistType::Person);

    let begin_date = meta
        .life_span
        .as_ref()
        .and_then(|ls| ls.begin.as_deref().map(format_mb_date_short))
        .unwrap_or_default();
    let end_date = meta
        .life_span
        .as_ref()
        .and_then(|ls| ls.end.as_deref().map(format_mb_date_short))
        .unwrap_or_default();

    let (location_display, location_clickable) = match &meta.location {
        Some(loc) => {
            // Tauri's gate: clickable when precision isn't "country" OR
            // a city is present somehow. Country-only locations stay as
            // plain text — there's nothing to drill into.
            let clickable = !matches!(loc.precision, LocationPrecision::Country)
                || loc.city.is_some();
            (loc.display_name.clone(), clickable)
        }
        None => (String::new(), false),
    };

    MbOrigin {
        is_person,
        begin_date,
        end_date,
        location_display,
        location_clickable,
    }
}
