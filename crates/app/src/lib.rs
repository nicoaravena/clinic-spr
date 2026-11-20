//! Use cases for the clinic platform, and the ports they need from the outside world.
//!
//! A *port* is a trait that says what a use case needs, such as "get a practitioner's schedule",
//! without saying how it's done. An *adapter* implements a port. This crate ships in-memory
//! adapters; the database ones come later and live in their own crate.
//!
//! Like the domain, this crate does no I/O of its own. Every port method takes the [`ClinicId`] it
//! works in, so no use case can reach data without naming the clinic it belongs to. Port methods
//! are `async` and can fail with a [`StorageError`], so the use cases are `async` too and pass that
//! error on.
//!
//! [`StorageError`]: ports::StorageError
//!
//! [`ClinicId`]: clinic_domain::ids::ClinicId

pub mod agenda;
pub mod booking;
pub mod hours;
pub mod in_memory;
pub mod ports;
pub mod weekly_hours;
