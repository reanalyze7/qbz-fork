/// Whether the caller should keep scanning (the remaining files in a folder,
/// or the remaining folders), or the scan was cancelled mid-loop (in which
/// case `Finished{Cancelled}` has already been emitted and every caller up
/// the chain must return immediately without running cleanup).
pub(super) enum ScanOutcome {
    Continue,
    Cancelled,
}
