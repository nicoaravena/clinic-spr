//! Setting a practitioner's weekly hours: the first use case that changes working hours.
//!
//! [`set_weekly_hours`] expands a [`WeeklyHours`] template over [`HORIZON_DAYS`] days from a
//! local start date, and puts the result in the practitioner's schedule from the start of that
//! date on. Earlier shifts stay as they were, and so do booked appointments: if one wouldn't fit
//! the new hours, nothing is saved.

use chrono::{Days, NaiveDate};
use clinic_domain::access::ScheduleAccess;
use clinic_domain::ids::{ClinicId, PractitionerId};
use clinic_domain::schedule::{ReplaceHoursError, Schedule};
use clinic_domain::working_hours::WorkingHours;
use thiserror::Error;

use crate::ports::ScheduleRepository;
use crate::weekly_hours::WeeklyHours;

/// How many days of shifts one call expands, starting on its `from` date.
pub const HORIZON_DAYS: u64 = 28;

/// Days of margin kept on either side of the horizon. Reading a local time near a daylight saving
/// change looks a day either side of it, which must stay within chrono's calendar.
const CALENDAR_MARGIN_DAYS: u64 = 2;

/// Whose hours to set, at which clinic, and from which local date.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SetHoursRequest {
    /// The clinic the practitioner works at.
    pub clinic: ClinicId,
    /// Whose hours to set.
    pub practitioner: PractitionerId,
    /// The weekly template and the time zone it's written in.
    pub hours: WeeklyHours,
    /// The first local date the template applies to, in its time zone.
    pub from: NaiveDate,
}

/// What [`set_weekly_hours`] did.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HoursSet {
    /// Whether the practitioner's schedule was created or already existed.
    pub outcome: HoursOutcome,
    /// The local date after the last one expanded: the template covers `[from, until)`.
    pub until: NaiveDate,
    /// The template's shifts over the horizon, in UTC. Shifts kept from before `from` aren't
    /// included.
    pub shifts: WorkingHours,
}

/// Whether [`set_weekly_hours`] created a schedule or changed one.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HoursOutcome {
    /// The practitioner had no schedule at the clinic, and now has one with these hours.
    Created,
    /// The practitioner's schedule had its hours replaced from `from` on.
    Replaced,
}

/// Sets the practitioner's hours from the start of `request.from` to `request.hours`, expanded
/// over [`HORIZON_DAYS`] days, and saves the schedule.
///
/// A practitioner without a schedule at the clinic gets a new one. Otherwise the shifts before
/// `from` starts are kept, everything after is replaced, and every booked appointment must still
/// fit. Setting the same hours twice leaves the same schedule. On failure nothing is saved.
/// `access` proves the caller may manage schedules.
///
/// `?Sized` lets `S` be a trait object, so `&mut dyn ScheduleRepository` works too.
pub fn set_weekly_hours<S>(
    schedules: &mut S,
    _access: &ScheduleAccess,
    request: SetHoursRequest,
) -> Result<HoursSet, SetHoursError>
where
    S: ScheduleRepository + ?Sized,
{
    let from = request.from;
    let until = from
        .checked_add_days(Days::new(HORIZON_DAYS))
        .ok_or(SetHoursError::OutOfRange)?;
    let within_calendar = from
        .checked_sub_days(Days::new(CALENDAR_MARGIN_DAYS))
        .and_then(|_| until.checked_add_days(Days::new(CALENDAR_MARGIN_DAYS)))
        .is_some();
    if !within_calendar {
        return Err(SetHoursError::OutOfRange);
    }

    let shifts = request.hours.working_hours(from, until);

    let (schedule, outcome) = match schedules.get(request.clinic, request.practitioner) {
        None => (Schedule::new(shifts.clone()), HoursOutcome::Created),
        Some(mut schedule) => {
            let cut = request.hours.day_start(from);
            let hours = schedule.hours().replace_from(cut, &shifts);
            schedule.replace_hours(hours)?;
            (schedule, HoursOutcome::Replaced)
        }
    };
    schedules.save(request.clinic, request.practitioner, schedule);

    Ok(HoursSet {
        outcome,
        until,
        shifts,
    })
}

/// Why [`set_weekly_hours`] refused to set the hours.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum SetHoursError {
    /// `from`, or the end of the horizon, is too close to the earliest or latest date chrono can
    /// represent.
    #[error("the hours would run past the earliest or latest supported date")]
    OutOfRange,
    /// The new hours would leave a booked appointment outside them.
    #[error(transparent)]
    Hours(#[from] ReplaceHoursError),
}
