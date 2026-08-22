use super::role_labels::performer_role_labels;
use super::Performer;

/// Build the lookup key Qobuz frontends use for role i18n: first char
/// lowercased, the rest with spaces stripped (mirrors Tauri `formatRole`).
fn role_key(role: &str) -> String {
    let mut chars = role.chars();
    match chars.next() {
        Some(first) => {
            let lowered: String = first.to_lowercase().collect();
            let rest: String = chars.filter(|c| *c != ' ').collect();
            format!("{lowered}{rest}")
        }
        None => String::new(),
    }
}

/// Fallback humanizer (mirrors Tauri `formatUnknownRole` minus the final
/// upper-casing, which the caller applies): insert a space before each
/// uppercase letter, then trim. e.g. "CustomRole" -> "Custom Role".
fn humanize_role(role: &str) -> String {
    let mut out = String::with_capacity(role.len() + 4);
    for (i, c) in role.chars().enumerate() {
        if i > 0 && c.is_uppercase() {
            out.push(' ');
        }
        out.push(c);
    }
    out.trim().to_string()
}

/// Human-readable role label, ported 1:1 from the Tauri `performerRoles` i18n
/// map (English) with the same `formatRole` key + `formatUnknownRole`
/// fallback. NOT upper-cased — the Track Info grid upper-cases at render
/// (Tauri uses CSS `text-transform: uppercase`).
pub fn format_role_label(role: &str) -> String {
    let key = role_key(role);
    for (k, label) in performer_role_labels() {
        if *k == key {
            return (*label).to_string();
        }
    }
    humanize_role(role)
}

/// Ordered, deduped role grouping — 1:1 with Tauri `getGroupedCredits`:
/// group performer names by role (dedup within a role, first-seen order),
/// then order roles as: Composer, Lyricist first (Composer before Lyricist);
/// MainArtist / "Main Artist" last; everything else alphabetical
/// (case-insensitive). Returns `(role, names)` pairs.
pub fn group_credits_ordered(performers: &[Performer]) -> Vec<(String, Vec<String>)> {
    // Preserve first-seen role order while grouping (mirror of JS object key
    // insertion order before the explicit sort).
    let mut order: Vec<String> = Vec::new();
    let mut grouped: std::collections::HashMap<String, Vec<String>> =
        std::collections::HashMap::new();

    for performer in performers {
        for role in &performer.roles {
            let entry = grouped.entry(role.clone()).or_insert_with(|| {
                order.push(role.clone());
                Vec::new()
            });
            if !entry.contains(&performer.name) {
                entry.push(performer.name.clone());
            }
        }
    }

    let is_first = |r: &str| {
        let r = r.to_lowercase();
        r == "composer" || r == "lyricist"
    };
    let is_last = |r: &str| {
        let r = r.to_lowercase();
        r == "mainartist" || r == "main artist"
    };

    order.sort_by(|a, b| {
        use std::cmp::Ordering;
        let (af, bf) = (is_first(a), is_first(b));
        let (al, bl) = (is_last(a), is_last(b));
        if af && !bf {
            return Ordering::Less;
        }
        if !af && bf {
            return Ordering::Greater;
        }
        if al && !bl {
            return Ordering::Greater;
        }
        if !al && bl {
            return Ordering::Less;
        }
        if af && bf {
            if a.to_lowercase() == "composer" {
                return Ordering::Less;
            }
            if b.to_lowercase() == "composer" {
                return Ordering::Greater;
            }
        }
        a.to_lowercase().cmp(&b.to_lowercase())
    });

    order
        .into_iter()
        .map(|role| {
            let names = grouped.remove(&role).unwrap_or_default();
            (role, names)
        })
        .collect()
}
