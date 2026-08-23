// crates/qbzd/src/cli/queue/ — the `queue list/add/remove/clear` verbs
// (02-cli-and-api.md §2.2). Each is exactly one HTTP request (§1.1); the pure
// index-translation and rendering helpers below are unit-tested without a
// running daemon.
//
// INDEX CONVENTION (normative, cross-doc-fixed, §3.3.13/§3.3.15/§2.2 queue
// remove note): the wire is 0-based everywhere (the same space as
// `GET /api/queue`'s `current_index`). This file is the ONLY place a 1-based
// position exists — `queue list`'s `#` column and `queue remove <INDEX>`'s
// argument. The translation happens exactly at this boundary:
//   - display: `cli_position(api_index_0based) -> 1-based` (used by the
//     `queue list` table renderer).
//   - input: `cli_index_to_api(cli_position_1based) -> 0-based, Result` (used
//     by `queue remove`; position 0 is a usage error — there is no "0th"
//     track).
mod fmt;
mod list;
mod mutate;
mod nav;
#[cfg(test)]
mod tests;

pub use list::list;
pub use mutate::{add, clear, remove};
pub use nav::{jump, move_, stop_after};

/// 1-based CLI position -> 0-based API index. Position 0 is a usage error
/// (there is no "0th" track) — the ONLY local validation this verb does;
/// everything else (out of range, playing track) is a server-side 404/400
/// the daemon is authoritative on.
pub fn cli_index_to_api(position: usize) -> Result<usize, String> {
    position.checked_sub(1).ok_or_else(|| {
        format!("invalid queue position '{position}' — positions start at 1 (see: qbzd queue list)")
    })
}

/// 0-based API index -> 1-based CLI display position (the inverse of
/// [`cli_index_to_api`] — used by `queue list`'s row numbering).
pub(super) fn cli_position(api_index: usize) -> usize {
    api_index + 1
}
