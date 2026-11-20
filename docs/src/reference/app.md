# app (crate)

Use cases, the ports they need, and in-memory adapters for those ports.

- **Source:** `crates/app/src/`
- **Introduced in:** [step 2.1](../steps/2.1-app-and-ports.md); the
  agenda query in [step 2.2](../steps/2.2-practitioner-agenda.md); weekly
  hours in [step 2.3](../steps/2.3-weekly-hours.md); setting them in
  [step 2.6](../steps/2.6-weekly-hours-endpoint.md); async, fallible ports
  in [step 3.1](../steps/3.1-async-ports.md)
- **API docs:** [`clinic_app`][api]

## Modules and public items

| Module | Items |
|---|---|
| `ports` | `ScheduleRepository` (`get`, `save`), `AppointmentTypes` (`find`), `StorageError` (`new`) |
| `in_memory` | `InMemorySchedules` (`new`), `InMemoryAppointmentTypes` (`new`, `add`), `DuplicateAppointmentType` (`name`) |
| `booking` | `book_appointment`, `BookingRequest`, `BookAppointmentError` (`UnknownPractitioner`, `UnknownAppointmentType`, `Booking(BookingError)`, `Storage(StorageError)`) |
| `agenda` | `practitioner_agenda`, `AgendaQuery`, `AgendaError` (`UnknownPractitioner`, `Storage(StorageError)`) |
| `weekly_hours` | `WeeklyShift` (`new`, `weekday`, `start`, `end`), `WeeklyHours` (`new`, `time_zone`, `shifts`, `day_start`, `working_hours`), `WeeklyHoursError` (`EndNotAfterStart`, `ShiftsNotSeparated { weekday }`) |
| `hours` | `set_weekly_hours`, `HORIZON_DAYS`, `SetHoursRequest`, `HoursSet`, `HoursOutcome` (`Created`, `Replaced`), `SetHoursError` (`OutOfRange`, `Hours(ReplaceHoursError)`, `Storage(StorageError)`) |

## Rules and how they are enforced

| Rule | How |
|---|---|
| Every storage call names its clinic | Every port method takes a `ClinicId` first |
| A storage failure is never mistaken for an answer | Port methods return `Result<_, StorageError>`, and every use case passes it on with `?`; tested with failing adapters in `tests/storage_errors.rs` |
| A booking or new hours that couldn't be saved aren't reported as done | The use cases return only after `save` succeeds; tested with an adapter whose `save` fails |
| Use cases can run on a multi-threaded runtime | Port methods return `impl Future + Send`; tested by requiring the use cases' futures to be `Send` |
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
| Only a role allowed to manage schedules can set hours | `set_weekly_hours` takes a `&ScheduleAccess` |
| Setting hours keeps the past | Shifts before the start of `from` are kept, via `WorkingHours::replace_from` |
| Setting hours never strands a booked appointment | `Schedule::replace_hours` refuses, and nothing is saved |
| Setting the same hours twice changes nothing | The result depends only on the request and the stored schedule; tested |
| A date near chrono's limits can't panic the server | `set_weekly_hours` returns `OutOfRange` unless two days either side of the horizon exist; tested at both ends in extreme zones |

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

### Ports and storage failures

Every port method is asynchronous and returns a `Result`:

| Method | Returns |
|---|---|
| `ScheduleRepository::get(clinic, practitioner)` | `Result<Option<Schedule>, StorageError>` |
| `ScheduleRepository::save(clinic, practitioner, schedule)` | `Result<(), StorageError>` |
| `AppointmentTypes::find(clinic, name)` | `Result<Option<AppointmentType>, StorageError>` |

Each is declared as returning `impl Future<Output = ...> + Send`, and
adapters implement it with a plain `async fn`. The ports can't be used as
trait objects (a `compile_fail` doctest shows it), so the use cases and
the api crate are generic over the adapter types instead.

`StorageError`'s message is always "storage failed". Its `source()` is the
cause it was built from with `StorageError::new`, such as a driver error or
a message. Each use case's error has a `Storage` variant that passes the
`StorageError` on unchanged (`#[error(transparent)]`), so the use case
errors are no longer `Clone`, `Copy` or `PartialEq`.

A use case stops at the first storage failure. In `book_appointment` and
`set_weekly_hours`, the result is returned only after `save` succeeded.
The in-memory adapters never fail.

`practitioner_agenda(schedules, access, query)` fails with
`UnknownPractitioner` if the practitioner has no schedule at `query.clinic`.
Otherwise it returns copies of the appointments that overlap `query.range`,
sorted by start, which may be none.

### Setting weekly hours

`set_weekly_hours(schedules, access, request)`:

1. Fails with `OutOfRange` unless the dates from two days before `from` to
   two days after `from` + `HORIZON_DAYS` (28) all exist in chrono's
   calendar.
2. Expands `request.hours` on the local dates `[from, until)`, where `until`
   is `from` + 28 days.
3. If the practitioner has no schedule at the clinic, saves a new one with
   those shifts (`Created`).
4. Otherwise replaces the schedule's hours from `hours.day_start(from)` on,
   keeping earlier shifts (`WorkingHours::replace_from`), and fails with
   `Hours(AppointmentOutsideHours)` if a booked appointment wouldn't fit
   (`Replaced` on success).

It returns the outcome, `until`, and the newly expanded shifts only (not
the kept ones). On failure nothing is saved.

`day_start(date)` is the instant `date` begins in the template's time zone:
its local midnight, or the instant the clocks jump to if midnight is
skipped (Santiago, 2026-09-06: 04:00 UTC).

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

- The ports take a `ClinicId` on every call, and nothing groups a read and
  a save into one unit of work. Step 3.2 brings tenant sessions.
- Appointment types are found by name. They get an ID when the database
  arrives.
- `get` returns a whole schedule, with every appointment ever booked, and
  `practitioner_agenda` filters it in memory. A database adapter will need
  a port method that loads only a date range.
- Appointments have no ID, so nothing can refer to one booking yet.
- The weekly template isn't stored, only its expansion. Nothing extends
  the hours when the horizon runs out; the client sends them again.
- Shifts can't cross midnight.

[api]: ../../../target/doc/clinic_app/index.html
