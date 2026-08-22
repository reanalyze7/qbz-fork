use crate::sparse_vector::SparseVector;

#[test]
fn test_add() {
    let mut a = SparseVector::new();
    a.set(1, 1.0);
    a.set(3, 2.0);

    let mut b = SparseVector::new();
    b.set(2, 1.5);
    b.set(3, 0.5);

    let c = a.add(&b);

    assert_eq!(c.get(1), 1.0);
    assert_eq!(c.get(2), 1.5);
    assert_eq!(c.get(3), 2.5);
    assert_eq!(c.nnz(), 3);
}

#[test]
fn test_dot_product() {
    let mut a = SparseVector::new();
    a.set(1, 2.0);
    a.set(3, 3.0);

    let mut b = SparseVector::new();
    b.set(1, 4.0);
    b.set(2, 5.0);
    b.set(3, 1.0);

    // dot = 2*4 + 3*1 = 11
    assert_eq!(a.dot(&b), 11.0);
}

#[test]
fn test_magnitude() {
    let mut vec = SparseVector::new();
    vec.set(0, 3.0);
    vec.set(1, 4.0);

    // magnitude = sqrt(9 + 16) = 5
    assert!((vec.magnitude() - 5.0).abs() < 1e-6);
}

#[test]
fn test_normalize() {
    let mut vec = SparseVector::new();
    vec.set(0, 3.0);
    vec.set(1, 4.0);

    let normalized = vec.normalize();

    assert!((normalized.magnitude() - 1.0).abs() < 1e-6);
    assert!((normalized.get(0) - 0.6).abs() < 1e-6);
    assert!((normalized.get(1) - 0.8).abs() < 1e-6);
}

#[test]
fn test_cosine_similarity_identical() {
    let mut vec = SparseVector::new();
    vec.set(0, 1.0);
    vec.set(1, 2.0);

    let sim = vec.cosine_similarity(&vec);
    assert!((sim - 1.0).abs() < 1e-6);
}

#[test]
fn test_cosine_similarity_orthogonal() {
    let mut a = SparseVector::new();
    a.set(0, 1.0);

    let mut b = SparseVector::new();
    b.set(1, 1.0);

    let sim = a.cosine_similarity(&b);
    assert!(sim.abs() < 1e-6);
}

#[test]
fn test_cosine_similarity_similar() {
    let mut a = SparseVector::new();
    a.set(0, 1.0);
    a.set(1, 1.0);

    let mut b = SparseVector::new();
    b.set(0, 1.0);
    b.set(1, 0.5);

    let sim = a.cosine_similarity(&b);
    // Should be high but not 1.0
    assert!(sim > 0.9);
    assert!(sim < 1.0);
}

#[test]
fn test_scale() {
    let mut vec = SparseVector::new();
    vec.set(0, 2.0);
    vec.set(1, 3.0);

    let scaled = vec.scale(2.0);

    assert_eq!(scaled.get(0), 4.0);
    assert_eq!(scaled.get(1), 6.0);
}
