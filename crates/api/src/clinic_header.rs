//! Which clinic a request is for, read from the `X-Clinic-Id` header.
//!
//! This is a stand-in until login arrives in M4. Then the clinic comes from the caller's token, and
//! handlers keep asking for a [`ClinicHeader`]-like extractor without changing.

use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use clinic_domain::ids::ClinicId;

use crate::error::ApiError;

/// The name of the header that names the clinic.
pub const CLINIC_HEADER: &str = "x-clinic-id";

/// An extractor for the clinic a request names in its `X-Clinic-Id` header.
///
/// The header must hold a whole number that fits in a `u64`, such as `1`. Otherwise the request is
/// refused before the handler runs, with [`ApiError::MissingClinic`] or
/// [`ApiError::InvalidClinic`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClinicHeader(pub ClinicId);

impl<S> FromRequestParts<S> for ClinicHeader
where
    S: Send + Sync,
{
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let value = parts
            .headers
            .get(CLINIC_HEADER)
            .ok_or(ApiError::MissingClinic)?;
        let id = value
            .to_str()
            .ok()
            .and_then(|text| text.parse::<u64>().ok())
            .ok_or(ApiError::InvalidClinic)?;

        Ok(Self(ClinicId::new(id)))
    }
}
