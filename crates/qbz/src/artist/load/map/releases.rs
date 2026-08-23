use std::collections::{BTreeMap, HashMap, HashSet};

use qbz_models::PageArtistReleaseGroup;

use crate::artist::data::{title_case, LabelData, ReleaseSection, RELEASE_SECTION_ORDER};
use crate::artist::track_map::map_release;

/// Releases: server-driven bucketing (the official webplayer is the
/// source of truth). Each `releases[]` group is keyed by its own
/// `type` (release_type) — we render EVERY non-empty bucket, in the
/// official order, trusting the server's key. We never re-derive
/// buckets by heuristic and never collapse them into a curated few.
/// Foreign-artist releases (guest spots that surface inside a group)
/// are filtered out, and ids are deduped across groups so a release
/// listed in more than one group appears once. The `awardedRelease`
/// bucket can appear twice in the array (a server quirk) — keying by
/// release_type naturally folds the two into one section.
///
/// Also collects labels from the artist's own album releases (only
/// group.type == "album", only own releases, dedupe by label id, sorted by
/// name — sidebar Labels section).
pub(crate) fn map_releases(
    groups: Option<Vec<PageArtistReleaseGroup>>,
) -> (Vec<ReleaseSection>, Vec<LabelData>) {
    let mut bucket_cards: HashMap<String, Vec<crate::home::CardData>> = HashMap::new();
    let mut bucket_has_more: HashMap<String, bool> = HashMap::new();
    let mut seen_release_ids: HashSet<String> = HashSet::new();
    let mut labels_by_id: BTreeMap<u64, String> = BTreeMap::new();

    for group in groups.into_iter().flatten() {
        let release_type = group.release_type.clone();
        let is_album_group = release_type == "album";
        *bucket_has_more.entry(release_type.clone()).or_insert(false) |= group.has_more;
        for release in group.items.into_iter() {
            // NO foreign-artist filter: the official webplayer renders every
            // item the server placed in the bucket — including releases
            // credited to the artist's band or where they only guest (e.g.
            // Vicky Psarakis' albums are credited to "Sicksense"). The old
            // `artist.id == page.id` filter dropped exactly those, hiding a
            // real Albums section. Trust the server's bucketing (D3).
            if seen_release_ids.contains(&release.id) {
                continue;
            }
            seen_release_ids.insert(release.id.clone());

            // Label collection — before consuming `release` into the card.
            if is_album_group {
                if let Some(label) = release.label.as_ref() {
                    labels_by_id
                        .entry(label.id)
                        .or_insert_with(|| label.name.clone());
                }
            }

            bucket_cards
                .entry(release_type.clone())
                .or_default()
                .push(map_release(release));
        }
    }

    let mut labels: Vec<LabelData> = labels_by_id
        .into_iter()
        .map(|(id, name)| LabelData {
            id: id.to_string(),
            name,
        })
        .collect();
    labels.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

    (build_sections(bucket_cards, bucket_has_more), labels)
}

/// Emit one section per non-empty bucket in the official on-screen order.
/// Buckets the server adds in the future that aren't in the order list are
/// appended at the end (still rendered, just untitled-mapped).
fn build_sections(
    mut bucket_cards: HashMap<String, Vec<crate::home::CardData>>,
    bucket_has_more: HashMap<String, bool>,
) -> Vec<ReleaseSection> {
    let mut release_sections: Vec<ReleaseSection> = Vec::new();
    for &(rt, title) in RELEASE_SECTION_ORDER {
        // "download" ("Purchase Only") is intentionally hidden — drain it so
        // it can't resurface in the leftovers pass, but emit no section.
        if rt == "download" {
            bucket_cards.remove(rt);
            continue;
        }
        // `.remove` drains the bucket so the leftovers pass below can't
        // re-emit an already-rendered type.
        if let Some(cards) = bucket_cards.remove(rt) {
            if cards.is_empty() {
                continue;
            }
            release_sections.push(ReleaseSection {
                release_type: rt.to_string(),
                title: title.to_string(),
                has_more: bucket_has_more.get(rt).copied().unwrap_or(false),
                cards,
            });
        }
    }
    // Any unknown bucket types the order list doesn't cover — append them
    // last, titled from their raw key (rare; keeps D3 faithful).
    let mut leftovers: Vec<(String, Vec<crate::home::CardData>)> = bucket_cards
        .into_iter()
        .filter(|(_, cards)| !cards.is_empty())
        .collect();
    leftovers.sort_by(|a, b| a.0.cmp(&b.0));
    for (rt, cards) in leftovers {
        let has_more = bucket_has_more.get(&rt).copied().unwrap_or(false);
        release_sections.push(ReleaseSection {
            title: title_case(&rt),
            release_type: rt,
            has_more,
            cards,
        });
    }
    release_sections
}
