# Glossary

## Domain

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
- **Patient**: a person a clinic treats. Appointments are for a patient,
  and clinical notes are about one.
- **Practitioner**: a member of a clinic's staff who sees patients and has an
  agenda.
- **Role**: what a member of a clinic's staff does there (practitioner,
  receptionist or administrator), which decides what they may access.
- **Schedule**: one practitioner's working hours and the appointments
  booked in them.
- **Shift**: a dated period, in UTC, when a practitioner works. Shifts are
  stored as time slots.
- **Slot (time slot)**: a span of time with a start (included) and an end
  (excluded).
- **Tenant**: a customer whose data is kept apart from every other
  customer's. Here, a clinic.
- **Version**: one state of a clinical note: its text, its author and when
  it was written. A version never changes.
- **Working hours**: all of a practitioner's shifts. A booking must fit
  inside one shift.

## Rust and design

- **Adapter**: a type that implements a port for one technology, such as
  `InMemorySchedules`. Swapping the adapter doesn't change the use case.
- **Associated const**: a constant declared in a trait that each
  implementation must provide, such as `Capability::NAME`.
- **Binary crate**: a crate with a `main` function that compiles to a
  program you can run. `clinic` is one; `clinic-domain` is a library.
- **Borrow checker**: the part of the compiler that enforces borrowing
  rules: many `&` or one `&mut` at a time, and no reference outliving its
  value.
- **Compile-fail doctest**: a code example in a doc comment, marked
  `compile_fail`, that passes only if it fails to compile. Used here to
  prove that fields are private.
- **Composition root**: the one place where the app's parts are created
  and wired together. Here, `main` in `crates/clinic`.
- **Crate**: the unit Rust compiles: a library or a binary. `clinic-domain`
  is a library crate.
- **Derive**: `#[derive(...)]` asks the compiler to write standard trait
  implementations, such as `Debug` or `PartialEq`, for a type.
- **Derive macro**: a library-provided `derive`, such as `thiserror`'s
  `Error`, that generates code for a type at compile time.
- **Dynamic dispatch**: calling a method through a trait object
  (`&dyn Trait`), where the method to run is looked up at runtime. One
  compiled copy serves every type.
- **Generic**: a function or type with a type parameter, such as
  `book_appointment<S: ScheduleRepository>`. The compiler makes one copy per
  concrete type used (*monomorphization*), so calls are dispatched
  statically.
- **Golden file**: a file holding the exact output a program must produce.
  A test compares each run against it. Here,
  `crates/clinic/tests/expected-output.txt`.
- **Half-open interval**: a range that includes its start and excludes its
  end. Back-to-back slots don't overlap.
- **Integration test**: a test in a crate's `tests/` directory. It is
  compiled as a separate crate and can only use the public API.
- **Invariant**: a rule that is always true for every value of a type, such
  as "start is before end".
- **Lifetime**: the span during which a reference is valid, written `'a`.
  The compiler checks that no reference outlives the value it points to.
- **Lifetime elision**: rules that let the compiler fill in lifetimes you
  don't write. A method's returned reference gets the lifetime of `&self`.
- **Move**: passing a value by value hands over ownership. The caller can't
  use it afterwards unless it cloned it first. `Schedule::book` moves the
  appointment type into the new appointment.
- **Newtype**: a tuple struct with one field that wraps another type, to give
  it a distinct type and its own rules. `ClinicId(u64)` is one.
- **Port**: a trait that says what a use case needs from the outside world,
  without saying how it is done, such as `ScheduleRepository`.
- **Precise capturing**: `+ use<'a>` on an `impl Trait` return type, which
  lists the lifetimes the hidden type may borrow. Without it, edition 2024
  captures every lifetime in scope.
- **Repository**: a port that stores and finds one kind of domain object,
  as if it were an in-memory collection.
- **`?` operator**: unwraps an `Ok` or `Some`, or returns the `Err` or
  `None` from the current function early.
- **`Sized` and `?Sized`**: `Sized` types have a size known at compile
  time. Type parameters are `Sized` unless marked `?Sized`, which lets them
  be a trait object such as `dyn ScheduleRepository`.
- **Smart constructor**: a function such as `new` or `parse` that checks its
  input and is the only way to build a type. Combined with private fields,
  it makes invalid values impossible to represent.
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
