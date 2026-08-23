use serde_json::{json, Value};

use super::defaults::default_prefs;
use super::model::{DiscoverPrefs, SectionPref};
use super::section_id::DiscoverySectionId;

impl DiscoverPrefs {
    // ---- JSON (persistence + migration) ----

    /// Serialize to the Tauri-compatible by-tab JSON object.
    pub fn to_json(&self) -> Value {
        let arr = |list: &[SectionPref]| -> Value {
            Value::Array(
                list.iter()
                    .map(|p| json!({ "id": p.id.as_str(), "enabled": p.enabled }))
                    .collect(),
            )
        };
        json!({
            "home": arr(&self.home),
            "editorPicks": arr(&self.editor_picks),
            "forYou": arr(&self.for_you),
            "showRecommendations": self.show_recommendations,
            "recoCacheTtlHours": self.reco_cache_ttl_hours,
        })
    }

    /// Migrate any persisted value into a complete, valid `DiscoverPrefs`.
    /// Three branches, IN ORDER (mirrors `migrate` in `sectionPrefs.ts`):
    ///   1. Array  -> legacy V1 home-only: reconcile as Home; the other two
    ///      tabs get raw defaults.
    ///   2. Object -> reconcile each of the 3 tabs against its defaults
    ///      (a missing tab key reconciles to that tab's defaults).
    ///   3. Anything else (null / number / string / parse failure upstream)
    ///      -> full defaults.
    pub fn migrate(value: &Value) -> DiscoverPrefs {
        let defaults = default_prefs();
        if let Some(arr) = value.as_array() {
            DiscoverPrefs {
                home: reconcile_list(Some(arr), &defaults.home),
                editor_picks: defaults.editor_picks,
                for_you: defaults.for_you,
                // Legacy V1 (home-only array) predates the flag -> default on.
                show_recommendations: true,
                // Legacy V1 predates the cache-window setting -> default 48h.
                reco_cache_ttl_hours: 48,
            }
        } else if value.is_object() {
            let get = |key: &str| value.get(key).and_then(|v| v.as_array());
            DiscoverPrefs {
                home: reconcile_list(get("home"), &defaults.home),
                editor_picks: reconcile_list(get("editorPicks"), &defaults.editor_picks),
                for_you: reconcile_list(get("forYou"), &defaults.for_you),
                // Missing key (older persisted blob) -> default on.
                show_recommendations: value
                    .get("showRecommendations")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(true),
                // Validate against the offered set; unknown / missing -> 48h.
                reco_cache_ttl_hours: value
                    .get("recoCacheTtlHours")
                    .and_then(|v| v.as_i64())
                    .filter(|h| [24, 36, 48, 72].contains(h))
                    .unwrap_or(48),
            }
        } else {
            defaults
        }
    }
}

/// Reconcile a persisted per-tab array against the tab's fallback defaults.
///
/// Output = (valid persisted ids in stored order) ++ (default ids not seen,
/// appended in default order). An entry is kept only if it is an object with a
/// string `id` that (a) maps to a known section, (b) is in the fallback id set,
/// and (c) has not already been seen (first-occurrence wins). `enabled` is
/// coerced to a strict bool (missing / non-bool -> false).
pub fn reconcile_list(persisted: Option<&Vec<Value>>, fallback: &[SectionPref]) -> Vec<SectionPref> {
    let Some(arr) = persisted else {
        return fallback.to_vec();
    };
    let allowed: std::collections::HashSet<DiscoverySectionId> =
        fallback.iter().map(|p| p.id).collect();

    let mut seen: std::collections::HashSet<DiscoverySectionId> = std::collections::HashSet::new();
    let mut out: Vec<SectionPref> = Vec::new();
    for entry in arr {
        let Some(obj) = entry.as_object() else {
            continue;
        };
        let Some(id) = obj.get("id").and_then(|v| v.as_str()).and_then(DiscoverySectionId::from_str)
        else {
            continue;
        };
        if !allowed.contains(&id) || seen.contains(&id) {
            continue;
        }
        let enabled = obj.get("enabled").and_then(|v| v.as_bool()).unwrap_or(false);
        seen.insert(id);
        out.push(SectionPref { id, enabled });
    }
    // Append every fallback entry whose id was not seen, in fallback order.
    for p in fallback {
        if !seen.contains(&p.id) {
            out.push(*p);
        }
    }
    out
}
