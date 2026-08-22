//! MusicBrainz integration: enable/resolve, suggestions, metadata,
//! discovery (tag-based and location-based), and musician resolution.
//! Split across files because the original section was ~700 lines.

mod discovery;
mod discovery_fallback;
mod discovery_location;
mod discovery_location_candidates;
mod discovery_location_genres;
mod enable;
mod metadata;
mod musician;
mod relationships;
mod suggestions;
mod validate;
