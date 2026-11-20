mod common;

use axum::http::StatusCode;
use serde_json::{Value, json};

use common::{app, booking, get, post, put};

fn hours_uri(practitioner: &str) -> String {
    format!("/practitioners/{practitioner}/hours")
}

/// Mondays 09:00-13:00 in Santiago from 2026-03-30. Santiago is at -03:00 until its clocks go
/// back on 2026-04-04, and at -04:00 after.
fn santiago_mondays() -> Value {
    json!({
        "time_zone": "America/Santiago",
        "from": "2026-03-30",
        "shifts": [{ "weekday": "Mon", "start": "09:00", "end": "13:00" }],
    })
}

fn shift(start: &str, end: &str) -> Value {
    json!({ "start": start, "end": end })
}

/// Sends `PUT /practitioners/{practitioner}/hours` at clinic 1 and returns the status and body.
async fn put_hours(app: axum::Router, practitioner: &str, body: Value) -> (StatusCode, Value) {
    put(app, Some("1"), &hours_uri(practitioner), body.to_string()).await
}

/// Asserts that `body` is refused with `status` and the error `code`.
async fn assert_refused(body: Value, status: StatusCode, code: &str) {
    let (actual, response) = put_hours(app(), "2", body).await;

    assert_eq!(actual, status, "{response}");
    assert_eq!(response["error"], code, "{response}");
}

#[tokio::test]
async fn hours_for_a_practitioner_without_a_schedule_are_created() {
    let (status, body) = put_hours(app(), "2", santiago_mondays()).await;

    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(
        body,
        json!({
            "practitioner_id": 2,
            "time_zone": "America/Santiago",
            "from": "2026-03-30",
            "until": "2026-04-27",
            "shifts": [
                shift("2026-03-30T12:00:00Z", "2026-03-30T16:00:00Z"),
                shift("2026-04-06T13:00:00Z", "2026-04-06T17:00:00Z"),
                shift("2026-04-13T13:00:00Z", "2026-04-13T17:00:00Z"),
                shift("2026-04-20T13:00:00Z", "2026-04-20T17:00:00Z"),
            ],
        })
    );
}

#[tokio::test]
async fn the_same_put_again_is_200_with_the_same_body() {
    let app = app();
    let (_, first) = put_hours(app.clone(), "2", santiago_mondays()).await;

    let (status, second) = put_hours(app, "2", santiago_mondays()).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(second, first);
}

#[tokio::test]
async fn the_new_hours_take_bookings() {
    let app = app();
    put_hours(app.clone(), "2", santiago_mondays()).await;
    let mut body = booking("2026-04-06T09:00:00-04:00");
    body["practitioner_id"] = json!(2);

    let (status, appointment) = post(app, Some("1"), body.to_string()).await;

    assert_eq!(status, StatusCode::CREATED, "{appointment}");
    assert_eq!(appointment["start"], "2026-04-06T13:00:00Z");
}

#[tokio::test]
async fn shifts_before_from_and_their_appointments_are_kept() {
    let app = app();
    let (status, _) = post(
        app.clone(),
        Some("1"),
        booking("2026-03-02T09:00:00Z").to_string(),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);

    let (status, _) = put_hours(
        app.clone(),
        "1",
        json!({
            "time_zone": "UTC",
            "from": "2026-03-03",
            "shifts": [{ "weekday": "Mon", "start": "14:00", "end": "18:00" }],
        }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    // 2026-03-02 still has its 09:00-12:00 shift, so 10:00 can still be booked.
    let (status, body) = post(
        app.clone(),
        Some("1"),
        booking("2026-03-02T10:00:00Z").to_string(),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "{body}");
    let uri = "/practitioners/1/agenda?from=2026-03-02T00:00:00Z&to=2026-03-03T00:00:00Z";
    let (_, agenda) = get(app, Some("1"), uri).await;
    assert_eq!(agenda["appointments"].as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn hours_that_leave_a_booked_appointment_outside_are_409() {
    let app = app();
    post(
        app.clone(),
        Some("1"),
        booking("2026-03-02T09:00:00Z").to_string(),
    )
    .await;

    let (status, body) = put_hours(
        app,
        "1",
        json!({
            "time_zone": "UTC",
            "from": "2026-03-02",
            "shifts": [{ "weekday": "Mon", "start": "14:00", "end": "18:00" }],
        }),
    )
    .await;

    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(
        body,
        json!({
            "error": "appointment_outside_hours",
            "message": "an existing appointment would be outside the new working hours",
        })
    );
}

#[tokio::test]
async fn no_shifts_is_allowed_and_gives_no_hours() {
    let mut body = santiago_mondays();
    body["shifts"] = json!([]);

    let (status, response) = put_hours(app(), "2", body).await;

    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(response["shifts"], json!([]));
}

#[tokio::test]
async fn weekday_names_are_read_in_any_case_and_full() {
    let mut body = santiago_mondays();
    body["shifts"][0]["weekday"] = json!("monday");

    let (status, response) = put_hours(app(), "2", body).await;

    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(response["shifts"].as_array().unwrap().len(), 4);
}

#[tokio::test]
async fn an_unknown_time_zone_is_422() {
    let mut body = santiago_mondays();
    body["time_zone"] = json!("Mars/Olympus");

    let (status, response) = put_hours(app(), "2", body).await;

    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(
        response,
        json!({
            "error": "invalid_time_zone",
            "message": "\"Mars/Olympus\" isn't a time zone name such as \"America/Santiago\"",
        })
    );
}

#[tokio::test]
async fn time_zone_names_are_matched_exactly() {
    let mut body = santiago_mondays();
    body["time_zone"] = json!("america/santiago");

    assert_refused(body, StatusCode::UNPROCESSABLE_ENTITY, "invalid_time_zone").await;
}

#[tokio::test]
async fn a_shift_that_does_not_end_after_it_starts_is_422() {
    let mut body = santiago_mondays();
    body["shifts"][0]["end"] = json!("09:00");

    assert_refused(
        body,
        StatusCode::UNPROCESSABLE_ENTITY,
        "shift_end_not_after_start",
    )
    .await;
}

#[tokio::test]
async fn overlapping_shifts_on_one_weekday_are_422() {
    let mut body = santiago_mondays();
    body["shifts"] = json!([
        { "weekday": "Mon", "start": "09:00", "end": "13:00" },
        { "weekday": "Mon", "start": "12:00", "end": "17:00" },
    ]);

    assert_refused(
        body,
        StatusCode::UNPROCESSABLE_ENTITY,
        "shifts_not_separated",
    )
    .await;
}

#[tokio::test]
async fn a_from_near_the_end_of_the_calendar_is_422() {
    let mut body = santiago_mondays();
    body["from"] = json!("+262142-12-20");

    assert_refused(body, StatusCode::UNPROCESSABLE_ENTITY, "out_of_range").await;
}

#[tokio::test]
async fn a_malformed_field_is_422_invalid_body() {
    let cases = [
        ("weekday", json!("Funday")),
        ("start", json!("9am")),
        ("end", json!("25:00")),
    ];
    for (field, value) in cases {
        let mut body = santiago_mondays();
        body["shifts"][0][field] = value;

        assert_refused(body, StatusCode::UNPROCESSABLE_ENTITY, "invalid_body").await;
    }

    let mut body = santiago_mondays();
    body["from"] = json!("30/03/2026");
    assert_refused(body, StatusCode::UNPROCESSABLE_ENTITY, "invalid_body").await;
}

#[tokio::test]
async fn missing_and_unknown_fields_are_422_invalid_body() {
    for field in ["time_zone", "from", "shifts"] {
        let mut body = santiago_mondays();
        body.as_object_mut().unwrap().remove(field);

        assert_refused(body, StatusCode::UNPROCESSABLE_ENTITY, "invalid_body").await;
    }

    let mut body = santiago_mondays();
    body["until"] = json!("2026-05-01");
    assert_refused(body, StatusCode::UNPROCESSABLE_ENTITY, "invalid_body").await;

    let mut body = santiago_mondays();
    body["shifts"][0]["note"] = json!("lunch");
    assert_refused(body, StatusCode::UNPROCESSABLE_ENTITY, "invalid_body").await;
}

#[tokio::test]
async fn an_id_that_is_not_a_whole_number_is_400() {
    for id in ["abc", "-1", "1.5"] {
        let (status, body) = put_hours(app(), id, santiago_mondays()).await;

        assert_eq!(status, StatusCode::BAD_REQUEST, "{id}");
        assert_eq!(body["error"], "invalid_path", "{id}");
    }
}

#[tokio::test]
async fn a_missing_clinic_header_is_400() {
    let (status, body) = put(app(), None, &hours_uri("2"), santiago_mondays().to_string()).await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["error"], "missing_clinic_id");
}

#[tokio::test]
async fn hours_set_at_one_clinic_do_not_touch_another() {
    let app = app();

    let (status, _) = put(
        app.clone(),
        Some("2"),
        &hours_uri("1"),
        json!({
            "time_zone": "UTC",
            "from": "2026-03-02",
            "shifts": [],
        })
        .to_string(),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);

    // Clinic 1's practitioner 1 still works 09:00-12:00 on 2026-03-02.
    let (status, body) = post(app, Some("1"), booking("2026-03-02T09:00:00Z").to_string()).await;
    assert_eq!(status, StatusCode::CREATED, "{body}");
}
