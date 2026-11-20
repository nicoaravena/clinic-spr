//! What every request handler shares: the storage behind the use cases.
//!
//! Handlers run concurrently, possibly on several threads, so the storage sits behind an [`Arc`]
//! (one copy, many owners). There's no lock here: each use case begins its own session, and the
//! storage decides how sessions are kept apart. The in-memory one runs them one at a time; a
//! database runs them as transactions.

use std::fmt;
use std::sync::Arc;

use clinic_app::ports::Storage;

/// The router's state: the storage the use cases begin their sessions in, shared by every request.
///
/// `St` is the storage type, such as `clinic_app`'s in-memory one. It's a type parameter rather
/// than a trait object because [`Storage`] has an associated type and `async` methods, which make
/// it unusable as a trait object.
///
/// Cloning it is cheap and every clone sees the same storage, which is what axum does for each
/// request.
pub struct AppState<St> {
    storage: Arc<St>,
}

impl<St> AppState<St>
where
    St: Storage + Send + Sync + 'static,
{
    /// State backed by `storage`.
    ///
    /// The storage must be `Send` and `Sync` so it can be used from the threads that serve
    /// requests, and `'static` so it doesn't borrow anything that could go away first.
    pub fn new(storage: St) -> Self {
        Self {
            storage: Arc::new(storage),
        }
    }

    /// The storage, for a use case to begin a session in.
    pub(crate) fn storage(&self) -> &St {
        &self.storage
    }
}

// Written out instead of derived: `#[derive(Clone)]` would require `St: Clone`, though only the
// `Arc` is cloned.
impl<St> Clone for AppState<St> {
    fn clone(&self) -> Self {
        Self {
            storage: Arc::clone(&self.storage),
        }
    }
}

impl<St> fmt::Debug for AppState<St> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // The storage doesn't have to implement `Debug`, so it isn't shown.
        f.debug_struct("AppState").finish_non_exhaustive()
    }
}
