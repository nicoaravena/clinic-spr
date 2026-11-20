mod common;

use axum::Router;
use axum::http::StatusCode;
use serde_json::{Value, json};

use common::{app, booking, get, post};

/// The app from `common`, with practitioner 1 at clinic 1 booked at 11:00 and 09:00, in that
/// order, so the agenda has to sort them.
async fn booked_app() -> Router {
    let app = app().await;
    for start in ["2026-03-02T11:00:00Z", "2026-03-02T09:00:00Z"] {
        let (status, body) = post(app.clone(), Some("1"), booking(start).to_string()).await;
        assert_eq!(status, StatusCode::CREATED, "seeding {start}: {body}");
    }
    app
}

fn agenda_uri(practitioner: &str, from: &str, to: &str) -> String {
    format!("/practitioners/{practitioner}/agenda?from={from}&to={to}")
}

fn appointment(start: &str, end: &str) -> Value {
    json!({
        "practitioner_id": 1,
        "patient_id": 7,
        "appointment_type": "Consultation",
        "start": start,
        "end": end,
        "status": "scheduled",
    })
}

#[tokio::test]
async fn the_agenda_lists_overlapping_appointments_sorted_by_start() {
    let uri = agenda_uri("1", "2026-03-02T09:00:00Z", "2026-03-02T12:00:00Z");

    let (status, body) = get(booked_app().await, Some("1"), &uri).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        body,
        json!({
            "practitioner_id": 1,
            "from": "2026-03-02T09:00:00Z",
            "to": "2026-03-02T12:00:00Z",
            "appointments": [
                appointment("2026-03-02T09:00:00Z", "2026-03-02T09:30:00Z"),
                appointment("2026-03-02T11:00:00Z", "2026-03-02T11:30:00Z"),
            ],
        })
    );
}

#[tokio::test]
async fn an_appointment_that_only_partly_overlaps_the_range_is_included() {
    let uri = agenda_uri("1", "2026-03-02T09:15:00Z", "2026-03-02T10:00:00Z");

    let (status, body) = get(booked_app().await, Some("1"), &uri).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        body["appointments"],
        json!([appointment("2026-03-02T09:00:00Z", "2026-03-02T09:30:00Z")])
    );
}

#[tokio::test]
async fn a_range_without_appointments_is_an_empty_list() {
    let uri = agenda_uri("1", "2026-03-02T09:30:00Z", "2026-03-02T11:00:00Z");

    let (status, body) = get(booked_app().await, Some("1"), &uri).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["appointments"], json!([]));
}

#[tokio::test]
async fn a_range_with_offsets_is_read_as_the_same_instants_in_utc() {
    // `+` means a space in a query string, so a positive offset must be sent as `%2B`.
    let uri = agenda_uri(
        "1",
        "2026-03-02T06:00:00-03:00",
        "2026-03-02T11:00:00%2B01:00",
    );

    let (status, body) = get(booked_app().await, Some("1"), &uri).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["from"], "2026-03-02T09:00:00Z");
    assert_eq!(body["to"], "2026-03-02T10:00:00Z");
    assert_eq!(
        body["appointments"],
        json!([appointment("2026-03-02T09:00:00Z", "2026-03-02T09:30:00Z")])
    );
}

#[tokio::test]
async fn another_clinic_cannot_read_this_clinics_agenda() {
    let uri = agenda_uri("1", "2026-03-02T09:00:00Z", "2026-03-02T12:00:00Z");

    let (status, body) = get(booked_app().await, Some("2"), &uri).await;

    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body["error"], "unknown_practitioner");
}

#[tokio::test]
async fn an_unknown_practitioner_is_not_found() {
    let uri = agenda_uri("9", "2026-03-02T09:00:00Z", "2026-03-02T12:00:00Z");

    let (status, body) = get(app().await, Some("1"), &uri).await;

    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body["error"], "unknown_practitioner");
}

#[tokio::test]
async fn a_missing_or_invalid_clinic_header_is_a_bad_request() {
    let uri = agenda_uri("1", "2026-03-02T09:00:00Z", "2026-03-02T12:00:00Z");

    let (missing, missing_body) = get(app().await, None, &uri).await;
    let (invalid, invalid_body) = get(app().await, Some("abc"), &uri).await;

    assert_eq!(missing, StatusCode::BAD_REQUEST);
    assert_eq!(missing_body["error"], "missing_clinic_id");
    assert_eq!(invalid, StatusCode::BAD_REQUEST);
    assert_eq!(invalid_body["error"], "invalid_clinic_id");
}

#[tokio::test]
async fn a_practitioner_id_that_is_not_a_whole_number_is_a_bad_request() {
    for practitioner in ["abc", "-1", "1.5"] {
        let uri = agenda_uri(practitioner, "2026-03-02T09:00:00Z", "2026-03-02T12:00:00Z");

        let (status, body) = get(app().await, Some("1"), &uri).await;

        assert_eq!(status, StatusCode::BAD_REQUEST, "id: {practitioner:?}");
        assert_eq!(body["error"], "invalid_path", "id: {practitioner:?}");
        assert!(
            body["message"].as_str().is_some_and(|m| !m.is_empty()),
            "id: {practitioner:?}"
        );
    }
}

#[tokio::test]
async fn a_missing_mistyped_or_unknown_query_parameter_is_a_bad_request() {
    let uris = [
        "/practitioners/1/agenda".to_owned(),
        "/practitioners/1/agenda?from=2026-03-02T09:00:00Z".to_owned(),
        agenda_uri("1", "10:00", "2026-03-02T12:00:00Z"),
        format!(
            "{}&room=3",
            agenda_uri("1", "2026-03-02T09:00:00Z", "2026-03-02T12:00:00Z")
        ),
    ];

    for uri in uris {
        let (status, body) = get(app().await, Some("1"), &uri).await;

        assert_eq!(status, StatusCode::BAD_REQUEST, "uri: {uri}");
        assert_eq!(body["error"], "invalid_query", "uri: {uri}");
        assert!(
            body["message"].as_str().is_some_and(|m| !m.is_empty()),
            "uri: {uri}"
        );
    }
}

#[tokio::test]
async fn a_range_that_does_not_end_after_it_starts_is_unprocessable() {
    for (from, to) in [
        ("2026-03-02T12:00:00Z", "2026-03-02T09:00:00Z"),
        ("2026-03-02T09:00:00Z", "2026-03-02T09:00:00Z"),
    ] {
        let (status, body) = get(app().await, Some("1"), &agenda_uri("1", from, to)).await;

        assert_eq!(
            status,
            StatusCode::UNPROCESSABLE_ENTITY,
            "from {from} to {to}"
        );
        assert_eq!(body["error"], "invalid_range", "from {from} to {to}");
    }
}
