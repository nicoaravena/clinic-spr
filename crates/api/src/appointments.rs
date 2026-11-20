//! `POST /appointments`: booking an appointment over HTTP.
//!
//! The handler is a thin adapter: it turns the JSON body into a
//! [`BookingRequest`], calls [`book_appointment`], and turns the result back into JSON. The booking
//! rules stay in the domain.

use axum::Json;
use axum::extract::State;
use axum::extract::rejection::JsonRejection;
use axum::http::StatusCode;
use chrono::{DateTime, Utc};
use clinic_app::booking::{BookingRequest, book_appointment};
use clinic_app::ports::Storage;
use clinic_domain::access::ScheduleAccess;
use clinic_domain::appointment::{Appointment, AppointmentStatus};
use clinic_domain::ids::{PatientId, PractitionerId};
use serde::{Deserialize, Serialize};

use crate::STAND_IN_CALLER;
use crate::clinic_header::ClinicHeader;
use crate::error::ApiError;
use crate::state::AppState;

/// The JSON body of `POST /appointments`.
///
/// `start` is an RFC 3339 timestamp with an offset, such as `"2026-03-02T10:00:00Z"` or
/// `"2026-03-02T07:00:00-03:00"`. It's converted to UTC when read. Fields not listed here are
/// refused.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BookAppointmentBody {
    /// Whose schedule to book into.
    pub practitioner_id: u64,
    /// Who the appointment is for.
    pub patient_id: u64,
    /// The name of one of the clinic's appointment types, matched exactly.
    pub appointment_type: String,
    /// When the appointment starts.
    pub start: DateTime<Utc>,
}

/// The JSON body of a booked appointment. Times are RFC 3339 in UTC.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AppointmentBody {
    /// Whose schedule the appointment is in.
    pub practitioner_id: u64,
    /// Who the appointment is for.
    pub patient_id: u64,
    /// The name of the appointment's type.
    pub appointment_type: String,
    /// When the appointment starts.
    pub start: DateTime<Utc>,
    /// When the appointment ends.
    pub end: DateTime<Utc>,
    /// `"scheduled"`, `"completed"`, `"cancelled"` or `"no_show"`.
    pub status: &'static str,
}

impl AppointmentBody {
    /// The JSON shape of `appointment`, booked in `practitioner`'s schedule.
    pub fn new(practitioner: PractitionerId, appointment: &Appointment) -> Self {
        Self {
            practitioner_id: practitioner.value(),
            patient_id: appointment.patient().value(),
            appointment_type: appointment.appointment_type().name().to_owned(),
            start: appointment.slot().start(),
            end: appointment.slot().end(),
            status: status_name(appointment.status()),
        }
    }
}

/// The name of a status in JSON. Written out here so the domain enum doesn't depend on serde.
fn status_name(status: AppointmentStatus) -> &'static str {
    match status {
        AppointmentStatus::Scheduled => "scheduled",
        AppointmentStatus::Completed => "completed",
        AppointmentStatus::Cancelled => "cancelled",
        AppointmentStatus::NoShow => "no_show",
    }
}

/// Books an appointment at the clinic named in `X-Clinic-Id`, answering `201 Created` with it.
///
/// The body is taken as a `Result` so a bad body becomes an [`ApiError`] with the same JSON shape
/// as every other error, instead of axum's plain-text rejection.
///
/// `St` is the storage in the [`AppState`]; [`router`](crate::router) picks it.
pub async fn book<St>(
    State(state): State<AppState<St>>,
    ClinicHeader(clinic): ClinicHeader,
    body: Result<Json<BookAppointmentBody>, JsonRejection>,
) -> Result<(StatusCode, Json<AppointmentBody>), ApiError>
where
    St: Storage + Send + Sync + 'static,
{
    let Json(body) = body?;
    let access: ScheduleAccess = STAND_IN_CALLER.grant()?;
    let practitioner = PractitionerId::new(body.practitioner_id);
    let request = BookingRequest {
        clinic,
        practitioner,
        patient: PatientId::new(body.patient_id),
        appointment_type: &body.appointment_type,
        start: body.start,
    };

    let appointment = book_appointment(state.storage(), &access, request).await?;

    Ok((
        StatusCode::CREATED,
        Json(AppointmentBody::new(practitioner, &appointment)),
    ))
}
