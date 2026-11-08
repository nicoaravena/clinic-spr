# Glossary

## Domain

- **Appointment type**: a kind of appointment a clinic offers, such as
  "Consultation", with a fixed duration. Each clinic defines its own.
- **Appointment status**: where an appointment is in its lifecycle:
  scheduled, completed, cancelled or no-show.
- **Clinic**: one customer of the platform, and one tenant. Its data must be
  invisible to every other clinic.
- **Practitioner**: a member of a clinic's staff who sees patients and has an
  agenda.
- **Shift**: a dated period, in UTC, when a practitioner works. Shifts are
  stored as time slots.
- **Slot (time slot)**: a span of time with a start (included) and an end
  (excluded).
- **Tenant**: a customer whose data is kept apart from every other
  customer's. Here, a clinic.
- **Working hours**: all of a practitioner's shifts. A booking must fit
  inside one shift.

## Rust and design

- **Compile-fail doctest**: a code example in a doc comment, marked
  `compile_fail`, that passes only if it fails to compile. Used here to
  prove that fields are private.
- **Crate**: the unit Rust compiles: a library or a binary. `clinic-domain`
  is a library crate.
- **Derive**: `#[derive(...)]` asks the compiler to write standard trait
  implementations, such as `Debug` or `PartialEq`, for a type.
- **Half-open interval**: a range that includes its start and excludes its
  end. Back-to-back slots don't overlap.
- **Integration test**: a test in a crate's `tests/` directory. It is
  compiled as a separate crate and can only use the public API.
- **Invariant**: a rule that is always true for every value of a type, such
  as "start is before end".
- **Newtype**: a tuple struct with one field that wraps another type, to give
  it a distinct type and its own rules. `ClinicId(u64)` is one.
- **Smart constructor**: a function such as `new` or `parse` that checks its
  input and is the only way to build a type. Combined with private fields,
  it makes invalid values impossible to represent.
- **Value object**: a value defined entirely by its contents, with no
  identity of its own. Two equal emails are the same email.
- **Workspace**: a set of crates that share one `Cargo.lock` and one
  `target/` directory.
