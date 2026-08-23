// crates/qbzd/src/tui/app/draw.rs — the top-level render tree: the frames
// shell layout (header/breadcrumb/sidebar/content/footer/help) plus the
// overlays, which sit above everything as the third navigation level.

use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::text::Line;
use ratatui::widgets::{Block, BorderType, Paragraph};
use ratatui::Frame;

use crate::tui::strings as s;
use crate::tui::theme;
use crate::tui::widgets;

use super::messages::DrawCtx;
use super::messages_worker::{Active, Overlay};
use super::nav::Focus;
use super::state::App;

impl App {
    pub fn draw(&self, f: &mut Frame) {
        let area = f.area();
        if area.width < 80 || area.height < 24 {
            let msg = s::too_small(area.width, area.height);
            f.render_widget(Paragraph::new(msg), area);
            return;
        }

        // FB3 frames layout: header · breadcrumb · [sidebar | content] · footer ·
        // help. The 80×24 floor budget is 1+1 (top chrome) + 1+1 (bottom chrome)
        // + a Min(3) body, so the content frame keeps ≥ 18 usable inner rows.
        let rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // header
                Constraint::Length(1), // breadcrumb
                Constraint::Min(3),    // body (sidebar + content)
                Constraint::Length(1), // footer (daemon state)
                Constraint::Length(1), // help bar
            ])
            .split(area);
        self.draw_header(f, rows[0]);
        self.draw_breadcrumb(f, rows[1]);

        let sidebar_w = widgets::sidebar_width(area.width);
        let wide = widgets::sidebar_is_wide(area.width);
        let body = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Length(sidebar_w), Constraint::Min(0)])
            .split(rows[2]);
        self.draw_sidebar(f, body[0], wide);

        // Content frame: accent border when the content owns focus, dim otherwise
        // (its title is gone — the breadcrumb names the section now).
        let border = if self.focus == Focus::Content {
            theme::accent()
        } else {
            theme::dim()
        };
        let block = Block::bordered()
            .border_type(BorderType::Rounded)
            .border_style(border);
        let inner = block.inner(body[1]);
        f.render_widget(block, body[1]);

        let ctx = DrawCtx {
            status: self.status.as_ref(),
        };
        match &self.active {
            Active::Account(sc) => sc.draw(f, inner, &ctx),
            Active::Audio(sc) => sc.draw(f, inner, &ctx),
            Active::Playback(sc) => sc.draw(f, inner, &ctx),
            Active::Network(sc) => sc.draw(f, inner, &ctx),
            Active::Bundle(sc) => sc.draw(f, inner, &ctx),
            Active::Wizard(sc) => sc.draw(f, inner, &ctx),
            Active::Scrobbler(sc) => sc.draw(f, inner, &ctx),
        }

        self.draw_footer(f, rows[3]);
        widgets::help_bar(f, rows[4], self.help_text());

        // Overlays (help / result / dirty-leave / busy) cover the WHOLE screen —
        // they are the third navigation level and sit above the frames.
        match &self.overlay {
            Overlay::Help => widgets::panel(
                f,
                area,
                s::HELP_TITLE,
                s::HELP_OVERLAY.lines().map(|l| Line::from(l.to_string())).collect(),
                0,
            ),
            Overlay::Result { title, lines } => {
                let body = lines.join("\n");
                widgets::modal(f, area, title, &body, s::RESULT_HINT);
            }
            Overlay::DirtyLeave { .. } => {
                widgets::modal(f, area, s::DIRTY_TITLE, s::DIRTY_BODY, s::DIRTY_HINT);
            }
            Overlay::ConfirmAbandon => {
                widgets::modal(f, area, s::WIZ_ABANDON_TITLE, s::WIZ_ABANDON_BODY, s::WIZ_ABANDON_HINT);
            }
            Overlay::None => {}
        }

        if let Some(label) = &self.busy {
            widgets::busy_overlay(f, area, label, self.busy_tick);
        }
    }
}
