use rusqlite::Connection;

use crate::error::CoreError;

// Restoring a snapshot must not make a captured laptop lease authoritative again.
pub(crate) fn invalidate_leases(connection: &Connection) -> Result<(), CoreError> {
    connection.execute("UPDATE inbox_generation SET status = CASE WHEN attempts < 3 THEN 'waiting' ELSE 'failed' END, reason = 'Restored snapshot; reconcile receipts and claim again', token = NULL, lease_until = NULL WHERE status IN ('preparing', 'pending_delivery')", [])?;
    Ok(())
}
