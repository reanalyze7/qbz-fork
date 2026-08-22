//! Skip-to-item-boundary pure logic. Boundary := source_item_id_hint, or
//! album_id as fallback when the hint is absent (no I/O; unit-testable alone).

use qbz_models::QueueTrack as CoreQueueTrack;

fn boundary_of(queue: &[CoreQueueTrack], i: usize) -> Option<&str> {
    queue
        .get(i)
        .and_then(|track| track.source_item_id_hint.as_deref().or(track.album_id.as_deref()))
}

/// Given a queue and a current index, find the next index whose item boundary
/// differs from the current.
pub fn next_item_index(queue: &[CoreQueueTrack], current: usize) -> Option<usize> {
    let current_boundary = boundary_of(queue, current)?;
    for i in (current + 1)..queue.len() {
        if boundary_of(queue, i) != Some(current_boundary) {
            return Some(i);
        }
    }
    None
}

/// Mirror: depending on elapsed ms and whether we're at item-start, either
/// restart the current item or jump to start of the previous item.
pub fn previous_item_index(
    queue: &[CoreQueueTrack],
    current: usize,
    current_elapsed_ms: u64,
) -> Option<usize> {
    if current >= queue.len() {
        return None;
    }
    let current_boundary = boundary_of(queue, current)?.to_string();

    let mut item_start = current;
    while item_start > 0 && boundary_of(queue, item_start - 1) == Some(current_boundary.as_str()) {
        item_start -= 1;
    }

    // If elapsed > 3s OR we are mid-item, seek to item_start.
    if current_elapsed_ms > 3_000 || current > item_start {
        return Some(item_start);
    }

    // Otherwise jump to start of previous item.
    if item_start == 0 {
        return Some(0);
    }
    let prev_boundary = boundary_of(queue, item_start - 1)?.to_string();
    let mut prev_start = item_start - 1;
    while prev_start > 0 && boundary_of(queue, prev_start - 1) == Some(prev_boundary.as_str()) {
        prev_start -= 1;
    }
    Some(prev_start)
}
