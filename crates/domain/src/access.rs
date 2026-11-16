//! Who may do what in a clinic.
//!
//! Access is proved by holding a capability: a value such as [`ClinicalAccess`] that only
//! [`Role::grant`] can create. Code that works on clinical content asks for a `&ClinicalAccess`,
//! so it can't be reached without the check.
//!
//! The fields are private, so a capability can't be built directly:
//!
//! ```compile_fail,E0451
//! use clinic_domain::access::ClinicalAccess;
//! use clinic_domain::ids::PractitionerId;
//!
//! ClinicalAccess {
//!     practitioner: PractitionerId::new(1),
//! };
//! ```
//!
//! The domain doesn't check who holds a [`Role`]. Whoever creates one, such as the login code, must
//! have authenticated the user first.

use thiserror::Error;

use crate::ids::PractitionerId;

/// What a member of a clinic's staff does there, which decides what they may access.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Role {
    /// Sees patients. The only role with access to clinical content.
    Practitioner(PractitionerId),
    /// Manages the schedules and patients' contact details.
    Receptionist,
    /// Runs the clinic and reviews who accessed what. Has no access to clinical content.
    Administrator,
}

impl Role {
    /// Grants the capability `C` if this role is allowed it.
    pub fn grant<C: Capability>(&self) -> Result<C, AccessDenied> {
        C::grant(self).ok_or(AccessDenied {
            capability: C::NAME,
        })
    }
}

/// Proof that a role was allowed some kind of access.
pub trait Capability: Sized {
    /// What the capability gives access to, for error messages.
    const NAME: &'static str;

    /// The capability, or `None` if `role` isn't allowed it.
    fn grant(role: &Role) -> Option<Self>;
}

/// Proof that a practitioner may read and write clinical content.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClinicalAccess {
    practitioner: PractitionerId,
}

impl ClinicalAccess {
    /// The practitioner who was granted access.
    pub fn practitioner(self) -> PractitionerId {
        self.practitioner
    }
}

impl Capability for ClinicalAccess {
    const NAME: &'static str = "clinical content";

    fn grant(role: &Role) -> Option<Self> {
        match role {
            Role::Practitioner(practitioner) => Some(Self {
                practitioner: *practitioner,
            }),
            Role::Receptionist | Role::Administrator => None,
        }
    }
}

/// Proof that a role may see and manage the clinic's schedules.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScheduleAccess {
    _private: (),
}

impl Capability for ScheduleAccess {
    const NAME: &'static str = "schedules";

    fn grant(role: &Role) -> Option<Self> {
        match role {
            Role::Practitioner(_) | Role::Receptionist | Role::Administrator => {
                Some(Self { _private: () })
            }
        }
    }
}

/// Proof that a role may review the record of who accessed what.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AuditAccess {
    _private: (),
}

impl Capability for AuditAccess {
    const NAME: &'static str = "the audit log";

    fn grant(role: &Role) -> Option<Self> {
        match role {
            Role::Administrator => Some(Self { _private: () }),
            Role::Practitioner(_) | Role::Receptionist => None,
        }
    }
}

/// Why [`Role::grant`] refused a capability.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
#[error("role may not access {capability}")]
pub struct AccessDenied {
    capability: &'static str,
}

impl AccessDenied {
    /// The [`Capability::NAME`] of the capability that was refused.
    pub fn capability(self) -> &'static str {
        self.capability
    }
}
