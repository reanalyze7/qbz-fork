// crates/qbzd/src/cli/settings/reload.rs — the import flow's reload-nudge
// disposition mapping, live-system enumeration, and the interactive device
// re-pick prompt.

use qbz_app::settings::bundle::{Bundle, DeviceChoice, LiveSystem};
use qbz_audio::{AudioBackendType, BackendManager};
use std::io::Write;

/// Pure mapping of the nudge outcome (+ whether a routing-critical field
/// changed) to the `done:` reload phrase, an optional stderr error, and the
/// exit code. Split from IO so the §5.3-step-7 contract is unit-testable.
pub(super) fn reload_disposition(
    outcome: crate::login::NudgeOutcome,
    routing_critical: bool,
) -> (String, Option<String>, i32) {
    use crate::login::NudgeOutcome::*;
    match outcome {
        Reloaded => {
            // §5.3 step 7 honesty rule: a routing-critical change re-inits the
            // output device on the spot — say so instead of hiding it.
            let line = if routing_critical {
                "daemon reloaded (was running; output device reinitialized — an in-flight track may gap)"
            } else {
                "daemon reloaded (was running)"
            };
            (line.to_string(), None, 0)
        }
        DaemonDown => (
            "daemon not running (changes apply on next start)".to_string(),
            None,
            0,
        ),
        ReloadRefused => (
            "daemon answered ping but refused the reload".to_string(),
            Some(
                "error: settings saved but the daemon did not reload — restart it: systemctl --user restart qbzd"
                    .to_string(),
            ),
            1,
        ),
    }
}

/// Enumerate the local audio system for [`bundle::plan`]: the available backends
/// + the devices of the bundle's intended backend (the picker's candidate list).
pub(super) fn build_live_system(bundle: &Bundle) -> LiveSystem {
    let backends: Vec<String> = BackendManager::available_backends()
        .into_iter()
        .filter_map(|b| serde_json::to_value(b).ok().and_then(|v| v.as_str().map(str::to_string)))
        .collect();

    let wanted: Option<AudioBackendType> = bundle
        .domains
        .get("audio")
        .and_then(|a| a.get("backend_type"))
        .and_then(|v| serde_json::from_value(v.clone()).ok());
    let backend = wanted.unwrap_or(AudioBackendType::SystemDefault);
    let devices = BackendManager::create_backend(backend)
        .and_then(|b| b.enumerate_devices())
        .map(|list| list.into_iter().map(|d| (d.id, d.name)).collect())
        .unwrap_or_default();

    LiveSystem { backends, devices }
}

/// Interactive device picker (04 §5.4). Numbered device list; the last entry is
/// always "system default"; an unparseable answer falls to system default.
pub(super) fn prompt_device(pick: &qbz_app::settings::bundle::DevicePick) -> DeviceChoice {
    println!(
        "audio device \"{}\" not found on this machine. Available on {}:",
        pick.wanted, pick.backend
    );
    for (i, (id, label)) in pick.options.iter().enumerate() {
        println!("  [{}] {}  {}", i + 1, id, label);
    }
    let sys_idx = pick.options.len() + 1;
    println!("  [{sys_idx}] system default");
    print!("pick a device [1-{sys_idx}]: ");
    let _ = std::io::stdout().flush();
    let mut line = String::new();
    let _ = std::io::stdin().read_line(&mut line);
    match line.trim().parse::<usize>() {
        Ok(n) if n >= 1 && n <= pick.options.len() => {
            let (id, label) = pick.options[n - 1].clone();
            println!();
            DeviceChoice::Device { id, label }
        }
        _ => {
            println!();
            DeviceChoice::SystemDefault
        }
    }
}
