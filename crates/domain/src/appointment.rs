//! What kind of appointment is booked, and where it is in its lifecycle.
//!
//! Each clinic defines its own appointment types. The fields are private, so the only way to build
//! a type is through [`AppointmentType::new`]:
//!
//! ```compile_fail,E0451
//! use chrono::TimeDelta;
//! use clinic_domain::appointment::AppointmentType;
//!
//! AppointmentType {
//!     name: String::new(),
//!     duration: TimeDelta::zero(),
//! };
//! ```

use std::error::Error;
use std::fmt;

use chrono::{DateTime, TimeDelta, Utc};

use crate::time_slot::TimeSlot;

/// A kind of appointment a clinic offers, such as "Consultation", with a fixed duration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppointmentType {
    name: String,
    duration: TimeDelta,
}

impl AppointmentType {
    /// Trims `name`, then fails if it is empty or if `duration` isn't positive.
    pub fn new(name: &str, duration: TimeDelta) -> Result<Self, AppointmentTypeError> {
        let name = name.trim();

        if name.is_empty() {
            return Err(AppointmentTypeError::EmptyName);
        }
        if duration <= TimeDelta::zero() {
            return Err(AppointmentTypeError::NonPositiveDuration);
        }

        Ok(Self {
            name: name.to_owned(),
            duration,
        })
    }

    /// The trimmed name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// How long an appointment of this type lasts. Always positive.
    pub fn duration(&self) -> TimeDelta {
        self.duration
    }

    /// The slot an appointment of this type would take if it started at `start`.
    ///
    /// Returns `None` if the slot would end outside the range of dates chrono can represent.
    pub fn slot_starting_at(&self, start: DateTime<Utc>) -> Option<TimeSlot> {
        let end = start.checked_add_signed(self.duration)?;

        TimeSlot::new(start, end).ok()
    }
}

/// Why [`AppointmentType::new`] rejected its input.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppointmentTypeError {
    /// The name was empty or only whitespace.
    EmptyName,
    /// The duration was zero or negative.
    NonPositiveDuration,
}

impl fmt::Display for AppointmentTypeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::EmptyName => "appointment type name is empty",
            Self::NonPositiveDuration => "appointment type duration must be positive",
        };
        f.write_str(message)
    }
}

impl Error for AppointmentTypeError {}

/// Where an appointment is in its lifecycle.
///
/// Every appointment starts as `Scheduled` and moves once to one of the final statuses.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppointmentStatus {
    /// Booked and not yet resolved.
    Scheduled,
    /// The patient was seen. Final.
    Completed,
    /// Called off before it happened. Final.
    Cancelled,
    /// The patient didn't turn up. Final.
    NoShow,
}

impl AppointmentStatus {
    /// Returns the new status, or `None` if the move isn't allowed.
    pub fn transition_to(self, next: Self) -> Option<Self> {
        match (self, next) {
            (Self::Scheduled, Self::Completed | Self::Cancelled | Self::NoShow) => Some(next),
            (Self::Scheduled, Self::Scheduled)
            | (Self::Completed | Self::Cancelled | Self::NoShow, _) => None,
        }
    }
}
