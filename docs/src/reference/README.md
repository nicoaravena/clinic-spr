# Architecture

## Crates

Today the workspace has two crates. The roadmap adds the others when they
are needed:

| Crate | Holds | Arrives in |
|---|---|---|
| `crates/domain` (`clinic-domain`) | entities, value objects, rules; no I/O | M1 (now) |
| `crates/clinic` (`clinic`) | the binary and composition root; a scripted day until M2 | M1 (now) |
| `crates/app` | use cases and repository traits | M2 |
| `crates/api` | axum handlers, as a library | M2 |
| `crates/db` | sqlx adapters, migrations, tenant access | M3 |

Dependencies point inward: `api -> app -> domain` and `db -> app -> domain`.
Only `clinic` depends on all of them, to wire them together.
The domain depends on nothing else in the workspace. Its only external
dependencies are `chrono`, without the system clock, and `thiserror`, which
generates error boilerplate at compile time.

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
| [ids](ids.md) | `ClinicId`, `PractitionerId` | 1.2 |
| [email](email.md) | `Email` | 1.2 |
| [time_slot](time_slot.md) | `TimeSlot` | 1.3 |
| [appointment](appointment.md) | `AppointmentType`, `AppointmentStatus` | 1.4 |
| [working_hours](working_hours.md) | `WorkingHours` | 1.5 |
| [schedule](schedule.md) | `Schedule`, `Appointment` (in `appointment`) | 1.6 |
| [access](access.md) | `Role`, `Capability`, `ClinicalAccess` | 1.7 |

## API docs

Exact signatures live in the
[rustdoc for `clinic_domain`](../../../target/doc/clinic_domain/index.html).
Run `cargo doc --no-deps` first (see the
[Introduction](../introduction.md#building-the-book)).
