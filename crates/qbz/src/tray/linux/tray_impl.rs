//! `QbzTray`: the actual `ksni::Tray` trait surface.

use ksni::{Icon, MenuItem, Orientation, ToolTip, Tray};

use super::super::Runtime;
use crate::AppWindow;

/// Now-playing info shown in the tooltip. Cleared when no track is loaded.
#[derive(Clone, Debug, Default)]
pub(super) struct NowPlaying {
    pub(super) title: String,
    pub(super) artist: String,
    pub(super) album: String,
}

pub(super) struct QbzTray {
    pub(super) runtime: Runtime,
    pub(super) weak: slint::Weak<AppWindow>,
    pub(super) handle: tokio::runtime::Handle,
    pub(super) icons: Vec<Icon>,
    pub(super) now_playing: Option<NowPlaying>,
    pub(super) is_playing: bool,
}

impl QbzTray {
    pub(super) fn play_pause(&self) {
        super::super::dispatch_play_pause(self.runtime.clone(), self.weak.clone(), self.handle.clone());
    }
}

impl Tray for QbzTray {
    fn id(&self) -> String {
        "com.blitzfc.qbz".into()
    }

    fn title(&self) -> String {
        "QBZ".into()
    }

    fn icon_name(&self) -> String {
        // Intentionally empty: SNI panels (KDE Plasma especially) prefer
        // IconName over IconPixmap when both are present, and resolving the
        // app id against the icon theme picks up the full colour app icon
        // instead of our themed monochrome glyph (issue #362). An empty name
        // forces panels to render IconPixmap directly.
        String::new()
    }

    fn icon_pixmap(&self) -> Vec<Icon> {
        self.icons.clone()
    }

    fn tool_tip(&self) -> ToolTip {
        let (title, description) = match &self.now_playing {
            Some(np) => {
                let header = if np.title.is_empty() {
                    "QBZ".to_string()
                } else {
                    np.title.clone()
                };
                let mut lines: Vec<String> = Vec::with_capacity(3);
                if !np.artist.is_empty() {
                    lines.push(qbz_i18n::t_args("by {}", &[np.artist.as_str()]));
                }
                if !np.album.is_empty() {
                    lines.push(np.album.clone());
                }
                lines.push(if self.is_playing {
                    qbz_i18n::t("Middle-click to pause")
                } else {
                    qbz_i18n::t("Middle-click to play")
                });
                lines.push(qbz_i18n::t("Scroll to adjust volume"));
                (header, lines.join("\n"))
            }
            None => (
                "QBZ".to_string(),
                qbz_i18n::t("Music Player\nNothing playing"),
            ),
        };
        ToolTip {
            title,
            description,
            icon_name: String::new(),
            icon_pixmap: vec![],
        }
    }

    /// Primary click (left) — toggle main window visibility.
    fn activate(&mut self, _x: i32, _y: i32) {
        log::info!("[tray] primary activate (left click)");
        super::super::toggle_window(&self.weak);
    }

    /// Secondary click (middle) — play/pause.
    fn secondary_activate(&mut self, _x: i32, _y: i32) {
        log::info!("[tray] secondary activate (middle click) -> play/pause");
        self.play_pause();
    }

    /// Mouse wheel — adjust volume in 5%-per-notch steps. Panels report ±120
    /// per notch; touch-pad scrolls produce smaller deltas, so we normalise by
    /// 120 and fall back to `signum()`.
    fn scroll(&mut self, delta: i32, orientation: Orientation) {
        if !matches!(orientation, Orientation::Vertical) || delta == 0 {
            return;
        }
        let mut ticks = delta / 120;
        if ticks == 0 {
            ticks = delta.signum();
        }
        log::debug!("[tray] scroll delta={} ticks={}", delta, ticks);
        // Positive ticks = wheel-up = volume up.
        super::super::dispatch_volume_delta(
            self.runtime.clone(),
            self.weak.clone(),
            self.handle.clone(),
            ticks,
        );
    }

    fn menu(&self) -> Vec<MenuItem<Self>> {
        super::menu::build_menu()
    }
}
