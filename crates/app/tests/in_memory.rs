use chrono::{DateTime, TimeDelta, TimeZone, Utc};
use clinic_app::in_memory::{
    DuplicateAppointmentType, InMemoryAppointmentTypes, InMemorySchedules,
};
use clinic_app::ports::{AppointmentTypes, ScheduleRepository};
use clinic_domain::appointment::AppointmentType;
use clinic_domain::ids::{ClinicId, PractitionerId};
use clinic_domain::schedule::Schedule;
use clinic_domain::time_slot::TimeSlot;
use clinic_domain::working_hours::WorkingHours;

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
async fn a_saved_schedule_can_be_read_back() {
    let mut schedules = InMemorySchedules::new();

    schedules
        .save(ClinicId::new(1), PractitionerId::new(1), mornings())
        .await
        .unwrap();

    assert_eq!(
        schedules
            .get(ClinicId::new(1), PractitionerId::new(1))
            .await
            .unwrap(),
        Some(mornings()),
    );
}

#[tokio::test]
async fn an_unknown_practitioner_has_no_schedule() {
    let schedules = InMemorySchedules::new();

    assert_eq!(
        schedules
            .get(ClinicId::new(1), PractitionerId::new(1))
            .await
            .unwrap(),
        None
    );
}

#[tokio::test]
async fn saving_again_replaces_the_schedule() {
    let mut schedules = InMemorySchedules::new();
    let empty = Schedule::new(WorkingHours::new(Vec::new()).unwrap());

    schedules
        .save(ClinicId::new(1), PractitionerId::new(1), mornings())
        .await
        .unwrap();
    schedules
        .save(ClinicId::new(1), PractitionerId::new(1), empty.clone())
        .await
        .unwrap();

    assert_eq!(
        schedules
            .get(ClinicId::new(1), PractitionerId::new(1))
            .await
            .unwrap(),
        Some(empty),
    );
}

#[tokio::test]
async fn a_schedule_is_invisible_to_another_clinic() {
    let mut schedules = InMemorySchedules::new();

    schedules
        .save(ClinicId::new(1), PractitionerId::new(1), mornings())
        .await
        .unwrap();

    // Same practitioner ID, different clinic.
    assert_eq!(
        schedules
            .get(ClinicId::new(2), PractitionerId::new(1))
            .await
            .unwrap(),
        None
    );
}

#[tokio::test]
async fn an_added_appointment_type_can_be_found_by_name() {
    let mut types = InMemoryAppointmentTypes::new();

    types.add(ClinicId::new(1), consultation()).unwrap();

    assert_eq!(
        types.find(ClinicId::new(1), "Consultation").await.unwrap(),
        Some(consultation()),
    );
    assert_eq!(
        types.find(ClinicId::new(1), "Follow-up").await.unwrap(),
        None
    );
}

#[tokio::test]
async fn an_appointment_type_is_invisible_to_another_clinic() {
    let mut types = InMemoryAppointmentTypes::new();

    types.add(ClinicId::new(1), consultation()).unwrap();

    assert_eq!(
        types.find(ClinicId::new(2), "Consultation").await.unwrap(),
        None
    );
}

#[tokio::test]
async fn refuses_a_duplicate_name_within_a_clinic() {
    let mut types = InMemoryAppointmentTypes::new();
    let longer = AppointmentType::new("Consultation", TimeDelta::minutes(45)).unwrap();

    types.add(ClinicId::new(1), consultation()).unwrap();
    let error = types.add(ClinicId::new(1), longer).unwrap_err();

    assert_eq!(error.name(), "Consultation");
    assert_eq!(
        types.find(ClinicId::new(1), "Consultation").await.unwrap(),
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
