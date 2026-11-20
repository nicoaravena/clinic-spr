//! What every request handler shares: the storage behind the use cases.
//!
//! Handlers run concurrently, possibly on several threads, so the shared storage sits behind an
//! [`Arc`] (one copy, many owners) and a [`Mutex`] (one request at a time may touch it).
//!
//! The mutex is tokio's, not the standard library's, because a handler keeps it locked while it
//! `.await`s the ports. A [`std::sync::MutexGuard`] isn't `Send`, so a handler holding one across an
//! `.await` couldn't move between threads and axum would refuse it. Waiting for tokio's lock also
//! lets the thread serve other requests meanwhile, instead of blocking it.

use std::fmt;
use std::sync::Arc;

use clinic_app::ports::{AppointmentTypes, ScheduleRepository};
use tokio::sync::{Mutex, MutexGuard};

/// The router's state: the adapters for the ports the use cases need, shared by every request.
///
/// `S` and `T` are the adapter types, such as the in-memory ones from `clinic_app`. They're type
/// parameters rather than trait objects because the ports' `async` methods make them unusable as
/// trait objects.
///
/// Cloning it is cheap and every clone sees the same storage, which is what axum does for each
/// request.
pub struct AppState<S, T> {
    store: Arc<Mutex<Store<S, T>>>,
}

impl<S, T> AppState<S, T>
where
    S: ScheduleRepository + Send + Sync + 'static,
    T: AppointmentTypes + Send + Sync + 'static,
{
    /// State backed by the given adapters.
    ///
    /// The adapters must be `Send` and `Sync` so the storage can be used from the threads that
    /// serve requests, and `'static` so it doesn't borrow anything that could go away first.
    pub fn new(schedules: S, types: T) -> Self {
        Self {
            store: Arc::new(Mutex::new(Store { schedules, types })),
        }
    }

    /// Exclusive access to the storage until the guard is dropped. Waits while another request
    /// holds it.
    ///
    /// Unlike the standard library's, tokio's mutex isn't poisoned when a request panics while
    /// holding it, so this can't fail. A panic could leave the storage half-changed; the in-memory
    /// adapters change it in one step, and the database adapters will use transactions instead.
    pub(crate) async fn lock(&self) -> MutexGuard<'_, Store<S, T>> {
        self.store.lock().await
    }
}

// Written out instead of derived: `#[derive(Clone)]` would require `S: Clone` and `T: Clone`,
// though only the `Arc` is cloned.
impl<S, T> Clone for AppState<S, T> {
    fn clone(&self) -> Self {
        Self {
            store: Arc::clone(&self.store),
        }
    }
}

impl<S, T> fmt::Debug for AppState<S, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // The adapters don't have to implement `Debug`, so they aren't shown.
        f.debug_struct("AppState").finish_non_exhaustive()
    }
}

/// The adapters, behind the lock.
pub(crate) struct Store<S, T> {
    pub(crate) schedules: S,
    pub(crate) types: T,
}
