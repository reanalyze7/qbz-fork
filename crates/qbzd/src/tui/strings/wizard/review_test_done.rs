// Review step.
pub const WIZ_GENERATING: &str = "generating per-DAC config…";
pub const WIZ_BACKUP_HINT: &str = "tip: back up ~/.config/pipewire + ~/.config/wireplumber before applying anything.";
pub const WIZ_REVIEW_FOOTER: &str =
    "the wizard NEVER writes these files — copy (c/C) or save (w), then apply them yourself";
pub const WIZ_SAVED_TO: &str = "saved to";
pub const WIZ_SAVE_FAILED: &str = "could not save";

pub fn wiz_copied_all(n: usize) -> String {
    if n == 1 {
        "copied 1 block".to_string()
    } else {
        format!("copied all {n} blocks")
    }
}

// Test step.
pub const WIZ_TEST_INTRO: &str = "Play a track through the daemon, then read the DAC's REAL negotiated rate back\n(from /proc/asound) — the requested vs negotiated rate is the bit-perfect proof.";
pub const WIZ_TEST_NOTHING: &str = "nothing playing yet — press t to start the current queue";
pub const WIZ_TEST_WAITING: &str = "waiting for the DAC to open a stream…";
pub const WIZ_TEST_MATCHED: &str = "✓ the DAC clock matches what QBZ requested — bit-perfect";
pub const WIZ_TEST_REFERENCE: &str = "known reference track:";
pub const WIZ_TEST_SEEDS_HEADER: &str = "reference tracks you can cast/queue to verify each rate:";

// Done step.
pub const WIZ_DONE_TITLE: &str = "All set";
pub const WIZ_DONE_REMINDER: &str = "reminder: QBZ never writes system audio configs. Apply the blocks you copied,\nthen restart your PipeWire/WirePlumber user services (or log out and back in).";
pub const WIZ_DONE_RESTART: &str = "on this box, that is:";
pub const WIZ_DONE_CTA: &str = "Enter finish";

pub fn wiz_done_summary(dacs: usize) -> String {
    match dacs {
        0 => "No DAC config was generated — re-run the wizard to select a DAC.".to_string(),
        1 => "Generated bit-perfect config for 1 DAC.".to_string(),
        n => format!("Generated bit-perfect config for {n} DACs."),
    }
}

// Confirm-abandon modal (Esc mid-wizard).
pub const WIZ_ABANDON_TITLE: &str = "Quit the wizard?";
pub const WIZ_ABANDON_BODY: &str = "Your selections and generated config will be discarded.";
pub const WIZ_ABANDON_HINT: &str = "y quit · Esc stay";
