use super::*;
use crate::sparse_vector::SparseVector;

fn unique_test_dir(name: &str) -> std::path::PathBuf {
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("qbz-reco-{name}-{}-{nonce}", std::process::id()))
}

#[test]
fn create_artist_index() {
    let dir = unique_test_dir("idx");
    let mut store = ArtistVectorStore::open_at(&dir).unwrap();

    let idx1 = store.get_or_create_idx("mbid-1", Some("Artist 1")).unwrap();
    let idx2 = store.get_or_create_idx("mbid-2", Some("Artist 2")).unwrap();
    let idx1_again = store.get_or_create_idx("mbid-1", None).unwrap();

    assert_eq!(idx1, 0);
    assert_eq!(idx2, 1);
    assert_eq!(idx1_again, idx1); // same index on re-create
    assert_eq!(store.get_mbid(idx1), Some("mbid-1"));
    assert_eq!(store.get_mbid(idx2), Some("mbid-2"));
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn store_and_retrieve_vector() {
    let dir = unique_test_dir("vec");
    let mut store = ArtistVectorStore::open_at(&dir).unwrap();

    store.get_or_create_idx("target-1", None).unwrap();
    store.get_or_create_idx("target-2", None).unwrap();

    let mut vec = SparseVector::new();
    vec.set(0, 1.0); // target-1
    vec.set(1, 0.5); // target-2
    store.set_vector("artist-a", &vec, "test").unwrap();

    let retrieved = store.get_vector("artist-a").unwrap();
    assert_eq!(retrieved.get(0), 1.0);
    assert_eq!(retrieved.get(1), 0.5);
    assert_eq!(retrieved.nnz(), 2);
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn related_artists_rank_by_summed_weight() {
    let dir = unique_test_dir("related");
    let mut s = ArtistVectorStore::open_at(&dir).unwrap();
    let _a = s.get_or_create_idx("mbid-a", Some("A")).unwrap();
    let b = s.get_or_create_idx("mbid-b", Some("B")).unwrap();
    let c = s.get_or_create_idx("mbid-c", Some("C")).unwrap();

    // A relates to B (1.0) and C (0.3) via the 'mb' source.
    let mut v = SparseVector::new();
    v.set(b, 1.0);
    v.set(c, 0.3);
    s.set_vector("mbid-a", &v, "mb").unwrap();

    let related = s.get_related_artists("mbid-a").unwrap();
    assert_eq!(related.len(), 2);
    assert_eq!(related[0].mbid, "mbid-b"); // higher weight first
    assert_eq!(related[0].name.as_deref(), Some("B"));
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn fresh_vector_check() {
    let dir = unique_test_dir("fresh");
    let mut store = ArtistVectorStore::open_at(&dir).unwrap();

    let vec = SparseVector::new();
    store.set_vector("artist-a", &vec, "test").unwrap();

    assert!(store.has_fresh_vector("artist-a", 86400)); // fresh within 1 day
    assert!(!store.has_fresh_vector("artist-a", 0)); // not fresh with 0s TTL
    assert!(!store.has_fresh_vector("nonexistent", 86400));
    let _ = std::fs::remove_dir_all(dir);
}
