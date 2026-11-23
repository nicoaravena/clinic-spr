//! A clinic as the platform knows it: which tenant it is, the slug it's known by, and whether it
//! may be used.
//!
//! Holding a [`Clinic`] means each part passed its checks: the only way to get a [`Slug`] is
//! [`Slug::parse`], and the fields can't be set from outside:
//!
//! ```compile_fail,E0451
//! use clinic_domain::clinic::{Clinic, ClinicStatus, Slug};
//! use clinic_domain::ids::ClinicId;
//!
//! let clinic = Clinic {
//!     id: ClinicId::new(1),
//!     slug: Slug::parse("centro").unwrap(),
//!     status: ClinicStatus::Active,
//! };
//! ```

use std::fmt;

use thiserror::Error;

use crate::ids::ClinicId;

const MAX_LENGTH: usize = 63;

/// The short name a clinic is known by, such as `centro-norte`.
///
/// It's made of lowercase ASCII letters and digits in groups joined by single hyphens, at most 63
/// bytes long. That's the shape of a DNS label, so a slug could name a subdomain later.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Slug(String);

impl Slug {
    /// Trims and lowercases `input`, then checks it.
    pub fn parse(input: &str) -> Result<Self, SlugError> {
        let slug = input.trim().to_lowercase();

        if slug.is_empty() {
            return Err(SlugError::Empty);
        }
        if slug.len() > MAX_LENGTH {
            return Err(SlugError::TooLong);
        }
        if !is_well_formed(&slug) {
            return Err(SlugError::InvalidFormat);
        }

        Ok(Self(slug))
    }

    /// The normalized slug.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Slug {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// Groups of `[a-z0-9]`, joined by single hyphens.
fn is_well_formed(slug: &str) -> bool {
    slug.split('-').all(|group| {
        !group.is_empty()
            && group
                .bytes()
                .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit())
    })
}

/// Why [`Slug::parse`] rejected its input.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum SlugError {
    /// Nothing was left after trimming.
    #[error("slug is empty")]
    Empty,
    /// Longer than 63 bytes.
    #[error("slug is too long")]
    TooLong,
    /// Not groups of lowercase letters and digits joined by single hyphens.
    #[error("slug has an invalid format")]
    InvalidFormat,
}

/// Whether a clinic may be used.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClinicStatus {
    /// The clinic is working normally.
    Active,
    /// The platform has suspended the clinic, for example for an unpaid plan.
    Suspended,
}

/// One tenant of the platform.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Clinic {
    id: ClinicId,
    slug: Slug,
    status: ClinicStatus,
}

impl Clinic {
    /// A clinic with these parts. Each part has already been checked, so this can't fail.
    pub fn new(id: ClinicId, slug: Slug, status: ClinicStatus) -> Self {
        Self { id, slug, status }
    }

    /// Which tenant this is.
    pub fn id(&self) -> ClinicId {
        self.id
    }

    /// The short name the clinic is known by.
    pub fn slug(&self) -> &Slug {
        &self.slug
    }

    /// Whether the clinic may be used.
    pub fn status(&self) -> ClinicStatus {
        self.status
    }
}
