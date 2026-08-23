use ratatui::layout::Rect;

// ============================ sidebar width (FB5) ============================

/// The left-nav sidebar width in columns (incl. border), a pure function of the
/// terminal width so the 80×24 floor keeps working (FB5). At ≥ 100 cols the
/// operator gets the roomy 28-col sidebar the owner asked for (labels spelled
/// out, room for a dim summary line); below that we fall back to the compact
/// 14-col rendering so the content frame never starves at the floor.
pub fn sidebar_width(term_width: u16) -> u16 {
    if term_width >= 100 {
        28
    } else {
        14
    }
}

/// True when `sidebar_width` is in its roomy tier (spelled-out labels + summary).
pub fn sidebar_is_wide(term_width: u16) -> bool {
    sidebar_width(term_width) >= 28
}

// ============================ word wrap (FB5) ============================

/// Word-boundary wrap `text` to `width` columns, no new dependency (FB5). Splits
/// on ASCII whitespace; a word longer than `width` is hard-split (the only place
/// a word is broken mid-word). Blank/whitespace-only input yields no lines. Each
/// embedded `\n` in the source is honored as a hard break first, then each
/// segment is word-wrapped, so pre-formatted copy keeps its intentional breaks.
pub fn wrap(text: &str, width: usize) -> Vec<String> {
    if width == 0 {
        return text.lines().map(str::to_string).collect();
    }
    let mut out: Vec<String> = Vec::new();
    for segment in text.split('\n') {
        if segment.trim().is_empty() {
            continue;
        }
        let mut cur = String::new();
        for word in segment.split_whitespace() {
            if word.chars().count() > width {
                if !cur.is_empty() {
                    out.push(std::mem::take(&mut cur));
                }
                let mut chunk = String::new();
                for ch in word.chars() {
                    if chunk.chars().count() == width {
                        out.push(std::mem::take(&mut chunk));
                    }
                    chunk.push(ch);
                }
                cur = chunk;
                continue;
            }
            let extra = usize::from(!cur.is_empty());
            if cur.chars().count() + extra + word.chars().count() > width {
                out.push(std::mem::take(&mut cur));
                cur.push_str(word);
            } else {
                if !cur.is_empty() {
                    cur.push(' ');
                }
                cur.push_str(word);
            }
        }
        if !cur.is_empty() {
            out.push(cur);
        }
    }
    out
}

/// The column (0-based, from the section inner edge) where every field's control
/// starts — the owner's "misma área de columna". ONE mechanism, applied on every
/// screen: `2` (indent) + the longest label + `2` (gap), clamped so the control
/// still has room. Pure so each screen derives an identical column for its own
/// label set.
pub fn control_column(labels: &[&str], width: u16) -> u16 {
    let max_label = labels
        .iter()
        .map(|l| l.chars().count())
        .max()
        .unwrap_or(0) as u16;
    let ceiling = width.saturating_sub(12).max(14);
    (2 + max_label + 2).clamp(14, ceiling)
}

/// The minimal vertical scroll (in rows) that keeps `[focus_top, focus_top +
/// focus_height)` inside a `viewport`-tall window over `total` rows (FB5). Pure so
/// the follow-focus math is unit-tested independent of any buffer.
pub fn follow_scroll(focus_top: u16, focus_height: u16, viewport: u16, total: u16) -> u16 {
    if total <= viewport || viewport == 0 {
        return 0;
    }
    let max_scroll = total - viewport;
    let mut scroll = 0u16;
    let focus_bottom = focus_top.saturating_add(focus_height.max(1));
    if focus_bottom > viewport {
        scroll = focus_bottom - viewport;
    }
    // A block taller than the viewport (or above the current offset): pin its top.
    if focus_top < scroll {
        scroll = focus_top;
    }
    scroll.min(max_scroll)
}

/// Center a fixed-size rect inside `area` (clamped to fit).
pub fn centered_rect(width: u16, height: u16, area: Rect) -> Rect {
    let w = width.min(area.width);
    let h = height.min(area.height);
    let x = area.x + (area.width.saturating_sub(w)) / 2;
    let y = area.y + (area.height.saturating_sub(h)) / 2;
    Rect { x, y, width: w, height: h }
}

