//! What the use cases need from storage, as traits.
//!
//! Every method takes the [`ClinicId`] first. An adapter must keep each clinic's data separate:
//! what was saved for one clinic is never returned for another, even under the same
//! [`PractitionerId`].
//!
//! Every method is asynchronous and can fail with a [`StorageError`], because a database adapter
//! waits on the network and the network can fail. The in-memory adapters never fail.
//!
//! Each method returns `impl Future<Output = ...> + Send`, which says that the future can move
//! between threads, as a multi-threaded runtime such as axum's needs. An implementation can still
//! write it as a plain `async fn`, and the compiler checks that its future is `Send`.
//!
//! A trait with methods like these can't be used as a trait object, because each implementation
//! returns a future of a different type:
//!
//! ```compile_fail,E0038
//! use clinic_app::in_memory::InMemorySchedules;
//! use clinic_app::ports::ScheduleRepository;
//!
//! let schedules: Box<dyn ScheduleRepository> = Box::new(InMemorySchedules::new());
//! ```

use std::error::Error;

use clinic_domain::appointment::AppointmentType;
use clinic_domain::ids::{ClinicId, PractitionerId};
use clinic_domain::schedule::Schedule;
use thiserror::Error;

/// Where each practitioner's [`Schedule`] is kept.
pub trait ScheduleRepository {
    /// A copy of the practitioner's schedule at `clinic`, or `None` if there is none.
    fn get(
        &self,
        clinic: ClinicId,
        practitioner: PractitionerId,
    ) -> impl Future<Output = Result<Option<Schedule>, StorageError>> + Send;

    /// Stores `schedule` as the practitioner's schedule at `clinic`, replacing any earlier one.
    ///
    /// On failure the earlier schedule is kept.
    fn save(
        &mut self,
        clinic: ClinicId,
        practitioner: PractitionerId,
        schedule: Schedule,
    ) -> impl Future<Output = Result<(), StorageError>> + Send;
}

/// The appointment types each clinic offers.
pub trait AppointmentTypes {
    /// The type named exactly `name` at `clinic`, or `None` if the clinic doesn't offer it.
    fn find(
        &self,
        clinic: ClinicId,
        name: &str,
    ) -> impl Future<Output = Result<Option<AppointmentType>, StorageError>> + Send;
}

/// A port couldn't read or write its storage.
///
/// The message is always "storage failed". What went wrong, such as a refused connection, is the
/// error's [`source`](Error::source): it belongs in the server's logs, not in an answer to a client.
#[derive(Debug, Error)]
#[error("storage failed")]
pub struct StorageError {
    #[source]
    source: Box<dyn Error + Send + Sync>,
}

impl StorageError {
    /// A storage failure caused by `source`: an error from a database driver, or just a message
    /// such as `"connection refused"`.
    pub fn new(source: impl Into<Box<dyn Error + Send + Sync>>) -> Self {
        Self {
            source: source.into(),
        }
    }
}
