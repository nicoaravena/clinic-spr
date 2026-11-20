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

use chrono::{DateTime, Utc};

use crate::time_slot::TimeSlot;

/// The shifts one practitioner works, sorted and separated by gaps.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkingHours {
    shifts: Vec<TimeSlot>,
}

impl WorkingHours {
    /// Sorts the shifts by start, then fails if any two overlap or touch.
    ///
    /// An empty list is allowed and accepts no slot.
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

    /// Whether a single shift fully contains `slot`.
    pub fn contains(&self, slot: &TimeSlot) -> bool {
        self.shifts.iter().any(|shift| shift.contains(slot))
    }

    /// These hours up to `cut`, and `replacement`'s hours from `cut` on.
    ///
    /// A shift of either that straddles `cut` is cut short there. If what's left of the two meets
    /// exactly at `cut`, it becomes one shift, so the result is always sorted and separated.
    pub fn replace_from(&self, cut: DateTime<Utc>, replacement: &WorkingHours) -> WorkingHours {
        // `TimeSlot::new` only fails for an empty piece, which the filters have already left out.
        let before = self
            .shifts
            .iter()
            .filter(|shift| shift.start() < cut)
            .map(|shift| TimeSlot::new(shift.start(), shift.end().min(cut)));
        let after = replacement
            .shifts
            .iter()
            .filter(|shift| shift.end() > cut)
            .map(|shift| TimeSlot::new(shift.start().max(cut), shift.end()));

        let mut shifts: Vec<TimeSlot> = Vec::new();
        for shift in before.chain(after) {
            let shift = shift.expect("a piece on one side of the cut is never empty");
            match shifts.last_mut() {
                Some(previous) if previous.end() == shift.start() => {
                    *previous = TimeSlot::new(previous.start(), shift.end())
                        .expect("a shift's end is after its start");
                }
                _ => shifts.push(shift),
            }
        }

        // Each side was sorted and separated, the sides don't overlap, and a meeting at `cut` was
        // merged above.
        Self { shifts }
    }
}

/// Why [`WorkingHours::new`] rejected its input.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkingHoursError {
    /// Two shifts overlap, or one ends exactly when the next starts.
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
