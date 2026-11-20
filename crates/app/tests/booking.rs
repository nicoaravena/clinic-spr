mod common;

use chrono::{DateTime, TimeDelta, TimeZone, Utc};
use clinic_app::booking::{BookAppointmentError, BookingRequest, book_appointment};
use clinic_app::in_memory::{InMemoryAppointmentTypes, InMemoryStorage};
use clinic_domain::access::{Role, ScheduleAccess};
use clinic_domain::appointment::{AppointmentStatus, AppointmentType};
use clinic_domain::ids::{ClinicId, PatientId, PractitionerId};
use clinic_domain::schedule::{BookingError, Schedule};
use clinic_domain::time_slot::TimeSlot;
use clinic_domain::working_hours::WorkingHours;

use common::{save_committed, stored};

fn clinic() -> ClinicId {
    ClinicId::new(1)
}

fn ana() -> PractitionerId {
    PractitionerId::new(1)
}

fn at(hour: u32, minute: u32) -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 3, 2, hour, minute, 0).unwrap()
}

fn slot(start: DateTime<Utc>, end: DateTime<Utc>) -> TimeSlot {
    TimeSlot::new(start, end).unwrap()
}

fn consultation() -> AppointmentType {
    AppointmentType::new("Consultation", TimeDelta::minutes(30)).unwrap()
}

fn access() -> ScheduleAccess {
    Role::Receptionist.grant().unwrap()
}

/// Storage whose appointment types are `types`, in which Ana works 09:00-12:00 at clinic 1.
async fn seeded_with(types: InMemoryAppointmentTypes) -> InMemoryStorage {
    let storage = InMemoryStorage::new(types);
    let hours = WorkingHours::new(vec![slot(at(9, 0), at(12, 0))]).unwrap();
    save_committed(&storage, clinic(), ana(), Schedule::new(hours)).await;
    storage
}

/// Ana works 09:00-12:00 at clinic 1, which offers a 30-minute consultation.
async fn seeded() -> InMemoryStorage {
    let mut types = InMemoryAppointmentTypes::new();
    types.add(clinic(), consultation()).unwrap();
    seeded_with(types).await
}

fn request(start: DateTime<Utc>) -> BookingRequest<'static> {
    BookingRequest {
        clinic: clinic(),
        practitioner: ana(),
        patient: PatientId::new(7),
        appointment_type: "Consultation",
        start,
    }
}

#[tokio::test]
async fn books_and_returns_the_appointment() {
    let storage = seeded().await;

    let appointment = book_appointment(&storage, &access(), request(at(9, 0)))
        .await
        .unwrap();

    assert_eq!(appointment.patient(), PatientId::new(7));
    assert_eq!(appointment.appointment_type(), &consultation());
    assert_eq!(appointment.slot(), slot(at(9, 0), at(9, 30)));
    assert_eq!(appointment.status(), AppointmentStatus::Scheduled);
}

#[tokio::test]
async fn saves_the_booking_in_the_schedule() {
    let storage = seeded().await;

    let appointment = book_appointment(&storage, &access(), request(at(9, 0)))
        .await
        .unwrap();

    let saved = stored(&storage, clinic(), ana()).await.unwrap();
    assert_eq!(saved.appointments(), [appointment]);
}

#[tokio::test]
async fn refuses_an_unknown_appointment_type() {
    let storage = seeded().await;
    let request = BookingRequest {
        appointment_type: "Check-up",
        ..request(at(9, 0))
    };

    let result = book_appointment(&storage, &access(), request).await;

    assert!(
        matches!(result, Err(BookAppointmentError::UnknownAppointmentType)),
        "{result:?}"
    );
}

#[tokio::test]
async fn refuses_an_unknown_practitioner() {
    let storage = seeded().await;
    let request = BookingRequest {
        practitioner: PractitionerId::new(2),
        ..request(at(9, 0))
    };

    let result = book_appointment(&storage, &access(), request).await;

    assert!(
        matches!(result, Err(BookAppointmentError::UnknownPractitioner)),
        "{result:?}"
    );
}

#[tokio::test]
async fn the_practitioner_is_checked_before_the_appointment_type() {
    let storage = seeded().await;
    let request = BookingRequest {
        practitioner: PractitionerId::new(2),
        appointment_type: "Check-up",
        ..request(at(9, 0))
    };

    let result = book_appointment(&storage, &access(), request).await;

    assert!(
        matches!(result, Err(BookAppointmentError::UnknownPractitioner)),
        "{result:?}"
    );
}

#[tokio::test]
async fn cannot_book_into_another_clinics_schedule() {
    let mut types = InMemoryAppointmentTypes::new();
    types.add(clinic(), consultation()).unwrap();
    types.add(ClinicId::new(2), consultation()).unwrap();
    let storage = seeded_with(types).await;
    let request = BookingRequest {
        clinic: ClinicId::new(2),
        ..request(at(9, 0))
    };

    let result = book_appointment(&storage, &access(), request).await;

    assert!(
        matches!(result, Err(BookAppointmentError::UnknownPractitioner)),
        "{result:?}"
    );
}

#[tokio::test]
async fn passes_on_the_domain_refusal() {
    let storage = seeded().await;
    book_appointment(&storage, &access(), request(at(9, 0)))
        .await
        .unwrap();

    let result = book_appointment(&storage, &access(), request(at(9, 15))).await;

    assert!(
        matches!(
            result,
            Err(BookAppointmentError::Booking(BookingError::Overlaps { existing }))
                if existing == slot(at(9, 0), at(9, 30))
        ),
        "{result:?}"
    );
}

#[tokio::test]
async fn a_refused_booking_leaves_the_stored_schedule_unchanged() {
    let storage = seeded().await;
    book_appointment(&storage, &access(), request(at(9, 0)))
        .await
        .unwrap();
    let before = stored(&storage, clinic(), ana()).await;

    let overlapping = book_appointment(&storage, &access(), request(at(9, 15))).await;
    let outside = book_appointment(&storage, &access(), request(at(11, 45))).await;

    assert!(overlapping.is_err());
    assert!(outside.is_err());
    assert_eq!(stored(&storage, clinic(), ana()).await, before);
}

#[tokio::test]
async fn the_booking_future_can_move_between_threads() {
    fn assert_send<F: Send>(future: F) -> F {
        future
    }
    let storage = seeded().await;

    // The future borrows its arguments until it's awaited, so `access` needs a name.
    let access = access();

    // A multi-threaded runtime such as axum's may resume the future on another thread.
    let booking = book_appointment(&storage, &access, request(at(9, 0)));

    assert!(assert_send(booking).await.is_ok());
}

#[test]
fn errors_have_readable_messages() {
    assert_eq!(
        BookAppointmentError::UnknownAppointmentType.to_string(),
        "clinic has no appointment type with that name",
    );
    assert_eq!(
        BookAppointmentError::UnknownPractitioner.to_string(),
        "clinic has no schedule for that practitioner",
    );
    // A domain refusal keeps the domain's message.
    assert_eq!(
        BookAppointmentError::from(BookingError::OutsideWorkingHours).to_string(),
        BookingError::OutsideWorkingHours.to_string(),
    );
}

#[test]
fn error_implements_the_standard_error_trait() {
    fn assert_is_error<E: std::error::Error>(_: E) {}

    assert_is_error(BookAppointmentError::UnknownPractitioner);
}
