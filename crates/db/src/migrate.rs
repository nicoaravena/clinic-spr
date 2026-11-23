//! Bringing the database schema up to date.

use clinic_app::ports::StorageError;
use sqlx::PgPool;
use sqlx::migrate::Migrator;

/// The migrations of the platform schema, from `migrations/platform/`.
///
/// `migrate!` reads the files when the crate compiles and embeds them in the binary, so the server
/// doesn't need the migration files at run time. Tests name this migrator in
/// `#[sqlx::test(migrator = "clinic_db::migrate::PLATFORM_MIGRATOR")]`.
pub static PLATFORM_MIGRATOR: Migrator = sqlx::migrate!("migrations/platform");

/// Runs every platform migration that hasn't run yet.
///
/// sqlx records what it ran in a `_sqlx_migrations` table, so running this again does nothing.
pub async fn migrate_platform(pool: &PgPool) -> Result<(), StorageError> {
    PLATFORM_MIGRATOR.run(pool).await.map_err(StorageError::new)
}
