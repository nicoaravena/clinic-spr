# app (crate)

Use cases, the ports they need, and in-memory adapters for those ports.

- **Source:** `crates/app/src/`
- **Introduced in:** [step 2.1](../steps/2.1-app-and-ports.md); the
  agenda query in [step 2.2](../steps/2.2-practitioner-agenda.md)
- **API docs:** [`clinic_app`][api]

## Modules and public items

| Module | Items |
|---|---|
| `ports` | `ScheduleRepository` (`get`, `save`), `AppointmentTypes` (`find`) |
| `in_memory` | `InMemorySchedules` (`new`), `InMemoryAppointmentTypes` (`new`, `add`), `DuplicateAppointmentType` (`name`) |
| `booking` | `book_appointment`, `BookingRequest`, `BookAppointmentError` (`UnknownPractitioner`, `UnknownAppointmentType`, `Booking(BookingError)`) |
| `agenda` | `practitioner_agenda`, `AgendaQuery`, `AgendaError` (`UnknownPractitioner`) |

## Rules and how they are enforced

| Rule | How |
|---|---|
| Every storage call names its clinic | Every port method takes a `ClinicId` first |
| One clinic can't see another's data | The in-memory adapters key by clinic; tested in `tests/in_memory.rs` |
| Appointment type names are unique within a clinic | `InMemoryAppointmentTypes::add` returns `DuplicateAppointmentType`; private fields, `compile_fail` doctest |
| Only a role allowed to manage schedules can book | `book_appointment` takes a `&ScheduleAccess` |
| A refused booking saves nothing | `book_appointment` calls `save` only after `Schedule::book` succeeds |
| The booking rules themselves stay in the domain | `book_appointment` calls `Schedule::book` and passes its `BookingError` on unchanged |
| Only a role allowed to see schedules can read an agenda | `practitioner_agenda` takes a `&ScheduleAccess` |
| Which appointments are in a range is a domain rule | `practitioner_agenda` calls `Schedule::appointments_in` |
| Reading an agenda changes nothing | `practitioner_agenda` takes `&S` and never calls `save` |

## Semantics

`book_appointment(schedules, types, access, request)` checks, in this order,
and stops at the first failure:

1. The practitioner has a schedule at `request.clinic`
   (`UnknownPractitioner`).
2. The clinic offers a type named exactly `request.appointment_type`
   (`UnknownAppointmentType`).
3. `Schedule::book` accepts it (`Booking(..)`, with the domain's error and
   message).

On success it saves the updated schedule and returns a copy of the new
appointment.

`book_appointment` is generic over both ports, with `?Sized` bounds, so it
takes concrete adapters and trait objects (`&mut dyn ScheduleRepository`)
alike.

`practitioner_agenda(schedules, access, query)` fails with
`UnknownPractitioner` if the practitioner has no schedule at `query.clinic`.
Otherwise it returns copies of the appointments that overlap `query.range`,
sorted by start, which may be none. It is generic over the port with a
`?Sized` bound, like `book_appointment`.

## Not yet

- The ports are synchronous. They become `async` in M3, with the database.
- Appointment types are found by name. They get an ID when the database
  arrives.
- `get` returns a whole schedule, with every appointment ever booked, and
  `practitioner_agenda` filters it in memory. A database adapter will need
  a port method that loads only a date range.
- Appointments have no ID, so nothing can refer to one booking yet.

[api]: ../../../target/doc/clinic_app/index.html
