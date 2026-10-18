use std::path::Path;
use std::time::Duration;

use rusqlite::config::DbConfig;
use rusqlite::{Connection, OpenFlags};

const BUSY_TIMEOUT: Duration = Duration::from_secs(5);

pub(super) fn open_writer(path: &Path) -> rusqlite::Result<Connection> {
    let connection = Connection::open(path)?;
    connection.busy_timeout(BUSY_TIMEOUT)?;
    connection.pragma_update(None, "foreign_keys", "ON")?;
    Ok(connection)
}

pub(super) fn open_reader(path: &Path) -> rusqlite::Result<Connection> {
    let connection = Connection::open(path)?;
    connection.busy_timeout(BUSY_TIMEOUT)?;
    connection.pragma_update(None, "query_only", "ON")?;
    Ok(connection)
}

pub(super) fn open_inspector(path: &Path) -> rusqlite::Result<Connection> {
    let connection = Connection::open_with_flags(
        path,
        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )?;
    connection.busy_timeout(BUSY_TIMEOUT)?;
    connection.set_db_config(DbConfig::SQLITE_DBCONFIG_NO_CKPT_ON_CLOSE, true)?;
    Ok(connection)
}
