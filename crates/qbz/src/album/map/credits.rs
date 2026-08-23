//! Header credit-line + release-date formatting.

use qbz_models::Album;

use super::super::data::ArtistCreditData;

/// Localize an ISO `YYYY-MM-DD` release date to a short readable form
/// ("Feb 19, 2026"), via the active locale. Empty when absent or unparseable
/// (the header simply omits the date segment, as before). Mirrors the
/// `info_modals` date formatter but with the short month (`%b`).
pub(super) fn format_release_date(iso: Option<&str>) -> String {
    let Some(raw) = iso.map(str::trim).filter(|s| !s.is_empty()) else {
        return String::new();
    };
    let head = raw.get(0..10).unwrap_or(raw);
    chrono::NaiveDate::parse_from_str(head, "%Y-%m-%d")
        .map(|d| {
            d.format_localized("%b %-d, %Y", crate::dates::current_locale())
                .to_string()
        })
        .unwrap_or_default()
}

/// Localized role suffix for a credit (E1). "" for the main artist (no suffix);
/// otherwise the first non-`main-artist` role, localized (e.g. "compositor").
fn credit_role(roles: Option<&Vec<String>>) -> String {
    let Some(roles) = roles else {
        return String::new();
    };
    roles
        .iter()
        .find(|r| r.as_str() != "main-artist")
        .map(|r| qbz_i18n::t(&qbz_qobuz::performers::format_role_label(r)))
        .unwrap_or_default()
}

/// Build the header credit line (E1): every credited artist with its role,
/// falling back to the single primary interpreter when the album carries no
/// `artists[]` array (some V2/discover shapes). A single album-level composer
/// is appended last.
///
/// This mirrors the official web player's `releaseArtistsMapper` exactly:
/// `mergeRoles([...album.artists, fallbackArtist, composerMapper(album.composer)])`.
/// The composer leg comes from the album-level `composer` field (a single
/// Artist), NOT from the per-track `composer` — deriving from the tracklist
/// over-credits every songwriter on non-classical albums (the player shows no
/// composer for e.g. Anthrax). The player also drops the composer when its
/// name is the localized "Various Composers" placeholder, detected by the
/// case-insensitive "VARIOUS" substring (bundle module 80145 / `hasAlbumComposer`).
pub(super) fn build_credits(album: &Album) -> Vec<ArtistCreditData> {
    let mut credits: Vec<ArtistCreditData> = match album.artists.as_ref().filter(|v| !v.is_empty())
    {
        Some(list) => list
            .iter()
            .map(|a| ArtistCreditData {
                id: a.id.to_string(),
                name: a.name.clone(),
                role: credit_role(a.roles.as_ref()),
            })
            .collect(),
        None => vec![ArtistCreditData {
            id: album.artist.id.to_string(),
            name: album.artist.name.clone(),
            role: String::new(),
        }],
    };

    // Append the album-level composer (mergeRoles-style: dedup by id, skip the
    // "Various Composers" placeholder).
    if let Some(comp) = album.composer.as_ref() {
        let id = comp.id.to_string();
        let already_credited = credits.iter().any(|c| c.id == id);
        let is_various = comp.name.to_uppercase().contains("VARIOUS");
        if !comp.name.is_empty() && !is_various && !already_credited {
            credits.push(ArtistCreditData {
                id,
                name: comp.name.clone(),
                role: qbz_i18n::t("Composer"),
            });
        }
    }
    credits
}
