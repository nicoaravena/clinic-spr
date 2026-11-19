//! The clinic platform's HTTP JSON API, as a library.
//!
//! [`router`] builds an axum [`Router`] with every route. This crate doesn't open a socket or
//! start a runtime: the `clinic` binary does that, and tests send requests to the router directly.
//!
//! Handlers are thin adapters. Each one reads the request, calls a use case from `clinic_app`, and
//! turns the result or the error into a response. [`error::ApiError`] decides the status codes.
//!
//! Routes:
//!
//! | Method and path | Handler |
//! |---|---|
//! | `POST /appointments` | [`appointments::book`] |
//!
//! Every route needs an `X-Clinic-Id` header naming the clinic, until login arrives in M4. See
//! [`clinic_header`].

use axum::Router;
use axum::routing::post;

pub mod appointments;
pub mod clinic_header;
pub mod error;
pub mod state;

use state::AppState;

/// Every route of the API, sharing `state`.
pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/appointments", post(appointments::book))
        .with_state(state)
}
