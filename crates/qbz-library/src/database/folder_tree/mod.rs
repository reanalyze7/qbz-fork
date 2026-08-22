//! Folder-tree (filesystem hierarchy) listing methods on
//! `LibraryDatabase`. Split out of the monolithic `database.rs` — see
//! `crate::database` for the overall module layout.

mod children;
mod children_query;
mod children_sort;
mod recursive;
mod tracks;

#[cfg(test)]
mod tests;
