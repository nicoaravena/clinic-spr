//! `SchemaName` is pure: these tests need no database.

use clinic_db::schema::SchemaName;
use clinic_domain::ids::ClinicId;

#[test]
fn is_clinic_followed_by_the_id() {
    assert_eq!(SchemaName::from(ClinicId::new(42)).as_str(), "clinic_42");
    assert_eq!(SchemaName::from(ClinicId::new(0)).as_str(), "clinic_0");
}

#[test]
fn displays_as_the_name() {
    assert_eq!(SchemaName::from(ClinicId::new(7)).to_string(), "clinic_7");
}

#[test]
fn the_largest_id_fits_in_a_postgres_identifier() {
    let name = SchemaName::from(ClinicId::new(u64::MAX));

    assert_eq!(name.as_str(), "clinic_18446744073709551615");
    assert!(
        name.as_str().len() <= 63,
        "Postgres truncates identifiers to 63 bytes"
    );
}

#[test]
fn uses_only_characters_that_need_no_quoting() {
    for id in [0, 1, 42, 9_007_199_254_740_993, u64::MAX] {
        let name = SchemaName::from(ClinicId::new(id));

        assert!(
            name.as_str()
                .bytes()
                .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_'),
            "{name} has a character that needs quoting",
        );
        assert!(name.as_str().starts_with(|c: char| c.is_ascii_lowercase()));
    }
}

#[test]
fn different_clinics_get_different_schemas() {
    assert_ne!(
        SchemaName::from(ClinicId::new(1)),
        SchemaName::from(ClinicId::new(11)),
    );
}
