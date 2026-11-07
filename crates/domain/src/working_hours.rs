//! The dated shifts a practitioner works, in UTC.
//!
//! Time zones stay outside the domain. Whatever creates the shifts converts them to UTC first.
//!
//! A slot fits only if a single shift contains it, so shifts must be separated by a gap: 09-12 and
//! 12-17 must be written as 09-17.
//!
//! The fields are private, so the only way to build working hours is through
//! [`WorkingHours::new`]:
//!
//! ```compile_fail,E0451
//! use clinic_domain::working_hours::WorkingHours;
//!
//! WorkingHours { shifts: Vec::new() };
//! ```

use std::error::Error;
use std::fmt;

use crate::time_slot::TimeSlot;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkingHours {
    shifts: Vec<TimeSlot>,
}

impl WorkingHours {
    pub fn new(mut shifts: Vec<TimeSlot>) -> Result<Self, WorkingHoursError> {
        shifts.sort_by_key(|shift| shift.start());

        if shifts
            .windows(2)
            .any(|pair| pair[1].start() <= pair[0].end())
        {
            return Err(WorkingHoursError::ShiftsNotSeparated);
        }

        Ok(Self { shifts })
    }

    /// The shifts, sorted by start.
    pub fn shifts(&self) -> &[TimeSlot] {
        &self.shifts
    }

    pub fn contains(&self, slot: &TimeSlot) -> bool {
        self.shifts.iter().any(|shift| shift.contains(slot))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkingHoursError {
    ShiftsNotSeparated,
}

impl fmt::Display for WorkingHoursError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::ShiftsNotSeparated => "working hours shifts must not overlap or touch",
        };
        f.write_str(message)
    }
}

impl Error for WorkingHoursError {}
