//! Paginator maths, extracted so it's unit-testable independent of the
//! Slint/async plumbing.

use super::PAGE_SIZE;

/// One page's bounds within a `total`-length list, `PAGE_SIZE` per page.
pub(super) struct PageBounds {
    /// Page index clamped into `[0, page_count)`.
    pub(super) page: usize,
    /// Total number of pages (at least 1).
    pub(super) page_count: usize,
    /// Zero-based slice start.
    pub(super) start: usize,
    /// Zero-based slice end (exclusive).
    pub(super) end: usize,
}

/// Compute the paginator bounds for `total` items at the requested
/// `page`. The page is clamped so a shrunk list never leaves the
/// paginator past the end. Extracted so the maths is unit-testable.
pub(super) fn paginate(total: usize, requested_page: usize) -> PageBounds {
    let page_count = total.div_ceil(PAGE_SIZE).max(1);
    let page = requested_page.min(page_count - 1);
    let start = page * PAGE_SIZE;
    let end = (start + PAGE_SIZE).min(total);
    PageBounds {
        page,
        page_count,
        start,
        end,
    }
}
