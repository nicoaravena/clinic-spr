# Glossary

## Domain

- **Agenda**: a practitioner's appointments in a span of time, sorted by
  start. An appointment is in the span if it overlaps it.
- **Appointment**: a booked visit: its patient, its appointment type, its
  slot and its status. Created only by booking it in a schedule.
- **Appointment type**: a kind of appointment a clinic offers, such as
  "Consultation", with a fixed duration. Each clinic defines its own.
- **Appointment status**: where an appointment is in its lifecycle:
  scheduled, completed, cancelled or no-show.
- **Booking**: adding an appointment to a schedule. Refused if it falls
  outside the working hours or overlaps another appointment.
- **Capability**: proof that a role was allowed some kind of access, such
  as `ClinicalAccess`. Only the domain can create one, and code that needs
  the access asks for it as a parameter.
- **Clinic**: one customer of the platform, and one tenant. Its data must be
  invisible to every other clinic.
- **Clinical content**: clinical notes and other medical information about a
  patient. Only practitioners may read or write it.
- **Clinical note**: a practitioner's note about a patient. It is never
  overwritten: each correction adds a new version.
- **Correction**: a new version of a clinical note, written by its author.
  The earlier versions are kept.
- **Daylight saving time (DST)**: moving a time zone's clocks forward in
  spring and back in autumn. Each change makes some local times skipped or
  repeated.
- **Horizon**: how far ahead weekly hours are expanded into dated shifts
  when they are set: 28 days from the `from` date. Dates past it have no
  shifts until the hours are set again.
- **Patient**: a person a clinic treats. Appointments are for a patient,
  and clinical notes are about one.
- **Practitioner**: a member of a clinic's staff who sees patients and has an
  agenda.
- **Repeated local time**: a clock reading that happens twice, because the
  clocks went back. A weekly shift uses its first occurrence.
- **Role**: what a member of a clinic's staff does there (practitioner,
  receptionist or administrator), which decides what they may access.
- **Schedule**: one practitioner's working hours and the appointments
  booked in them.
- **Shift**: a dated period, in UTC, when a practitioner works. Shifts are
  stored as time slots.
- **Skipped local time**: a clock reading that never happens, because the
  clocks jumped forward over it. A weekly shift reads it as the instant the
  clocks jump to.
- **Slot (time slot)**: a span of time with a start (included) and an end
  (excluded).
- **Tenant**: a customer whose data is kept apart from every other
  customer's. Here, a clinic.
- **Time zone**: a region's rules for turning UTC into local clock time,
  including when daylight saving starts and ends, such as
  `America/Santiago`. Each clinic's weekly hours are written in one.
- **Version**: one state of a clinical note: its text, its author and when
  it was written. A version never changes.
- **Weekly hours**: the shifts a practitioner works each week, as weekdays
  and local times in a time zone. They are expanded into dated UTC shifts.
- **Working hours**: all of a practitioner's shifts. A booking must fit
  inside one shift.

## Rust and design

- **Adapter**: a type that implements a port for one technology, such as
  `InMemorySchedules`. Swapping the adapter doesn't change the use case.
- **`Arc`**: an atomically reference-counted pointer. Clones share one
  value, which is dropped when the last clone goes. It hands out only `&`,
  so changing the value needs something like a `Mutex` inside.
- **Associated const**: a constant declared in a trait that each
  implementation must provide, such as `Capability::NAME`.
- **Associated type**: a type declared in a trait that each implementation
  chooses, such as `FromRequestParts::Rejection`.
- **`async fn`**: a function that returns a future instead of running. Its
  body runs when the future is polled and can pause at each `.await`.
- **`async fn` in a trait**: a trait method that returns a future. Declared
  as `fn get(..) -> impl Future<Output = ..> + Send` so callers know the
  future is `Send`; an implementation can still write `async fn`. Each
  implementation's future is a different type, so the trait isn't dyn
  compatible.
- **Binary crate**: a crate with a `main` function that compiles to a
  program you can run. `clinic` is one; `clinic-domain` is a library.
- **Borrow checker**: the part of the compiler that enforces borrowing
  rules: many `&` or one `&mut` at a time, and no reference outliving its
  value.
- **Compile-fail doctest**: a code example in a doc comment, marked
  `compile_fail`, that passes only if it fails to compile. Used here to
  prove that fields are private.
- **Composition root**: the one place where the app's parts are created
  and wired together. Here, `main` in `crates/clinic`: it picks the
  in-memory adapters, builds the router and serves it.
- **Crate**: the unit Rust compiles: a library or a binary. `clinic-domain`
  is a library crate.
- **Data transfer object (DTO)**: a type that describes a wire format and
  nothing else, such as `BookAppointmentBody`. It derives serde so the
  domain types don't have to.
- **Derive**: `#[derive(...)]` asks the compiler to write standard trait
  implementations, such as `Debug` or `PartialEq`, for a type.
- **Derive macro**: a library-provided `derive`, such as `thiserror`'s
  `Error`, that generates code for a type at compile time.
- **Dyn compatible**: a trait that can be used as a trait object
  (`dyn Trait`), formerly called "object safe". A trait whose methods
  return `impl Trait`, as async ones do, isn't: the compiler couldn't know
  the size of what they return. Shown by a `compile_fail` doctest on
  `clinic_app::ports`.
- **Dynamic dispatch**: calling a method through a trait object
  (`&dyn Trait`), where the method to run is looked up at runtime. One
  compiled copy serves every type.
- **Extractor**: a type that axum builds from a request before calling a
  handler, such as `Json<T>`, `State<T>` or `ClinicHeader`. If it can't, its
  rejection becomes the response.
- **Future**: a value that represents work in progress. It does nothing
  until a runtime polls it, and each poll runs it to its next `.await`.
- **Generic**: a function or type with a type parameter, such as
  `book_appointment<S: ScheduleRepository>`. The compiler makes one copy per
  concrete type used (*monomorphization*), so calls are dispatched
  statically.
- **Error source**: the error that caused another one, returned by
  `Error::source`. Following sources gives a chain, such as "storage failed:
  connection refused". thiserror's `#[source]` and `#[from]` set it, and
  `#[error(transparent)]` passes the inner error's message and source on.
- **Ephemeral port**: a free port the OS picks when a program asks to
  listen on port 0. Tests use it so several servers can run at once.
- **Golden file**: a file holding the exact output a program must produce.
  A test compares each run against it. Here,
  `crates/clinic/tests/expected-output.txt`, the transcript of the curl
  walkthrough.
- **Graceful shutdown**: stopping a server by first refusing new
  connections, then letting the requests in progress finish, then exiting.
  The opposite of killing it mid-request.
- **Half-open interval**: a range that includes its start and excludes its
  end. Back-to-back slots don't overlap.
- **Handler**: an async function that axum calls for a route. Its
  parameters are extractors and its return value implements
  `IntoResponse`.
- **`impl Trait` return**: a return type such as
  `impl Iterator<Item = &Appointment>`, which promises a type implementing
  the trait without naming it. The compiler still knows the concrete type,
  so calls are static.
- **Idempotent**: doing it twice has the same effect as doing it once.
  `PUT` must be idempotent, so sending the same hours again changes
  nothing; `POST /appointments` isn't.
- **Integration test**: a test in a crate's `tests/` directory. It is
  compiled as a separate crate and can only use the public API.
- **Invariant**: a rule that is always true for every value of a type, such
  as "start is before end".
- **Iterator adapter**: a method such as `map`, `filter` or `take_while`
  that wraps one iterator in another. Adapters are lazy: nothing runs until
  the items are pulled out, by `for`, `collect` or `next`.
- **Lifetime**: the span during which a reference is valid, written `'a`.
  The compiler checks that no reference outlives the value it points to.
- **Lifetime elision**: rules that let the compiler fill in lifetimes you
  don't write. A method's returned reference gets the lifetime of `&self`.
- **Lost update**: two writers read the same value, both change it, and
  the second save silently replaces the first. Prevented here by holding
  one lock from the read to the save.
- **Match guard**: an `if` condition after a pattern in a `match` arm. The
  arm runs only if the pattern fits and the condition holds.
- **Move**: passing a value by value hands over ownership. The caller can't
  use it afterwards unless it cloned it first. `Schedule::book` moves the
  appointment type into the new appointment.
- **`Mutex`**: a lock that lets one owner at a time reach a value. `lock()`
  returns a guard, and dropping the guard unlocks. The standard library's
  mutex is *poisoned* if a thread panics while holding it, and its guard
  isn't `Send`, so it can't be held across an `.await` in a future that
  must be `Send`. tokio's `Mutex` is locked with `.lock().await`, can be
  held across `.await`, and isn't poisoned. The api crate uses tokio's since
  step 3.1.
- **Naive time**: chrono's `NaiveTime`, `NaiveDate` and `NaiveDateTime`: a
  clock or calendar reading with no time zone, so not yet an instant.
- **Newtype**: a tuple struct with one field that wraps another type, to give
  it a distinct type and its own rules. `ClinicId(u64)` is one.
- **Parsing at the edge**: turning text from outside, such as a time
  zone name, into a typed value as soon as it arrives, in the delivery
  layer. The use cases and the domain only ever see the type.
- **Path parameter**: a part of a route's path that varies, such as `{id}`
  in `/practitioners/{id}/agenda`. axum's `Path` extractor reads it.
- **Port**: a trait that says what a use case needs from the outside world,
  without saying how it is done, such as `ScheduleRepository`.
- **Precise capturing**: `+ use<'a>` on an `impl Trait` return type, which
  lists the lifetimes the hidden type may borrow. Without it, edition 2024
  captures every lifetime in scope.
- **`PUT`**: the HTTP method that sets a resource to what the body says,
  creating it if needed. Here it answers 201 when it created and 200 when
  it replaced.
- **Query**: a use case that only reads, such as `practitioner_agenda`. It
  takes its ports by `&` and never saves.
- **Query string**: the part of a URL after `?`, such as
  `from=...&to=...`. axum's `Query` extractor reads it into a struct. In
  it, `+` means a space, so a literal `+` is sent as `%2B`.
- **Rejection**: the error an extractor returns when it can't build itself
  from a request. It implements `IntoResponse`.
- **Repository**: a port that stores and finds one kind of domain object,
  as if it were an in-memory collection.
- **Runtime (async)**: the library that polls futures and schedules them
  on threads. Rust doesn't ship one; this project uses tokio.
- **`?` operator**: unwraps an `Ok` or `Some`, or returns the `Err` or
  `None` from the current function early.
- **`Send` and `Sync`**: `Send` means a value may move to another thread;
  `Sync` means it may be shared by reference between threads. The compiler
  works them out from a type's fields. axum requires handlers' futures to
  be `Send`.
- **`Sized` and `?Sized`**: `Sized` types have a size known at compile
  time. Type parameters are `Sized` unless marked `?Sized`, which lets them
  be a trait object such as `dyn Error`, or `str`. The use cases had
  `?Sized` bounds until step 3.1, when the ports stopped being dyn
  compatible.
- **Signal**: a message the OS sends a process. SIGINT is what Ctrl-C
  sends; SIGTERM is what `kill` and `docker stop` send. Both ask the
  `clinic` server to shut down gracefully.
- **Smart constructor**: a function such as `new` or `parse` that checks its
  input and is the only way to build a type. Combined with private fields,
  it makes invalid values impossible to represent.
- **Status code**: the number at the start of an HTTP response. Here: 200
  read or replaced, 201 booked or created, 400 bad header, path, query or
  malformed JSON, 403 role not allowed, 404 no such practitioner in the
  path, 409 clashes with an existing appointment, 415 not JSON, 422
  well-formed but refused, 500 server fault.
- **Storage error**: `StorageError`, what a port returns when its storage
  can't be read or written. Its message is always "storage failed"; the
  cause is its error source, logged by the server and never sent to a
  client, which gets a 500.
- **Trait**: a named set of methods, functions or constants that types can
  implement, such as `Capability`.
- **Trait bound**: a requirement on a generic type, such as
  `C: Capability`, meaning "any type that implements `Capability`".
- **Trait object**: `dyn Trait`, a value whose concrete type is known only
  at runtime, used behind a pointer such as `Box`. `Box<dyn Error>` holds
  any error.
- **Use case**: one thing the application does for a user, such as
  booking an appointment, written as a function over ports. It lives in
  `crates/app`.
- **Value object**: a value defined entirely by its contents, with no
  identity of its own. Two equal emails are the same email.
- **Workspace**: a set of crates that share one `Cargo.lock` and one
  `target/` directory.
