//! A practitioner's agenda: the first query.
//!
//! [`practitioner_agenda`] gets the practitioner's schedule through the port and asks the domain's
//! [`Schedule::appointments_in`](clinic_domain::schedule::Schedule::appointments_in) which
//! appointments fall in the range. It changes nothing.

use clinic_domain::access::ScheduleAccess;
use clinic_domain::appointment::Appointment;
use clinic_domain::ids::{ClinicId, PractitionerId};
use clinic_domain::time_slot::TimeSlot;
use thiserror::Error;

use crate::ports::ScheduleRepository;

/// Whose agenda to read, at which clinic, and for what span of time.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AgendaQuery {
    /// The clinic whose schedules to read.
    pub clinic: ClinicId,
    /// Whose agenda to read.
    pub practitioner: PractitionerId,
    /// The span of time to read. Appointments that overlap it are included.
    pub range: TimeSlot,
}

/// The practitioner's appointments that overlap `query.range`, sorted by start.
///
/// Fails if the practitioner has no schedule at the clinic. An empty agenda is not an error.
/// `access` proves the caller may see schedules.
///
/// `?Sized` lets `S` be a trait object, so `&dyn ScheduleRepository` works too.
pub fn practitioner_agenda<S>(
    schedules: &S,
    _access: &ScheduleAccess,
    query: AgendaQuery,
) -> Result<Vec<Appointment>, AgendaError>
where
    S: ScheduleRepository + ?Sized,
{
    let schedule = schedules
        .get(query.clinic, query.practitioner)
        .ok_or(AgendaError::UnknownPractitioner)?;

    // `schedule` is dropped when this function returns, so the borrowed appointments are copied
    // out before then.
    Ok(schedule.appointments_in(query.range).cloned().collect())
}

/// Why [`practitioner_agenda`] couldn't read an agenda.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum AgendaError {
    /// The practitioner has no schedule at this clinic.
    #[error("clinic has no schedule for that practitioner")]
    UnknownPractitioner,
}
