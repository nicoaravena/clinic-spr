use chrono::{DateTime, TimeDelta, TimeZone, Utc};
use clinic_domain::appointment::{AppointmentStatus, AppointmentType};
use clinic_domain::ids::PatientId;
use clinic_domain::schedule::{BookingError, Schedule};
use clinic_domain::time_slot::TimeSlot;
use clinic_domain::working_hours::WorkingHours;

fn on(day: u32, hour: u32, minute: u32) -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 3, day, hour, minute, 0).unwrap()
}

fn at(hour: u32, minute: u32) -> DateTime<Utc> {
    on(2, hour, minute)
}

fn slot(start: DateTime<Utc>, end: DateTime<Utc>) -> TimeSlot {
    TimeSlot::new(start, end).unwrap()
}

fn patient() -> PatientId {
    PatientId::new(1)
}

fn consultation() -> AppointmentType {
    AppointmentType::new("Consultation", TimeDelta::minutes(30)).unwrap()
}

/// 09:00-13:00 and 14:00-18:00, with a lunch break between them.
fn day_with_lunch_break() -> Schedule {
    Schedule::new(
        WorkingHours::new(vec![slot(at(9, 0), at(13, 0)), slot(at(14, 0), at(18, 0))]).unwrap(),
    )
}

fn assert_rejected(start: DateTime<Utc>, expected: BookingError) {
    let mut schedule = day_with_lunch_break();

    assert_eq!(
        schedule.book(patient(), start, consultation()),
        Err(expected)
    );
}

#[test]
fn books_an_appointment_inside_a_shift() {
    let mut schedule = day_with_lunch_break();

    let appointment = schedule.book(patient(), at(9, 0), consultation()).unwrap();

    assert_eq!(appointment.appointment_type(), &consultation());
    assert_eq!(appointment.slot(), slot(at(9, 0), at(9, 30)));
    assert_eq!(appointment.status(), AppointmentStatus::Scheduled);
}

#[test]
fn the_appointment_records_its_patient() {
    let mut schedule = day_with_lunch_break();

    let appointment = schedule
        .book(PatientId::new(7), at(9, 0), consultation())
        .unwrap();

    assert_eq!(appointment.patient(), PatientId::new(7));
}

#[test]
fn keeps_the_booked_appointment() {
    let mut schedule = day_with_lunch_break();

    schedule.book(patient(), at(9, 0), consultation()).unwrap();

    let slots: Vec<TimeSlot> = schedule.appointments().iter().map(|a| a.slot()).collect();
    assert_eq!(slots, [slot(at(9, 0), at(9, 30))]);
}

#[test]
fn books_in_the_second_shift() {
    let mut schedule = day_with_lunch_break();

    assert!(schedule.book(patient(), at(15, 0), consultation()).is_ok());
}

#[test]
fn books_an_appointment_ending_exactly_at_the_end_of_a_shift() {
    let mut schedule = day_with_lunch_break();

    assert!(schedule.book(patient(), at(17, 30), consultation()).is_ok());
}

#[test]
fn books_back_to_back_appointments() {
    let mut schedule = day_with_lunch_break();

    schedule.book(patient(), at(9, 0), consultation()).unwrap();

    assert!(schedule.book(patient(), at(9, 30), consultation()).is_ok());
    assert_eq!(schedule.appointments().len(), 2);
}

#[test]
fn rejects_an_appointment_during_the_gap_between_shifts() {
    assert_rejected(at(13, 0), BookingError::OutsideWorkingHours);
}

#[test]
fn rejects_an_appointment_running_past_the_end_of_a_shift() {
    assert_rejected(at(17, 45), BookingError::OutsideWorkingHours);
}

#[test]
fn rejects_an_appointment_on_a_day_without_a_shift() {
    assert_rejected(on(3, 9, 0), BookingError::OutsideWorkingHours);
}

#[test]
fn empty_working_hours_reject_every_appointment() {
    let mut schedule = Schedule::new(WorkingHours::new(Vec::new()).unwrap());

    assert_eq!(
        schedule.book(patient(), at(9, 0), consultation()),
        Err(BookingError::OutsideWorkingHours),
    );
}

#[test]
fn rejects_an_appointment_that_overlaps_an_existing_one() {
    let mut schedule = day_with_lunch_break();
    schedule.book(patient(), at(9, 0), consultation()).unwrap();

    assert_eq!(
        schedule.book(patient(), at(9, 15), consultation()),
        Err(BookingError::Overlaps {
            existing: slot(at(9, 0), at(9, 30)),
        }),
    );
}

#[test]
fn rejects_an_appointment_at_the_same_time_as_an_existing_one() {
    let mut schedule = day_with_lunch_break();
    schedule.book(patient(), at(9, 0), consultation()).unwrap();

    assert_eq!(
        schedule.book(patient(), at(9, 0), consultation()),
        Err(BookingError::Overlaps {
            existing: slot(at(9, 0), at(9, 30)),
        }),
    );
}

#[test]
fn working_hours_are_checked_before_overlap() {
    let mut schedule = day_with_lunch_break();
    schedule
        .book(patient(), at(17, 30), consultation())
        .unwrap();

    // 17:45-18:15 overlaps 17:30-18:00 and also runs past the end of the shift.
    assert_eq!(
        schedule.book(patient(), at(17, 45), consultation()),
        Err(BookingError::OutsideWorkingHours),
    );
}

#[test]
fn rejects_an_appointment_ending_out_of_range() {
    assert_rejected(DateTime::<Utc>::MAX_UTC, BookingError::OutOfRange);
}

#[test]
fn a_rejected_booking_leaves_the_schedule_unchanged() {
    let mut schedule = day_with_lunch_break();
    schedule.book(patient(), at(9, 0), consultation()).unwrap();
    let before = schedule.clone();

    assert!(schedule.book(patient(), at(9, 15), consultation()).is_err());
    assert!(schedule.book(patient(), at(13, 0), consultation()).is_err());

    assert_eq!(schedule, before);
}

#[test]
fn exposes_the_appointments_sorted_by_start() {
    let mut schedule = day_with_lunch_break();

    schedule.book(patient(), at(15, 0), consultation()).unwrap();
    schedule.book(patient(), at(9, 0), consultation()).unwrap();
    schedule.book(patient(), at(11, 0), consultation()).unwrap();

    let starts: Vec<DateTime<Utc>> = schedule
        .appointments()
        .iter()
        .map(|a| a.slot().start())
        .collect();
    assert_eq!(starts, [at(9, 0), at(11, 0), at(15, 0)]);
}

#[test]
fn error_has_a_readable_message() {
    assert_eq!(
        BookingError::OutOfRange.to_string(),
        "appointment would end after the latest supported time",
    );
    assert_eq!(
        BookingError::OutsideWorkingHours.to_string(),
        "appointment is outside the practitioner's working hours",
    );
    assert_eq!(
        BookingError::Overlaps {
            existing: slot(at(9, 0), at(9, 30)),
        }
        .to_string(),
        "appointment overlaps an existing appointment",
    );
}

#[test]
fn error_implements_the_standard_error_trait() {
    fn assert_is_error<E: std::error::Error>(_: E) {}

    assert_is_error(BookingError::OutsideWorkingHours);
}
