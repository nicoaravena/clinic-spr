//! The practitioner routes, over HTTP:
//!
//! - `GET /practitioners/{id}/agenda` reads a practitioner's agenda.
//! - `PUT /practitioners/{id}/hours` sets their weekly hours.
//!
//! The handlers are thin adapters: they read the practitioner from the path and the rest from the
//! query string or body, call [`practitioner_agenda`] or [`set_weekly_hours`], and turn the result
//! into JSON.

use axum::Json;
use axum::extract::rejection::{JsonRejection, PathRejection, QueryRejection};
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use chrono::{DateTime, NaiveDate, NaiveTime, Utc, Weekday};
use chrono_tz::Tz;
use clinic_app::agenda::{AgendaQuery, practitioner_agenda};
use clinic_app::hours::{HoursOutcome, SetHoursRequest, set_weekly_hours};
use clinic_app::ports::{AppointmentTypes, ScheduleRepository};
use clinic_app::weekly_hours::{WeeklyHours, WeeklyShift};
use clinic_domain::access::ScheduleAccess;
use clinic_domain::ids::PractitionerId;
use clinic_domain::time_slot::TimeSlot;
use serde::{Deserialize, Serialize};

use crate::STAND_IN_CALLER;
use crate::appointments::AppointmentBody;
use crate::clinic_header::ClinicHeader;
use crate::error::ApiError;
use crate::state::AppState;

/// The query string of `GET /practitioners/{id}/agenda`: `?from=<time>&to=<time>`.
///
/// Both are RFC 3339 timestamps with an offset, converted to UTC when read. In a query string `+`
/// means a space, so a positive offset must be sent as `%2B`, as in `10:00:00%2B01:00`. Parameters
/// not listed here are refused.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AgendaParams {
    /// The start of the span to read.
    pub from: DateTime<Utc>,
    /// The end of the span to read. Must be after `from`.
    pub to: DateTime<Utc>,
}

/// The JSON body of an agenda. Times are RFC 3339 in UTC.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AgendaBody {
    /// Whose agenda this is.
    pub practitioner_id: u64,
    /// The start of the span that was read.
    pub from: DateTime<Utc>,
    /// The end of the span that was read.
    pub to: DateTime<Utc>,
    /// The appointments that overlap the span, sorted by start. Empty if there are none.
    pub appointments: Vec<AppointmentBody>,
}

/// Reads the agenda of the practitioner in the path, at the clinic named in `X-Clinic-Id`,
/// answering `200 OK` with it.
///
/// Like the body in [`book`](crate::appointments::book), the path and the query are taken as
/// `Result`s so that a bad one becomes an [`ApiError`] with the usual JSON shape.
pub async fn agenda<S, T>(
    State(state): State<AppState<S, T>>,
    ClinicHeader(clinic): ClinicHeader,
    id: Result<Path<u64>, PathRejection>,
    params: Result<Query<AgendaParams>, QueryRejection>,
) -> Result<Json<AgendaBody>, ApiError>
where
    S: ScheduleRepository + Send + Sync + 'static,
    T: AppointmentTypes + Send + Sync + 'static,
{
    let Path(id) = id?;
    let Query(params) = params?;
    let access: ScheduleAccess = STAND_IN_CALLER.grant()?;
    let practitioner = PractitionerId::new(id);
    let query = AgendaQuery {
        clinic,
        practitioner,
        range: TimeSlot::new(params.from, params.to)?,
    };

    // Reading needs the lock too: a booking may be saving the same schedule on another thread.
    // The guard is a temporary, dropped at the end of the statement, after the `.await`.
    let appointments = practitioner_agenda(&state.lock().await.schedules, &access, query).await?;

    Ok(Json(AgendaBody {
        practitioner_id: practitioner.value(),
        from: params.from,
        to: params.to,
        appointments: appointments
            .iter()
            .map(|appointment| AppointmentBody::new(practitioner, appointment))
            .collect(),
    }))
}

/// The JSON body of `PUT /practitioners/{id}/hours`.
///
/// ```json
/// {
///   "time_zone": "America/Santiago",
///   "from": "2026-03-30",
///   "shifts": [{ "weekday": "Mon", "start": "09:00", "end": "13:00" }]
/// }
/// ```
///
/// Fields not listed here are refused.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SetHoursBody {
    /// The IANA name of the time zone the shifts are written in, such as `"America/Santiago"`.
    /// Matched exactly, case included.
    pub time_zone: String,
    /// The first local date the hours apply to, such as `"2026-03-30"`.
    pub from: NaiveDate,
    /// The shifts worked each week. May be empty.
    pub shifts: Vec<WeeklyShiftBody>,
}

/// One weekly shift in [`SetHoursBody`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WeeklyShiftBody {
    /// The day of the week, in English, short or full and in any case: `"Mon"`, `"monday"`.
    pub weekday: Weekday,
    /// When the shift starts, in local time: `"09:00"` or `"09:00:00"`.
    pub start: NaiveTime,
    /// When the shift ends, in local time, on the same day. Must be after `start`.
    pub end: NaiveTime,
}

/// The JSON body of hours that were set. Shift times are RFC 3339 in UTC.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct HoursBody {
    /// Whose hours these are.
    pub practitioner_id: u64,
    /// The time zone the shifts were written in.
    pub time_zone: &'static str,
    /// The first local date the hours apply to.
    pub from: NaiveDate,
    /// The local date after the last one expanded. Shifts from then on aren't set yet.
    pub until: NaiveDate,
    /// The shifts on the dates from `from` up to `until`, sorted by start. Shifts before `from`
    /// aren't included.
    pub shifts: Vec<ShiftBody>,
}

/// One dated shift in [`HoursBody`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct ShiftBody {
    /// When the shift starts.
    pub start: DateTime<Utc>,
    /// When the shift ends.
    pub end: DateTime<Utc>,
}

/// Sets the weekly hours of the practitioner in the path, at the clinic named in `X-Clinic-Id`.
///
/// Answers `201 Created` if the practitioner had no schedule at the clinic, and `200 OK` if their
/// hours were replaced. Sending the same body again changes nothing and gives the same answer, as
/// `PUT` should.
pub async fn set_hours<S, T>(
    State(state): State<AppState<S, T>>,
    ClinicHeader(clinic): ClinicHeader,
    id: Result<Path<u64>, PathRejection>,
    body: Result<Json<SetHoursBody>, JsonRejection>,
) -> Result<(StatusCode, Json<HoursBody>), ApiError>
where
    S: ScheduleRepository + Send + Sync + 'static,
    T: AppointmentTypes + Send + Sync + 'static,
{
    let Path(id) = id?;
    let Json(body) = body?;
    let time_zone: Tz = body
        .time_zone
        .parse()
        .map_err(|_| ApiError::InvalidTimeZone {
            name: body.time_zone.clone(),
        })?;
    let shifts = body
        .shifts
        .iter()
        .map(|shift| WeeklyShift::new(shift.weekday, shift.start, shift.end))
        .collect::<Result<Vec<_>, _>>()?;
    let access: ScheduleAccess = STAND_IN_CALLER.grant()?;
    let practitioner = PractitionerId::new(id);
    let request = SetHoursRequest {
        clinic,
        practitioner,
        hours: WeeklyHours::new(time_zone, shifts)?,
        from: body.from,
    };

    // As in `book`, the lock is held from reading the schedule to saving it.
    let set = set_weekly_hours(&mut state.lock().await.schedules, &access, request).await?;

    let status = match set.outcome {
        HoursOutcome::Created => StatusCode::CREATED,
        HoursOutcome::Replaced => StatusCode::OK,
    };
    Ok((
        status,
        Json(HoursBody {
            practitioner_id: practitioner.value(),
            time_zone: time_zone.name(),
            from: body.from,
            until: set.until,
            shifts: set
                .shifts
                .shifts()
                .iter()
                .map(|shift| ShiftBody {
                    start: shift.start(),
                    end: shift.end(),
                })
                .collect(),
        }),
    ))
}
