//! Typed identifiers, so the compiler rejects an ID of the wrong kind.
//!
//! ```compile_fail,E0308
//! use clinic_domain::ids::{ClinicId, PractitionerId};
//!
//! fn load_clinic(_: ClinicId) {}
//!
//! load_clinic(PractitionerId::new(1));
//! ```

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ClinicId(u64);

impl ClinicId {
    pub fn new(value: u64) -> Self {
        Self(value)
    }

    pub fn value(self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PractitionerId(u64);

impl PractitionerId {
    pub fn new(value: u64) -> Self {
        Self(value)
    }

    pub fn value(self) -> u64 {
        self.0
    }
}
