use chrono::{DateTime, TimeDelta, TimeZone, Utc};
use clinic_app::booking::{BookAppointmentError, BookingRequest, book_appointment};
use clinic_app::in_memory::{InMemoryAppointmentTypes, InMemorySchedules};
use clinic_app::ports::{AppointmentTypes, ScheduleRepository};
use clinic_domain::access::{Role, ScheduleAccess};
use clinic_domain::appointment::{AppointmentStatus, AppointmentType};
use clinic_domain::ids::{ClinicId, PatientId, PractitionerId};
use clinic_domain::schedule::{BookingError, Schedule};
use clinic_domain::time_slot::TimeSlot;
use clinic_domain::working_hours::WorkingHours;

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

/// Ana works 09:00-12:00 at clinic 1, which offers a 30-minute consultation.
fn seeded() -> (InMemorySchedules, InMemoryAppointmentTypes) {
    let mut schedules = InMemorySchedules::new();
    let hours = WorkingHours::new(vec![slot(at(9, 0), at(12, 0))]).unwrap();
    schedules.save(clinic(), ana(), Schedule::new(hours));

    let mut types = InMemoryAppointmentTypes::new();
    types.add(clinic(), consultation()).unwrap();

    (schedules, types)
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

#[test]
fn books_and_returns_the_appointment() {
    let (mut schedules, types) = seeded();

    let appointment =
        book_appointment(&mut schedules, &types, &access(), request(at(9, 0))).unwrap();

    assert_eq!(appointment.patient(), PatientId::new(7));
    assert_eq!(appointment.appointment_type(), &consultation());
    assert_eq!(appointment.slot(), slot(at(9, 0), at(9, 30)));
    assert_eq!(appointment.status(), AppointmentStatus::Scheduled);
}

#[test]
fn saves_the_booking_in_the_schedule() {
    let (mut schedules, types) = seeded();

    let appointment =
        book_appointment(&mut schedules, &types, &access(), request(at(9, 0))).unwrap();

    let saved = schedules.get(clinic(), ana()).unwrap();
    assert_eq!(saved.appointments(), [appointment]);
}

#[test]
fn refuses_an_unknown_appointment_type() {
    let (mut schedules, types) = seeded();
    let request = BookingRequest {
        appointment_type: "Check-up",
        ..request(at(9, 0))
    };

    assert_eq!(
        book_appointment(&mut schedules, &types, &access(), request),
        Err(BookAppointmentError::UnknownAppointmentType),
    );
}

#[test]
fn refuses_an_unknown_practitioner() {
    let (mut schedules, types) = seeded();
    let request = BookingRequest {
        practitioner: PractitionerId::new(2),
        ..request(at(9, 0))
    };

    assert_eq!(
        book_appointment(&mut schedules, &types, &access(), request),
        Err(BookAppointmentError::UnknownPractitioner),
    );
}

#[test]
fn the_practitioner_is_checked_before_the_appointment_type() {
    let (mut schedules, types) = seeded();
    let request = BookingRequest {
        practitioner: PractitionerId::new(2),
        appointment_type: "Check-up",
        ..request(at(9, 0))
    };

    assert_eq!(
        book_appointment(&mut schedules, &types, &access(), request),
        Err(BookAppointmentError::UnknownPractitioner),
    );
}

#[test]
fn cannot_book_into_another_clinics_schedule() {
    let (mut schedules, mut types) = seeded();
    types.add(ClinicId::new(2), consultation()).unwrap();
    let request = BookingRequest {
        clinic: ClinicId::new(2),
        ..request(at(9, 0))
    };

    assert_eq!(
        book_appointment(&mut schedules, &types, &access(), request),
        Err(BookAppointmentError::UnknownPractitioner),
    );
}

#[test]
fn passes_on_the_domain_refusal() {
    let (mut schedules, types) = seeded();
    book_appointment(&mut schedules, &types, &access(), request(at(9, 0))).unwrap();

    assert_eq!(
        book_appointment(&mut schedules, &types, &access(), request(at(9, 15))),
        Err(BookAppointmentError::Booking(BookingError::Overlaps {
            existing: slot(at(9, 0), at(9, 30)),
        })),
    );
}

#[test]
fn a_refused_booking_leaves_the_stored_schedule_unchanged() {
    let (mut schedules, types) = seeded();
    book_appointment(&mut schedules, &types, &access(), request(at(9, 0))).unwrap();
    let before = schedules.get(clinic(), ana());

    let overlapping = book_appointment(&mut schedules, &types, &access(), request(at(9, 15)));
    let outside = book_appointment(&mut schedules, &types, &access(), request(at(11, 45)));

    assert!(overlapping.is_err());
    assert!(outside.is_err());
    assert_eq!(schedules.get(clinic(), ana()), before);
}

#[test]
fn works_through_trait_objects() {
    let (mut schedules, types) = seeded();
    let schedules: &mut dyn ScheduleRepository = &mut schedules;
    let types: &dyn AppointmentTypes = &types;

    assert!(book_appointment(schedules, types, &access(), request(at(9, 0))).is_ok());
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
