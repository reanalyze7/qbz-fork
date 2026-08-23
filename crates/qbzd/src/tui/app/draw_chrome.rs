// crates/qbzd/src/tui/app/draw_chrome.rs — the header/breadcrumb/footer chrome
// rows + the context-sensitive help-bar text (the sidebar itself is drawn in
// `draw_sidebar.rs`).

use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use crate::tui::strings as s;
use crate::tui::theme;

use super::messages_worker::Active;
use super::nav::{breadcrumb_nodes, section_title, Focus};
use super::state::App;
use super::worker_fns_ext::{footer_state, playing_extra};

impl App {
    /// Header row: `QBZ Daemon Setup` (accent-bold, left) · `qbzd <version>`
    /// (dim, right). One row, always visible.
    pub(super) fn draw_header(&self, f: &mut Frame, area: Rect) {
        let title = s::APP_TITLE;
        let version = format!("qbzd {}", env!("CARGO_PKG_VERSION"));
        let used = 1 + title.chars().count() + version.chars().count() + 1;
        let pad = (area.width as usize).saturating_sub(used);
        let line = Line::from(vec![
            Span::raw(" "),
            Span::styled(title.to_string(), theme::accent_bold()),
            Span::raw(" ".repeat(pad)),
            Span::styled(version, theme::dim()),
            Span::raw(" "),
        ]);
        f.render_widget(Paragraph::new(line), area);
    }

    /// Breadcrumb row (max 2 levels): dim `Setup ›` prefix + accent current node;
    /// while a field is edited it becomes `<Section> › <Field>`.
    pub(super) fn draw_breadcrumb(&self, f: &mut Frame, area: Rect) {
        let section = section_title(self.active_section);
        let (prefix, current) = breadcrumb_nodes(section, self.active_editing_label());
        let line = Line::from(vec![
            Span::raw(" "),
            Span::styled(prefix.to_string(), theme::dim()),
            Span::styled(" › ".to_string(), theme::dim()),
            Span::styled(current.to_string(), theme::accent()),
        ]);
        f.render_widget(Paragraph::new(line), area);
    }

    /// The daemon-state footer, color-coded via `footer_state`. Never color
    /// alone — every state spells itself out.
    pub(super) fn draw_footer(&self, f: &mut Frame, area: Rect) {
        let playing = self.status.as_ref().and_then(playing_extra);
        let (text, style) = footer_state(self.reachable, self.auth.logged_in, playing);
        f.render_widget(
            Paragraph::new(Line::from(Span::styled(text, style))),
            area,
        );
    }

    pub(super) fn help_text(&self) -> &'static str {
        match self.focus {
            Focus::Nav => s::HELP_NAV,
            Focus::Content => match &self.active {
                Active::Audio(sc) => {
                    if sc.is_dirty() {
                        s::HELP_AUDIO_DIRTY
                    } else {
                        s::HELP_AUDIO_CLEAN
                    }
                }
                Active::Wizard(sc) => sc.help_text(),
                Active::Scrobbler(_) => s::HELP_SCROBBLER,
                _ => {
                    if self.active_is_dirty() {
                        s::HELP_CONTENT_DIRTY
                    } else {
                        s::HELP_CONTENT_CLEAN
                    }
                }
            },
        }
    }
}
