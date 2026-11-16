use chrono::{DateTime, TimeZone, Utc};
use clinic_domain::access::{ClinicalAccess, Role};
use clinic_domain::clinical_note::{ClinicalNote, NoteError, NoteVersion};
use clinic_domain::ids::{PatientId, PractitionerId};

fn at(hour: u32, minute: u32) -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 3, 2, hour, minute, 0).unwrap()
}

fn access_for(practitioner: u64) -> ClinicalAccess {
    Role::Practitioner(PractitionerId::new(practitioner))
        .grant::<ClinicalAccess>()
        .unwrap()
}

fn note_by_ana() -> ClinicalNote {
    ClinicalNote::write(&access_for(1), PatientId::new(5), "Mild fever.", at(9, 30)).unwrap()
}

fn texts(note: &ClinicalNote, access: &ClinicalAccess) -> Vec<String> {
    note.history(access)
        .map(|version| version.text().to_owned())
        .collect()
}

#[test]
fn write_records_the_text_author_time_and_patient() {
    let access = access_for(1);
    let note = note_by_ana();
    let current = note.current(&access);

    assert_eq!(note.patient(), PatientId::new(5));
    assert_eq!(current.text(), "Mild fever.");
    assert_eq!(current.author(), PractitionerId::new(1));
    assert_eq!(current.written_at(), at(9, 30));
    assert_eq!(note.history(&access).count(), 1);
}

#[test]
fn write_trims_the_text() {
    let access = access_for(1);
    let note =
        ClinicalNote::write(&access, PatientId::new(5), "  Mild fever.\n", at(9, 30)).unwrap();

    assert_eq!(note.current(&access).text(), "Mild fever.");
}

#[test]
fn write_refuses_blank_text() {
    let result = ClinicalNote::write(&access_for(1), PatientId::new(5), " \n\t", at(9, 30));

    assert_eq!(result.unwrap_err(), NoteError::EmptyText);
}

#[test]
fn correct_adds_a_version_and_keeps_the_original() {
    let access = access_for(1);
    let mut note = note_by_ana();

    let corrected = note
        .correct(&access, "Mild fever, 38.2 C.", at(9, 45))
        .unwrap();
    assert_eq!(corrected.text(), "Mild fever, 38.2 C.");
    assert_eq!(corrected.written_at(), at(9, 45));

    assert_eq!(note.current(&access).text(), "Mild fever, 38.2 C.");
    assert_eq!(
        texts(&note, &access),
        ["Mild fever.", "Mild fever, 38.2 C."],
    );
}

#[test]
fn history_lists_every_version_oldest_first() {
    let access = access_for(1);
    let mut note = note_by_ana();
    note.correct(&access, "Second.", at(10, 0)).unwrap();
    note.correct(&access, "Third.", at(11, 0)).unwrap();

    let times: Vec<_> = note.history(&access).map(NoteVersion::written_at).collect();

    assert_eq!(texts(&note, &access), ["Mild fever.", "Second.", "Third."]);
    assert_eq!(times, [at(9, 30), at(10, 0), at(11, 0)]);
}

#[test]
fn any_practitioner_can_read_the_note() {
    let note = note_by_ana();

    assert_eq!(note.current(&access_for(2)).text(), "Mild fever.");
}

#[test]
fn only_the_author_may_correct() {
    let mut note = note_by_ana();

    let error = note
        .correct(&access_for(2), "Something else.", at(10, 0))
        .unwrap_err();

    assert_eq!(
        error,
        NoteError::NotAuthor {
            author: PractitionerId::new(1)
        },
    );
    assert_eq!(texts(&note, &access_for(1)), ["Mild fever."]);
}

#[test]
fn correct_refuses_blank_text() {
    let access = access_for(1);
    let mut note = note_by_ana();

    let error = note.correct(&access, "   ", at(10, 0)).unwrap_err();

    assert_eq!(error, NoteError::EmptyText);
    assert_eq!(texts(&note, &access), ["Mild fever."]);
}

#[test]
fn correct_refuses_unchanged_text() {
    let access = access_for(1);
    let mut note = note_by_ana();

    let error = note
        .correct(&access, " Mild fever. ", at(10, 0))
        .unwrap_err();

    assert_eq!(error, NoteError::Unchanged);
    assert_eq!(note.history(&access).count(), 1);
}

#[test]
fn correct_refuses_a_time_before_the_latest_version() {
    let access = access_for(1);
    let mut note = note_by_ana();

    let error = note.correct(&access, "Earlier.", at(9, 0)).unwrap_err();

    assert_eq!(error, NoteError::BeforeLatest { latest: at(9, 30) });
    assert_eq!(note.history(&access).count(), 1);
}

#[test]
fn correct_accepts_the_same_time_as_the_latest_version() {
    let access = access_for(1);
    let mut note = note_by_ana();

    assert!(note.correct(&access, "Same minute.", at(9, 30)).is_ok());
}

#[test]
fn author_is_checked_before_the_text() {
    let mut note = note_by_ana();

    let error = note.correct(&access_for(2), "", at(9, 0)).unwrap_err();

    assert!(matches!(error, NoteError::NotAuthor { .. }));
}

#[test]
fn errors_have_readable_messages() {
    assert_eq!(
        NoteError::EmptyText.to_string(),
        "clinical note text is empty"
    );
    assert_eq!(
        NoteError::Unchanged.to_string(),
        "correction doesn't change the note",
    );
    assert_eq!(
        NoteError::BeforeLatest { latest: at(9, 30) }.to_string(),
        "correction is dated before the latest version",
    );
    assert_eq!(
        NoteError::NotAuthor {
            author: PractitionerId::new(1)
        }
        .to_string(),
        "only the note's author may correct it",
    );
}

#[test]
fn error_implements_the_standard_error_trait() {
    fn assert_is_error<E: std::error::Error>(_: E) {}

    assert_is_error(NoteError::EmptyText);
}
