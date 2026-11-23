//! The connection pool.

use std::time::Duration;

use clinic_app::ports::StorageError;
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;

/// The most connections the pool opens at once.
const MAX_CONNECTIONS: u32 = 10;

/// How long a caller waits for a free connection, or for a new one to open, before giving up.
const ACQUIRE_TIMEOUT: Duration = Duration::from_secs(5);

/// Opens a pool of connections to the database at `url`, such as
/// `postgres://clinic:clinic@localhost:5433/clinic`.
///
/// It opens one connection straight away, so a wrong URL or a server that isn't running fails
/// here, at startup, and not on the first request.
pub async fn connect(url: &str) -> Result<PgPool, StorageError> {
    PgPoolOptions::new()
        .max_connections(MAX_CONNECTIONS)
        .acquire_timeout(ACQUIRE_TIMEOUT)
        .connect(url)
        .await
        .map_err(StorageError::new)
}
