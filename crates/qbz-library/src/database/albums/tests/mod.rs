//! Test suite for the `albums` metadata-grouping domain, split by
//! concern: `common` (shared fixtures), `basic_tests` (merge/fallback/
//! orphan/VA detection), `folder_mode_tests` (the Saint Seiya
//! compilation regression), `metadata_edge_tests` (track fetch by
//! group key, and the #447/#507 title/year/artist regressions).

mod basic_tests;
mod common;
mod folder_mode_tests;
mod metadata_edge_tests;
mod metadata_title_year_tests;
