# app (crate)

Use cases, the ports they need, and in-memory adapters for those ports.

- **Source:** `crates/app/src/`
- **Introduced in:** [step 2.1](../steps/2.1-app-and-ports.md); the
  agenda query in [step 2.2](../steps/2.2-practitioner-agenda.md); weekly
  hours in [step 2.3](../steps/2.3-weekly-hours.md)
- **API docs:** [`clinic_app`][api]

## Modules and public items

| Module | Items |
|---|---|
| `ports` | `ScheduleRepository` (`get`, `save`), `AppointmentTypes` (`find`) |
| `in_memory` | `InMemorySchedules` (`new`), `InMemoryAppointmentTypes` (`new`, `add`), `DuplicateAppointmentType` (`name`) |
| `booking` | `book_appointment`, `BookingRequest`, `BookAppointmentError` (`UnknownPractitioner`, `UnknownAppointmentType`, `Booking(BookingError)`) |
| `agenda` | `practitioner_agenda`, `AgendaQuery`, `AgendaError` (`UnknownPractitioner`) |
| `weekly_hours` | `WeeklyShift` (`new`, `weekday`, `start`, `end`), `WeeklyHours` (`new`, `time_zone`, `shifts`, `working_hours`), `WeeklyHoursError` (`EndNotAfterStart`, `ShiftsNotSeparated { weekday }`) |

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
| A weekly shift ends after it starts, on the same day | `WeeklyShift::new` returns `EndNotAfterStart`; private fields |
| Weekly shifts on the same weekday don't overlap or touch | `WeeklyHours::new` returns `ShiftsNotSeparated`; private fields, `compile_fail` doctest |
| Time zones stay out of the domain | `WeeklyHours::working_hours` converts to UTC before building `WorkingHours` |
| Expanding weekly hours can't fail | Skipped and repeated local times map forward only; tested over a year in three zones |

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

### Weekly hours

`WeeklyHours::working_hours(from, until)` takes each local date from `from`
up to, but not including, `until`. For every template shift on that date's
weekday, it reads the start and end in the template's time zone and
converts them to UTC:

| Local reading | Becomes |
|---|---|
| Happens once | That instant |
| Happens twice (clocks went back) | The first occurrence |
| Never happens (clocks jumped forward) | The instant the clocks jump to |

A shift that lies entirely in a skipped hour is dropped for that date. Two
shifts separated only by a skipped hour are merged into one. The result is
always valid `WorkingHours`; an empty or reversed range gives no shifts.

Examples (Europe/Madrid):

| Date | Template shift | UTC shift |
|---|---|---|
| Mon 2026-03-23 | 09:00-13:00 | 08:00-12:00 |
| Mon 2026-03-30 | 09:00-13:00 | 07:00-11:00 |
| Sun 2026-03-29 | 02:30-05:00 | 01:00-03:00 |
| Sun 2026-03-29 | 02:10-02:50 | none |
| Sun 2026-10-25 | 00:00-02:30 | 22:00 (the 24th)-00:30 |

## Not yet

- The ports are synchronous. They become `async` in M3, with the database.
- Appointment types are found by name. They get an ID when the database
  arrives.
- `get` returns a whole schedule, with every appointment ever booked, and
  `practitioner_agenda` filters it in memory. A database adapter will need
  a port method that loads only a date range.
- Appointments have no ID, so nothing can refer to one booking yet.
- No use case expands weekly hours or stores them. Shifts can't cross
  midnight.

[api]: ../../../target/doc/clinic_app/index.html
