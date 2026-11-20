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
//! | `GET /practitioners/{id}/agenda` | [`practitioners::agenda`] |
//! | `PUT /practitioners/{id}/hours` | [`practitioners::set_hours`] |
//!
//! Every route needs an `X-Clinic-Id` header naming the clinic, until login arrives in M4. See
//! [`clinic_header`].
//!
//! The router, its state and its handlers are generic over the [`Storage`], so this crate doesn't
//! depend on which one the binary picks.

use axum::Router;
use axum::routing::{get, post, put};
use clinic_app::ports::Storage;
use clinic_domain::access::Role;

pub mod appointments;
pub mod clinic_header;
pub mod error;
pub mod practitioners;
pub mod state;

use state::AppState;

/// Who every request acts as until login arrives in M4.
///
/// Every role may manage schedules today, so this changes no outcome. In M4 the caller's role
/// comes from their token instead.
const STAND_IN_CALLER: Role = Role::Receptionist;

/// Every route of the API, sharing `state`.
///
/// Each handler is named with the storage type, as in `appointments::book::<St>`, because a
/// generic function has to be given its type parameters before it can be used as a value.
pub fn router<St>(state: AppState<St>) -> Router
where
    St: Storage + Send + Sync + 'static,
{
    Router::new()
        .route("/appointments", post(appointments::book::<St>))
        .route(
            "/practitioners/{id}/agenda",
            get(practitioners::agenda::<St>),
        )
        .route(
            "/practitioners/{id}/hours",
            put(practitioners::set_hours::<St>),
        )
        .with_state(state)
}
