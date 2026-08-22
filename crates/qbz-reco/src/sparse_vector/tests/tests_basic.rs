use crate::sparse_vector::SparseVector;

#[test]
fn test_set_and_get() {
    let mut vec = SparseVector::new();
    vec.set(5, 1.0);
    vec.set(10, 2.0);
    vec.set(3, 0.5);

    assert_eq!(vec.get(5), 1.0);
    assert_eq!(vec.get(10), 2.0);
    assert_eq!(vec.get(3), 0.5);
    assert_eq!(vec.get(7), 0.0); // Not set
    assert_eq!(vec.nnz(), 3);

    // Indices should be sorted
    assert_eq!(vec.indices(), &[3, 5, 10]);
}

#[test]
fn test_update_value() {
    let mut vec = SparseVector::new();
    vec.set(5, 1.0);
    vec.set(5, 2.0);

    assert_eq!(vec.get(5), 2.0);
    assert_eq!(vec.nnz(), 1);
}

#[test]
fn test_remove_on_zero() {
    let mut vec = SparseVector::new();
    vec.set(5, 1.0);
    vec.set(5, 0.0);

    assert_eq!(vec.get(5), 0.0);
    assert_eq!(vec.nnz(), 0);
}

#[test]
fn test_from_parts() {
    let vec = SparseVector::from_parts(vec![1, 3, 5], vec![0.5, 1.0, 1.5]);

    assert_eq!(vec.get(1), 0.5);
    assert_eq!(vec.get(3), 1.0);
    assert_eq!(vec.get(5), 1.5);
    assert_eq!(vec.nnz(), 3);
}

#[test]
fn test_empty_vector() {
    let vec = SparseVector::new();

    assert!(vec.is_empty());
    assert_eq!(vec.magnitude(), 0.0);
    assert_eq!(vec.normalize().nnz(), 0);
}
