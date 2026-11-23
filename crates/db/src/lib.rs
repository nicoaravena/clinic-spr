//! Postgres storage for the clinic platform, through sqlx.
//!
//! The database has one *platform* schema, `platform`, with what the platform knows about every
//! clinic, and, from step 3.4, one schema per clinic, `clinic_<id>`, with that clinic's own data.
//! This crate is an adapter: it implements what the use cases need from storage, and every failure
//! is a [`StorageError`].
//!
//! - [`pool::connect`] opens the connection pool.
//! - [`migrate::migrate_platform`] brings the platform schema up to date.
//! - [`platform::Platform`] reads `platform.clinics`.
//! - [`schema::SchemaName`] is the name of a clinic's schema, derived from its
//!   [`ClinicId`](clinic_domain::ids::ClinicId) and never from request input.
//!
//! [`StorageError`]: clinic_app::ports::StorageError

pub mod migrate;
pub mod platform;
pub mod pool;
pub mod schema;
