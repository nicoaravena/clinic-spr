//! `GET /practitioners/{id}/agenda`: reading a practitioner's agenda over HTTP.
//!
//! The handler is a thin adapter: it reads the practitioner from the path and the range from the
//! query string, calls [`practitioner_agenda`], and turns the appointments into JSON.

use axum::Json;
use axum::extract::rejection::{PathRejection, QueryRejection};
use axum::extract::{Path, Query, State};
use chrono::{DateTime, Utc};
use clinic_app::agenda::{AgendaQuery, practitioner_agenda};
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
pub async fn agenda(
    State(state): State<AppState>,
    ClinicHeader(clinic): ClinicHeader,
    id: Result<Path<u64>, PathRejection>,
    params: Result<Query<AgendaParams>, QueryRejection>,
) -> Result<Json<AgendaBody>, ApiError> {
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
    let appointments = practitioner_agenda(state.lock()?.schedules.as_ref(), &access, query)?;

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
