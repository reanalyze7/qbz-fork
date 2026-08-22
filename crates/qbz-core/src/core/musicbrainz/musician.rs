//! Musician resolution + appearances.

use qbz_integrations::musicbrainz::{
    AlbumAppearance, MusicianAppearances, MusicianConfidence, ResolvedMusician,
};
use qbz_models::FrontendAdapter;

use crate::error::CoreError;

use super::super::QbzCore;

impl<A: FrontendAdapter + Send + Sync + 'static> QbzCore<A> {
    /// Resolve a musician (band member / collaborator) to the
    /// strongest available identity: MBID via MusicBrainz, exact-
    /// name Qobuz artist match, or just an appears-on count. Ports
    /// v2_resolve_musician end to end so the artist-network sidebar
    /// click can route the user to the right view (artist page when
    /// confident, MusicianPageView otherwise).
    pub async fn musicbrainz_resolve_musician(
        &self,
        name: &str,
        role: &str,
    ) -> Result<ResolvedMusician, CoreError> {
        // MusicBrainz opt-out / unavailable: a disabled or failed resolve is NOT
        // fatal here — fall through to the Qobuz exact-name / appears-on fallback
        // below so the musician page still degrades cleanly when MB is off. (Was
        // `?`, which propagated the disabled-client Err before the Qobuz fallback
        // and left the musician page blank/stuck when MB is disabled.)
        let resolved_artist = match self.musicbrainz.resolve_artist(name).await {
            Ok(found) => found,
            Err(e) => {
                log::debug!(
                    "musicbrainz_resolve_musician: MB resolve unavailable for {name:?}: {e}"
                );
                None
            }
        };

        let normalized_target = name.trim().to_lowercase();
        let artist_results = self.search_artists(name, 10, 0, None).await?;
        let exact = artist_results
            .items
            .iter()
            .find(|artist| artist.name.trim().to_lowercase() == normalized_target);

        if let Some(artist) = exact {
            let qobuz_artist_id = i64::try_from(artist.id).ok();
            return Ok(ResolvedMusician {
                name: name.to_string(),
                role: role.to_string(),
                mbid: None,
                qobuz_artist_id,
                confidence: MusicianConfidence::Confirmed,
                bands: Vec::new(),
                appears_on_count: 0,
            });
        }

        let album_results = self.search_albums(name, 20, 0, None).await?;
        let appears_on_count = album_results.total as usize;

        let confidence = if appears_on_count > 0 {
            MusicianConfidence::Contextual
        } else if resolved_artist.is_some() {
            MusicianConfidence::Weak
        } else {
            MusicianConfidence::None
        };

        Ok(ResolvedMusician {
            name: name.to_string(),
            role: role.to_string(),
            mbid: resolved_artist.as_ref().map(|a| a.mbid.clone()),
            qobuz_artist_id: None,
            confidence,
            bands: Vec::new(),
            appears_on_count,
        })
    }

    /// Fetch the paginated list of albums on which `name` appears,
    /// for the Appears On grid in MusicianPageView. Ports
    /// v2_get_musician_appearances — same Qobuz album search +
    /// per-row mapping with `role_on_album = role`.
    pub async fn musicbrainz_get_musician_appearances(
        &self,
        name: &str,
        role: &str,
        limit: u32,
        offset: u32,
    ) -> Result<MusicianAppearances, CoreError> {
        let results = self.search_albums(name, limit, offset, None).await?;
        let albums = results
            .items
            .into_iter()
            .map(|album| AlbumAppearance {
                album_id: album.id,
                album_title: album.title,
                album_artwork: album.image.large.or(album.image.small).unwrap_or_default(),
                artist_name: album.artist.name,
                year: album.release_date_original,
                role_on_album: role.to_string(),
            })
            .collect::<Vec<_>>();
        Ok(MusicianAppearances {
            albums,
            total: results.total as usize,
        })
    }
}
