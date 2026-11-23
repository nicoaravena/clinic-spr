# db (crate)

Postgres storage through sqlx: the connection pool, migrations, the platform
schema, and the names of clinic schemas.

- **Source:** `crates/db/src/`, migrations in `crates/db/migrations/`
- **Introduced in:** [step 3.3](../steps/3.3-db-crate.md)
- **API docs:** [`clinic_db`][api]

## Modules and public items

| Module | Items |
|---|---|
| `pool` | `connect(url)` |
| `migrate` | `PLATFORM_MIGRATOR`, `migrate_platform(pool)` |
| `platform` | `Platform` (`new`, `clinic`) |
| `schema` | `SchemaName` (`From<ClinicId>`, `as_str`, `Display`) |

## Database layout

| Schema | Holds | Created by |
|---|---|---|
| `platform` | `clinics`: what the platform knows about every clinic | platform migration `0001` |
| `public` | `_sqlx_migrations`: which platform migrations ran | `migrate_platform` |
| `clinic_<id>` | one clinic's own data | provisioning, from [step 3.4](../roadmap.md) |

`platform.clinics`:

| Column | Type | Rule |
|---|---|---|
| `id` | `bigint`, primary key | a `ClinicId`; `>= 0` |
| `slug` | `text`, unique | same rule as `Slug::parse` |
| `schema_name` | `text` | always `'clinic_' \|\| id` |
| `status` | `text`, default `'active'` | `'active'` or `'suspended'` |
| `created_at` | `timestamptz`, default `now()` | |

## Rules and how they are enforced

| Rule | How |
|---|---|
| A clinic's schema name never comes from request input | `SchemaName` has a private field and is built only `From<ClinicId>`; `compile_fail` doctest (E0423) |
| A schema name never needs quoting and fits Postgres's 63-byte limit | Always `clinic_` and digits, at most 27 bytes; tested up to `u64::MAX` |
| The recorded schema name follows from the id | `CHECK (schema_name = 'clinic_' \|\| id)`; tested |
| Slugs are unique and well formed, even for rows not written by this crate | `UNIQUE` and a `CHECK` with the same rule as the domain; tested |
| Status is one the domain knows | `CHECK (status IN ('active', 'suspended'))`; an unknown text read back is a `StorageError` |
| Queries match the schema | `query!` is checked against the database (or the `.sqlx` cache) at compile time |
| Every failure is a `StorageError` | `connect`, `migrate_platform` and `Platform::clinic` map sqlx errors with `StorageError::new`; tested with a closed pool and a closed port |
| Running the migrations again is harmless | sqlx's `_sqlx_migrations` table; tested |
| A green test run includes the database tests | `#[sqlx::test]` panics without Postgres instead of skipping |

## Semantics

`connect(url)` opens a pool of up to 10 connections and opens one at once.
A caller waits up to 5 seconds for a connection before getting a
`StorageError`, which also applies to a server that refuses connections.

`Platform::clinic(id)` returns `Ok(None)` for an id that isn't in the table,
including any id above `i64::MAX`, which can't be stored in a `bigint`. A
row that doesn't make a valid `Clinic` is a `StorageError`.

## Working with the database

| Task | Command |
|---|---|
| Start Postgres | `docker compose up -d --wait` |
| Point sqlx at it | `cp .env.example .env` |
| Migrate the development database (`query!` checks against it) | `cargo sqlx migrate run --source crates/db/migrations/platform` |
| Refresh the offline cache after changing a query or migration | `cargo sqlx prepare --workspace` |
| Check the cache is current | `cargo sqlx prepare --workspace --check` |
| Build without a database | `SQLX_OFFLINE=true cargo build` |

sqlx-cli is installed once with
`cargo install sqlx-cli@0.9.1 --locked --no-default-features --features postgres,rustls`.

## Not yet

- No schema per clinic yet, and no tenant transactions: step 3.4.
- No adapters for the ports: step 3.5.
- The server doesn't use this crate until step 3.7.
- No `plan` column. It comes when a plan changes behavior.

[api]: ../../../target/doc/clinic_db/index.html
