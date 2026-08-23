//! Pick (and lazily compile+cache) the pipeline for the active mode:
//! line-bed (mode 5) uses its own line-strip pipeline; the fullscreen
//! scenes map mode -> `SHADER_SOURCES` index.

use slint::wgpu_28::wgpu;

use super::super::resources::{build_linebed_pipeline, build_scene_pipeline, GpuResources, SHADER_SOURCES};

/// Bounds-guard: fall back to the plasma pipeline (index 0) if a mode is
/// ever out of range, so the underlay degrades gracefully instead of
/// panicking on an indexing error.
pub(super) fn pick_pipeline<'a>(
    res: &'a mut GpuResources,
    device: &wgpu::Device,
    mode: i32,
) -> Option<&'a wgpu::RenderPipeline> {
    if mode == 5 {
        if res.linebed_pipeline.is_none() {
            res.linebed_pipeline = Some(build_linebed_pipeline(device, &res.pipeline_layout));
        }
        return res.linebed_pipeline.as_ref();
    }
    // modes 1-4 → pipelines[0..3]; mode 6 (liquid spectrum) → pipelines[4];
    // mode 7 (ambient, app-wide) → pipelines[5]. mode 5 is line_bed's own
    // pipeline above, so the index skips it.
    let idx = if mode == 6 {
        4
    } else if mode == 7 {
        5
    } else {
        (mode - 1) as usize
    };
    let idx = if idx < SHADER_SOURCES.len() { idx } else { 0 };
    if res.pipelines[idx].is_none() {
        res.pipelines[idx] = Some(build_scene_pipeline(device, &res.pipeline_layout, idx));
    }
    res.pipelines[idx].as_ref()
}
