//! Bounded availability-probe subprocess runner.

use std::process::Command;

/// Runs `program args...` with all stdio discarded and reports whether it
/// exited successfully within `timeout`. Availability probing only — a missing
/// binary, a failing exit, or a timeout all mean "not available". Bounded so
/// the audio thread can never hang on a wedged daemon (#591/#592: playback
/// start must fail fast and fall back, not sit behind the 45s UI watchdog).
pub(crate) fn probe_command_ok(program: &str, args: &[&str], timeout: std::time::Duration) -> bool {
    use std::process::Stdio;
    let mut child = match Command::new(program)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
    {
        Ok(c) => c,
        Err(_) => return false, // binary not present (sandbox / not installed)
    };
    let start = std::time::Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(status)) => return status.success(),
            Ok(None) => {
                if start.elapsed() >= timeout {
                    let _ = child.kill();
                    let _ = child.wait();
                    log::warn!(
                        "[PipeWire Backend] availability probe `{}` timed out after {:?}",
                        program,
                        timeout
                    );
                    return false;
                }
                std::thread::sleep(std::time::Duration::from_millis(25));
            }
            Err(e) => {
                log::warn!(
                    "[PipeWire Backend] availability probe `{}` failed: {}",
                    program,
                    e
                );
                return false;
            }
        }
    }
}
