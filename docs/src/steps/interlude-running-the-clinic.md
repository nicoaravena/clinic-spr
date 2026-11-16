# Interlude: a runnable clinic

## Goal

Give every step something you can run. `crates/clinic` is a binary whose
`main` plays one scripted day at a clinic. Each roadmap step extends that
day to use what the step added. From M2 the same binary becomes the server,
so by the end of the roadmap it is the full app.

This isn't a roadmap step. It was added after step 1.7, and the day was
backfilled with steps 1.2 to 1.7.

## What changed

- `crates/clinic/Cargo.toml`: a new package, `clinic`, that depends on
  `clinic-domain` and `chrono`. The workspace's `crates/*` glob picks it up.
- `crates/clinic/src/main.rs`: the scripted day.
- `crates/clinic/tests/run.rs` and `crates/clinic/tests/expected-output.txt`:
  a golden-output test.
- `docs/ROADMAP.md`: each step now extends the day, and `crates/clinic` is
  the binary and composition root. `crates/api` becomes a library of axum
  handlers.

## Rust concepts

### Binary and library crates

A package with `src/main.rs` builds a program; one with `src/lib.rs` builds
a library that other crates use. `clinic` is the first binary. It depends on
`clinic-domain` with a path dependency:

```toml
clinic-domain = { path = "../domain" }
```

and uses it like any external crate, through its public API only.
`cargo run -p clinic` picks the package to run, which will matter once the
workspace has several binaries.

Book: [Packages and Crates](https://doc.rust-lang.org/book/ch07-01-packages-and-crates.html).

### `main` returning `Result`, and `Box<dyn Error>`

```rust,ignore
fn main() -> Result<(), Box<dyn Error>> {
    let ana_email = Email::parse("ana@example.com")?;
    let consultation = AppointmentType::new("Consultation", TimeDelta::minutes(30))?;
    let hours = WorkingHours::new(vec![/* ... */])?;
    // ...
    Ok(())
}
```

Each `?` here meets a different error type: `EmailError`,
`AppointmentTypeError`, `WorkingHoursError`. `Box<dyn Error>` is a *trait
object*: a pointer to any value that implements `Error`, whose concrete type
is known only at runtime. The standard library implements
`From<E> for Box<dyn Error>` for every error `E`, and `?` calls `From`, so
every domain error converts on the way out. If `main` returns `Err`, Rust
prints it to stderr and exits with code 1.

This fits a binary, which only reports errors. A library should keep its
error enums, so callers can `match` on them.

Book: [`?` in `main`](https://doc.rust-lang.org/book/ch09-02-recoverable-errors-with-result.html#where-the--operator-can-be-used)
and [Trait Objects](https://doc.rust-lang.org/book/ch18-02-trait-objects.html).

### Expected and unexpected errors

```rust,ignore
match schedule.book(start, appointment_type.clone()) {
    Ok(appointment) => println!("  {request} booked for {}", span(appointment.slot())),
    Err(error @ BookingError::Overlaps { existing }) => {
        println!("  {request} refused: {error} ({})", span(existing));
    }
    Err(error) => println!("  {request} refused: {error}"),
}
```

A refusal that the day is meant to show is printed with `match` or
`if let`. A failure that should never happen goes through `?` and stops the
run. `error @ BookingError::Overlaps { existing }` binds the whole error to
`error` and its field to `existing` in one pattern.

Book: [`@` Bindings](https://doc.rust-lang.org/book/ch19-03-pattern-syntax.html#-bindings).

### A closure for formatting

```rust,ignore
let span = |slot: TimeSlot| {
    format!("{}-{}", slot.start().format("%H:%M"), slot.end().format("%H:%M"))
};
```

`TimeSlot` has no `Display`, and the domain shouldn't decide how times look
on screen. A closure inside `main` keeps that formatting at the edge.

Book: [Closures](https://doc.rust-lang.org/book/ch13-01-closures.html).

### Testing the real binary

```rust,ignore
const EXPECTED: &str = include_str!("expected-output.txt");

let output = Command::new(env!("CARGO_BIN_EXE_clinic")).output()?;
assert_eq!(String::from_utf8_lossy(&output.stdout), EXPECTED);
```

Cargo builds the binary before a package's integration tests, and sets
`CARGO_BIN_EXE_<name>` to its path. `include_str!` embeds the golden file at
compile time. The test runs the real program, end to end, with no extra
dependencies.

Book: [Integration Tests](https://doc.rust-lang.org/book/ch11-03-test-organization.html#integration-tests).

## Design choices worth questioning

- **One `main`, not one function per step.** The day reads as one story,
  and each step's change shows up as a diff of `main` and of the golden
  file. If `main` grows too long, it can be split by topic (staff, booking,
  access) instead of by step.
- **Exact output, not a few `contains` checks.** Any change to the output
  fails the test, even a reworded message. That's deliberate: the golden
  file is the record of what the app does, and updating it is one command.
- **Expected refusals aren't asserted in `main`.** If a refusal stopped
  happening, its line would just be missing. The golden test catches that.
- **Named `clinic`, not `demo`.** It becomes the real binary in M2, where it
  stops playing the day and starts serving HTTP. Then the day becomes a curl
  walkthrough.

## Tests

`tests/run.rs` runs the binary, checks it exits with code 0, and compares
stdout with `expected-output.txt`.

## See it run

```sh
cargo run -p clinic
```

```text
Clinic 1 opens on Monday 2026-03-02. All times are UTC.

[1.2] Staff
  Practitioner 1 registered as ana@example.com
  Email "ana@example" refused: email address has an invalid format

[1.3] Time slots
  Slot 10:00-09:00 refused: time slot end must be after its start

[1.4] Appointment types
  Consultation lasts 30 min
  Follow-up lasts 15 min
  Check-up lasting 0 min refused: appointment type duration must be positive

[1.5] Working hours
  Practitioner 1 works 09:00-12:00
  Practitioner 1 works 13:00-17:00

[1.6] Booking
  Consultation at 09:00 booked for 09:00-09:30
  Follow-up at 09:15 refused: appointment overlaps an existing appointment (09:00-09:30)
  Consultation at 11:45 refused: appointment is outside the practitioner's working hours
  Follow-up at 13:00 booked for 13:00-13:15
  Agenda for practitioner 1:
    09:00-09:30 Consultation (Scheduled)
    13:00-13:15 Follow-up (Scheduled)

[1.7] Access
  Receptionist asks for clinical content: refused: role may not access clinical content
  Administrator asks for clinical content: refused: role may not access clinical content
  Administrator asks for the audit log: granted
  Practitioner 1 asks for clinical content: granted to practitioner 1
```

## Commits

- *(pending: added when the developer commits)*
