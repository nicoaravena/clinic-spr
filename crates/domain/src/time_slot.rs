//! A span of time that always starts before it ends.
//!
//! The end is exclusive, so a slot ending at 09:30 doesn't overlap one starting at 09:30.
//!
//! The fields are private, so the only way to build a slot is through [`TimeSlot::new`]:
//!
//! ```compile_fail,E0451
//! use chrono::{TimeZone, Utc};
//! use clinic_domain::time_slot::TimeSlot;
//!
//! let start = Utc.with_ymd_and_hms(2026, 3, 2, 10, 0, 0).unwrap();
//! let end = Utc.with_ymd_and_hms(2026, 3, 2, 9, 0, 0).unwrap();
//!
//! TimeSlot { start, end };
//! ```

use std::error::Error;
use std::fmt;

use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TimeSlot {
    start: DateTime<Utc>,
    end: DateTime<Utc>,
}

impl TimeSlot {
    /// Fails unless `start` is strictly before `end`.
    pub fn new(start: DateTime<Utc>, end: DateTime<Utc>) -> Result<Self, TimeSlotError> {
        if start >= end {
            return Err(TimeSlotError::EndNotAfterStart);
        }

        Ok(Self { start, end })
    }

    /// When the slot starts (inclusive).
    pub fn start(&self) -> DateTime<Utc> {
        self.start
    }

    /// When the slot ends (exclusive).
    pub fn end(&self) -> DateTime<Utc> {
        self.end
    }

    /// Whether the two slots share any time. Back-to-back slots don't overlap.
    pub fn overlaps(&self, other: &TimeSlot) -> bool {
        self.start < other.end && other.start < self.end
    }

    /// Whether `other` lies entirely within this slot. A slot contains itself.
    pub fn contains(&self, other: &TimeSlot) -> bool {
        self.start <= other.start && other.end <= self.end
    }
}

/// Why [`TimeSlot::new`] rejected its input.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimeSlotError {
    /// The end was equal to or before the start.
    EndNotAfterStart,
}

impl fmt::Display for TimeSlotError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::EndNotAfterStart => "time slot end must be after its start",
        };
        f.write_str(message)
    }
}

impl Error for TimeSlotError {}
