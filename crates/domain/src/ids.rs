//! Typed identifiers, so the compiler rejects an ID of the wrong kind.
//!
//! ```compile_fail,E0308
//! use clinic_domain::ids::{ClinicId, PractitionerId};
//!
//! fn load_clinic(_: ClinicId) {}
//!
//! load_clinic(PractitionerId::new(1));
//! ```

/// Identifies a clinic, which is one tenant of the platform.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ClinicId(u64);

impl ClinicId {
    /// Wraps a raw value. Any `u64` is accepted.
    pub fn new(value: u64) -> Self {
        Self(value)
    }

    /// The raw value, for storage or display.
    pub fn value(self) -> u64 {
        self.0
    }
}

/// Identifies a practitioner within a clinic.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PractitionerId(u64);

impl PractitionerId {
    /// Wraps a raw value. Any `u64` is accepted.
    pub fn new(value: u64) -> Self {
        Self(value)
    }

    /// The raw value, for storage or display.
    pub fn value(self) -> u64 {
        self.0
    }
}
