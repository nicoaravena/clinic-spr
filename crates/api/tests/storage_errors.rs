mod common;

use axum::Router;
use axum::http::StatusCode;
use clinic_api::router;
use clinic_api::state::AppState;
use clinic_app::ports::{AppointmentTypes, ScheduleRepository, StorageError};
use clinic_domain::appointment::AppointmentType;
use clinic_domain::ids::{ClinicId, PractitionerId};
use clinic_domain::schedule::Schedule;
use serde_json::{Value, json};

use common::{booking, get, post, put};

/// The cause every call fails with. It must never reach a client.
const CAUSE: &str = "connection refused by db.internal:5432";

/// Storage that can't be reached: every call fails.
struct Unreachable;

impl ScheduleRepository for Unreachable {
    async fn get(
        &self,
        _clinic: ClinicId,
        _practitioner: PractitionerId,
    ) -> Result<Option<Schedule>, StorageError> {
        Err(StorageError::new(CAUSE))
    }

    async fn save(
        &mut self,
        _clinic: ClinicId,
        _practitioner: PractitionerId,
        _schedule: Schedule,
    ) -> Result<(), StorageError> {
        Err(StorageError::new(CAUSE))
    }
}

impl AppointmentTypes for Unreachable {
    async fn find(
        &self,
        _clinic: ClinicId,
        _name: &str,
    ) -> Result<Option<AppointmentType>, StorageError> {
        Err(StorageError::new(CAUSE))
    }
}

fn unreachable_app() -> Router {
    router(AppState::new(Unreachable, Unreachable))
}

/// A 500 with the same body for every storage failure, and nothing about the cause.
fn assert_internal(status: StatusCode, body: &Value) {
    assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
    assert_eq!(
        body,
        &json!({ "error": "internal", "message": "internal server error" })
    );
    assert!(!body.to_string().contains("connection refused"));
}

#[tokio::test]
async fn booking_answers_500_when_storage_fails() {
    let (status, body) = post(
        unreachable_app(),
        Some("1"),
        booking("2026-03-02T10:00:00Z").to_string(),
    )
    .await;

    assert_internal(status, &body);
}

#[tokio::test]
async fn the_agenda_answers_500_when_storage_fails() {
    let (status, body) = get(
        unreachable_app(),
        Some("1"),
        "/practitioners/1/agenda?from=2026-03-02T09:00:00Z&to=2026-03-02T12:00:00Z",
    )
    .await;

    assert_internal(status, &body);
}

#[tokio::test]
async fn setting_hours_answers_500_when_storage_fails() {
    let body = json!({
        "time_zone": "America/Santiago",
        "from": "2026-03-30",
        "shifts": [{ "weekday": "Mon", "start": "09:00", "end": "13:00" }],
    });

    let (status, body) = put(
        unreachable_app(),
        Some("1"),
        "/practitioners/1/hours",
        body.to_string(),
    )
    .await;

    assert_internal(status, &body);
}

#[tokio::test]
async fn a_bad_request_is_still_refused_before_storage_is_touched() {
    let (status, body) = post(unreachable_app(), None, "{}".to_owned()).await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["error"], "missing_clinic_id");
}
