//! Booking an appointment: the first use case.
//!
//! [`book_appointment`] looks up what it needs through the ports, lets the domain's
//! [`Schedule::book`](clinic_domain::schedule::Schedule::book) decide, and saves the result.

use chrono::{DateTime, Utc};
use clinic_domain::access::ScheduleAccess;
use clinic_domain::appointment::Appointment;
use clinic_domain::ids::{ClinicId, PatientId, PractitionerId};
use clinic_domain::schedule::BookingError;
use thiserror::Error;

use crate::ports::{AppointmentTypes, ScheduleRepository, StorageError};

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
/// Storage can fail at any of them, or when saving. The booking is only returned once it's saved,
/// and if a check fails nothing is saved. `access` proves the caller may manage schedules.
pub async fn book_appointment<S, T>(
    schedules: &mut S,
    types: &T,
    _access: &ScheduleAccess,
    request: BookingRequest<'_>,
) -> Result<Appointment, BookAppointmentError>
where
    S: ScheduleRepository,
    T: AppointmentTypes,
{
    let mut schedule = schedules
        .get(request.clinic, request.practitioner)
        .await?
        .ok_or(BookAppointmentError::UnknownPractitioner)?;
    let appointment_type = types
        .find(request.clinic, request.appointment_type)
        .await?
        .ok_or(BookAppointmentError::UnknownAppointmentType)?;

    let appointment = schedule
        .book(request.patient, request.start, appointment_type)?
        .clone();
    schedules
        .save(request.clinic, request.practitioner, schedule)
        .await?;

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
    /// The schedule or the appointment types couldn't be read, or the booking couldn't be saved.
    #[error(transparent)]
    Storage(#[from] StorageError),
}
