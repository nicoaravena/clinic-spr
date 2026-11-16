//! Adapters that keep everything in memory, for tests and for the first server.
//!
//! Nothing survives a restart. The database adapters replace these.
//!
//! The fields are private, so the only way to add an appointment type is
//! [`InMemoryAppointmentTypes::add`], which keeps names unique within a clinic:
//!
//! ```compile_fail,E0451
//! use clinic_app::in_memory::InMemoryAppointmentTypes;
//!
//! InMemoryAppointmentTypes {
//!     types: Default::default(),
//! };
//! ```

use std::collections::HashMap;

use clinic_domain::appointment::AppointmentType;
use clinic_domain::ids::{ClinicId, PractitionerId};
use clinic_domain::schedule::Schedule;
use thiserror::Error;

use crate::ports::{AppointmentTypes, ScheduleRepository};

/// A [`ScheduleRepository`] backed by a `HashMap`.
///
/// The key includes the clinic, so the same practitioner ID at two clinics names two schedules.
#[derive(Debug, Clone, Default)]
pub struct InMemorySchedules {
    schedules: HashMap<(ClinicId, PractitionerId), Schedule>,
}

impl InMemorySchedules {
    /// A repository with no schedules.
    pub fn new() -> Self {
        Self::default()
    }
}

impl ScheduleRepository for InMemorySchedules {
    fn get(&self, clinic: ClinicId, practitioner: PractitionerId) -> Option<Schedule> {
        self.schedules.get(&(clinic, practitioner)).cloned()
    }

    fn save(&mut self, clinic: ClinicId, practitioner: PractitionerId, schedule: Schedule) {
        self.schedules.insert((clinic, practitioner), schedule);
    }
}

/// An [`AppointmentTypes`] catalog backed by a `HashMap`, with names unique within each clinic.
#[derive(Debug, Clone, Default)]
pub struct InMemoryAppointmentTypes {
    types: HashMap<ClinicId, Vec<AppointmentType>>,
}

impl InMemoryAppointmentTypes {
    /// A catalog in which no clinic offers anything.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds `appointment_type` to what `clinic` offers.
    ///
    /// Fails, leaving the catalog unchanged, if the clinic already has a type with the same name.
    /// Names are compared exactly, so "Consultation" and "consultation" are different types.
    pub fn add(
        &mut self,
        clinic: ClinicId,
        appointment_type: AppointmentType,
    ) -> Result<(), DuplicateAppointmentType> {
        let offered = self.types.entry(clinic).or_default();

        if offered
            .iter()
            .any(|existing| existing.name() == appointment_type.name())
        {
            return Err(DuplicateAppointmentType {
                name: appointment_type.name().to_owned(),
            });
        }

        offered.push(appointment_type);
        Ok(())
    }
}

impl AppointmentTypes for InMemoryAppointmentTypes {
    fn find(&self, clinic: ClinicId, name: &str) -> Option<AppointmentType> {
        self.types
            .get(&clinic)?
            .iter()
            .find(|appointment_type| appointment_type.name() == name)
            .cloned()
    }
}

/// Why [`InMemoryAppointmentTypes::add`] refused a type.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("clinic already has an appointment type named {name:?}")]
pub struct DuplicateAppointmentType {
    name: String,
}

impl DuplicateAppointmentType {
    /// The name that was already taken.
    pub fn name(&self) -> &str {
        &self.name
    }
}
