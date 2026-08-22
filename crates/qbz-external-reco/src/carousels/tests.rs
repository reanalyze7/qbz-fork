use std::collections::HashSet;

use super::artist_rails::compose_artist_rails;
use crate::types::{ArtistReco, RecoSource};

fn reco(id: u64) -> ArtistReco {
    ArtistReco {
        qobuz_artist_id: id,
        name: format!("Artist {id}"),
        image_url: String::new(),
        subtitle: String::new(),
        source: RecoSource::LastFm,
    }
}

fn ids(recos: &[ArtistReco]) -> Vec<u64> {
    recos.iter().map(|r| r.qobuz_artist_id).collect()
}

#[test]
fn compose_excludes_ids_from_visible_and_overflow() {
    let common = vec![reco(1), reco(2), reco(3), reco(4)];
    let recent = vec![reco(5), reco(6)];
    let excluded = HashSet::from([2, 4, 5]);
    let (c, r) = compose_artist_rails(common, recent, &excluded, 2);
    assert_eq!(ids(&c.visible), vec![1, 3]);
    assert!(c.overflow.is_empty(), "excluded ids leave no overflow");
    assert_eq!(ids(&r.visible), vec![6]);
}

#[test]
fn compose_dedups_cross_rail_common_wins() {
    let common = vec![reco(1), reco(2)];
    let recent = vec![reco(2), reco(3)];
    let (c, r) = compose_artist_rails(common, recent, &HashSet::new(), 20);
    assert_eq!(ids(&c.visible), vec![1, 2]);
    assert_eq!(ids(&r.visible), vec![3], "id 2 appears only in common");
}

#[test]
fn compose_dedups_within_rail() {
    let common = vec![reco(1), reco(1), reco(2)];
    let (c, _r) = compose_artist_rails(common, Vec::new(), &HashSet::new(), 20);
    assert_eq!(ids(&c.visible), vec![1, 2]);
}

#[test]
fn compose_splits_visible_and_overflow_in_order() {
    let common: Vec<ArtistReco> = (1..=5).map(reco).collect();
    let excluded = HashSet::from([1]);
    let (c, _r) = compose_artist_rails(common, Vec::new(), &excluded, 3);
    // The exclusion punches a hole in the first window; the next
    // candidates move up into the visible rows, the rest is overflow —
    // both in pool order (the backfill contract).
    assert_eq!(ids(&c.visible), vec![2, 3, 4]);
    assert_eq!(ids(&c.overflow), vec![5]);
}

#[test]
fn compose_cross_rail_dedup_also_covers_overflow() {
    // An id sitting in common's OVERFLOW must not resurface in recent's
    // visible rows (it is still claimed for potential backfill).
    let common: Vec<ArtistReco> = (1..=3).map(reco).collect();
    let recent = vec![reco(3), reco(4)];
    let (c, r) = compose_artist_rails(common, recent, &HashSet::new(), 2);
    assert_eq!(ids(&c.visible), vec![1, 2]);
    assert_eq!(ids(&c.overflow), vec![3]);
    assert_eq!(ids(&r.visible), vec![4], "id 3 is claimed by common");
    assert!(r.overflow.is_empty());
}
