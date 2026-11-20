//! What the server knows when it starts.
//!
//! There's no endpoint yet to set a practitioner's hours or a clinic's appointment types, and no
//! database, so the server starts with fixed data in the in-memory adapters. Step 2.6 adds an
//! endpoint for the hours, and M3 moves storage to Postgres.

use std::error::Error;

use chrono::{DateTime, TimeDelta, TimeZone, Utc};
use clinic_api::state::AppState;
use clinic_app::in_memory::{InMemoryAppointmentTypes, InMemorySchedules};
use clinic_app::ports::ScheduleRepository;
use clinic_domain::appointment::AppointmentType;
use clinic_domain::ids::{ClinicId, PractitionerId};
use clinic_domain::schedule::Schedule;
use clinic_domain::time_slot::TimeSlot;
use clinic_domain::working_hours::WorkingHours;

/// The demo clinics:
///
/// - Clinic 1 offers a 30-minute "Consultation" and a 15-minute "Follow-up". Its practitioner 1
///   works 09:00-12:00 and 13:00-17:00 UTC on Monday 2026-03-02, with nothing booked.
/// - Clinic 2 offers "Consultation" too, but has no practitioners. It shows that one clinic can't
///   reach another's data.
pub(crate) fn demo_state() -> Result<AppState, Box<dyn Error>> {
    let first = ClinicId::new(1);
    let second = ClinicId::new(2);
    let consultation = AppointmentType::new("Consultation", TimeDelta::minutes(30))?;
    let follow_up = AppointmentType::new("Follow-up", TimeDelta::minutes(15))?;

    let mut types = InMemoryAppointmentTypes::new();
    types.add(first, consultation.clone())?;
    types.add(first, follow_up)?;
    types.add(second, consultation)?;

    let hours = WorkingHours::new(vec![
        TimeSlot::new(at(9, 0), at(12, 0))?,
        TimeSlot::new(at(13, 0), at(17, 0))?,
    ])?;
    let mut schedules = InMemorySchedules::new();
    schedules.save(first, PractitionerId::new(1), Schedule::new(hours));

    Ok(AppState::new(schedules, types))
}

/// A time on Monday 2026-03-02, in UTC.
fn at(hour: u32, minute: u32) -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 3, 2, hour, minute, 0).unwrap()
}
