use chrono::{DateTime, TimeDelta, TimeZone, Utc};
use clinic_domain::appointment::{AppointmentStatus, AppointmentType, AppointmentTypeError};

const FINAL_STATUSES: [AppointmentStatus; 3] = [
    AppointmentStatus::Completed,
    AppointmentStatus::Cancelled,
    AppointmentStatus::NoShow,
];

const ALL_STATUSES: [AppointmentStatus; 4] = [
    AppointmentStatus::Scheduled,
    AppointmentStatus::Completed,
    AppointmentStatus::Cancelled,
    AppointmentStatus::NoShow,
];

fn at(hour: u32, minute: u32) -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 3, 2, hour, minute, 0).unwrap()
}

fn consultation() -> AppointmentType {
    AppointmentType::new("Consultation", TimeDelta::minutes(30)).unwrap()
}

#[test]
fn exposes_the_name_and_duration_it_was_created_with() {
    let appointment_type = consultation();

    assert_eq!(appointment_type.name(), "Consultation");
    assert_eq!(appointment_type.duration(), TimeDelta::minutes(30));
}

#[test]
fn trims_the_name() {
    let appointment_type = AppointmentType::new("  Follow-up  ", TimeDelta::minutes(15)).unwrap();

    assert_eq!(appointment_type.name(), "Follow-up");
}

#[test]
fn rejects_an_empty_name() {
    assert_eq!(
        AppointmentType::new("", TimeDelta::minutes(30)),
        Err(AppointmentTypeError::EmptyName),
    );
}

#[test]
fn rejects_a_whitespace_only_name() {
    assert_eq!(
        AppointmentType::new("   ", TimeDelta::minutes(30)),
        Err(AppointmentTypeError::EmptyName),
    );
}

#[test]
fn rejects_a_zero_duration() {
    assert_eq!(
        AppointmentType::new("Consultation", TimeDelta::zero()),
        Err(AppointmentTypeError::NonPositiveDuration),
    );
}

#[test]
fn rejects_a_negative_duration() {
    assert_eq!(
        AppointmentType::new("Consultation", TimeDelta::minutes(-30)),
        Err(AppointmentTypeError::NonPositiveDuration),
    );
}

#[test]
fn builds_a_slot_that_lasts_for_the_duration() {
    let slot = consultation().slot_starting_at(at(9, 0)).unwrap();

    assert_eq!(slot.start(), at(9, 0));
    assert_eq!(slot.end(), at(9, 30));
}

#[test]
fn builds_no_slot_when_the_end_is_out_of_range() {
    assert_eq!(
        consultation().slot_starting_at(DateTime::<Utc>::MAX_UTC),
        None
    );
}

#[test]
fn error_has_a_readable_message() {
    assert_eq!(
        AppointmentTypeError::EmptyName.to_string(),
        "appointment type name is empty",
    );
    assert_eq!(
        AppointmentTypeError::NonPositiveDuration.to_string(),
        "appointment type duration must be positive",
    );
}

#[test]
fn error_implements_the_standard_error_trait() {
    fn assert_is_error<E: std::error::Error>(_: E) {}

    assert_is_error(AppointmentTypeError::EmptyName);
}

#[test]
fn a_scheduled_appointment_can_reach_every_final_status() {
    for next in FINAL_STATUSES {
        assert_eq!(
            AppointmentStatus::Scheduled.transition_to(next),
            Some(next),
            "expected Scheduled to move to {next:?}",
        );
    }
}

#[test]
fn a_scheduled_appointment_cannot_move_to_scheduled() {
    assert_eq!(
        AppointmentStatus::Scheduled.transition_to(AppointmentStatus::Scheduled),
        None,
    );
}

#[test]
fn a_final_status_cannot_move_to_any_status() {
    for current in FINAL_STATUSES {
        for next in ALL_STATUSES {
            assert_eq!(
                current.transition_to(next),
                None,
                "expected {current:?} not to move to {next:?}",
            );
        }
    }
}
