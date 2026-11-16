//! Clinical notes that are never overwritten: a correction adds a new version and keeps the old.
//!
//! Every function that reads or writes a note's content asks for a
//! [`ClinicalAccess`], so only practitioners can reach it. The author of each version is taken from
//! that capability, never from a parameter.
//!
//! The fields are private, so the only way to build a note is through [`ClinicalNote::write`], and
//! the only way to change one is [`ClinicalNote::correct`], which appends:
//!
//! ```compile_fail,E0451
//! use chrono::{TimeZone, Utc};
//! use clinic_domain::clinical_note::{ClinicalNote, NoteVersion};
//! use clinic_domain::ids::{PatientId, PractitionerId};
//!
//! let original = NoteVersion {
//!     text: String::from("Mild fever."),
//!     author: PractitionerId::new(1),
//!     written_at: Utc.with_ymd_and_hms(2026, 3, 2, 9, 30, 0).unwrap(),
//! };
//! ClinicalNote {
//!     patient: PatientId::new(5),
//!     original,
//!     corrections: Vec::new(),
//! };
//! ```
//!
//! A version borrowed from a note can't be held while the note is corrected. The borrow checker
//! refuses, because `current` returns a reference tied to the note:
//!
//! ```compile_fail,E0502
//! use chrono::{TimeZone, Utc};
//! use clinic_domain::access::{ClinicalAccess, Role};
//! use clinic_domain::clinical_note::ClinicalNote;
//! use clinic_domain::ids::{PatientId, PractitionerId};
//!
//! let access = Role::Practitioner(PractitionerId::new(1))
//!     .grant::<ClinicalAccess>()
//!     .unwrap();
//! let at = Utc.with_ymd_and_hms(2026, 3, 2, 9, 30, 0).unwrap();
//! let mut note = ClinicalNote::write(&access, PatientId::new(5), "Mild fever.", at).unwrap();
//!
//! let current = note.current(&access);
//! note.correct(&access, "Mild fever, 38.2 C.", at).unwrap();
//! println!("{}", current.text());
//! ```

use std::iter;

use chrono::{DateTime, Utc};
use thiserror::Error;

use crate::access::ClinicalAccess;
use crate::ids::{PatientId, PractitionerId};

/// One version of a clinical note: its text, who wrote it and when. A version never changes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NoteVersion {
    text: String,
    author: PractitionerId,
    written_at: DateTime<Utc>,
}

impl NoteVersion {
    /// The trimmed text. Never empty.
    pub fn text(&self) -> &str {
        &self.text
    }

    /// The practitioner who wrote this version.
    pub fn author(&self) -> PractitionerId {
        self.author
    }

    /// When this version was written.
    pub fn written_at(&self) -> DateTime<Utc> {
        self.written_at
    }
}

/// A practitioner's note about a patient, with every version it has had.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClinicalNote {
    patient: PatientId,
    original: NoteVersion,
    corrections: Vec<NoteVersion>,
}

impl ClinicalNote {
    /// Writes a new note about `patient`, authored by the practitioner who holds `access`.
    ///
    /// Trims `text`, then fails with [`NoteError::EmptyText`] if nothing is left.
    pub fn write(
        access: &ClinicalAccess,
        patient: PatientId,
        text: &str,
        at: DateTime<Utc>,
    ) -> Result<Self, NoteError> {
        let text = non_empty(text)?;

        Ok(Self {
            patient,
            original: NoteVersion {
                text: text.to_owned(),
                author: access.practitioner(),
                written_at: at,
            },
            corrections: Vec::new(),
        })
    }

    /// The patient the note is about.
    pub fn patient(&self) -> PatientId {
        self.patient
    }

    /// The latest version.
    pub fn current(&self, _access: &ClinicalAccess) -> &NoteVersion {
        self.corrections.last().unwrap_or(&self.original)
    }

    /// Every version, oldest first. The first is the original note.
    ///
    /// The iterator borrows the note but not `access`: `use<'a>` says it captures only `'a`.
    pub fn history<'a>(
        &'a self,
        _access: &ClinicalAccess,
    ) -> impl Iterator<Item = &'a NoteVersion> + use<'a> {
        iter::once(&self.original).chain(&self.corrections)
    }

    /// Corrects the note by adding a new version. Earlier versions are kept unchanged.
    ///
    /// The checks run in this order, and the first one that fails decides the error:
    ///
    /// 1. The holder of `access` must be the note's author.
    /// 2. The trimmed `text` must not be empty.
    /// 3. It must differ from the current text.
    /// 4. `at` must not be before the current version was written.
    ///
    /// On failure the note is unchanged.
    pub fn correct(
        &mut self,
        access: &ClinicalAccess,
        text: &str,
        at: DateTime<Utc>,
    ) -> Result<&NoteVersion, NoteError> {
        let author = self.original.author;
        if access.practitioner() != author {
            return Err(NoteError::NotAuthor { author });
        }

        let text = non_empty(text)?;
        let latest = self.current(access);
        if text == latest.text {
            return Err(NoteError::Unchanged);
        }
        if at < latest.written_at {
            return Err(NoteError::BeforeLatest {
                latest: latest.written_at,
            });
        }

        self.corrections.push(NoteVersion {
            text: text.to_owned(),
            author,
            written_at: at,
        });

        Ok(self.current(access))
    }
}

/// `text` without surrounding whitespace, or `EmptyText` if nothing is left.
fn non_empty(text: &str) -> Result<&str, NoteError> {
    let text = text.trim();

    if text.is_empty() {
        return Err(NoteError::EmptyText);
    }

    Ok(text)
}

/// Why [`ClinicalNote::write`] or [`ClinicalNote::correct`] refused its input.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum NoteError {
    /// The text was empty or only whitespace.
    #[error("clinical note text is empty")]
    EmptyText,
    /// The correction has the same text as the current version.
    #[error("correction doesn't change the note")]
    Unchanged,
    /// The correction is dated before the current version.
    #[error("correction is dated before the latest version")]
    BeforeLatest {
        /// When the current version was written.
        latest: DateTime<Utc>,
    },
    /// Someone other than the note's author tried to correct it.
    #[error("only the note's author may correct it")]
    NotAuthor {
        /// The practitioner who wrote the note.
        author: PractitionerId,
    },
}
