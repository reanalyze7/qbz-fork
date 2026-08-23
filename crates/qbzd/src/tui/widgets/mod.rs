// crates/qbzd/src/tui/widgets/ — reusable ratatui primitives for the setup TUI.
//
// Two interactive sub-widgets (a filterable select popup and a line input) plus
// the pure render helpers every screen shares (field rows, group headers, the
// bottom help bar, centered modals, spinner). No screen logic lives here —
// screens own their staged state and drive these.
mod field;
mod help;
mod layout;
mod lines;
mod overlay;
mod scroll;
mod section;
mod select_popup;
mod select_popup_draw;
mod text_fit;
mod text_input;
#[cfg(test)]
mod tests;

pub use field::{field_block, focus_style, mask, Field};
pub use help::{help_bar, help_spans};
pub use layout::{centered_rect, control_column, follow_scroll, sidebar_is_wide, sidebar_width, wrap};
pub use lines::{action_line, blank, err_line, note_line, warn_line, wrapped_note};
pub use overlay::{busy_overlay, modal, panel, spinner_frame};
pub use scroll::sections_scroll;
pub use section::{push_section, sections, FocusAnchor, Section};
pub use select_popup::{SelectOutcome, SelectPopup};
pub use text_input::{InputOutcome, TextInput};
