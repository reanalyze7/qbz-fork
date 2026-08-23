use serde_json::Value;

use crate::cli::client::ApiClient;
use crate::paths::ProfileRoots;

use super::cli_position;
use super::fmt::fmt_mmss;

// ============================ list ============================

/// `qbzd queue list [--json]` — renders `GET /api/queue` (§2.2/§3.3.13).
/// Exit: 0 · 1 · 3 (no needs_auth in `list`'s Errors column, §3.3.13).
pub async fn list(host: Option<String>, json: bool, roots: &ProfileRoots) -> i32 {
    let client = ApiClient::new(host, roots);
    match client.get("/api/queue").await {
        Ok(v) => {
            if json {
                println!("{}", serde_json::to_string(&v).unwrap_or_default());
            } else {
                print!("{}", render_queue_list(&v));
            }
            0
        }
        Err(e) => {
            eprintln!("{e}");
            e.exit_code()
        }
    }
}

/// The §2.2 table header, verbatim (column starts: `#`@4, `track`@7,
/// `artist`@47, `len` right-aligned in the trailing 4-wide field).
pub(super) const HEADER: &str = "    #  track                                   artist            len";

/// How many played rows render above the current track. A render cap only —
/// the wire `history` field is the full list (the §2.2 example shows one
/// played row; three keeps a long session's table from being mostly past).
const HISTORY_RENDER_CAP: usize = 3;

/// The §2.2 `queue list` table: up to [`HISTORY_RENDER_CAP`] played rows
/// (from the response's additive `history` field, recent-first on the wire,
/// rendered oldest-first), the current track marked `->`, then `upcoming` —
/// all numbered with the 1-based display position via `cli_position`.
///
/// History-row numbering is the linear-play reconstruction: positions count
/// back from the current track (`current - 1`, `current - 2`, …), which is
/// exact in the §2.2 example's sequential case. Under shuffle or after
/// index jumps a played track's TRUE absolute position is not derivable
/// from the wire shape (history entries carry no index), so those rows'
/// numbers are best-effort context — `queue remove` against them stays
/// safe: the daemon re-validates every index server-side (§3.3.15).
///
/// When nothing is current (`current_index: null`), `upcoming` already
/// holds the ENTIRE queue (`QueueManager::get_state_full`,
/// qbz-player/src/queue.rs:1036-1082), numbering starts at 1, and no
/// history rows render (there is no current row to anchor them to).
pub(super) fn render_queue_list(v: &Value) -> String {
    let current_index = v.get("current_index").and_then(|i| i.as_u64()).map(|i| i as usize);
    let total = v.get("total_tracks").and_then(|t| t.as_u64()).unwrap_or(0);
    let shuffle = v.get("shuffle").and_then(|s| s.as_bool()).unwrap_or(false);
    let repeat = v.get("repeat").and_then(|r| r.as_str()).unwrap_or("off");

    let mut out = String::new();
    out.push_str(HEADER);
    out.push('\n');

    let current_track = v.get("current_track").filter(|t| !t.is_null());
    if let (Some(idx), Some(track)) = (current_index, current_track) {
        let cur_pos = cli_position(idx);
        // Played rows: the `take` most recent history entries, printed
        // oldest-first so the most recent lands directly above the current
        // row, numbered backwards from it (never below position 1).
        if let Some(history) = v.get("history").and_then(|h| h.as_array()) {
            let take = HISTORY_RENDER_CAP.min(cur_pos.saturating_sub(1)).min(history.len());
            for (i, played) in history[..take].iter().rev().enumerate() {
                out.push_str(&render_row(false, cur_pos - take + i, played));
            }
        }
        out.push_str(&render_row(true, cur_pos, track));
    }

    let upcoming_start = current_index.map(|i| cli_position(i) + 1).unwrap_or(1);
    if let Some(upcoming) = v.get("upcoming").and_then(|u| u.as_array()) {
        for (i, track) in upcoming.iter().enumerate() {
            out.push_str(&render_row(false, upcoming_start + i, track));
        }
    }

    out.push_str(&format!(
        "{total} tracks · shuffle {} · repeat {repeat}\n",
        if shuffle { "on" } else { "off" }
    ));
    out
}

fn render_row(is_current: bool, position: usize, track: &Value) -> String {
    let marker = if is_current { "->" } else { "  " };
    let title = track.get("title").and_then(|t| t.as_str()).unwrap_or("");
    let artist = track.get("artist").and_then(|a| a.as_str()).unwrap_or("");
    let dur = track.get("duration_secs").and_then(|d| d.as_u64()).unwrap_or(0);
    format!(
        "{marker}{position:>3}  {title:<40}{artist:<17}{len:>4}\n",
        len = fmt_mmss(dur)
    )
}
