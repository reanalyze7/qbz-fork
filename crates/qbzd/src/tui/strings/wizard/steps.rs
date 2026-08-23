// Step names (breadcrumb `Wizard › <step>`).
pub const WIZ_STEP_WELCOME: &str = "Welcome";
pub const WIZ_STEP_CHECK: &str = "Check";
pub const WIZ_STEP_SELECT: &str = "Select DACs";
pub const WIZ_STEP_REVIEW: &str = "Review";
pub const WIZ_STEP_TEST: &str = "Test";
pub const WIZ_STEP_DONE: &str = "Done";

// Per-step help bars.
pub const WIZ_HELP_WELCOME: &str = "Enter start · → next · Tab nav · q quit";
pub const WIZ_HELP_CHECK: &str = "up/down move · Enter override · → next · ← back · Esc quit wizard";
pub const WIZ_HELP_SELECT: &str = "up/down move · Space toggle · m manual · → next · ← back · Esc quit";
pub const WIZ_HELP_REVIEW: &str =
    "up/down block · c copy · C copy all · w save · PgUp/PgDn scroll · → next · ← back";
pub const WIZ_HELP_TEST: &str = "t play test · r re-read · → next (skip) · ← back · Esc quit wizard";
pub const WIZ_HELP_DONE: &str = "Enter finish · ← back · Esc close";

// Welcome step.
pub const WIZ_WELCOME_TITLE: &str = "HiFi / DAC Setup Wizard";
pub const WIZ_WELCOME_BODY: &str = "This wizard checks your PipeWire/ALSA audio stack, finds your DAC(s), and\ngenerates the exact bit-perfect config for each one. It never touches a system\nfile — you copy the blocks and apply them yourself.\n\nSteps: Check the stack · Select DACs · Review the config · Test playback.";
pub const WIZ_WELCOME_CTA: &str = "Enter start";

// Check step.
pub const WIZ_DISTRO: &str = "Distribution";
pub const WIZ_INIT: &str = "Init system";
pub const WIZ_HEALTH_CHECKING: &str = "checking your audio stack…";
pub const WIZ_HEALTH_READY: &str = "✓ your audio stack is ready for bit-perfect playback";
pub const WIZ_HEALTH_ATTENTION: &str = "! some pieces need attention before bit-perfect playback will work:";
pub const WIZ_NO_REMEDIATION: &str = "nothing to change — the commands below are for reference only.";

pub fn wiz_sandbox_note(name: &str) -> String {
    format!(
        "running inside {name} — the host audio stack can't be probed from here; \
the commands below are the reference setup for the distro/init you pick."
    )
}

// Select-DACs step.
pub const WIZ_SELECT_INTRO: &str = "Detected outputs — check the DAC(s) you want bit-perfect config for:";
pub const WIZ_DETECTING: &str = "detecting DACs…";
pub const WIZ_DAC_BADGE: &str = "  [likely DAC]";
pub const WIZ_DEFAULT_BADGE: &str = "  [default]";
pub const WIZ_NO_DACS: &str = "no outputs enumerated — is PipeWire running and pw-dump installed?\nyou can still enter a node.name manually with 'm'.";
pub const WIZ_MANUAL_HINT: &str = "m — enter a PipeWire node.name manually (alsa_output.* / alsa_input.*)";
pub const WIZ_MANUAL_ACCEPTED: &str = "manual node:";
pub const WIZ_MANUAL_TITLE: &str = "Manual node.name";
pub const WIZ_MANUAL_BODY: &str = "Paste a PipeWire node.name (must contain alsa_output or alsa_input):";
pub const WIZ_MANUAL_INVALID: &str = "not a valid node.name — it must contain alsa_output or alsa_input";
pub const WIZ_SELECT_GATE: &str = "select at least one DAC (or enter a node.name with 'm') before continuing";
