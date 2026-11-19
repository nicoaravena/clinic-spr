# api (crate)

The HTTP JSON API, as a library: an axum `Router` and its handlers.

- **Source:** `crates/api/src/`
- **Introduced in:** [step 2.4](../steps/2.4-http-api.md)
- **API docs:** [`clinic_api`][api]

## Modules and public items

| Module | Items |
|---|---|
| crate root | `router` |
| `state` | `AppState` (`new`) |
| `clinic_header` | `ClinicHeader`, `CLINIC_HEADER` |
| `appointments` | `book` (handler), `BookAppointmentBody`, `AppointmentBody` (`new`) |
| `error` | `ApiError` (`status`, `code`; `MissingClinic`, `InvalidClinic`, `InvalidBody`, `AccessDenied`, `Booking`, `Internal`) |

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
| The caller's role may not do this (not reachable before M4) | 403 | `access_denied` |
| No schedule for that practitioner at this clinic | 422 | `unknown_practitioner` |
| The clinic offers no appointment type with that name | 422 | `unknown_appointment_type` |
| The appointment would end after chrono's latest date | 422 | `out_of_range` |
| Not inside a single shift of the working hours | 422 | `outside_working_hours` |
| Overlaps an appointment already booked | 409 | `overlaps` |
| The storage lock was poisoned by a panic | 500 | `internal` |

## Rules and how they are enforced

| Rule | How |
|---|---|
| Every request names its clinic | `book` takes a `ClinicHeader`, which refuses the request before the handler runs |
| One clinic can't book into another's schedules | The clinic from the header is passed to the ports; tested over HTTP |
| Booking rules stay in the domain | `book` calls `book_appointment` and maps its errors; it checks nothing itself |
| Two requests can't book the same slot | The store's lock is held across `book_appointment`'s read, book and save; tested with 20 concurrent requests |
| Every refusal has a deliberate status | `ApiError::status` matches every `BookingError` without a wildcard, so a new one doesn't compile until it's mapped |
| The domain doesn't depend on serde | JSON shapes are separate DTOs in `appointments` |

[api]: ../../../target/doc/clinic_api/index.html
