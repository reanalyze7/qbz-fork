//! Per-frame audio driver input + the album-art palette pushed on track
//! change.

/// Per-frame audio drivers handed to [`super::render_frame`] from the 30 fps
/// drain. `time` and resolution come from the render state; the album-art
/// palette is pushed separately via [`set_palette`] (it changes on track
/// change, not per tick). Energy bands and log bands are ALREADY smoothed
/// upstream (qbz-audio) — pass them raw, do not EMA again.
pub struct FrameAudio {
    pub level: f32,
    pub level_smooth: f32,
    pub beat: f32,
    pub phase: f32,
    pub transient: f32,
    pub energy: [f32; 5], // sub, bass, mid, presence, air
    pub bands: [f32; 8],  // 8 log FFT bands (paired from the 16 bars)
    /// Spectral-ribbon feed (mode 4): the latest 512-band frame to paint as a
    /// new column (None = no new frame this tick), the playback fraction 0..1
    /// for the column position, and a reset flag (track change / seek → clear).
    pub spectral: Option<Vec<f32>>,
    pub progress: f32,
    pub reset: bool,
    /// Smoothed fraction (0..1) of the highest active frequency band — drives the
    /// spectral-ribbon real-time ceiling line (mode 4). 0 for the other modes.
    pub spectral_peak: f32,
}

/// Album-art palette triad, normalized rgb (0..1, a = 1). Lives in its own
/// thread-local so a track's colors can be pushed before the render pipeline
/// exists (`set_palette` may run before `setup()`), and read on every frame.
#[derive(Clone, Copy)]
pub(super) struct Palette {
    pub(super) primary: [f32; 4],
    pub(super) secondary: [f32; 4],
    pub(super) accent: [f32; 4],
}
impl Palette {
    /// Matches the `ImmersiveState` defaults #00dcc8 / #9632ff / #3fd9c8 so a
    /// shader opened before album art resolves still gets sensible colors.
    pub(super) const DEFAULT: Palette = Palette {
        primary: [0.0, 0.862_745, 0.784_314, 1.0],
        secondary: [0.588_235, 0.196_078, 1.0, 1.0],
        accent: [0.247_059, 0.850_980, 0.784_314, 1.0],
    };
}

/// Push the album-art palette triad. Called on track change (playback.rs), not
/// per frame, from the UI thread. Stored in a thread-local independent of the
/// render pipeline so it survives if pushed before `setup()`.
pub fn set_palette(primary: slint::Color, secondary: slint::Color, accent: slint::Color) {
    fn norm(c: slint::Color) -> [f32; 4] {
        [
            c.red() as f32 / 255.0,
            c.green() as f32 / 255.0,
            c.blue() as f32 / 255.0,
            1.0,
        ]
    }
    super::lifecycle::PALETTE.with(|p| {
        *p.borrow_mut() = Palette {
            primary: norm(primary),
            secondary: norm(secondary),
            accent: norm(accent),
        };
    });
}
