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
use axum::extract::rejection::JsonRejection;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use clinic_app::booking::BookAppointmentError;
use clinic_domain::access::AccessDenied;
use clinic_domain::schedule::BookingError;
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
    /// The caller's role may not do this.
    #[error(transparent)]
    AccessDenied(#[from] AccessDenied),
    /// The booking use case refused the request.
    #[error(transparent)]
    Booking(#[from] BookAppointmentError),
    /// Something went wrong on the server's side. The details aren't shown to the client.
    #[error("internal server error")]
    Internal,
}

impl ApiError {
    /// The HTTP status this error is answered with.
    pub fn status(&self) -> StatusCode {
        match self {
            Self::MissingClinic | Self::InvalidClinic => StatusCode::BAD_REQUEST,
            Self::InvalidBody { status, .. } => *status,
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
            Self::Internal => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    /// The stable code in the `error` field of the response body.
    pub fn code(&self) -> &'static str {
        match self {
            Self::MissingClinic => "missing_clinic_id",
            Self::InvalidClinic => "invalid_clinic_id",
            Self::InvalidBody { .. } => "invalid_body",
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
