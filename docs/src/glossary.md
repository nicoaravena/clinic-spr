# Glossary

## Domain

- **Appointment**: a booked visit: its appointment type, its slot and its
  status. Created only by booking it in a schedule.
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
- **Working hours**: all of a practitioner's shifts. A booking must fit
  inside one shift.

## Rust and design

- **Associated const**: a constant declared in a trait that each
  implementation must provide, such as `Capability::NAME`.
- **Binary crate**: a crate with a `main` function that compiles to a
  program you can run. `clinic` is one; `clinic-domain` is a library.
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
- **Golden file**: a file holding the exact output a program must produce.
  A test compares each run against it. Here,
  `crates/clinic/tests/expected-output.txt`.
- **Half-open interval**: a range that includes its start and excludes its
  end. Back-to-back slots don't overlap.
- **Integration test**: a test in a crate's `tests/` directory. It is
  compiled as a separate crate and can only use the public API.
- **Invariant**: a rule that is always true for every value of a type, such
  as "start is before end".
- **Move**: passing a value by value hands over ownership. The caller can't
  use it afterwards unless it cloned it first. `Schedule::book` moves the
  appointment type into the new appointment.
- **Newtype**: a tuple struct with one field that wraps another type, to give
  it a distinct type and its own rules. `ClinicId(u64)` is one.
- **`?` operator**: unwraps an `Ok` or `Some`, or returns the `Err` or
  `None` from the current function early.
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
- **Value object**: a value defined entirely by its contents, with no
  identity of its own. Two equal emails are the same email.
- **Workspace**: a set of crates that share one `Cargo.lock` and one
  `target/` directory.
