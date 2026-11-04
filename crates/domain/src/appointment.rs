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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppointmentType {
    name: String,
    duration: TimeDelta,
}

impl AppointmentType {
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

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn duration(&self) -> TimeDelta {
        self.duration
    }

    /// Returns `None` if the slot would end outside the range of dates chrono can represent.
    pub fn slot_starting_at(&self, start: DateTime<Utc>) -> Option<TimeSlot> {
        let end = start.checked_add_signed(self.duration)?;

        TimeSlot::new(start, end).ok()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppointmentTypeError {
    EmptyName,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppointmentStatus {
    Scheduled,
    Completed,
    Cancelled,
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
