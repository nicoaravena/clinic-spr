use clinic_domain::clinic::{Clinic, ClinicStatus, Slug, SlugError};
use clinic_domain::ids::ClinicId;

#[test]
fn accepts_a_valid_slug() {
    let slug = Slug::parse("centro-medico-2").unwrap();

    assert_eq!(slug.as_str(), "centro-medico-2");
}

#[test]
fn trims_surrounding_whitespace_and_lowercases() {
    let slug = Slug::parse("  Centro-Norte\n").unwrap();

    assert_eq!(slug.as_str(), "centro-norte");
}

#[test]
fn displays_as_the_normalized_slug() {
    let slug = Slug::parse("Centro").unwrap();

    assert_eq!(slug.to_string(), "centro");
}

#[test]
fn rejects_an_empty_slug() {
    assert_eq!(Slug::parse(""), Err(SlugError::Empty));
    assert_eq!(Slug::parse("   "), Err(SlugError::Empty));
}

#[test]
fn accepts_63_bytes_and_rejects_64() {
    let longest = "a".repeat(63);

    assert_eq!(Slug::parse(&longest).unwrap().as_str(), longest);
    assert_eq!(Slug::parse(&"a".repeat(64)), Err(SlugError::TooLong));
}

#[test]
fn rejects_characters_other_than_letters_digits_and_hyphens() {
    for input in [
        "centro_norte",
        "centro norte",
        "centro.cl",
        "clínica",
        "a/b",
        "x;drop",
    ] {
        assert_eq!(
            Slug::parse(input),
            Err(SlugError::InvalidFormat),
            "{input:?} should be rejected",
        );
    }
}

#[test]
fn rejects_leading_trailing_and_doubled_hyphens() {
    for input in ["-centro", "centro-", "centro--norte", "-"] {
        assert_eq!(
            Slug::parse(input),
            Err(SlugError::InvalidFormat),
            "{input:?} should be rejected",
        );
    }
}

#[test]
fn slug_errors_have_messages() {
    assert_eq!(SlugError::Empty.to_string(), "slug is empty");
    assert_eq!(SlugError::TooLong.to_string(), "slug is too long");
    assert_eq!(
        SlugError::InvalidFormat.to_string(),
        "slug has an invalid format",
    );
}

#[test]
fn a_clinic_keeps_its_id_slug_and_status() {
    let clinic = Clinic::new(
        ClinicId::new(7),
        Slug::parse("centro").unwrap(),
        ClinicStatus::Suspended,
    );

    assert_eq!(clinic.id(), ClinicId::new(7));
    assert_eq!(clinic.slug().as_str(), "centro");
    assert_eq!(clinic.status(), ClinicStatus::Suspended);
}
