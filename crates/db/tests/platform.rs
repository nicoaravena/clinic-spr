//! Looking clinics up in `platform.clinics`.
//!
//! Each `#[sqlx::test]` gets a fresh database with the platform migrations run and
//! `fixtures/clinics.sql` loaded: clinic 1 `centro` (active) and clinic 2 `norte` (suspended).

use clinic_db::platform::Platform;
use clinic_domain::clinic::{Clinic, ClinicStatus, Slug};
use clinic_domain::ids::ClinicId;
use sqlx::PgPool;

#[sqlx::test(
    migrator = "clinic_db::migrate::PLATFORM_MIGRATOR",
    fixtures("clinics")
)]
async fn finds_an_active_clinic(pool: PgPool) {
    let platform = Platform::new(pool);

    let clinic = platform.clinic(ClinicId::new(1)).await.unwrap();

    assert_eq!(
        clinic,
        Some(Clinic::new(
            ClinicId::new(1),
            Slug::parse("centro").unwrap(),
            ClinicStatus::Active,
        )),
    );
}

#[sqlx::test(
    migrator = "clinic_db::migrate::PLATFORM_MIGRATOR",
    fixtures("clinics")
)]
async fn finds_a_suspended_clinic(pool: PgPool) {
    let platform = Platform::new(pool);

    let clinic = platform.clinic(ClinicId::new(2)).await.unwrap().unwrap();

    assert_eq!(clinic.slug().as_str(), "norte");
    assert_eq!(clinic.status(), ClinicStatus::Suspended);
}

#[sqlx::test(
    migrator = "clinic_db::migrate::PLATFORM_MIGRATOR",
    fixtures("clinics")
)]
async fn an_unknown_clinic_is_none(pool: PgPool) {
    let platform = Platform::new(pool);

    assert_eq!(platform.clinic(ClinicId::new(3)).await.unwrap(), None);
}

#[sqlx::test(
    migrator = "clinic_db::migrate::PLATFORM_MIGRATOR",
    fixtures("clinics")
)]
async fn an_id_too_large_for_the_table_is_none(pool: PgPool) {
    let platform = Platform::new(pool);

    assert_eq!(
        platform.clinic(ClinicId::new(u64::MAX)).await.unwrap(),
        None
    );
}

#[sqlx::test(migrator = "clinic_db::migrate::PLATFORM_MIGRATOR")]
async fn a_closed_pool_is_a_storage_error(pool: PgPool) {
    pool.close().await;
    let platform = Platform::new(pool);

    let error = platform.clinic(ClinicId::new(1)).await.unwrap_err();

    assert_eq!(error.to_string(), "storage failed");
}
