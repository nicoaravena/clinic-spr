//! What the use cases need from storage, as traits.
//!
//! Every method takes the [`ClinicId`] first. An adapter must keep each clinic's data separate:
//! what was saved for one clinic is never returned for another, even under the same
//! [`PractitionerId`].
//!
//! The ports are synchronous for now. They become `async` when a database adapter needs it.

use clinic_domain::appointment::AppointmentType;
use clinic_domain::ids::{ClinicId, PractitionerId};
use clinic_domain::schedule::Schedule;

/// Where each practitioner's [`Schedule`] is kept.
pub trait ScheduleRepository {
    /// A copy of the practitioner's schedule at `clinic`, or `None` if there is none.
    fn get(&self, clinic: ClinicId, practitioner: PractitionerId) -> Option<Schedule>;

    /// Stores `schedule` as the practitioner's schedule at `clinic`, replacing any earlier one.
    fn save(&mut self, clinic: ClinicId, practitioner: PractitionerId, schedule: Schedule);
}

/// The appointment types each clinic offers.
pub trait AppointmentTypes {
    /// The type named exactly `name` at `clinic`, or `None` if the clinic doesn't offer it.
    fn find(&self, clinic: ClinicId, name: &str) -> Option<AppointmentType>;
}
