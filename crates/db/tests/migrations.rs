//! The platform migrations, and the rules the `platform.clinics` table enforces on its own.
//!
//! Each `#[sqlx::test]` gets a fresh database. They need the Postgres from `compose.yaml` and
//! `DATABASE_URL` (in `.env`), and fail without them.

use clinic_db::migrate::migrate_platform;
use sqlx::PgPool;

const CHECK_VIOLATION: &str = "23514";
const UNIQUE_VIOLATION: &str = "23505";

/// Inserts a row into `platform.clinics`, with every value bound as a parameter.
async fn insert(
    pool: &PgPool,
    id: i64,
    slug: &str,
    schema_name: &str,
    status: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO platform.clinics (id, slug, schema_name, status) VALUES ($1, $2, $3, $4)",
    )
    .bind(id)
    .bind(slug)
    .bind(schema_name)
    .bind(status)
    .execute(pool)
    .await?;
    Ok(())
}

/// The SQLSTATE that `result` failed with.
fn sqlstate(result: Result<(), sqlx::Error>) -> String {
    let error = result.expect_err("the insert should be rejected");
    let database_error = error.as_database_error().expect("a database error");

    database_error.code().expect("a SQLSTATE").into_owned()
}

#[sqlx::test(migrations = false)]
async fn migrating_twice_is_harmless(pool: PgPool) {
    migrate_platform(&pool).await.unwrap();
    migrate_platform(&pool).await.unwrap();

    let clinics: i64 = sqlx::query_scalar("SELECT count(*) FROM platform.clinics")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(clinics, 0);
}

#[sqlx::test(migrator = "clinic_db::migrate::PLATFORM_MIGRATOR")]
async fn a_new_clinic_is_active(pool: PgPool) {
    sqlx::query(
        "INSERT INTO platform.clinics (id, slug, schema_name) VALUES (1, 'centro', 'clinic_1')",
    )
    .execute(&pool)
    .await
    .unwrap();

    let status: String = sqlx::query_scalar("SELECT status FROM platform.clinics WHERE id = 1")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(status, "active");
}

#[sqlx::test(migrator = "clinic_db::migrate::PLATFORM_MIGRATOR")]
async fn the_schema_name_must_match_the_id(pool: PgPool) {
    let result = insert(&pool, 1, "centro", "clinic_2", "active").await;

    assert_eq!(sqlstate(result), CHECK_VIOLATION);
}

#[sqlx::test(migrator = "clinic_db::migrate::PLATFORM_MIGRATOR")]
async fn the_slug_must_be_well_formed(pool: PgPool) {
    for slug in [
        "Centro",
        "centro--norte",
        "-centro",
        "centro-",
        "centro_norte",
        "",
    ] {
        let result = insert(&pool, 1, slug, "clinic_1", "active").await;

        assert_eq!(sqlstate(result), CHECK_VIOLATION, "slug {slug:?}");
    }
}

#[sqlx::test(migrator = "clinic_db::migrate::PLATFORM_MIGRATOR")]
async fn the_slug_must_be_at_most_63_bytes(pool: PgPool) {
    insert(&pool, 1, &"a".repeat(63), "clinic_1", "active")
        .await
        .unwrap();

    let result = insert(&pool, 2, &"b".repeat(64), "clinic_2", "active").await;

    assert_eq!(sqlstate(result), CHECK_VIOLATION);
}

#[sqlx::test(migrator = "clinic_db::migrate::PLATFORM_MIGRATOR")]
async fn two_clinics_cant_share_a_slug(pool: PgPool) {
    insert(&pool, 1, "centro", "clinic_1", "active")
        .await
        .unwrap();

    let result = insert(&pool, 2, "centro", "clinic_2", "active").await;

    assert_eq!(sqlstate(result), UNIQUE_VIOLATION);
}

#[sqlx::test(migrator = "clinic_db::migrate::PLATFORM_MIGRATOR")]
async fn the_status_must_be_known(pool: PgPool) {
    let result = insert(&pool, 1, "centro", "clinic_1", "closed").await;

    assert_eq!(sqlstate(result), CHECK_VIOLATION);
}

#[sqlx::test(migrator = "clinic_db::migrate::PLATFORM_MIGRATOR")]
async fn the_id_cant_be_negative(pool: PgPool) {
    let result = insert(&pool, -1, "centro", "clinic_-1", "active").await;

    assert_eq!(sqlstate(result), CHECK_VIOLATION);
}
