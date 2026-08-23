//! The full markdown diagnostics report (uploaded diagnostics paste) — the
//! same data `refresh` gathers, formatted as System/Audio/Graphics/
//! Environment/Playback sections.

mod audio;
mod env;
mod graphics;
mod playback;
mod system;

use super::collect::gather_blocking;
use super::Runtime;

/// Append a `- **key:** value` markdown bullet (one self-contained line, so it
/// renders correctly without relying on trailing-whitespace hard breaks).
pub(super) fn md_line(out: &mut String, key: &str, value: &str) {
    out.push_str("- **");
    out.push_str(key);
    out.push_str(":** ");
    out.push_str(value);
    out.push('\n');
}

/// Build a COMPLETE, human-readable markdown diagnostics report — the same data
/// `refresh` gathers, formatted for the uploaded paste. The caller appends logs
/// separately, so this is the non-log body. Runs in an async tokio context, so
/// `tokio::task::spawn_blocking` works without an explicit handle.
pub async fn build_full_report(runtime: &Runtime) -> String {
    // (a) blocking: the three settings stores + /proc + /sys + CPAL sinks.
    let collected = tokio::task::spawn_blocking(gather_blocking).await;

    let (d, sys, active_output, available_outputs, active_fmt) = match collected {
        Ok(v) => v,
        Err(e) => {
            return format!(
                "# qbz diagnostics\n\n**Version:** {}\n\nFailed to gather diagnostics: {e}\n",
                env!("CARGO_PKG_VERSION")
            );
        }
    };

    // (b) async core snapshot for the Playback section.
    let pb = runtime.core().get_playback_state();
    let track = runtime.core().current_track().await;

    let mut out = String::new();
    out.push_str("# qbz diagnostics\n\n");
    md_line(&mut out, "Version", env!("CARGO_PKG_VERSION"));
    md_line(
        &mut out,
        "Generated",
        &chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
    );

    system::write_section(&mut out, &sys);
    audio::write_section(&mut out, &d, active_output.as_deref(), &available_outputs, &active_fmt);
    graphics::write_section(&mut out, &d);
    env::write_section(&mut out, &d);
    playback::write_section(&mut out, &pb, track.as_ref());

    out
}
