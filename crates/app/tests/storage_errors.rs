//! What the use cases do when storage fails: they stop and pass the [`StorageError`] on.

use std::error::Error;

use chrono::{DateTime, NaiveDate, NaiveTime, TimeDelta, TimeZone, Utc, Weekday};
use chrono_tz::America::Santiago;
use clinic_app::agenda::{AgendaError, AgendaQuery, practitioner_agenda};
use clinic_app::booking::{BookAppointmentError, BookingRequest, book_appointment};
use clinic_app::hours::{SetHoursError, SetHoursRequest, set_weekly_hours};
use clinic_app::in_memory::{InMemoryAppointmentTypes, InMemorySchedules};
use clinic_app::ports::{AppointmentTypes, ScheduleRepository, StorageError};
use clinic_app::weekly_hours::{WeeklyHours, WeeklyShift};
use clinic_domain::access::{Role, ScheduleAccess};
use clinic_domain::appointment::AppointmentType;
use clinic_domain::ids::{ClinicId, PatientId, PractitionerId};
use clinic_domain::schedule::Schedule;
use clinic_domain::time_slot::TimeSlot;
use clinic_domain::working_hours::WorkingHours;

/// Storage that can't be reached: every call fails.
struct Unreachable;

impl ScheduleRepository for Unreachable {
    async fn get(
        &self,
        _clinic: ClinicId,
        _practitioner: PractitionerId,
    ) -> Result<Option<Schedule>, StorageError> {
        Err(StorageError::new("connection refused"))
    }

    async fn save(
        &mut self,
        _clinic: ClinicId,
        _practitioner: PractitionerId,
        _schedule: Schedule,
    ) -> Result<(), StorageError> {
        Err(StorageError::new("connection refused"))
    }
}

impl AppointmentTypes for Unreachable {
    async fn find(
        &self,
        _clinic: ClinicId,
        _name: &str,
    ) -> Result<Option<AppointmentType>, StorageError> {
        Err(StorageError::new("connection refused"))
    }
}

/// Schedules that can be read but not written: `get` reads the wrapped schedules, `save` fails.
struct ReadOnly(InMemorySchedules);

impl ScheduleRepository for ReadOnly {
    async fn get(
        &self,
        clinic: ClinicId,
        practitioner: PractitionerId,
    ) -> Result<Option<Schedule>, StorageError> {
        self.0.get(clinic, practitioner).await
    }

    async fn save(
        &mut self,
        _clinic: ClinicId,
        _practitioner: PractitionerId,
        _schedule: Schedule,
    ) -> Result<(), StorageError> {
        Err(StorageError::new("disk full"))
    }
}

fn clinic() -> ClinicId {
    ClinicId::new(1)
}

fn ana() -> PractitionerId {
    PractitionerId::new(1)
}

fn at(hour: u32, minute: u32) -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 3, 2, hour, minute, 0).unwrap()
}

fn access() -> ScheduleAccess {
    Role::Receptionist.grant().unwrap()
}

/// Ana works 09:00-12:00 at clinic 1, which offers a 30-minute consultation.
async fn seeded() -> (InMemorySchedules, InMemoryAppointmentTypes) {
    let mut schedules = InMemorySchedules::new();
    let hours = WorkingHours::new(vec![TimeSlot::new(at(9, 0), at(12, 0)).unwrap()]).unwrap();
    schedules
        .save(clinic(), ana(), Schedule::new(hours))
        .await
        .unwrap();

    let mut types = InMemoryAppointmentTypes::new();
    let consultation = AppointmentType::new("Consultation", TimeDelta::minutes(30)).unwrap();
    types.add(clinic(), consultation).unwrap();

    (schedules, types)
}

fn booking() -> BookingRequest<'static> {
    BookingRequest {
        clinic: clinic(),
        practitioner: ana(),
        patient: PatientId::new(7),
        appointment_type: "Consultation",
        start: at(9, 0),
    }
}

fn mondays() -> SetHoursRequest {
    let shift = WeeklyShift::new(
        Weekday::Mon,
        NaiveTime::from_hms_opt(9, 0, 0).unwrap(),
        NaiveTime::from_hms_opt(13, 0, 0).unwrap(),
    )
    .unwrap();
    SetHoursRequest {
        clinic: clinic(),
        practitioner: ana(),
        hours: WeeklyHours::new(Santiago, vec![shift]).unwrap(),
        from: NaiveDate::from_ymd_opt(2026, 3, 30).unwrap(),
    }
}

#[tokio::test]
async fn booking_fails_when_schedules_are_unreachable() {
    let (_, types) = seeded().await;

    let result = book_appointment(&mut Unreachable, &types, &access(), booking()).await;

    assert!(
        matches!(result, Err(BookAppointmentError::Storage(_))),
        "{result:?}"
    );
}

#[tokio::test]
async fn booking_fails_when_appointment_types_are_unreachable() {
    let (mut schedules, _) = seeded().await;

    let result = book_appointment(&mut schedules, &Unreachable, &access(), booking()).await;

    assert!(
        matches!(result, Err(BookAppointmentError::Storage(_))),
        "{result:?}"
    );
}

#[tokio::test]
async fn a_booking_that_cant_be_saved_is_an_error_not_a_success() {
    let (schedules, types) = seeded().await;
    let mut schedules = ReadOnly(schedules);

    let result = book_appointment(&mut schedules, &types, &access(), booking()).await;

    assert!(
        matches!(result, Err(BookAppointmentError::Storage(_))),
        "{result:?}"
    );
    let stored = schedules.0.get(clinic(), ana()).await.unwrap().unwrap();
    assert!(stored.appointments().is_empty());
}

#[tokio::test]
async fn the_agenda_fails_when_schedules_are_unreachable() {
    let query = AgendaQuery {
        clinic: clinic(),
        practitioner: ana(),
        range: TimeSlot::new(at(9, 0), at(12, 0)).unwrap(),
    };

    let result = practitioner_agenda(&Unreachable, &access(), query).await;

    assert!(matches!(result, Err(AgendaError::Storage(_))), "{result:?}");
}

#[tokio::test]
async fn setting_hours_fails_when_schedules_are_unreachable() {
    let result = set_weekly_hours(&mut Unreachable, &access(), mondays()).await;

    assert!(
        matches!(result, Err(SetHoursError::Storage(_))),
        "{result:?}"
    );
}

#[tokio::test]
async fn hours_that_cant_be_saved_are_an_error_not_a_success() {
    let mut schedules = ReadOnly(InMemorySchedules::new());

    let result = set_weekly_hours(&mut schedules, &access(), mondays()).await;

    assert!(
        matches!(result, Err(SetHoursError::Storage(_))),
        "{result:?}"
    );
}

#[test]
fn the_message_is_generic_and_the_cause_is_its_source() {
    let error = StorageError::new("connection refused");

    assert_eq!(error.to_string(), "storage failed");
    assert_eq!(
        error.source().map(ToString::to_string),
        Some("connection refused".to_owned()),
    );
}

#[test]
fn a_use_case_error_passes_the_storage_error_on_unchanged() {
    let error = BookAppointmentError::from(StorageError::new("connection refused"));

    assert_eq!(error.to_string(), "storage failed");
    assert_eq!(
        error.source().map(ToString::to_string),
        Some("connection refused".to_owned()),
    );
}

#[test]
fn a_storage_error_can_cross_threads() {
    fn assert_thread_safe<E: Error + Send + Sync + 'static>() {}

    assert_thread_safe::<StorageError>();
}
