//! Helpers shared by the app's integration tests.

// Each test file is its own crate and uses a different subset of these helpers.
#![allow(dead_code)]

use clinic_app::in_memory::InMemoryStorage;
use clinic_app::ports::{ScheduleRepository, Session, Storage};
use clinic_domain::ids::{ClinicId, PractitionerId};
use clinic_domain::schedule::Schedule;

/// Saves `schedule` for `practitioner` at `clinic` in a session of its own, and commits it.
pub(crate) async fn save_committed(
    storage: &InMemoryStorage,
    clinic: ClinicId,
    practitioner: PractitionerId,
    schedule: Schedule,
) {
    let mut session = storage.begin(clinic).await.unwrap();
    session.save(practitioner, schedule).await.unwrap();
    session.commit().await.unwrap();
}

/// The practitioner's schedule at `clinic`, as a new session sees it.
pub(crate) async fn stored(
    storage: &InMemoryStorage,
    clinic: ClinicId,
    practitioner: PractitionerId,
) -> Option<Schedule> {
    let session = storage.begin(clinic).await.unwrap();
    session.get(practitioner).await.unwrap()
}
