mod common;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::response::IntoResponse;
use clinic_api::error::ApiError;
use clinic_api::state::AppState;
use clinic_app::agenda::AgendaError;
use clinic_app::booking::BookAppointmentError;
use clinic_app::hours::SetHoursError;
use clinic_app::in_memory::InMemoryStorage;
use clinic_app::ports::StorageError;
use clinic_domain::access::{ClinicalAccess, Role};
use clinic_domain::schedule::BookingError;
use clinic_domain::time_slot::{TimeSlot, TimeSlotError};
use serde_json::json;

use common::{app, at, booking, post, send};

#[tokio::test]
async fn a_valid_booking_is_created_and_returned() {
    let (status, body) = post(
        app().await,
        Some("1"),
        booking("2026-03-02T10:00:00Z").to_string(),
    )
    .await;

    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(
        body,
        json!({
            "practitioner_id": 1,
            "patient_id": 7,
            "appointment_type": "Consultation",
            "start": "2026-03-02T10:00:00Z",
            "end": "2026-03-02T10:30:00Z",
            "status": "scheduled",
        })
    );
}

#[tokio::test]
async fn a_booking_is_saved_so_the_same_slot_is_refused_next_time() {
    let app = app().await;
    let body = booking("2026-03-02T10:00:00Z").to_string();

    let (first, _) = post(app.clone(), Some("1"), body.clone()).await;
    let (second, error) = post(app, Some("1"), body).await;

    assert_eq!(first, StatusCode::CREATED);
    assert_eq!(second, StatusCode::CONFLICT);
    assert_eq!(error["error"], "overlaps");
}

#[tokio::test]
async fn a_start_with_an_offset_is_read_as_the_same_instant_in_utc() {
    let (status, body) = post(
        app().await,
        Some("1"),
        booking("2026-03-02T07:00:00-03:00").to_string(),
    )
    .await;

    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(body["start"], "2026-03-02T10:00:00Z");
}

#[tokio::test]
async fn a_missing_clinic_header_is_a_bad_request() {
    let (status, body) = post(
        app().await,
        None,
        booking("2026-03-02T10:00:00Z").to_string(),
    )
    .await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["error"], "missing_clinic_id");
}

#[tokio::test]
async fn a_clinic_header_that_is_not_a_whole_number_is_a_bad_request() {
    for clinic in ["abc", "-1", "1.5", ""] {
        let (status, body) = post(
            app().await,
            Some(clinic),
            booking("2026-03-02T10:00:00Z").to_string(),
        )
        .await;

        assert_eq!(status, StatusCode::BAD_REQUEST, "X-Clinic-Id: {clinic:?}");
        assert_eq!(
            body["error"], "invalid_clinic_id",
            "X-Clinic-Id: {clinic:?}"
        );
    }
}

#[tokio::test]
async fn another_clinic_cannot_book_into_this_clinics_practitioner() {
    let (status, body) = post(
        app().await,
        Some("2"),
        booking("2026-03-02T10:00:00Z").to_string(),
    )
    .await;

    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(body["error"], "unknown_practitioner");
}

#[tokio::test]
async fn an_unknown_appointment_type_is_unprocessable() {
    let mut body = booking("2026-03-02T10:00:00Z");
    body["appointment_type"] = json!("Check-up");

    let (status, body) = post(app().await, Some("1"), body.to_string()).await;

    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(body["error"], "unknown_appointment_type");
}

#[tokio::test]
async fn a_booking_outside_working_hours_is_unprocessable() {
    let (status, body) = post(
        app().await,
        Some("1"),
        booking("2026-03-02T11:45:00Z").to_string(),
    )
    .await;

    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(body["error"], "outside_working_hours");
}

#[tokio::test]
async fn an_overlapping_booking_is_a_conflict_with_a_readable_message() {
    let app = app().await;
    post(
        app.clone(),
        Some("1"),
        booking("2026-03-02T10:00:00Z").to_string(),
    )
    .await;

    let (status, body) = post(app, Some("1"), booking("2026-03-02T10:15:00Z").to_string()).await;

    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(
        body,
        json!({
            "error": "overlaps",
            "message": "appointment overlaps an existing appointment",
        })
    );
}

#[tokio::test]
async fn a_body_that_is_not_valid_json_is_rejected_as_invalid_body() {
    let (status, body) = post(app().await, Some("1"), "{not json".to_owned()).await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["error"], "invalid_body");
}

#[tokio::test]
async fn a_body_with_a_missing_wrong_or_unknown_field_is_rejected_as_invalid_body() {
    let missing = json!({ "practitioner_id": 1, "patient_id": 7, "start": "2026-03-02T10:00:00Z" });
    let mut wrong_type = booking("2026-03-02T10:00:00Z");
    wrong_type["patient_id"] = json!("seven");
    let mut unknown = booking("2026-03-02T10:00:00Z");
    unknown["room"] = json!(3);
    let not_a_time = booking("10:00");

    for body in [missing, wrong_type, unknown, not_a_time] {
        let (status, error) = post(app().await, Some("1"), body.to_string()).await;

        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "body: {body}");
        assert_eq!(error["error"], "invalid_body", "body: {body}");
        assert!(
            error["message"].as_str().is_some_and(|m| !m.is_empty()),
            "body: {body}"
        );
    }
}

#[tokio::test]
async fn a_body_without_a_json_content_type_is_unsupported() {
    let request = Request::builder()
        .method("POST")
        .uri("/appointments")
        .header("x-clinic-id", "1")
        .body(Body::from(booking("2026-03-02T10:00:00Z").to_string()))
        .unwrap();

    let (status, body) = send(app().await, request).await;

    assert_eq!(status, StatusCode::UNSUPPORTED_MEDIA_TYPE);
    assert_eq!(body["error"], "invalid_body");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn concurrent_bookings_of_the_same_slot_let_exactly_one_through() {
    let app = app().await;
    let body = booking("2026-03-02T10:00:00Z").to_string();

    let requests: Vec<_> = (0..20)
        .map(|_| tokio::spawn(post(app.clone(), Some("1"), body.clone())))
        .collect();
    let mut statuses = Vec::new();
    for request in requests {
        statuses.push(request.await.unwrap().0);
    }

    let created = statuses
        .iter()
        .filter(|status| **status == StatusCode::CREATED)
        .count();
    let conflicts = statuses
        .iter()
        .filter(|status| **status == StatusCode::CONFLICT)
        .count();
    assert_eq!((created, conflicts), (1, 19));
}

#[test]
fn every_api_error_has_its_documented_status_and_code() {
    let existing = TimeSlot::new(at(10, 0), at(10, 30)).unwrap();
    let denied = Role::Receptionist.grant::<ClinicalAccess>().unwrap_err();
    let cases = [
        (
            ApiError::MissingClinic,
            StatusCode::BAD_REQUEST,
            "missing_clinic_id",
        ),
        (
            ApiError::InvalidClinic,
            StatusCode::BAD_REQUEST,
            "invalid_clinic_id",
        ),
        (
            ApiError::from(BookAppointmentError::UnknownPractitioner),
            StatusCode::UNPROCESSABLE_ENTITY,
            "unknown_practitioner",
        ),
        (
            ApiError::from(BookAppointmentError::UnknownAppointmentType),
            StatusCode::UNPROCESSABLE_ENTITY,
            "unknown_appointment_type",
        ),
        (
            ApiError::from(BookAppointmentError::Booking(BookingError::OutOfRange)),
            StatusCode::UNPROCESSABLE_ENTITY,
            "out_of_range",
        ),
        (
            ApiError::from(BookAppointmentError::Booking(
                BookingError::OutsideWorkingHours,
            )),
            StatusCode::UNPROCESSABLE_ENTITY,
            "outside_working_hours",
        ),
        (
            ApiError::from(BookAppointmentError::Booking(BookingError::Overlaps {
                existing,
            })),
            StatusCode::CONFLICT,
            "overlaps",
        ),
        (
            ApiError::from(denied),
            StatusCode::FORBIDDEN,
            "access_denied",
        ),
        (
            ApiError::from(BookAppointmentError::from(StorageError::new("down"))),
            StatusCode::INTERNAL_SERVER_ERROR,
            "internal",
        ),
        (
            ApiError::from(AgendaError::from(StorageError::new("down"))),
            StatusCode::INTERNAL_SERVER_ERROR,
            "internal",
        ),
        (
            ApiError::from(SetHoursError::from(StorageError::new("down"))),
            StatusCode::INTERNAL_SERVER_ERROR,
            "internal",
        ),
        (
            ApiError::from(TimeSlotError::EndNotAfterStart),
            StatusCode::UNPROCESSABLE_ENTITY,
            "invalid_range",
        ),
        (
            ApiError::from(AgendaError::UnknownPractitioner),
            StatusCode::NOT_FOUND,
            "unknown_practitioner",
        ),
    ];

    for (error, status, code) in cases {
        assert_eq!(error.status(), status, "{error:?}");
        assert_eq!(error.code(), code, "{error:?}");
        assert_eq!(error.into_response().status(), status);
    }
}

#[test]
fn app_state_can_be_shared_between_tasks_and_threads() {
    fn assert_shareable<T: Clone + Send + Sync + 'static>() {}

    assert_shareable::<AppState<InMemoryStorage>>();
}
