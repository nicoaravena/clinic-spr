//! What the use cases do when storage fails: they stop, pass the [`StorageError`] on, and leave
//! nothing saved.

mod common;

use std::error::Error;

use chrono::{DateTime, NaiveDate, NaiveTime, TimeDelta, TimeZone, Utc, Weekday};
use chrono_tz::America::Santiago;
use clinic_app::agenda::{AgendaError, AgendaQuery, practitioner_agenda};
use clinic_app::booking::{BookAppointmentError, BookingRequest, book_appointment};
use clinic_app::hours::{SetHoursError, SetHoursRequest, set_weekly_hours};
use clinic_app::in_memory::{InMemoryAppointmentTypes, InMemorySession, InMemoryStorage};
use clinic_app::ports::{AppointmentTypes, ScheduleRepository, Session, Storage, StorageError};
use clinic_app::weekly_hours::{WeeklyHours, WeeklyShift};
use clinic_domain::access::{Role, ScheduleAccess};
use clinic_domain::appointment::AppointmentType;
use clinic_domain::ids::{ClinicId, PatientId, PractitionerId};
use clinic_domain::schedule::Schedule;
use clinic_domain::time_slot::TimeSlot;
use clinic_domain::working_hours::WorkingHours;

use common::{save_committed, stored};

/// The step a [`Faulty`] storage fails at.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Step {
    Begin,
    Get,
    Find,
    Save,
    Commit,
}

/// In-memory storage, except that one step always fails with "connection refused".
struct Faulty {
    inner: InMemoryStorage,
    failing: Step,
}

/// A session of a [`Faulty`] storage.
struct FaultySession {
    inner: InMemorySession,
    failing: Step,
}

/// Fails if `step` is the one that's `failing`.
fn check(failing: Step, step: Step) -> Result<(), StorageError> {
    if failing == step {
        Err(StorageError::new("connection refused"))
    } else {
        Ok(())
    }
}

impl Storage for Faulty {
    type Session = FaultySession;

    async fn begin(&self, clinic: ClinicId) -> Result<FaultySession, StorageError> {
        check(self.failing, Step::Begin)?;
        Ok(FaultySession {
            inner: self.inner.begin(clinic).await?,
            failing: self.failing,
        })
    }
}

impl ScheduleRepository for FaultySession {
    async fn get(&self, practitioner: PractitionerId) -> Result<Option<Schedule>, StorageError> {
        check(self.failing, Step::Get)?;
        self.inner.get(practitioner).await
    }

    async fn save(
        &mut self,
        practitioner: PractitionerId,
        schedule: Schedule,
    ) -> Result<(), StorageError> {
        check(self.failing, Step::Save)?;
        self.inner.save(practitioner, schedule).await
    }
}

impl AppointmentTypes for FaultySession {
    async fn find(&self, name: &str) -> Result<Option<AppointmentType>, StorageError> {
        check(self.failing, Step::Find)?;
        self.inner.find(name).await
    }
}

impl Session for FaultySession {
    async fn commit(self) -> Result<(), StorageError> {
        check(self.failing, Step::Commit)?;
        self.inner.commit().await
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

/// Ana works 09:00-12:00 at clinic 1, which offers a 30-minute consultation, and `failing` fails.
async fn seeded(failing: Step) -> Faulty {
    let mut types = InMemoryAppointmentTypes::new();
    let consultation = AppointmentType::new("Consultation", TimeDelta::minutes(30)).unwrap();
    types.add(clinic(), consultation).unwrap();

    let inner = InMemoryStorage::new(types);
    let hours = WorkingHours::new(vec![TimeSlot::new(at(9, 0), at(12, 0)).unwrap()]).unwrap();
    save_committed(&inner, clinic(), ana(), Schedule::new(hours)).await;

    Faulty { inner, failing }
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

fn agenda() -> AgendaQuery {
    AgendaQuery {
        clinic: clinic(),
        practitioner: ana(),
        range: TimeSlot::new(at(9, 0), at(12, 0)).unwrap(),
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
async fn booking_fails_at_every_step_that_storage_fails_and_saves_nothing() {
    for failing in [Step::Begin, Step::Get, Step::Find, Step::Save, Step::Commit] {
        let storage = seeded(failing).await;

        let result = book_appointment(&storage, &access(), booking()).await;

        assert!(
            matches!(result, Err(BookAppointmentError::Storage(_))),
            "{failing:?}: {result:?}"
        );
        let saved = stored(&storage.inner, clinic(), ana()).await.unwrap();
        assert!(saved.appointments().is_empty(), "{failing:?}");
    }
}

#[tokio::test]
async fn the_agenda_fails_when_a_session_cant_begin_or_read() {
    for failing in [Step::Begin, Step::Get] {
        let storage = seeded(failing).await;

        let result = practitioner_agenda(&storage, &access(), agenda()).await;

        assert!(
            matches!(result, Err(AgendaError::Storage(_))),
            "{failing:?}: {result:?}"
        );
    }
}

#[tokio::test]
async fn the_agenda_reads_without_committing() {
    let storage = seeded(Step::Commit).await;

    let result = practitioner_agenda(&storage, &access(), agenda()).await;

    assert!(result.is_ok(), "{result:?}");
}

#[tokio::test]
async fn setting_hours_fails_at_every_step_that_storage_fails_and_saves_nothing() {
    for failing in [Step::Begin, Step::Get, Step::Save, Step::Commit] {
        let storage = Faulty {
            inner: InMemoryStorage::default(),
            failing,
        };

        let result = set_weekly_hours(&storage, &access(), mondays()).await;

        assert!(
            matches!(result, Err(SetHoursError::Storage(_))),
            "{failing:?}: {result:?}"
        );
        assert_eq!(
            stored(&storage.inner, clinic(), ana()).await,
            None,
            "{failing:?}"
        );
    }
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
