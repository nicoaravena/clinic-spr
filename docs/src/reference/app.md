# app (crate)

Use cases, the ports they need, and in-memory adapters for those ports.

- **Source:** `crates/app/src/`
- **Introduced in:** [step 2.1](../steps/2.1-app-and-ports.md); the
  agenda query in [step 2.2](../steps/2.2-practitioner-agenda.md); weekly
  hours in [step 2.3](../steps/2.3-weekly-hours.md); setting them in
  [step 2.6](../steps/2.6-weekly-hours-endpoint.md); async, fallible ports
  in [step 3.1](../steps/3.1-async-ports.md); tenant sessions in
  [step 3.2](../steps/3.2-tenant-sessions.md)
- **API docs:** [`clinic_app`][api]

## Modules and public items

| Module | Items |
|---|---|
| `ports` | `Storage` (`Session`, `begin`), `Session` (`commit`), `ScheduleRepository` (`get`, `save`), `AppointmentTypes` (`find`), `StorageError` (`new`) |
| `in_memory` | `InMemoryStorage` (`new`, `default`), `InMemorySession`, `InMemoryAppointmentTypes` (`new`, `add`), `DuplicateAppointmentType` (`name`) |
| `booking` | `book_appointment`, `BookingRequest`, `BookAppointmentError` (`UnknownPractitioner`, `UnknownAppointmentType`, `Booking(BookingError)`, `Storage(StorageError)`) |
| `agenda` | `practitioner_agenda`, `AgendaQuery`, `AgendaError` (`UnknownPractitioner`, `Storage(StorageError)`) |
| `weekly_hours` | `WeeklyShift` (`new`, `weekday`, `start`, `end`), `WeeklyHours` (`new`, `time_zone`, `shifts`, `day_start`, `working_hours`), `WeeklyHoursError` (`EndNotAfterStart`, `ShiftsNotSeparated { weekday }`) |
| `hours` | `set_weekly_hours`, `HORIZON_DAYS`, `SetHoursRequest`, `HoursSet`, `HoursOutcome` (`Created`, `Replaced`), `SetHoursError` (`OutOfRange`, `Hours(ReplaceHoursError)`, `Storage(StorageError)`) |

## Rules and how they are enforced

| Rule | How |
|---|---|
| Every storage call names its clinic | Storage is reached only through a `Session`, and `Storage::begin` takes the `ClinicId` |
| A use case's reads and writes succeed or fail together | Each use case runs in one session and keeps its changes only by calling `commit`; dropping the session discards them; tested in `tests/in_memory.rs` |
| A committed session can't be used again | `commit` takes the session by value; `compile_fail` doctest (E0382) |
| A storage failure is never mistaken for an answer | Port methods, `begin` and `commit` return `Result<_, StorageError>`, and every use case passes it on with `?`; tested in `tests/storage_errors.rs` with a storage that fails at each step |
| A booking or new hours that couldn't be saved aren't reported as done | The use cases return only after `commit` succeeds; tested with failing `save` and failing `commit`, after which nothing is stored |
| Two bookings can't both see a slot free | The in-memory session holds the storage's lock from `begin` to `commit` or drop |
| Use cases can run on a multi-threaded runtime | Port methods return `impl Future + Send`; tested by requiring the use cases' futures to be `Send` |
| One clinic can't see another's data | A session reads and writes only the clinic it began at; the in-memory storage keys by clinic; tested in `tests/in_memory.rs` |
| Appointment type names are unique within a clinic | `InMemoryAppointmentTypes::add` returns `DuplicateAppointmentType`; private fields, `compile_fail` doctest |
| Only a role allowed to manage schedules can book | `book_appointment` takes a `&ScheduleAccess` |
| A refused booking saves nothing | `book_appointment` returns before `save` and `commit`, and the dropped session keeps nothing |
| The booking rules themselves stay in the domain | `book_appointment` calls `Schedule::book` and passes its `BookingError` on unchanged |
| Only a role allowed to see schedules can read an agenda | `practitioner_agenda` takes a `&ScheduleAccess` |
| Which appointments are in a range is a domain rule | `practitioner_agenda` calls `Schedule::appointments_in` |
| Reading an agenda changes nothing | `practitioner_agenda` never calls `save` or `commit`; tested with a storage whose `commit` fails |
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

`book_appointment(storage, access, request)` begins a session at
`request.clinic`, then checks, in this order, and stops at the first
failure:

1. The practitioner has a schedule at `request.clinic`
   (`UnknownPractitioner`).
2. The clinic offers a type named exactly `request.appointment_type`
   (`UnknownAppointmentType`).
3. `Schedule::book` accepts it (`Booking(..)`, with the domain's error and
   message).

On success it saves the updated schedule, commits, and returns a copy of
the new appointment.

### Sessions

A use case reaches storage only through a session: a unit of work at one
clinic.

| Method | Returns |
|---|---|
| `Storage::begin(clinic)` | `Result<Self::Session, StorageError>` |
| `Session::commit(self)` | `Result<(), StorageError>` |
| `ScheduleRepository::get(practitioner)` | `Result<Option<Schedule>, StorageError>` |
| `ScheduleRepository::save(practitioner, schedule)` | `Result<(), StorageError>` |
| `AppointmentTypes::find(name)` | `Result<Option<AppointmentType>, StorageError>` |

`Storage::Session` is an associated type: each storage names its own
session type, which implements both ports and `Session`. What a session
saves is seen by its own `get` at once, and by other sessions only after
`commit`. Dropping a session without committing it discards its changes.

Every method is declared as returning `impl Future<Output = ...> + Send`,
and adapters implement it with a plain `async fn`. The traits can't be used
as trait objects (a `compile_fail` doctest shows it), so the use cases and
the api crate are generic over the storage type instead.

`InMemoryStorage` keeps every clinic's data behind one
`Arc<tokio::sync::Mutex<_>>`. `begin` takes the lock with `lock_owned`, and
the `InMemorySession` holds the guard until it's committed or dropped, so
sessions run one at a time. A session keeps its saved schedules in a map of
its own; `commit` copies them into the storage. A task that begins a second
session while holding one waits forever.

`StorageError`'s message is always "storage failed". Its `source()` is the
cause it was built from with `StorageError::new`, such as a driver error or
a message. Each use case's error has a `Storage` variant that passes the
`StorageError` on unchanged (`#[error(transparent)]`), so the use case
errors are no longer `Clone`, `Copy` or `PartialEq`.

A use case stops at the first storage failure, and its session is dropped
with whatever it saved. In `book_appointment` and `set_weekly_hours`, the
result is returned only after `commit` succeeded. The in-memory adapters
never fail.

`practitioner_agenda(storage, access, query)` begins a session at
`query.clinic` and fails with `UnknownPractitioner` if the practitioner has
no schedule there. Otherwise it returns copies of the appointments that
overlap `query.range`, sorted by start, which may be none. It doesn't
commit: it drops the session.

### Setting weekly hours

`set_weekly_hours(storage, access, request)` works in one session at
`request.clinic`:

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

It commits, then returns the outcome, `until`, and the newly expanded
shifts only (not the kept ones). On failure nothing is saved.

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

- The in-memory storage runs one session at a time for every clinic. The
  database runs them as concurrent transactions, with a row lock and an
  exclusion constraint (step 3.6) doing the lock's job.
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
