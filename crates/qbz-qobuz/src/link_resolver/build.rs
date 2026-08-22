use super::{LinkResolverError, ResolvedLink};

/// Build a ResolvedLink from entity type and raw ID string.
pub(super) fn build_resolved_link(
    entity_type: &str,
    raw_id: &str,
) -> Result<ResolvedLink, LinkResolverError> {
    match entity_type {
        "album" => {
            // Album IDs are strings (e.g., "0060254728933")
            if raw_id.is_empty() {
                return Err(LinkResolverError::InvalidId(raw_id.to_string()));
            }
            Ok(ResolvedLink::OpenAlbum(raw_id.to_string()))
        }
        "track" => {
            let id = raw_id
                .parse::<u64>()
                .map_err(|_| LinkResolverError::InvalidId(raw_id.to_string()))?;
            Ok(ResolvedLink::OpenTrack(id))
        }
        "artist" | "interpreter" => {
            let id = raw_id
                .parse::<u64>()
                .map_err(|_| LinkResolverError::InvalidId(raw_id.to_string()))?;
            Ok(ResolvedLink::OpenArtist(id))
        }
        "playlist" => {
            let id = raw_id
                .parse::<u64>()
                .map_err(|_| LinkResolverError::InvalidId(raw_id.to_string()))?;
            Ok(ResolvedLink::OpenPlaylist(id))
        }
        _ => Err(LinkResolverError::UnknownEntityType(
            entity_type.to_string(),
        )),
    }
}
