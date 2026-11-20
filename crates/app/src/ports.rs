//! What the use cases need from storage, as traits.
//!
//! Storage is reached through a *session*: a unit of work at one clinic. A use case asks a
//! [`Storage`] to [`begin`](Storage::begin) a session for a [`ClinicId`], reads and writes through
//! the session's ports, and ends it with [`commit`](Session::commit). Its changes are kept only if
//! it commits. A session that is dropped instead, because a check failed or an error was returned
//! early, keeps nothing: dropping it is the rollback. Until the session ends, what it saved is seen
//! only by itself.
//!
//! The clinic is named once, when the session begins, and the port methods don't take it. A session
//! never returns data saved for another clinic, even under the same [`PractitionerId`].
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
//! use clinic_app::ports::ScheduleRepository;
//!
//! fn read_through(_schedules: &dyn ScheduleRepository) {}
//! ```
//!
//! [`commit`](Session::commit) takes the session by value, so once it's committed it can't be used
//! again:
//!
//! ```compile_fail,E0382
//! use clinic_app::in_memory::InMemoryStorage;
//! use clinic_app::ports::{ScheduleRepository, Session, Storage};
//! use clinic_domain::ids::{ClinicId, PractitionerId};
//!
//! async fn twice(storage: &InMemoryStorage) {
//!     let session = storage.begin(ClinicId::new(1)).await.unwrap();
//!     session.commit().await.unwrap();
//!     session.get(PractitionerId::new(1)).await.unwrap();
//! }
//! ```

use std::error::Error;

use clinic_domain::appointment::AppointmentType;
use clinic_domain::ids::{ClinicId, PractitionerId};
use clinic_domain::schedule::Schedule;
use thiserror::Error;

/// Where sessions come from: the whole storage, for every clinic.
pub trait Storage {
    /// The kind of session this storage begins. Each adapter has its own, such as one that wraps
    /// a database transaction.
    type Session: Session;

    /// Starts a unit of work at `clinic`.
    ///
    /// The session may hold resources, such as a lock or a database connection, until it's
    /// committed or dropped, so a task should end one session before it begins another.
    fn begin(
        &self,
        clinic: ClinicId,
    ) -> impl Future<Output = Result<Self::Session, StorageError>> + Send;
}

/// A unit of work at one clinic, through which the use cases read and write.
///
/// Changes are kept only if [`commit`](Self::commit) succeeds. Dropping a session without
/// committing it discards them.
pub trait Session: ScheduleRepository + AppointmentTypes + Send {
    /// Keeps every change made in this session, and ends it.
    ///
    /// On failure nothing is kept.
    fn commit(self) -> impl Future<Output = Result<(), StorageError>> + Send;
}

/// Where each practitioner's [`Schedule`] is kept.
pub trait ScheduleRepository {
    /// A copy of the practitioner's schedule, or `None` if there is none.
    fn get(
        &self,
        practitioner: PractitionerId,
    ) -> impl Future<Output = Result<Option<Schedule>, StorageError>> + Send;

    /// Stores `schedule` as the practitioner's schedule, replacing any earlier one.
    ///
    /// On failure the earlier schedule is kept.
    fn save(
        &mut self,
        practitioner: PractitionerId,
        schedule: Schedule,
    ) -> impl Future<Output = Result<(), StorageError>> + Send;
}

/// The appointment types a clinic offers.
pub trait AppointmentTypes {
    /// The type named exactly `name`, or `None` if the clinic doesn't offer it.
    fn find(
        &self,
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
