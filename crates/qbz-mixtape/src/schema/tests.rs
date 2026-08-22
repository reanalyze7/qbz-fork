use super::*;

#[test]
fn migrations_are_idempotent() {
    let conn = Connection::open_in_memory().unwrap();
    run_mixtape_migrations(&conn).unwrap();
    run_mixtape_migrations(&conn).unwrap();

    // Both tables exist
    let collections: i64 = conn.query_row(
        "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='mixtape_collections'",
        [], |r| r.get(0),
    ).unwrap();
    assert_eq!(collections, 1);
    let items: i64 = conn.query_row(
        "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='mixtape_collection_items'",
        [], |r| r.get(0),
    ).unwrap();
    assert_eq!(items, 1);
}

#[test]
fn adds_column_when_session_table_exists() {
    let conn = Connection::open_in_memory().unwrap();
    conn.execute(
        "CREATE TABLE session_queue_state (user_id INTEGER, extra TEXT)",
        [],
    ).unwrap();
    run_mixtape_migrations(&conn).unwrap();

    let has_col: i64 = conn.query_row(
        "SELECT COUNT(*) FROM pragma_table_info('session_queue_state') WHERE name = 'source_collection_id'",
        [], |r| r.get(0),
    ).unwrap();
    assert_eq!(has_col, 1);
}

#[test]
fn tolerates_missing_session_table() {
    let conn = Connection::open_in_memory().unwrap();
    // Don't create session_queue_state
    run_mixtape_migrations(&conn).unwrap(); // should not panic
}
