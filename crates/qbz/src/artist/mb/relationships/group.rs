use std::collections::HashMap;

use super::MbRelationshipRow;

pub(super) fn group_relations(
    rels: Vec<qbz_integrations::musicbrainz::RelatedArtist>,
    default_role: &str,
) -> Vec<MbRelationshipRow> {
    struct Pending {
        name: String,
        roles: Vec<String>,
        begin: Option<String>,
        end: Option<String>,
    }
    let mut by_mbid: HashMap<String, Pending> = HashMap::new();
    let mut order: Vec<String> = Vec::new();
    for r in rels {
        let begin = r.period.as_ref().and_then(|p| p.begin.clone());
        let end = r.period.as_ref().and_then(|p| p.end.clone());
        match by_mbid.get_mut(&r.mbid) {
            Some(existing) => {
                if let Some(role) = r.role.clone() {
                    if !existing.roles.iter().any(|rr| rr == &role) {
                        existing.roles.push(role);
                    }
                }
            }
            None => {
                order.push(r.mbid.clone());
                let mut roles = Vec::new();
                if let Some(role) = r.role.clone() {
                    roles.push(role);
                }
                by_mbid.insert(
                    r.mbid.clone(),
                    Pending {
                        name: r.name,
                        roles,
                        begin,
                        end,
                    },
                );
            }
        }
    }
    order
        .into_iter()
        .filter_map(|mbid| by_mbid.remove(&mbid).map(|p| (mbid, p)))
        .map(|(mbid, p)| {
            let period = format_period(p.begin.as_deref(), p.end.as_deref());
            let tooltip = if !p.roles.is_empty() {
                let roles_joined = p.roles.join(", ");
                if period.is_empty() {
                    roles_joined
                } else {
                    format!("{} ({})", roles_joined, period)
                }
            } else if !period.is_empty() {
                period.clone()
            } else {
                p.name.clone()
            };
            let role = p
                .roles
                .first()
                .cloned()
                .unwrap_or_else(|| default_role.to_string());
            MbRelationshipRow {
                mbid,
                name: p.name,
                role,
                tooltip,
            }
        })
        .collect()
}

fn format_period(begin: Option<&str>, end: Option<&str>) -> String {
    if begin.is_some() || end.is_some() {
        let b = begin.unwrap_or("?");
        let e = end.unwrap_or("present");
        format!("{} - {}", b, e)
    } else {
        String::new()
    }
}
