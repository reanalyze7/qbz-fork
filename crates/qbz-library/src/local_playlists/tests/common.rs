//! Shared test setup used by both `playlist_tests` and `track_tests`.

use rusqlite::Connection;

use crate::local_playlists::init_schema;

pub(super) fn fresh_db() -> Connection {
    let conn = Connection::open_in_memory().unwrap();
    init_schema(&conn).unwrap();
    conn
}
