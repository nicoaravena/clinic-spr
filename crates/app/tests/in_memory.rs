mod common;

use chrono::{DateTime, TimeDelta, TimeZone, Utc};
use clinic_app::in_memory::{DuplicateAppointmentType, InMemoryAppointmentTypes, InMemoryStorage};
use clinic_app::ports::{AppointmentTypes, ScheduleRepository, Storage};
use clinic_domain::appointment::AppointmentType;
use clinic_domain::ids::{ClinicId, PractitionerId};
use clinic_domain::schedule::Schedule;
use clinic_domain::time_slot::TimeSlot;
use clinic_domain::working_hours::WorkingHours;

use common::{save_committed, stored};

fn at(hour: u32, minute: u32) -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 3, 2, hour, minute, 0).unwrap()
}

fn mornings() -> Schedule {
    Schedule::new(WorkingHours::new(vec![TimeSlot::new(at(9, 0), at(12, 0)).unwrap()]).unwrap())
}

fn consultation() -> AppointmentType {
    AppointmentType::new("Consultation", TimeDelta::minutes(30)).unwrap()
}

#[tokio::test]
async fn a_committed_schedule_can_be_read_back_by_a_later_session() {
    let storage = InMemoryStorage::new(InMemoryAppointmentTypes::new());

    save_committed(
        &storage,
        ClinicId::new(1),
        PractitionerId::new(1),
        mornings(),
    )
    .await;

    assert_eq!(
        stored(&storage, ClinicId::new(1), PractitionerId::new(1)).await,
        Some(mornings()),
    );
}

#[tokio::test]
async fn an_unknown_practitioner_has_no_schedule() {
    let storage = InMemoryStorage::new(InMemoryAppointmentTypes::new());

    assert_eq!(
        stored(&storage, ClinicId::new(1), PractitionerId::new(1)).await,
        None
    );
}

#[tokio::test]
async fn a_session_sees_its_own_uncommitted_save() {
    let storage = InMemoryStorage::new(InMemoryAppointmentTypes::new());
    let mut session = storage.begin(ClinicId::new(1)).await.unwrap();

    session
        .save(PractitionerId::new(1), mornings())
        .await
        .unwrap();

    assert_eq!(
        session.get(PractitionerId::new(1)).await.unwrap(),
        Some(mornings()),
    );
}

#[tokio::test]
async fn a_session_dropped_without_commit_saves_nothing() {
    let storage = InMemoryStorage::new(InMemoryAppointmentTypes::new());

    {
        let mut session = storage.begin(ClinicId::new(1)).await.unwrap();
        session
            .save(PractitionerId::new(1), mornings())
            .await
            .unwrap();
        // `session` is dropped here, without `commit`.
    }

    assert_eq!(
        stored(&storage, ClinicId::new(1), PractitionerId::new(1)).await,
        None
    );
}

#[tokio::test]
async fn a_dropped_session_keeps_the_earlier_schedule() {
    let storage = InMemoryStorage::new(InMemoryAppointmentTypes::new());
    let empty = Schedule::new(WorkingHours::new(Vec::new()).unwrap());
    save_committed(
        &storage,
        ClinicId::new(1),
        PractitionerId::new(1),
        mornings(),
    )
    .await;

    let mut session = storage.begin(ClinicId::new(1)).await.unwrap();
    session.save(PractitionerId::new(1), empty).await.unwrap();
    drop(session);

    assert_eq!(
        stored(&storage, ClinicId::new(1), PractitionerId::new(1)).await,
        Some(mornings()),
    );
}

#[tokio::test]
async fn saving_again_replaces_the_schedule() {
    let storage = InMemoryStorage::new(InMemoryAppointmentTypes::new());
    let empty = Schedule::new(WorkingHours::new(Vec::new()).unwrap());

    save_committed(
        &storage,
        ClinicId::new(1),
        PractitionerId::new(1),
        mornings(),
    )
    .await;
    save_committed(
        &storage,
        ClinicId::new(1),
        PractitionerId::new(1),
        empty.clone(),
    )
    .await;

    assert_eq!(
        stored(&storage, ClinicId::new(1), PractitionerId::new(1)).await,
        Some(empty),
    );
}

#[tokio::test]
async fn a_schedule_is_invisible_to_another_clinic() {
    let storage = InMemoryStorage::new(InMemoryAppointmentTypes::new());

    save_committed(
        &storage,
        ClinicId::new(1),
        PractitionerId::new(1),
        mornings(),
    )
    .await;

    // Same practitioner ID, different clinic.
    assert_eq!(
        stored(&storage, ClinicId::new(2), PractitionerId::new(1)).await,
        None
    );
}

#[tokio::test]
async fn clones_of_the_storage_share_their_data() {
    let storage = InMemoryStorage::new(InMemoryAppointmentTypes::new());
    let clone = storage.clone();

    save_committed(&clone, ClinicId::new(1), PractitionerId::new(1), mornings()).await;

    assert_eq!(
        stored(&storage, ClinicId::new(1), PractitionerId::new(1)).await,
        Some(mornings()),
    );
}

#[tokio::test]
async fn an_added_appointment_type_can_be_found_by_name() {
    let mut types = InMemoryAppointmentTypes::new();
    types.add(ClinicId::new(1), consultation()).unwrap();
    let storage = InMemoryStorage::new(types);

    let session = storage.begin(ClinicId::new(1)).await.unwrap();

    assert_eq!(
        session.find("Consultation").await.unwrap(),
        Some(consultation())
    );
    assert_eq!(session.find("Follow-up").await.unwrap(), None);
}

#[tokio::test]
async fn an_appointment_type_is_invisible_to_another_clinic() {
    let mut types = InMemoryAppointmentTypes::new();
    types.add(ClinicId::new(1), consultation()).unwrap();
    let storage = InMemoryStorage::new(types);

    let session = storage.begin(ClinicId::new(2)).await.unwrap();

    assert_eq!(session.find("Consultation").await.unwrap(), None);
}

#[tokio::test]
async fn refuses_a_duplicate_name_within_a_clinic() {
    let mut types = InMemoryAppointmentTypes::new();
    let longer = AppointmentType::new("Consultation", TimeDelta::minutes(45)).unwrap();

    types.add(ClinicId::new(1), consultation()).unwrap();
    let error = types.add(ClinicId::new(1), longer).unwrap_err();

    assert_eq!(error.name(), "Consultation");
    let session = InMemoryStorage::new(types)
        .begin(ClinicId::new(1))
        .await
        .unwrap();
    assert_eq!(
        session.find("Consultation").await.unwrap(),
        Some(consultation()),
        "the first type must be kept",
    );
}

#[test]
fn allows_the_same_name_in_another_clinic() {
    let mut types = InMemoryAppointmentTypes::new();

    types.add(ClinicId::new(1), consultation()).unwrap();

    assert!(types.add(ClinicId::new(2), consultation()).is_ok());
}

#[test]
fn duplicate_error_has_a_readable_message() {
    let mut types = InMemoryAppointmentTypes::new();
    types.add(ClinicId::new(1), consultation()).unwrap();

    let error: DuplicateAppointmentType = types.add(ClinicId::new(1), consultation()).unwrap_err();

    assert_eq!(
        error.to_string(),
        "clinic already has an appointment type named \"Consultation\"",
    );
}
