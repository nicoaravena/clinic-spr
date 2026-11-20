//! Booking an appointment: the first use case.
//!
//! [`book_appointment`] begins a session at the clinic, looks up what it needs through the
//! session's ports, lets the domain's [`Schedule::book`](clinic_domain::schedule::Schedule::book)
//! decide, saves the result, and commits. The session makes the whole read-modify-write one unit
//! of work.

use chrono::{DateTime, Utc};
use clinic_domain::access::ScheduleAccess;
use clinic_domain::appointment::Appointment;
use clinic_domain::ids::{ClinicId, PatientId, PractitionerId};
use clinic_domain::schedule::BookingError;
use thiserror::Error;

use crate::ports::{AppointmentTypes, ScheduleRepository, Session, Storage, StorageError};

/// What to book, for whom, with whom, and when.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BookingRequest<'a> {
    /// The clinic the booking is made at.
    pub clinic: ClinicId,
    /// Whose schedule to book into.
    pub practitioner: PractitionerId,
    /// Who the appointment is for.
    pub patient: PatientId,
    /// The name of one of the clinic's appointment types.
    pub appointment_type: &'a str,
    /// When the appointment starts.
    pub start: DateTime<Utc>,
}

/// Books the appointment described by `request` and saves it in the practitioner's schedule.
///
/// The checks run in this order, and the first one that fails decides the error:
///
/// 1. The practitioner must have a schedule at the clinic.
/// 2. The clinic must offer the named appointment type.
/// 3. The domain's booking rules must accept it.
///
/// Storage can fail at any of them, or when beginning, saving or committing. The booking is only
/// returned once it's committed. If a check fails, the session is dropped and nothing is saved.
/// `access` proves the caller may manage schedules.
pub async fn book_appointment<St>(
    storage: &St,
    _access: &ScheduleAccess,
    request: BookingRequest<'_>,
) -> Result<Appointment, BookAppointmentError>
where
    St: Storage + Sync,
{
    let mut session = storage.begin(request.clinic).await?;
    let mut schedule = session
        .get(request.practitioner)
        .await?
        .ok_or(BookAppointmentError::UnknownPractitioner)?;
    let appointment_type = session
        .find(request.appointment_type)
        .await?
        .ok_or(BookAppointmentError::UnknownAppointmentType)?;

    let appointment = schedule
        .book(request.patient, request.start, appointment_type)?
        .clone();
    session.save(request.practitioner, schedule).await?;
    session.commit().await?;

    Ok(appointment)
}

/// Why [`book_appointment`] refused a booking.
///
/// It isn't `Clone` or `PartialEq`, because a [`StorageError`] holds its cause as a boxed error.
#[derive(Debug, Error)]
pub enum BookAppointmentError {
    /// The practitioner has no schedule at this clinic.
    #[error("clinic has no schedule for that practitioner")]
    UnknownPractitioner,
    /// The clinic offers no appointment type with that name.
    #[error("clinic has no appointment type with that name")]
    UnknownAppointmentType,
    /// The domain's booking rules refused it.
    #[error(transparent)]
    Booking(#[from] BookingError),
    /// A session couldn't begin, the schedule or the appointment types couldn't be read, or the
    /// booking couldn't be saved or committed.
    #[error(transparent)]
    Storage(#[from] StorageError),
}
