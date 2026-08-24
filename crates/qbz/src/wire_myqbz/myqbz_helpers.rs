use crate::*;

use myqbz::Grid;

// Re-issue mosaic artwork jobs for a grid after a toolbar rebuild (the row
// set / order changed, so visible cards need their covers reloaded).
pub(crate) fn refresh_covers(window: &AppWindow, grid: Grid, image_cache: &artwork::ImageCache) {
    let jobs = myqbz::artwork_jobs(window, grid);
    artwork::spawn_loads(jobs, window.as_weak(), image_cache.clone());
}
