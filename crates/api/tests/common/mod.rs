//! Helpers shared by the API's integration tests.

// Each test file is its own crate and uses a different subset of these helpers.
#![allow(dead_code)]

use axum::Router;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use chrono::{DateTime, TimeDelta, TimeZone, Utc};
use clinic_api::router;
use clinic_api::state::AppState;
use clinic_app::in_memory::{InMemoryAppointmentTypes, InMemorySchedules};
use clinic_app::ports::ScheduleRepository;
use clinic_domain::appointment::AppointmentType;
use clinic_domain::ids::{ClinicId, PractitionerId};
use clinic_domain::schedule::Schedule;
use clinic_domain::time_slot::TimeSlot;
use clinic_domain::working_hours::WorkingHours;
use http_body_util::BodyExt;
use serde_json::{Value, json};
use tower::ServiceExt;

pub(crate) fn at(hour: u32, minute: u32) -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 3, 2, hour, minute, 0).unwrap()
}

/// Clinic 1: practitioner 1 works 09:00-12:00 UTC on 2026-03-02 and the clinic offers a 30-minute
/// "Consultation". Clinic 2 offers "Consultation" too but has no schedules.
pub(crate) fn app() -> Router {
    let mut schedules = InMemorySchedules::new();
    let hours = WorkingHours::new(vec![TimeSlot::new(at(9, 0), at(12, 0)).unwrap()]).unwrap();
    schedules.save(
        ClinicId::new(1),
        PractitionerId::new(1),
        Schedule::new(hours),
    );

    let mut types = InMemoryAppointmentTypes::new();
    let consultation = AppointmentType::new("Consultation", TimeDelta::minutes(30)).unwrap();
    types.add(ClinicId::new(1), consultation.clone()).unwrap();
    types.add(ClinicId::new(2), consultation).unwrap();

    router(AppState::new(schedules, types))
}

/// The body of `POST /appointments` for a Consultation with practitioner 1 for patient 7.
pub(crate) fn booking(start: &str) -> Value {
    json!({
        "practitioner_id": 1,
        "patient_id": 7,
        "appointment_type": "Consultation",
        "start": start,
    })
}

/// Sends `POST /appointments` with a JSON content type and returns the status and JSON body.
pub(crate) async fn post(app: Router, clinic: Option<&str>, body: String) -> (StatusCode, Value) {
    let mut request = Request::builder()
        .method("POST")
        .uri("/appointments")
        .header("content-type", "application/json");
    if let Some(clinic) = clinic {
        request = request.header("x-clinic-id", clinic);
    }
    send(app, request.body(Body::from(body)).unwrap()).await
}

/// Sends `GET uri` and returns the status and JSON body.
pub(crate) async fn get(app: Router, clinic: Option<&str>, uri: &str) -> (StatusCode, Value) {
    let mut request = Request::builder().method("GET").uri(uri);
    if let Some(clinic) = clinic {
        request = request.header("x-clinic-id", clinic);
    }
    send(app, request.body(Body::empty()).unwrap()).await
}

/// Sends `request` to `app` and returns the status and the body parsed as JSON.
pub(crate) async fn send(app: Router, request: Request<Body>) -> (StatusCode, Value) {
    let response = app.oneshot(request).await.unwrap();
    let status = response.status();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let body = serde_json::from_slice(&bytes).unwrap_or_else(|error| {
        panic!(
            "body should be JSON ({error}): {}",
            String::from_utf8_lossy(&bytes)
        )
    });
    (status, body)
}
