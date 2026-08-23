use ratatui::text::Line;

use crate::tui::widgets::{field_block, Field};

fn cells(line: &Line) -> String {
    line.spans.iter().map(|s| s.content.as_ref()).collect()
}

#[test]
fn field_block_anchors_the_control_and_right_aligns_the_widget() {
    let f = Field {
        label: "Backend",
        value: "PipeWire".to_string(),
        widget: "[select]",
        focused: false,
        enabled: true,
        reason: None,
        description: None,
    };
    let block = field_block(&f, 21, 62);
    assert_eq!(block.len(), 1, "no description → just the control row");
    let row = cells(&block[0]);
    assert_eq!(row.chars().count(), 62, "the row spans the full width");
    // Label indented by 2, value starts at the control column.
    assert!(row.starts_with("  Backend"));
    assert_eq!(&row[21..29], "PipeWire", "value begins exactly at the control column");
    assert!(row.trim_end().ends_with("[select]"), "widget marker right-aligned");
}

#[test]
fn field_block_truncates_a_long_value_with_ellipsis() {
    let f = Field {
        label: "Output device",
        value: "a-really-long-alsa-hardware-device-identifier-that-overflows".to_string(),
        widget: "[select]",
        focused: false,
        enabled: true,
        reason: None,
        description: None,
    };
    let block = field_block(&f, 21, 40); // narrow → must truncate
    let row = cells(&block[0]);
    assert_eq!(row.chars().count(), 40);
    assert!(row.contains('…'), "the overflowing value is truncated with an ellipsis");
}

#[test]
fn field_block_wraps_the_disabled_reason_under_the_label() {
    let f = Field {
        label: "Gapless playback",
        value: "off".to_string(),
        widget: "[toggle]",
        focused: false,
        enabled: false,
        reason: Some("off while Audio > Streaming only on"),
        description: None,
    };
    let block = field_block(&f, 21, 40);
    assert!(block.len() >= 2, "the reason wraps onto its own dim row(s)");
    // Every description row is indented and within the width.
    for row in &block[1..] {
        let t = cells(row);
        assert!(t.starts_with("    "), "description indented under the label");
        assert!(t.chars().count() <= 40);
    }
}
