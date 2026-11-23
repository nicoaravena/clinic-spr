//! The name of the Postgres schema that holds one clinic's data.
//!
//! SQL can bind values as parameters, but not identifiers: a schema name has to be written into the
//! SQL text itself. If that text came from a request, a crafted name could change the statement.
//! So a [`SchemaName`] can only be made from a [`ClinicId`], and is always `clinic_` followed by
//! the id's digits. Those characters never need quoting, and the longest name, for `u64::MAX`, is
//! 27 bytes, well under Postgres's limit of 63.
//!
//! There's no way to make one from a string:
//!
//! ```compile_fail,E0423
//! use clinic_db::schema::SchemaName;
//!
//! let name = SchemaName(String::from("public; DROP TABLE platform.clinics"));
//! ```

use std::fmt;

use clinic_domain::ids::ClinicId;

/// The schema of one clinic, `clinic_<id>`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SchemaName(String);

impl SchemaName {
    /// The name, ready to be written into SQL as an identifier.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<ClinicId> for SchemaName {
    fn from(clinic: ClinicId) -> Self {
        Self(format!("clinic_{}", clinic.value()))
    }
}

impl fmt::Display for SchemaName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}
