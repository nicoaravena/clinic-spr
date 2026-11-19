//! What every request handler shares: the storage behind the use cases.
//!
//! Handlers run concurrently, possibly on several threads, so the shared storage sits behind an
//! [`Arc`] (one copy, many owners) and a [`Mutex`] (one request at a time may touch it).

use std::fmt;
use std::sync::{Arc, Mutex, MutexGuard};

use clinic_app::ports::{AppointmentTypes, ScheduleRepository};

use crate::error::ApiError;

/// The router's state: the ports the use cases need, shared by every request.
///
/// Cloning it is cheap and every clone sees the same storage, which is what axum does for each
/// request.
#[derive(Clone)]
pub struct AppState {
    store: Arc<Mutex<Store>>,
}

impl AppState {
    /// State backed by the given adapters, such as the in-memory ones from `clinic_app`.
    ///
    /// The adapters must be `Send` so the storage can move between the threads that serve
    /// requests, and `'static` so it doesn't borrow anything that could go away first.
    pub fn new<S, T>(schedules: S, types: T) -> Self
    where
        S: ScheduleRepository + Send + 'static,
        T: AppointmentTypes + Send + 'static,
    {
        Self {
            store: Arc::new(Mutex::new(Store {
                schedules: Box::new(schedules),
                types: Box::new(types),
            })),
        }
    }

    /// Exclusive access to the storage until the guard is dropped.
    ///
    /// Fails if a request panicked while it held the lock, because the storage may then be
    /// half-changed.
    pub(crate) fn lock(&self) -> Result<MutexGuard<'_, Store>, ApiError> {
        self.store.lock().map_err(|_| ApiError::Internal)
    }
}

impl fmt::Debug for AppState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // The adapters are trait objects without `Debug`, so they aren't shown.
        f.debug_struct("AppState").finish_non_exhaustive()
    }
}

/// The adapters, as trait objects so the router doesn't depend on which ones they are.
pub(crate) struct Store {
    pub(crate) schedules: Box<dyn ScheduleRepository + Send>,
    pub(crate) types: Box<dyn AppointmentTypes + Send>,
}
