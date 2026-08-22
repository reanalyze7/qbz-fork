//! Resolving artist metadata and location from MusicBrainz artist responses

use crate::musicbrainz::genre::extract_affinity_seeds;
use crate::musicbrainz::{
    Area, ArtistFullResponse, ArtistLocation, ArtistMetadata, ArtistType, LocationPrecision,
};

use super::country_codes::country_code_to_name;

/// Extract artist metadata from the full MB response
pub fn extract_metadata(response: &ArtistFullResponse) -> ArtistMetadata {
    let artist_type = ArtistType::from(response.artist_type.as_deref());

    // Resolve location: prefer begin_area (city-level), fallback to area (country)
    let location = resolve_location(
        response.begin_area.as_ref(),
        response.area.as_ref(),
        response.country.as_deref(),
    );

    // Extract affinity seeds from tags
    let tags = response.tags.as_deref().unwrap_or(&[]);
    let affinity_seeds = extract_affinity_seeds(tags);

    ArtistMetadata {
        mbid: response.id.clone(),
        name: response.name.clone(),
        artist_type,
        life_span: response.life_span.clone(),
        location,
        affinity_seeds,
    }
}

/// Resolve the most precise location from MB area data
fn resolve_location(
    begin_area: Option<&Area>,
    area: Option<&Area>,
    country: Option<&str>,
) -> Option<ArtistLocation> {
    let cc = country.map(|c| c.to_lowercase());

    // Try begin_area first (formation/birth location — typically city-level)
    if let Some(ba) = begin_area {
        let is_city = ba
            .area_type
            .as_deref()
            .map(|t| t.eq_ignore_ascii_case("city") || t.eq_ignore_ascii_case("municipality"))
            .unwrap_or(false);

        let is_subdivision = ba
            .area_type
            .as_deref()
            .map(|t| t.eq_ignore_ascii_case("subdivision"))
            .unwrap_or(false);

        // MB's "country" field is where the artist is active (not where born).
        // When we have a city-level begin_area, display only the city name
        // to avoid incorrect country attribution (e.g., Zimmer: born Frankfurt,
        // but country=US because he works in the US).
        let precision = if is_city {
            LocationPrecision::City
        } else if is_subdivision {
            LocationPrecision::State
        } else {
            LocationPrecision::City // best guess
        };

        return Some(ArtistLocation {
            city: Some(ba.name.clone()),
            area_id: Some(ba.id.clone()),
            country: country.map(|c| country_code_to_name(c)),
            country_code: cc,
            display_name: ba.name.clone(),
            precision,
        });
    }

    // Fallback to area (usually country-level)
    if let Some(a) = area {
        let is_country = a
            .area_type
            .as_deref()
            .map(|t| t.eq_ignore_ascii_case("country"))
            .unwrap_or(false);

        if is_country {
            return Some(ArtistLocation {
                city: None,
                area_id: Some(a.id.clone()),
                country: Some(a.name.clone()),
                country_code: cc,
                display_name: a.name.clone(),
                precision: LocationPrecision::Country,
            });
        }

        // Non-country area (could be city without begin_area)
        let country_name = country.map(|c| country_code_to_name(c));
        let display = if let Some(ref cn) = country_name {
            format!("{}, {}", a.name, cn)
        } else {
            a.name.clone()
        };

        return Some(ArtistLocation {
            city: Some(a.name.clone()),
            area_id: Some(a.id.clone()),
            country: country_name,
            country_code: cc,
            display_name: display,
            precision: LocationPrecision::City,
        });
    }

    // Country code only (no area data)
    if let Some(raw_cc) = country {
        let name = country_code_to_name(raw_cc);
        return Some(ArtistLocation {
            city: None,
            area_id: None,
            country: Some(name.clone()),
            country_code: cc,
            display_name: name,
            precision: LocationPrecision::Country,
        });
    }

    None
}
