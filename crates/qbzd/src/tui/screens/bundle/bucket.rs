use qbz_app::settings::bundle::PlanLine;
use ratatui::style::Modifier;
use ratatui::text::{Line, Span};

use crate::tui::theme;

pub(super) enum BucketKind {
    Applied,
    Adapted,
    Skipped,
}

pub(super) fn bucket(lines: &mut Vec<Line<'static>>, title: &str, rows: &[PlanLine], kind: BucketKind) {
    // Bucket headers carry a semantic tint (applies=ok, adapted=warn, skipped=dim);
    // the count and label stand on their own without it.
    let head_style = match kind {
        BucketKind::Applied => theme::ok().add_modifier(Modifier::BOLD),
        BucketKind::Adapted => theme::warn().add_modifier(Modifier::BOLD),
        BucketKind::Skipped => theme::dim().add_modifier(Modifier::BOLD),
    };
    lines.push(Line::from(Span::styled(
        format!("{title} ({})", rows.len()),
        head_style,
    )));
    for l in rows {
        let text = match kind {
            BucketKind::Applied => format!("  {} = {}", l.key, l.new),
            BucketKind::Adapted => format!(
                "  {} {} -> {} ({})",
                l.key,
                l.old.as_deref().unwrap_or(""),
                l.new,
                l.why
            ),
            BucketKind::Skipped => format!("  {}  {}", l.key, l.why),
        };
        lines.push(Line::from(text));
    }
}
