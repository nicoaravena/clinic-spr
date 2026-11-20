//! Adapters that keep everything in memory, for tests and for the first server.
//!
//! Nothing survives a restart, and nothing ever fails: every port method returns `Ok`. The database
//! adapters replace these.
//!
//! [`InMemoryStorage`] keeps its data behind one lock, and each [`InMemorySession`] holds that lock
//! from [`begin`](Storage::begin) until it's committed or dropped. Sessions therefore run one at a
//! time, so a use case's "read the schedule, book, save" can't interleave with another one's. A
//! task that begins a second session while it still holds the first waits for itself forever.
//!
//! The lock is tokio's, not the standard library's, because a session is held across the
//! `.await`s of a use case. A [`std::sync::MutexGuard`] isn't `Send`, so a handler holding one
//! across an `.await` couldn't move between threads and axum would refuse it. Waiting for tokio's
//! lock also lets the thread serve other requests meanwhile, instead of blocking it.
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
use std::fmt;
use std::sync::Arc;

use clinic_domain::appointment::AppointmentType;
use clinic_domain::ids::{ClinicId, PractitionerId};
use clinic_domain::schedule::Schedule;
use thiserror::Error;
use tokio::sync::{Mutex, OwnedMutexGuard};

use crate::ports::{AppointmentTypes, ScheduleRepository, Session, Storage, StorageError};

/// A [`Storage`] backed by `HashMap`s, for every clinic.
///
/// Cloning it is cheap, and every clone sees the same data.
#[derive(Debug, Clone, Default)]
pub struct InMemoryStorage {
    data: Arc<Mutex<Data>>,
}

/// What an [`InMemoryStorage`] keeps, behind its lock.
#[derive(Debug, Default)]
struct Data {
    /// The key includes the clinic, so the same practitioner ID at two clinics names two
    /// schedules.
    schedules: HashMap<(ClinicId, PractitionerId), Schedule>,
    types: InMemoryAppointmentTypes,
}

impl InMemoryStorage {
    /// Storage with no schedules, whose clinics offer the appointment types in `types`.
    ///
    /// [`InMemoryStorage::default`] is storage in which no clinic offers anything.
    pub fn new(types: InMemoryAppointmentTypes) -> Self {
        Self {
            data: Arc::new(Mutex::new(Data {
                schedules: HashMap::new(),
                types,
            })),
        }
    }
}

impl Storage for InMemoryStorage {
    type Session = InMemorySession;

    /// Waits until no other session is open, then starts one at `clinic`.
    async fn begin(&self, clinic: ClinicId) -> Result<InMemorySession, StorageError> {
        // `lock_owned` locks a clone of the `Arc`, so the guard owns its share of the data instead
        // of borrowing `self`, and the session can outlive this call.
        let data = Arc::clone(&self.data).lock_owned().await;
        Ok(InMemorySession {
            clinic,
            data,
            pending: HashMap::new(),
        })
    }
}

/// A unit of work at one clinic in an [`InMemoryStorage`].
///
/// Saved schedules wait in the session until [`commit`](Session::commit) copies them into the
/// storage. There is no `impl Drop`: dropping the session drops what was waiting, and releases the
/// lock, which is all a rollback has to do here.
pub struct InMemorySession {
    clinic: ClinicId,
    data: OwnedMutexGuard<Data>,
    pending: HashMap<PractitionerId, Schedule>,
}

// Written out instead of derived: the guard's `Debug` would print the whole storage.
impl fmt::Debug for InMemorySession {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("InMemorySession")
            .field("clinic", &self.clinic)
            .field("pending", &self.pending)
            .finish_non_exhaustive()
    }
}

impl ScheduleRepository for InMemorySession {
    async fn get(&self, practitioner: PractitionerId) -> Result<Option<Schedule>, StorageError> {
        // What this session saved comes first: it's newer than what's in the storage.
        let schedule = match self.pending.get(&practitioner) {
            Some(schedule) => Some(schedule),
            None => self.data.schedules.get(&(self.clinic, practitioner)),
        };
        Ok(schedule.cloned())
    }

    async fn save(
        &mut self,
        practitioner: PractitionerId,
        schedule: Schedule,
    ) -> Result<(), StorageError> {
        self.pending.insert(practitioner, schedule);
        Ok(())
    }
}

impl AppointmentTypes for InMemorySession {
    async fn find(&self, name: &str) -> Result<Option<AppointmentType>, StorageError> {
        Ok(self.data.types.find(self.clinic, name))
    }
}

impl Session for InMemorySession {
    async fn commit(self) -> Result<(), StorageError> {
        // Taking the session apart is allowed because it has no `impl Drop`. The guard is dropped
        // at the end of the function, which releases the lock.
        let Self {
            clinic,
            mut data,
            pending,
        } = self;
        data.schedules.extend(
            pending
                .into_iter()
                .map(|(practitioner, schedule)| ((clinic, practitioner), schedule)),
        );
        Ok(())
    }
}

/// The appointment types each clinic offers, with names unique within each clinic.
///
/// It's what an [`InMemoryStorage`] is created with. Sessions look types up in it through the
/// [`AppointmentTypes`] port.
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

    /// The type named exactly `name` at `clinic`, if it offers one.
    fn find(&self, clinic: ClinicId, name: &str) -> Option<AppointmentType> {
        self.types.get(&clinic).and_then(|offered| {
            offered
                .iter()
                .find(|appointment_type| appointment_type.name() == name)
                .cloned()
        })
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
