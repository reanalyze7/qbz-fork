//! Raw MusicBrainz response -> `ArtistRelationships` mapping.

use qbz_integrations::musicbrainz::{ArtistFullResponse, ArtistRelationships, Period, RelatedArtist};

/// Extract ArtistRelationships from a raw MusicBrainz response (verbatim port —
/// the only two relation types production uses: `member of band` + `collaboration`).
pub(super) fn extract_relationships(artist: &ArtistFullResponse) -> ArtistRelationships {
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
                        // We're viewing a BAND, the related artist is a MEMBER
                        if related.ended {
                            past_members.push(related);
                        } else {
                            members.push(related);
                        }
                    } else {
                        // We're viewing a PERSON, the related artist is a BAND/GROUP
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

    ArtistRelationships {
        members,
        past_members,
        groups,
        collaborators,
    }
}
