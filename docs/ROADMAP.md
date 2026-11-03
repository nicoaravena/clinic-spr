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

## Progress

Current step: **1.2**

## How each step works

1. **The agent writes the tests first**, then the code, keeping each step small enough to read in
   one sitting.
2. **The agent runs the checks:** `cargo fmt --check`,
   `cargo clippy --all-targets -- -D warnings`, `cargo test`, and confirms the expected tests ran.
3. **The agent explains the step:** what changed, the Rust concepts involved, pointers to The Rust
   Book, and the design choices worth questioning.
4. **The developer reviews** the code and asks for changes.
5. **The developer commits** under their own name, with Conventional Commits.
6. **Hard stop.** The next step starts only when the developer asks for it.

## Target structure (grows as needed, not scaffolded up front)

```
Cargo.toml              workspace
crates/domain/          entities, value objects, rules; no I/O dependencies
crates/app/             use cases and repository traits (from M2)
crates/db/              sqlx adapters, migrations, tenant access (from M3)
crates/api/             axum handlers, the binary, composition root (from M2)
docs/ROADMAP.md         this file
```

Dependencies point inward: `api -> app -> domain`, and `db -> app -> domain`.

## Milestones

### M1: Domain rules (no async, no database)

| Step | Build | Rust concepts |
|---|---|---|
| 1.1 | `git init`, workspace, `domain` crate, one test | cargo, crates, modules, `#[test]` |
| 1.2 | `ClinicId`, `PractitionerId`, `Email` with validation | structs, newtypes, privacy, `Result`, derive |
| 1.3 | `TimeSlot` (start before end, `overlaps`) | borrowing, `&self`, operators, chrono |
| 1.4 | `AppointmentType` with a fixed duration, `AppointmentStatus` | enums, `match`, `Option` |
| 1.5 | `WorkingHours` (slot fits within a practitioner's hours) | `Vec`, slices, iterators, closures |
| 1.6 | `Schedule::book` (no overlap, within hours) | `thiserror`, `?`, ownership in APIs |
| 1.7 | `Role` and permissions (only practitioners touch clinical notes) | traits, types that make bad states unrepresentable |
| 1.8 | `ClinicalNote` that keeps versions on correction | lifetimes, immutability by design |

### M2: First HTTP slice (in-memory storage)

Delivers `POST /appointments` and `GET /practitioners/{id}/agenda`, working through curl.
Concepts: traits as ports, generics vs `dyn`, `Arc`, `Mutex`, async/await, axum extractors, serde,
and mapping domain errors to HTTP status codes.

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

- **Every step:** fmt, clippy and tests pass, and the code is reviewed.
- **M2 onward:** the milestone ends with a real run of the binary, exercised with curl.
- **M3 onward:** integration tests run against real Postgres, including the cross-clinic isolation
  test.
