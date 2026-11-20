//! One practitioner's working hours and the appointments booked in them.
//!
//! [`Schedule::book`] is the only way to add an appointment. It accepts one only if it fits inside
//! a single shift and doesn't overlap an appointment already booked.
//!
//! The fields are private, so the only way to build a schedule is through [`Schedule::new`]:
//!
//! ```compile_fail,E0451
//! use clinic_domain::schedule::Schedule;
//! use clinic_domain::working_hours::WorkingHours;
//!
//! Schedule {
//!     hours: WorkingHours::new(Vec::new()).unwrap(),
//!     appointments: Vec::new(),
//! };
//! ```

use chrono::{DateTime, Utc};
use thiserror::Error;

use crate::appointment::{Appointment, AppointmentType};
use crate::ids::PatientId;
use crate::time_slot::TimeSlot;
use crate::working_hours::WorkingHours;

/// One practitioner's working hours and their appointments, sorted by start.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Schedule {
    hours: WorkingHours,
    appointments: Vec<Appointment>,
}

impl Schedule {
    /// An empty schedule for the given working hours.
    pub fn new(hours: WorkingHours) -> Self {
        Self {
            hours,
            appointments: Vec::new(),
        }
    }

    /// The working hours appointments must fit in.
    pub fn hours(&self) -> &WorkingHours {
        &self.hours
    }

    /// Replaces the working hours, keeping every booked appointment.
    ///
    /// Fails if a booked appointment wouldn't fit inside a single shift of `hours`, naming the
    /// earliest one. On failure the schedule is unchanged.
    pub fn replace_hours(&mut self, hours: WorkingHours) -> Result<(), ReplaceHoursError> {
        if let Some(outside) = self
            .appointments
            .iter()
            .find(|appointment| !hours.contains(&appointment.slot()))
        {
            return Err(ReplaceHoursError::AppointmentOutsideHours {
                existing: outside.slot(),
            });
        }

        self.hours = hours;
        Ok(())
    }

    /// The booked appointments, sorted by start.
    pub fn appointments(&self) -> &[Appointment] {
        &self.appointments
    }

    /// The booked appointments that overlap `range`, sorted by start.
    ///
    /// The range is half-open like every [`TimeSlot`]: an appointment that ends exactly when the
    /// range starts, or starts exactly when it ends, isn't in it. One that straddles either edge is.
    pub fn appointments_in(&self, range: TimeSlot) -> impl Iterator<Item = &Appointment> {
        // Appointments are sorted by start and never overlap, so their ends are sorted too. Every
        // appointment that ends before the range comes first, and every one that starts after it
        // comes last, so the walk can stop at the first one past the end.
        self.appointments
            .iter()
            .skip_while(move |appointment| appointment.slot().end() <= range.start())
            .take_while(move |appointment| appointment.slot().start() < range.end())
    }

    /// Books an appointment of `appointment_type` for `patient`, starting at `start`.
    ///
    /// The checks run in this order, and the first one that fails decides the error:
    ///
    /// 1. The appointment must end within the range of dates chrono can represent.
    /// 2. It must fit inside a single shift of the working hours.
    /// 3. It must not overlap an appointment already booked.
    ///
    /// On failure the schedule is unchanged.
    pub fn book(
        &mut self,
        patient: PatientId,
        start: DateTime<Utc>,
        appointment_type: AppointmentType,
    ) -> Result<&Appointment, BookingError> {
        let slot = appointment_type
            .slot_starting_at(start)
            .ok_or(BookingError::OutOfRange)?;

        if !self.hours.contains(&slot) {
            return Err(BookingError::OutsideWorkingHours);
        }
        if let Some(existing) = self
            .appointments
            .iter()
            .find(|appointment| appointment.slot().overlaps(&slot))
        {
            return Err(BookingError::Overlaps {
                existing: existing.slot(),
            });
        }

        let index = self
            .appointments
            .partition_point(|appointment| appointment.slot().start() <= slot.start());
        self.appointments.insert(
            index,
            Appointment::scheduled(patient, appointment_type, slot),
        );

        Ok(&self.appointments[index])
    }
}

/// Why [`Schedule::book`] refused a booking.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum BookingError {
    /// The appointment would end after the latest time chrono can represent.
    #[error("appointment would end after the latest supported time")]
    OutOfRange,
    /// No single shift of the working hours contains the appointment.
    #[error("appointment is outside the practitioner's working hours")]
    OutsideWorkingHours,
    /// The appointment overlaps one already booked.
    #[error("appointment overlaps an existing appointment")]
    Overlaps {
        /// The slot of the appointment already booked.
        existing: TimeSlot,
    },
}

/// Why [`Schedule::replace_hours`] refused new working hours.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum ReplaceHoursError {
    /// A booked appointment wouldn't fit inside a single shift of the new hours.
    #[error("an existing appointment would be outside the new working hours")]
    AppointmentOutsideHours {
        /// The slot of the earliest such appointment.
        existing: TimeSlot,
    },
}
