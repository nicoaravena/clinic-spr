# Architecture

## Crates

Today the workspace has one crate. The roadmap adds the others when they are
needed:

| Crate | Holds | Arrives in |
|---|---|---|
| `crates/domain` (`clinic-domain`) | entities, value objects, rules; no I/O | M1 (now) |
| `crates/app` | use cases and repository traits | M2 |
| `crates/api` | axum handlers, the binary, composition root | M2 |
| `crates/db` | sqlx adapters, migrations, tenant access | M3 |

Dependencies point inward: `api -> app -> domain` and `db -> app -> domain`.
The domain depends on nothing else in the workspace, and its only external
dependency is `chrono`, without the system clock.

## Rules that hold across the domain

- **UTC only.** Every time is a `DateTime<Utc>`. Time zones are converted at
  the edges, never inside the domain.
- **Valid by construction.** Types with rules have private fields and a
  constructor that checks them (`new` or `parse`). Holding a value means its
  rules hold. Each module proves its fields are private with a
  `compile_fail` doctest.
- **Errors are enums.** Each module has its own error enum, which implements
  `Display` and `std::error::Error`.
- **No clock, no I/O.** Callers pass in times and data.

## Modules

| Module | Main types | Step |
|---|---|---|
| [ids](ids.md) | `ClinicId`, `PractitionerId` | 1.2 |
| [email](email.md) | `Email` | 1.2 |
| [time_slot](time_slot.md) | `TimeSlot` | 1.3 |
| [appointment](appointment.md) | `AppointmentType`, `AppointmentStatus` | 1.4 |
| [working_hours](working_hours.md) | `WorkingHours` | 1.5 |

`schedule` holds only the step 1.1 placeholder and is replaced in step 1.6.

## API docs

Exact signatures live in the
[rustdoc for `clinic_domain`](../../../target/doc/clinic_domain/index.html).
Run `cargo doc --no-deps` first (see the
[Introduction](../introduction.md#building-the-book)).
