//! Every way a request can fail, and the HTTP response each one becomes.
//!
//! Every error response has the same JSON body:
//!
//! ```json
//! { "error": "overlaps", "message": "appointment overlaps an existing appointment" }
//! ```
//!
//! `error` is a stable code that clients can match on. `message` is for people and may change.

use axum::Json;
use axum::extract::rejection::{JsonRejection, PathRejection, QueryRejection};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use clinic_app::agenda::AgendaError;
use clinic_app::booking::BookAppointmentError;
use clinic_app::hours::SetHoursError;
use clinic_app::weekly_hours::WeeklyHoursError;
use clinic_domain::access::AccessDenied;
use clinic_domain::schedule::{BookingError, ReplaceHoursError};
use clinic_domain::time_slot::TimeSlotError;
use serde::Serialize;
use thiserror::Error;

/// Why a request failed.
#[derive(Debug, Error)]
pub enum ApiError {
    /// The request has no `X-Clinic-Id` header.
    #[error("the X-Clinic-Id header is missing")]
    MissingClinic,
    /// The `X-Clinic-Id` header isn't a whole number that fits in a `u64`.
    #[error("the X-Clinic-Id header must be a non-negative whole number")]
    InvalidClinic,
    /// The body isn't the JSON the route expects. Keeps the status axum chose for it.
    #[error("{message}")]
    InvalidBody {
        /// 400 for malformed JSON, 415 for a missing JSON content type, 422 for a missing,
        /// mistyped or unknown field.
        status: StatusCode,
        /// What axum found wrong with the body.
        message: String,
    },
    /// A path parameter, such as the `{id}` in `/practitioners/{id}/agenda`, isn't what the route
    /// expects. Keeps the status axum chose for it.
    #[error("{message}")]
    InvalidPath {
        /// 400 for a value that doesn't parse. 500 if the route and the handler disagree on the
        /// parameters, which is a bug in this crate.
        status: StatusCode,
        /// What axum found wrong with the path.
        message: String,
    },
    /// The query string is missing a parameter, has a mistyped or unknown one, or doesn't parse.
    #[error("{message}")]
    InvalidQuery {
        /// 400, the status axum chose for it.
        status: StatusCode,
        /// What axum found wrong with the query string.
        message: String,
    },
    /// A requested span of time doesn't end after it starts.
    #[error(transparent)]
    InvalidRange(#[from] TimeSlotError),
    /// A time zone isn't one of the IANA names, such as `America/Santiago`. Names are matched
    /// exactly, case included.
    #[error("{name:?} isn't a time zone name such as \"America/Santiago\"")]
    InvalidTimeZone {
        /// The name that was sent.
        name: String,
    },
    /// A weekly template's shifts aren't valid.
    #[error(transparent)]
    WeeklyHours(#[from] WeeklyHoursError),
    /// The caller's role may not do this.
    #[error(transparent)]
    AccessDenied(#[from] AccessDenied),
    /// The booking use case refused the request.
    #[error(transparent)]
    Booking(#[from] BookAppointmentError),
    /// The agenda query refused the request.
    #[error(transparent)]
    Agenda(#[from] AgendaError),
    /// Setting a practitioner's hours was refused.
    #[error(transparent)]
    Hours(#[from] SetHoursError),
    /// Something went wrong on the server's side. The details aren't shown to the client.
    #[error("internal server error")]
    Internal,
}

impl ApiError {
    /// The HTTP status this error is answered with.
    pub fn status(&self) -> StatusCode {
        match self {
            Self::MissingClinic | Self::InvalidClinic => StatusCode::BAD_REQUEST,
            Self::InvalidBody { status, .. }
            | Self::InvalidPath { status, .. }
            | Self::InvalidQuery { status, .. } => *status,
            Self::InvalidRange(_) | Self::InvalidTimeZone { .. } | Self::WeeklyHours(_) => {
                StatusCode::UNPROCESSABLE_ENTITY
            }
            Self::AccessDenied(_) => StatusCode::FORBIDDEN,
            Self::Booking(error) => match error {
                BookAppointmentError::Booking(BookingError::Overlaps { .. }) => {
                    StatusCode::CONFLICT
                }
                BookAppointmentError::UnknownPractitioner
                | BookAppointmentError::UnknownAppointmentType
                | BookAppointmentError::Booking(
                    BookingError::OutOfRange | BookingError::OutsideWorkingHours,
                ) => StatusCode::UNPROCESSABLE_ENTITY,
            },
            // The practitioner is the path, so a missing one is a missing resource. Booking names
            // the practitioner in its body instead, which is why it answers 422 above.
            Self::Agenda(AgendaError::UnknownPractitioner) => StatusCode::NOT_FOUND,
            Self::Hours(error) => match error {
                SetHoursError::OutOfRange => StatusCode::UNPROCESSABLE_ENTITY,
                SetHoursError::Hours(ReplaceHoursError::AppointmentOutsideHours { .. }) => {
                    StatusCode::CONFLICT
                }
            },
            Self::Internal => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    /// The stable code in the `error` field of the response body.
    pub fn code(&self) -> &'static str {
        match self {
            Self::MissingClinic => "missing_clinic_id",
            Self::InvalidClinic => "invalid_clinic_id",
            Self::InvalidBody { .. } => "invalid_body",
            Self::InvalidPath { .. } => "invalid_path",
            Self::InvalidQuery { .. } => "invalid_query",
            Self::InvalidRange(_) => "invalid_range",
            Self::InvalidTimeZone { .. } => "invalid_time_zone",
            Self::WeeklyHours(WeeklyHoursError::EndNotAfterStart) => "shift_end_not_after_start",
            Self::WeeklyHours(WeeklyHoursError::ShiftsNotSeparated { .. }) => {
                "shifts_not_separated"
            }
            Self::AccessDenied(_) => "access_denied",
            Self::Booking(error) => match error {
                BookAppointmentError::UnknownPractitioner => "unknown_practitioner",
                BookAppointmentError::UnknownAppointmentType => "unknown_appointment_type",
                BookAppointmentError::Booking(BookingError::OutOfRange) => "out_of_range",
                BookAppointmentError::Booking(BookingError::OutsideWorkingHours) => {
                    "outside_working_hours"
                }
                BookAppointmentError::Booking(BookingError::Overlaps { .. }) => "overlaps",
            },
            Self::Agenda(AgendaError::UnknownPractitioner) => "unknown_practitioner",
            Self::Hours(SetHoursError::OutOfRange) => "out_of_range",
            Self::Hours(SetHoursError::Hours(ReplaceHoursError::AppointmentOutsideHours {
                ..
            })) => "appointment_outside_hours",
            Self::Internal => "internal",
        }
    }
}

impl From<JsonRejection> for ApiError {
    fn from(rejection: JsonRejection) -> Self {
        Self::InvalidBody {
            status: rejection.status(),
            message: rejection.body_text(),
        }
    }
}

impl From<PathRejection> for ApiError {
    fn from(rejection: PathRejection) -> Self {
        Self::InvalidPath {
            status: rejection.status(),
            message: rejection.body_text(),
        }
    }
}

impl From<QueryRejection> for ApiError {
    fn from(rejection: QueryRejection) -> Self {
        Self::InvalidQuery {
            status: rejection.status(),
            message: rejection.body_text(),
        }
    }
}

/// The JSON body of every error response.
#[derive(Serialize)]
struct ErrorBody {
    error: &'static str,
    message: String,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let body = ErrorBody {
            error: self.code(),
            message: self.to_string(),
        };

        (self.status(), Json(body)).into_response()
    }
}
