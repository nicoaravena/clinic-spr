# Architecture

## Crates

Today the workspace has five crates. The roadmap adds the others when they
are needed:

| Crate | Holds | Arrives in |
|---|---|---|
| `crates/domain` (`clinic-domain`) | entities, value objects, rules; no I/O | M1 (now) |
| `crates/clinic` (`clinic`) | the binary and composition root: the HTTP server; see [clinic](clinic.md) | M1 (now) |
| `crates/app` (`clinic-app`) | use cases, ports (repository traits) and in-memory adapters; see [app](app.md) | 2.1 (now) |
| `crates/api` (`clinic-api`) | axum router and handlers, as a library; see [api](api.md) | 2.4 (now) |
| `crates/db` (`clinic-db`) | the Postgres pool, migrations, the platform schema and, from 3.4, tenant access; see [db](db.md) | 3.3 (now) |

Dependencies point inward: `api -> app -> domain` and `db -> app -> domain`.
Only `clinic` depends on all of them, to wire them together (on `db` from
step 3.7, when the server moves to Postgres).
The domain depends on nothing else in the workspace, and `app` depends only
on the domain. The domain's only external
dependencies are `chrono`, without the system clock, and `thiserror`, which
generates error boilerplate at compile time. `app` adds `chrono-tz`, and tokio's `sync` feature for the in-memory
storage's lock. Only
`api` uses `serde`, and it defines every route with `axum`. `clinic` uses
`axum` only to serve the router, and it's the only crate that starts a tokio
runtime. Only `db` uses `sqlx`, so neither the domain, the use cases nor the
handlers know there's a database.

## Rules that hold across the domain

- **UTC only.** Every time is a `DateTime<Utc>`. Time zones are converted at
  the edges, never inside the domain.
- **Valid by construction.** Types with rules have private fields and a
  constructor that checks them (`new` or `parse`). Holding a value means its
  rules hold. Each module proves its fields are private with a
  `compile_fail` doctest.
- **Errors are enums.** Each module has its own error enum, which implements
  `Display` and `std::error::Error`, by hand (steps 1.2 to 1.5) or with
  `thiserror` (from step 1.6).
- **No clock, no I/O.** Callers pass in times and data.

## Modules

| Module | Main types | Step |
|---|---|---|
| [ids](ids.md) | `ClinicId`, `PractitionerId`, `PatientId` (1.8) | 1.2 |
| [email](email.md) | `Email` | 1.2 |
| [time_slot](time_slot.md) | `TimeSlot` | 1.3 |
| [appointment](appointment.md) | `AppointmentType`, `AppointmentStatus` | 1.4 |
| [working_hours](working_hours.md) | `WorkingHours` | 1.5 |
| [schedule](schedule.md) | `Schedule`, `Appointment` (in `appointment`) | 1.6 |
| [access](access.md) | `Role`, `Capability`, `ClinicalAccess` | 1.7 |
| [clinical_note](clinical_note.md) | `ClinicalNote`, `NoteVersion` | 1.8 |
| [clinic](clinic_entity.md) | `Clinic`, `Slug`, `ClinicStatus` | 3.3 |

## API docs

Exact signatures live in the
[rustdoc for `clinic_domain`](../../../target/doc/clinic_domain/index.html).
Run `cargo doc --no-deps` first (see the
[Introduction](../introduction.md#building-the-book)).
