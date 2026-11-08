//! Domain rules for the clinic platform: entities, value objects and the rules between them.
//!
//! This crate does no I/O. It has no async code, no database and no HTTP, so every rule can be
//! tested with plain values. All times are in UTC, and time zones are handled outside the domain.

pub mod appointment;
pub mod email;
pub mod ids;
pub mod schedule;
pub mod time_slot;
pub mod working_hours;
