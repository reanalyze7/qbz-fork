use serde_json::Value;

use crate::settings::bundle::{Bundle, BundleSource, ImportPlan, LiveSystem, PlanLine, ProfilePaths, SCHEMA_VERSION};

pub(super) fn scratch(name: &str) -> ProfilePaths {
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let base = std::env::temp_dir().join(format!(
        "qbz-bundle-{name}-{}-{nonce}",
        std::process::id()
    ));
    ProfilePaths {
        config_root: base.join("config"),
        data_root: base.join("data"),
    }
}

pub(super) fn cleanup(p: &ProfilePaths) {
    let _ = std::fs::remove_dir_all(p.data_root.parent().unwrap_or(&p.data_root));
}

pub(super) fn live() -> LiveSystem {
    LiveSystem {
        backends: vec!["SystemDefault".into(), "Alsa".into(), "PipeWire".into()],
        devices: vec![
            ("hw:1,0".into(), "Topping D90".into()),
            ("hw:0,0".into(), "Onboard".into()),
        ],
    }
}

pub(super) fn bundle_with(domains: serde_json::Value) -> Bundle {
    let obj = domains.as_object().cloned().unwrap_or_default();
    Bundle {
        schema_version: SCHEMA_VERSION,
        created_at: "2026-07-14T09:30:00Z".into(),
        source: BundleSource {
            app_version: "2.0.2".into(),
            profile: "desktop".into(),
            hostname: "workstation".into(),
        },
        domains: obj,
    }
}

pub(super) fn find(lines: &[PlanLine], key: &str) -> Option<PlanLine> {
    lines.iter().find(|l| l.key == key).cloned()
}

pub(super) fn find_contains(lines: &[PlanLine], needle: &str) -> Option<PlanLine> {
    lines.iter().find(|l| l.key.contains(needle)).cloned()
}

pub(super) fn write_of<'a>(plan: &'a ImportPlan, key: &str) -> Option<&'a Value> {
    plan.writes.iter().find(|(k, _)| k == key).map(|(_, v)| v)
}
