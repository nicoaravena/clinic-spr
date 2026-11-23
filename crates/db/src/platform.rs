//! What the platform knows about every clinic, in the `platform.clinics` table.

use clinic_app::ports::StorageError;
use clinic_domain::clinic::{Clinic, ClinicStatus, Slug};
use clinic_domain::ids::ClinicId;
use sqlx::PgPool;

/// Reads the platform schema.
///
/// Cloning it is cheap: the clones share one pool.
#[derive(Debug, Clone)]
pub struct Platform {
    pool: PgPool,
}

impl Platform {
    /// Reads through `pool`, whose database has the platform migrations applied.
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// The clinic with this id, or `None` if there is none.
    ///
    /// A row that doesn't make a valid [`Clinic`] is a [`StorageError`]. The table's constraints
    /// should make that impossible.
    pub async fn clinic(&self, id: ClinicId) -> Result<Option<Clinic>, StorageError> {
        // The column is a bigint, so an id above `i64::MAX` can't be in the table.
        let Ok(key) = i64::try_from(id.value()) else {
            return Ok(None);
        };

        let row = sqlx::query!(
            "SELECT id, slug, status FROM platform.clinics WHERE id = $1",
            key,
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(StorageError::new)?;

        row.map(|row| clinic_from_row(row.id, &row.slug, &row.status))
            .transpose()
    }
}

/// Rebuilds a [`Clinic`] from the columns of its row.
fn clinic_from_row(id: i64, slug: &str, status: &str) -> Result<Clinic, StorageError> {
    let id =
        u64::try_from(id).map_err(|_| StorageError::new(format!("clinic id {id} is negative")))?;
    let slug = Slug::parse(slug).map_err(StorageError::new)?;
    let status = match status {
        "active" => ClinicStatus::Active,
        "suspended" => ClinicStatus::Suspended,
        other => {
            return Err(StorageError::new(format!(
                "clinic {id} has an unknown status {other:?}"
            )));
        }
    };

    Ok(Clinic::new(ClinicId::new(id), slug, status))
}
