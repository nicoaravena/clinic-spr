# api (crate)

The HTTP JSON API, as a library: an axum `Router` and its handlers.

- **Source:** `crates/api/src/`
- **Introduced in:** [step 2.4](../steps/2.4-http-api.md); the agenda
  route in [step 2.5](../steps/2.5-serving-http.md); the hours route in
  [step 2.6](../steps/2.6-weekly-hours-endpoint.md); generic over the
  storage adapters, with storage failures as 500, in
  [step 3.1](../steps/3.1-async-ports.md); generic over one `Storage`,
  without a lock, in [step 3.2](../steps/3.2-tenant-sessions.md)
- **Served by:** the [`clinic` binary](clinic.md)
- **API docs:** [`clinic_api`][api]

## Modules and public items

| Module | Items |
|---|---|
| crate root | `router::<St>` |
| `state` | `AppState<St>` (`new`) |
| `clinic_header` | `ClinicHeader`, `CLINIC_HEADER` |
| `appointments` | `book` (handler), `BookAppointmentBody`, `AppointmentBody` (`new`) |
| `practitioners` | `agenda` (handler), `AgendaParams`, `AgendaBody`; `set_hours` (handler), `SetHoursBody`, `WeeklyShiftBody`, `HoursBody`, `ShiftBody` |
| `error` | `ApiError` (`status`, `code`; `MissingClinic`, `InvalidClinic`, `InvalidBody`, `InvalidPath`, `InvalidQuery`, `InvalidRange`, `InvalidTimeZone`, `WeeklyHours`, `AccessDenied`, `Booking`, `Agenda`, `Hours`) |

## Routes

### `POST /appointments`

Books an appointment through `book_appointment`.

Headers:

- `X-Clinic-Id: <u64>`: the clinic to act for. Required until login
  arrives in M4.
- `Content-Type: application/json`.

Body (unknown fields are refused):

```json
{
  "practitioner_id": 1,
  "patient_id": 7,
  "appointment_type": "Consultation",
  "start": "2026-03-02T10:00:00Z"
}
```

`start` is RFC 3339 with any offset. It's converted to UTC.

Answer: `201 Created` with the appointment. Times are in UTC.

```json
{
  "practitioner_id": 1,
  "patient_id": 7,
  "appointment_type": "Consultation",
  "start": "2026-03-02T10:00:00Z",
  "end": "2026-03-02T10:30:00Z",
  "status": "scheduled"
}
```

`status` is one of `scheduled`, `completed`, `cancelled` and `no_show`.

### `GET /practitioners/{id}/agenda`

Reads a practitioner's agenda through `practitioner_agenda`.

- `{id}`: the practitioner, a whole number that fits in a `u64`.
- Headers: `X-Clinic-Id: <u64>`, as above.
- Query string: `from` and `to`, both required, RFC 3339 with any offset.
  Unknown parameters are refused. `to` must be after `from`. In a query
  string `+` means a space, so send a positive offset as `%2B`, as in
  `from=2026-03-02T10:00:00%2B01:00`.

```sh
curl -s 'localhost:3000/practitioners/1/agenda?from=2026-03-02T09:00:00Z&to=2026-03-02T17:00:00Z' \
  -H 'X-Clinic-Id: 1'
```

Answer: `200 OK` with the appointments that overlap the range, sorted by
start, each in the same shape `POST /appointments` answers with. An empty
range is an empty list, not an error. `from` and `to` are echoed in UTC.

```json
{
  "practitioner_id": 1,
  "from": "2026-03-02T09:00:00Z",
  "to": "2026-03-02T17:00:00Z",
  "appointments": [
    {
      "practitioner_id": 1,
      "patient_id": 1,
      "appointment_type": "Consultation",
      "start": "2026-03-02T10:00:00Z",
      "end": "2026-03-02T10:30:00Z",
      "status": "scheduled"
    }
  ]
}
```

### `PUT /practitioners/{id}/hours`

Sets a practitioner's weekly hours through `set_weekly_hours`.

- `{id}`: the practitioner, a whole number that fits in a `u64`.
- Headers: `X-Clinic-Id: <u64>` and `Content-Type: application/json`.

Body (unknown fields are refused, in the body and in each shift):

```json
{
  "time_zone": "America/Santiago",
  "from": "2026-03-30",
  "shifts": [{ "weekday": "Mon", "start": "09:00", "end": "13:00" }]
}
```

- `time_zone`: an IANA name, matched exactly (`america/santiago` is
  refused).
- `from`: the first local date the hours apply to, `YYYY-MM-DD`.
- `shifts`: may be empty. `weekday` is an English name, short or full, in
  any case. `start` and `end` are local times, `HH:MM` or `HH:MM:SS`, on
  the same day; `end` must be after `start`. Shifts on the same weekday
  must not overlap or touch.

The template is expanded over the 28 local dates from `from` and replaces
the practitioner's hours from the start of `from` on. Earlier shifts and
appointments are kept.

Answer: `201 Created` if the practitioner had no schedule at the clinic,
`200 OK` if their hours were replaced. Sending the same body again gives
the same answer with `200` and changes nothing. `shifts` holds only the
new UTC shifts, for the dates `[from, until)`.

```json
{
  "practitioner_id": 2,
  "time_zone": "America/Santiago",
  "from": "2026-03-30",
  "until": "2026-04-27",
  "shifts": [
    { "start": "2026-03-30T12:00:00Z", "end": "2026-03-30T16:00:00Z" },
    { "start": "2026-04-06T13:00:00Z", "end": "2026-04-06T17:00:00Z" },
    { "start": "2026-04-13T13:00:00Z", "end": "2026-04-13T17:00:00Z" },
    { "start": "2026-04-20T13:00:00Z", "end": "2026-04-20T17:00:00Z" }
  ]
}
```

## Errors

Every error has the same body. `error` is stable and meant for programs;
`message` is meant for people and may change.

```json
{ "error": "overlaps", "message": "appointment overlaps an existing appointment" }
```

| Cause | Status | `error` |
|---|---|---|
| No `X-Clinic-Id` header | 400 | `missing_clinic_id` |
| `X-Clinic-Id` isn't a whole number that fits in a `u64` | 400 | `invalid_clinic_id` |
| Malformed JSON | 400 | `invalid_body` |
| No JSON content type | 415 | `invalid_body` |
| Missing, mistyped or unknown field | 422 | `invalid_body` |
| A path parameter doesn't parse, such as `/practitioners/abc/agenda` | 400 | `invalid_path` |
| A query parameter is missing, mistyped or unknown | 400 | `invalid_query` |
| `to` isn't after `from` | 422 | `invalid_range` |
| `time_zone` isn't an IANA name | 422 | `invalid_time_zone` |
| A weekly shift doesn't end after it starts | 422 | `shift_end_not_after_start` |
| Two weekly shifts on one weekday overlap or touch | 422 | `shifts_not_separated` |
| The hours' `from` is too close to chrono's earliest or latest date | 422 | `out_of_range` |
| New hours would leave a booked appointment outside them | 409 | `appointment_outside_hours` |
| The caller's role may not do this (not reachable before M4) | 403 | `access_denied` |
| No schedule for that practitioner at this clinic, named in a body | 422 | `unknown_practitioner` |
| No schedule for that practitioner at this clinic, named in the path | 404 | `unknown_practitioner` |
| The clinic offers no appointment type with that name | 422 | `unknown_appointment_type` |
| The appointment would end after chrono's latest date | 422 | `out_of_range` |
| Not inside a single shift of the working hours | 422 | `outside_working_hours` |
| Overlaps an appointment already booked | 409 | `overlaps` |
| Storage failed (never with the in-memory adapters) | 500 | `internal` |

Every 500 has the message `internal server error`, whatever the cause. The
cause, with its chain of sources, is written to stderr as one line, such as
`clinic-api: 500 Internal Server Error: storage failed: connection refused`.
It can name hosts or tables, so it isn't sent to the client.

## Storage

`router`, `AppState` and the handlers are generic over `St: Storage + Send
+ Sync + 'static`. The `clinic` binary picks the type. It's a type
parameter rather than a trait object because `Storage` has an associated
type and async methods, neither of which works through `dyn`.

`AppState` holds the storage in an `Arc`, without a lock. Each handler
passes `state.storage()` to its use case, which begins a session, and the
storage decides how sessions are kept apart: the in-memory one runs them one
at a time (see [app](app.md#sessions)).

## Rules and how they are enforced

| Rule | How |
|---|---|
| Every request names its clinic | Every handler takes a `ClinicHeader`, which refuses the request before the handler runs |
| One clinic can't book into or read another's schedules | The use case begins its session at the clinic from the header; tested over HTTP for both routes |
| Rules stay in the domain and `app` | `book` calls `book_appointment`, `agenda` calls `practitioner_agenda` and `set_hours` calls `set_weekly_hours`, and each maps the errors; none checks a rule itself |
| Text is parsed into types at the edge | `set_hours` parses the time zone name into a `Tz`, and serde reads dates, times and weekdays, before any use case runs |
| A read never sees a half-saved schedule | A session's changes are seen by others only after `commit`, and the in-memory sessions run one at a time |
| Two requests can't book the same slot | `book_appointment` reads, books and saves in one session, which holds the in-memory storage's lock throughout; tested with 20 concurrent requests |
| A storage failure doesn't leak details | Every 5xx answers the fixed message `internal server error` and logs the cause to stderr; tested over HTTP for every route in `tests/storage_errors.rs` |
| Every refusal has a deliberate status | `ApiError::status` matches every `BookAppointmentError`, `AgendaError`, `WeeklyHoursError` and `SetHoursError` variant without a wildcard, `Storage` included, so a new one doesn't compile until it's mapped |
| The domain doesn't depend on serde | JSON shapes are separate DTOs in `appointments` and `practitioners` |

[api]: ../../../target/doc/clinic_api/index.html
