# Roadmap: learn Rust by building the clinic platform

## Context

`README.md` describes a multi-tenant clinic platform. The main goal is learning Rust; the app is the
vehicle. Work moves in small steps, each checked before the next one starts.

Decisions:
- **Way of working:** the agent writes the code and explains it; the developer reviews. Changed from
  "developer writes, agent guides" after step 1.1.
- **Interface:** HTTP JSON API only.
- **Isolation:** one Postgres schema per clinic.
- **Stack:** axum + tokio + sqlx + chrono.
- **Layout:** Cargo workspace whose crate boundaries keep the domain free of I/O.
- **Order:** tests before code, and pure domain logic before async and SQL.
- **Time:** the domain works only in UTC. A clinic, its staff and its patients can each be in a
  different time zone, so time zones are handled at the edges.
- **Ports:** repository traits are sync in M2. They become async in M3, when sqlx needs it.
- **Tenant over HTTP:** until login arrives in M4, a request names its clinic in an `X-Clinic-Id`
  header. In M4 the token takes its place and the routes don't change.

## Progress

Current step: **3.1** (M2 is done; 3.1 has not started)

## How each step works

1. **The agent writes the tests first**, then the code, keeping each step small enough to read in
   one sitting.
2. **The agent extends the running clinic:** `crates/clinic` is the server, and
   `crates/clinic/tests/walkthrough.rs` plays a curl walkthrough against it that grows with every
   step. The agent adds requests for what the new step makes possible, runs
   `UPDATE_EXPECTED=1 cargo test -p clinic --test walkthrough`, and reviews the diff of
   `crates/clinic/tests/expected-output.txt`. That diff shows what changed in the running app.
   (Until step 2.5, `main` played a scripted day and the same file held its output.)
3. **The agent runs the checks:** `cargo fmt --check`,
   `cargo clippy --all-targets -- -D warnings`, `cargo test`, and confirms the expected tests ran.
4. **The agent explains the step:** what changed, the Rust concepts involved, pointers to The Rust
   Book, and the design choices worth questioning.
5. **The agent records the step in the book** (`docs/src/`): a step page in `steps/`, a new or
   updated page in `reference/`, glossary terms, and doc comments on new public items. Then it
   checks that `mdbook build docs` and `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps` pass.
6. **The developer reviews** the code and asks for changes.
7. **The developer commits** under their own name, with Conventional Commits.
8. **Hard stop.** The next step starts only when the developer asks for it.

## Target structure (grows as needed, not scaffolded up front)

```text
Cargo.toml              workspace
crates/domain/          entities, value objects, rules; no I/O dependencies
crates/app/             use cases and repository traits (from M2)
crates/db/              sqlx adapters, migrations, tenant access (from M3)
crates/api/             axum handlers, as a library (from M2)
crates/clinic/          the binary and composition root: a scripted day until 2.5, then the server
docs/ROADMAP.md         this file
docs/book.toml, src/    the learning book (mdBook): step journal, reference, glossary
```

Dependencies point inward: `api -> app -> domain`, and `db -> app -> domain`. Only `clinic` depends
on all of them, to wire them together.

## Milestones

### M1: Domain rules (no async, no database)

| Step | Build | Rust concepts |
|---|---|---|
| 1.1 | `git init`, workspace, `domain` crate, one test | cargo, crates, modules, `#[test]` |
| 1.2 | `ClinicId`, `PractitionerId`, `Email` with validation | structs, newtypes, privacy, `Result`, derive |
| 1.3 | `TimeSlot` (start before end, `overlaps`) | borrowing, `&self`, operators, chrono |
| 1.4 | Clinic-defined `AppointmentType` with a duration, `AppointmentStatus` | enums, `match`, `Option` |
| 1.5 | `WorkingHours` as dated UTC shifts (slot fits within a practitioner's hours) | `Vec`, slices, iterators, closures |
| 1.6 | `Schedule::book` (no overlap, within hours) | `thiserror`, `?`, ownership in APIs |
| 1.7 | `Role` and permissions (only practitioners touch clinical notes) | traits, types that make bad states unrepresentable |
| 1.8 | `ClinicalNote` that keeps versions on correction | lifetimes, immutability by design |

### M2: First HTTP slice (in-memory storage)

Delivers `POST /appointments` and `GET /practitioners/{id}/agenda`, working through curl.

| Step | Build | Rust concepts |
|---|---|---|
| 2.1 | `app` crate: `ScheduleRepository` and `AppointmentTypes` ports, in-memory adapters, `book_appointment` use case; appointments record their patient | traits as ports, generics vs `dyn`, `#[from]` error wrapping, `HashMap` |
| 2.2 | `practitioner_agenda` query (appointments in a UTC range) | iterators over borrowed data, `impl Trait` returns |
| 2.3 | Weekly template to dated UTC shifts in the clinic's time zone, daylight saving included (adds `chrono-tz`) | `chrono-tz`, `LocalResult`, ambiguous and missing local times |
| 2.4 | `api` crate: axum, `POST /appointments` with `X-Clinic-Id`, serde DTOs, domain errors to HTTP status codes | async/await, tokio, `Arc<Mutex<_>>`, `Send`/`Sync`, extractors, serde |
| 2.5 | `GET /practitioners/{id}/agenda`; the `clinic` binary serves HTTP; the scripted day becomes a curl walkthrough | composition root, graceful shutdown, integration tests over HTTP |
| 2.6 | Endpoint to set a practitioner's weekly hours: a time zone name parsed to a `Tz`, expanded over a fixed horizon into their schedule | parsing at the edge, path extractors, `PUT` semantics |

In step 2.5 the `clinic` binary switches from its scripted day to serving HTTP, and the scripted
day becomes a curl walkthrough, checked by a test that runs it against the real binary.
Concepts: traits as ports, generics vs `dyn`, `Arc`, `Mutex`, async/await, axum extractors, serde,
and mapping domain errors to HTTP status codes.
Appointment types are defined per clinic, so this milestone also needs somewhere to store and look
them up.
`WorkingHours` holds dated UTC shifts, so something outside the domain must create them, for example
by expanding a weekly template using the time zone the hours were defined in. That's where daylight
saving changes must be handled correctly.

### M3: Postgres with one schema per clinic

- **Platform schema:** `platform.clinics` holds id, slug, schema name, plan and status.
- **Tenant schema:** each clinic gets `clinic_<id>` with its own tables. Provisioning a clinic
  creates the schema and runs the tenant migrations.
- **Enforced access:** the only way to get a tenant connection is `TenantDb::begin(&ClinicId)`,
  which returns a `TenantTx` that runs `SET LOCAL search_path` inside a transaction. Repositories
  accept only `&mut TenantTx`, so forgetting the tenant is a compile error. `SET LOCAL` also stops
  the setting leaking through the connection pool.
- **Schema names:** these come from a validated type and never from request input, because SQL
  identifiers can't be bound as parameters and would otherwise allow injection.
- **Overlap guard:** a Postgres exclusion constraint (`btree_gist`) backs up the domain rule, so two
  concurrent bookings can't both succeed.

Concepts: sqlx, compile-time checked queries, transactions, integration tests, Docker Compose.
**Key test:** data written for clinic A can't be seen from clinic B.

Delivers the same three endpoints as M2, backed by Postgres: the walkthrough runs against a real
database and its output doesn't change, except where M3 adds behavior (unknown clinics, the
overlap guard).

| Step | Build | Rust concepts |
|---|---|---|
| 3.1 | Ports become `async` and fallible (`StorageError`, mapped to 500); in-memory adapters and handlers follow. No behavior change | `async fn` in traits, `Send` bounds on returned futures, `tokio::sync::Mutex` vs `std` |
| 3.2 | Tenant sessions: `begin(ClinicId)` returns a session that implements the ports and ends with `commit`. Port methods lose their `ClinicId` argument. No behavior change | associated types, ownership of a unit of work, `Drop` as rollback |
| 3.3 | `db` crate and `compose.yaml` (Postgres): pool, platform migrations, `platform.clinics`, `SchemaName` derived from `ClinicId`. DB test harness | sqlx pool, `migrate!`, `query!` and the offline `.sqlx` cache, integration tests against Docker |
| 3.4 | Provisioning: create `clinic_<id>`, run tenant migrations, record per-schema migration status. `TenantDb::begin` and `TenantTx` with `SET LOCAL search_path`. **Isolation test** at the database level | transactions, `&mut` borrows that can't escape, identifiers that can't be bound |
| 3.5 | Postgres adapters for `ScheduleRepository` and `AppointmentTypes` inside a `TenantTx`. One contract test suite runs against both the in-memory and the Postgres adapters. Domain gains a way to rebuild a stored `Schedule` | row to domain mapping, `TryFrom`, generic test suites |
| 3.6 | Overlap guard: exclusion constraint (`btree_gist`) on appointments, the practitioner's row locked on load, `23P01` mapped to the overlap error (409). Concurrency test: of two parallel bookings for one slot, exactly one succeeds | database errors as domain errors, `tokio::join!`, races and how to test them |
| 3.7 | The server on Postgres: `DATABASE_URL`, platform migration and idempotent demo provisioning at startup (replaces `seed.rs`), unknown clinic is a 404, isolation shown over HTTP in the walkthrough. Milestone run with curl | configuration at the edge, startup that fails early, `Arc` without `Mutex` |
| 3.8 | GitLab CI job for fmt, clippy and tests with a Postgres service, next to the docs jobs | CI services, `SQLX_OFFLINE` |

Notes for the steps:

- **Why sessions (3.2) before the database:** today one `Mutex` around all storage makes "read the
  schedule, book, save" atomic. Postgres has no such lock, so the use case needs a transaction
  around the whole read-modify-write. The session gives it one, and makes the tenant part of the
  type instead of an argument that can be wrong. `api -> app` stays free of sqlx: the `db` crate
  implements the session by wrapping a `TenantTx`.
- **Saving a schedule:** `Schedule` is saved as a whole today. In Postgres that would delete and
  re-insert every appointment on each booking, which loses concurrent writes. The adapter instead
  inserts only new appointments and replaces only shifts from the changed date, under the row lock
  from 3.6.
- **Appointments have no ID yet.** The tables use a surrogate key that the domain doesn't see. An
  `AppointmentId` arrives when an endpoint needs to name one (cancel, reschedule).
- **Tenant migrations and `query!`:** sqlx's migrator records what it ran in `_sqlx_migrations` in
  the current `search_path`, so running it per schema gives per-schema status for free. It runs its
  own transactions, so it needs a connection with a session-level `search_path` that is closed
  afterwards, not `SET LOCAL`. Compile-time checks of tenant queries need a migrated template
  schema in the development database.
- **Tests need Postgres from 3.3 on:** `docker compose up -d` before `cargo test`. Database tests
  fail rather than skip when it's missing, so a green run always includes the isolation test.

### M4: Identity and access

Login with hashed passwords (argon2), a request principal, and tenant resolution from the token.
Receptionists are blocked from clinical content.
Concepts: custom axum extractors, middleware (tower), and secrets from environment variables.

### M5: Patient records and audit

Patients, append-only clinical notes, and an audit row for every read and change, written in the
same transaction. Includes the administrator's audit view.

### M6: Support plane

Separate routes and a separate principal type for onboarding, suspending clinics, changing plans and
managing clinic admin accounts. The support code gets no access to clinical repositories, and every
support action is recorded. Includes self-service clinic signup.

### M7: Data protection

Patient data export, deletion on request, and retention rules. **To decide when we get there:**
how deletion fits with records that must never be overwritten. It's a legal question as well as a
technical one.

### M8: Fairness and running costs

Per-clinic rate limits, connection limits, per-clinic usage metering, and structured logs. Includes
routing from `ClinicId` to a pool, so a large clinic can move to its own database.

## Risks

- **Schema per clinic costs more as clinics grow.** Each migration must run across every schema and
  can partly fail, so per-schema migration status has to be tracked from M3. Thousands of schemas
  also bloat the Postgres catalog and slow tooling. That's acceptable here, and M3 makes the limits
  visible.
- **Schema separation isn't a security boundary if everything shares one database role.** A bug that
  uses the wrong `ClinicId` still crosses clinics. `TenantDb` lowers that risk and M3 tests for it.
  Per-schema database roles are a possible hardening step later.
- **Scope is large.** M1 to M3 already cover most core Rust and the isolation design, so that's a
  natural point to reassess whether M4 to M8 are still worth doing for learning.
- **This isn't ready for real patient data.** Completing the roadmap doesn't replace a legal and
  security review, and only synthetic data should be used.

## Verification

- **Every step:** fmt, clippy and tests pass, the walkthrough shows the new behavior, and the
  code is reviewed.
- **M2 onward:** the milestone ends with a real run of the binary, exercised with curl.
- **M3 onward:** integration tests run against real Postgres, including the cross-clinic isolation
  test.
