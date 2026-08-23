// crates/qbzd/src/tui/screens/wizard/step.rs — the six-step transition table.

use crate::tui::strings as s;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WStep {
    Welcome,
    Check,
    SelectDacs,
    Review,
    Test,
    Done,
}

/// The linear order of the six steps (drives next/prev + the breadcrumb).
pub const STEP_ORDER: [WStep; 6] = [
    WStep::Welcome,
    WStep::Check,
    WStep::SelectDacs,
    WStep::Review,
    WStep::Test,
    WStep::Done,
];

fn step_index(step: WStep) -> usize {
    STEP_ORDER.iter().position(|s| *s == step).unwrap_or(0)
}

/// The next step (None at Done). Pure — the step-transition table test pins it.
pub fn next_step(step: WStep) -> Option<WStep> {
    STEP_ORDER.get(step_index(step) + 1).copied()
}

/// The previous step (None at Welcome).
pub fn prev_step(step: WStep) -> Option<WStep> {
    let i = step_index(step);
    if i == 0 {
        None
    } else {
        STEP_ORDER.get(i - 1).copied()
    }
}

impl WStep {
    /// The breadcrumb / step-name (`Wizard › <this>`).
    pub fn title(self) -> &'static str {
        match self {
            WStep::Welcome => s::WIZ_STEP_WELCOME,
            WStep::Check => s::WIZ_STEP_CHECK,
            WStep::SelectDacs => s::WIZ_STEP_SELECT,
            WStep::Review => s::WIZ_STEP_REVIEW,
            WStep::Test => s::WIZ_STEP_TEST,
            WStep::Done => s::WIZ_STEP_DONE,
        }
    }
}
