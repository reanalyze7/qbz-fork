use ratatui::layout::Rect;
use ratatui::text::Line;

use crate::tui::widgets::{sections_scroll, FocusAnchor, Section};

#[test]
fn sections_scroll_indicates_and_brings_the_focused_block_into_view() {
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;
    // Four 4-line boxes = 4 * (4 + 2) = 24 rows into a 12-row area → overflow.
    let mk = |t: &str| {
        Section::new(t, false, (0..4).map(|i| Line::from(format!("{t}-{i}"))).collect())
    };
    let secs = vec![mk("A"), mk("B"), mk("C"), mk("D")];
    // Focus the LAST box's first line — it starts well below the fold.
    let anchor = Some(FocusAnchor { section: 3, inner_line: 0, height: 1 });
    let mut term = Terminal::new(TestBackend::new(30, 12)).unwrap();
    term.draw(|f| sections_scroll(f, Rect::new(0, 0, 30, 12), &secs, anchor))
        .unwrap();
    let buf = term.backend().buffer().clone();
    let mut out = String::new();
    for y in 0..12 {
        for x in 0..30 {
            out.push_str(buf[(x, y)].symbol());
        }
        out.push('\n');
    }
    assert!(out.contains('▲'), "content hidden above → up indicator: \n{out}");
    assert!(out.contains('▼'), "content hidden below → down indicator");
    assert!(out.contains("D-0"), "the focused block is scrolled into view");
    assert!(!out.contains("A-0"), "the top box scrolled out of view");
}
