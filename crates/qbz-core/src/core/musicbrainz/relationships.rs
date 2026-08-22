//! Artist relationships (band members, member-of groups, collaborators)
//! for the Relationships section of the sidebar.

use qbz_integrations::musicbrainz::{ArtistRelationships, Period, RelatedArtist};
use qbz_models::FrontendAdapter;

use crate::error::CoreError;

use super::super::QbzCore;

impl<A: FrontendAdapter + Send + Sync + 'static> QbzCore<A> {
    /// Fetch the artist relationships (band members, member-of groups,
    /// collaborators) for the Relationships section of the sidebar.
    /// Splits `member of band` by direction: backward direction lists
    /// members of *this* artist (still-active vs ended -> past), forward
    /// direction lists groups this artist is a member of.
    pub async fn musicbrainz_get_artist_relationships(
        &self,
        mbid: &str,
    ) -> Result<ArtistRelationships, CoreError> {
        if let Ok(guard) = self.musicbrainz_cache.lock() {
            if let Some(cache) = guard.as_ref() {
                if let Ok(Some(cached)) = cache.get_artist_relations(mbid) {
                    return Ok(cached);
                }
            }
        }

        let artist = self
            .musicbrainz
            .get_artist_with_relations(mbid)
            .await
            .map_err(|e| CoreError::Internal(e.to_string()))?;

        let mut members = Vec::new();
        let mut past_members = Vec::new();
        let mut groups = Vec::new();
        let mut collaborators = Vec::new();

        if let Some(relations) = &artist.relations {
            for relation in relations {
                let Some(related_artist) = &relation.artist else {
                    continue;
                };

                let related = RelatedArtist {
                    mbid: related_artist.id.clone(),
                    name: related_artist.name.clone(),
                    role: relation
                        .attributes
                        .as_ref()
                        .and_then(|a| a.first().cloned()),
                    period: Some(Period {
                        begin: relation.begin.clone(),
                        end: relation.end.clone(),
                    }),
                    ended: relation.ended.unwrap_or(false),
                };

                match relation.relation_type.as_str() {
                    "member of band" => {
                        if relation.direction.as_deref() == Some("backward") {
                            if related.ended {
                                past_members.push(related);
                            } else {
                                members.push(related);
                            }
                        } else {
                            groups.push(related);
                        }
                    }
                    "collaboration" => {
                        collaborators.push(related);
                    }
                    _ => {}
                }
            }
        }

        let result = ArtistRelationships {
            members,
            past_members,
            groups,
            collaborators,
        };

        if let Ok(guard) = self.musicbrainz_cache.lock() {
            if let Some(cache) = guard.as_ref() {
                let _ = cache.set_artist_relations(mbid, &result);
            }
        }

        Ok(result)
    }
}
