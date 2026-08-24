use rusqlite::Connection;

pub(super) fn fresh_db() -> Connection {
    let conn = Connection::open_in_memory().unwrap();
    crate::schema::run_mixtape_migrations(&conn).unwrap();
    conn
}

mod collections_tests;
mod items_tests;
