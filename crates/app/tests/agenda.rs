use chrono::{DateTime, TimeDelta, TimeZone, Utc};
use clinic_app::agenda::{AgendaError, AgendaQuery, practitioner_agenda};
use clinic_app::in_memory::InMemorySchedules;
use clinic_app::ports::ScheduleRepository;
use clinic_domain::access::{Role, ScheduleAccess};
use clinic_domain::appointment::{AppointmentStatus, AppointmentType};
use clinic_domain::ids::{ClinicId, PatientId, PractitionerId};
use clinic_domain::schedule::Schedule;
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

/// Ana works 09:00-12:00 at clinic 1, with consultations at 09:00 for patient 7 and at 11:00 for
/// patient 8.
fn seeded() -> InMemorySchedules {
    let hours = WorkingHours::new(vec![slot(at(9, 0), at(12, 0))]).unwrap();
    let mut schedule = Schedule::new(hours);
    schedule
        .book(PatientId::new(7), at(9, 0), consultation())
        .unwrap();
    schedule
        .book(PatientId::new(8), at(11, 0), consultation())
        .unwrap();

    let mut schedules = InMemorySchedules::new();
    schedules.save(clinic(), ana(), schedule);
    schedules
}

fn query(range: TimeSlot) -> AgendaQuery {
    AgendaQuery {
        clinic: clinic(),
        practitioner: ana(),
        range,
    }
}

#[test]
fn returns_the_appointments_in_the_range_sorted_by_start() {
    let schedules = seeded();

    let agenda =
        practitioner_agenda(&schedules, &access(), query(slot(at(9, 0), at(12, 0)))).unwrap();

    assert_eq!(agenda.len(), 2);
    assert_eq!(agenda[0].patient(), PatientId::new(7));
    assert_eq!(agenda[0].appointment_type(), &consultation());
    assert_eq!(agenda[0].slot(), slot(at(9, 0), at(9, 30)));
    assert_eq!(agenda[0].status(), AppointmentStatus::Scheduled);
    assert_eq!(agenda[1].patient(), PatientId::new(8));
    assert_eq!(agenda[1].slot(), slot(at(11, 0), at(11, 30)));
}

#[test]
fn leaves_out_appointments_outside_the_range() {
    let schedules = seeded();

    let agenda =
        practitioner_agenda(&schedules, &access(), query(slot(at(10, 0), at(12, 0)))).unwrap();

    assert_eq!(agenda.len(), 1);
    assert_eq!(agenda[0].slot(), slot(at(11, 0), at(11, 30)));
}

#[test]
fn a_range_with_no_appointments_gives_an_empty_agenda() {
    let schedules = seeded();

    assert_eq!(
        practitioner_agenda(&schedules, &access(), query(slot(at(9, 30), at(11, 0)))),
        Ok(Vec::new()),
    );
}

#[test]
fn refuses_an_unknown_practitioner() {
    let schedules = seeded();
    let query = AgendaQuery {
        practitioner: PractitionerId::new(2),
        ..query(slot(at(9, 0), at(12, 0)))
    };

    assert_eq!(
        practitioner_agenda(&schedules, &access(), query),
        Err(AgendaError::UnknownPractitioner),
    );
}

#[test]
fn cannot_read_another_clinics_agenda() {
    let schedules = seeded();
    let query = AgendaQuery {
        clinic: ClinicId::new(2),
        ..query(slot(at(9, 0), at(12, 0)))
    };

    assert_eq!(
        practitioner_agenda(&schedules, &access(), query),
        Err(AgendaError::UnknownPractitioner),
    );
}

#[test]
fn works_through_a_trait_object() {
    let schedules = seeded();
    let schedules: &dyn ScheduleRepository = &schedules;

    let agenda = practitioner_agenda(schedules, &access(), query(slot(at(9, 0), at(12, 0))));

    assert_eq!(agenda.map(|agenda| agenda.len()), Ok(2));
}

#[test]
fn errors_have_readable_messages() {
    assert_eq!(
        AgendaError::UnknownPractitioner.to_string(),
        "clinic has no schedule for that practitioner",
    );
}

#[test]
fn error_implements_the_standard_error_trait() {
    fn assert_is_error<E: std::error::Error>(_: E) {}

    assert_is_error(AgendaError::UnknownPractitioner);
}
