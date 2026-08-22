//! Album query, grouping, and artwork/metadata-mutation methods on
//! `LibraryDatabase`. Split out of the monolithic `database.rs` — see
//! `crate::database` for the overall module layout.

mod artwork;
mod cover_fallback;
mod filter;
mod filter_sql;
mod filter_sql_query;
mod group_metadata;
mod metadata_count;
mod metadata_grouped;
mod metadata_grouped_query;
mod metadata_page;
mod metadata_page_mapping;
mod metadata_page_query;
mod metadata_tracks;
mod track_metadata_by_id;
mod tracks;

#[cfg(test)]
mod tests;
