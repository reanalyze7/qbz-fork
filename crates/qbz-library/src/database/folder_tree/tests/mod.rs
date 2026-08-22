//! Test suite for `folder_tree`, split by concern: `common` (shared
//! fixture layout), `children_tests` (`list_folder_children`),
//! `tracks_tests` (`list_folder_tracks` / `list_folder_tracks_recursive`),
//! `network_tests` (network-folder exclusion across listing
//! primitives), `network_count_tests` (`count_folder_tracks_recursive`).

mod children_tests;
mod common;
mod network_count_tests;
mod network_tests;
mod tracks_tests;
