use std::path::Path;
use std::time::Duration;

use rusqlite::Connection;

const BUSY_TIMEOUT: Duration = Duration::from_secs(5);

pub(super) fn open(path: &Path) -> rusqlite::Result<Connection> {
    let connection = Connection::open(path)?;
    connection.busy_timeout(BUSY_TIMEOUT)?;
    connection.pragma_update(None, "foreign_keys", "ON")?;
    connection
        .pragma_update_and_check(None, "journal_mode", "WAL", |row| row.get::<_, String>(0))?;
    Ok(connection)
}
