use clinic_domain::email::{Email, EmailError};

#[test]
fn accepts_a_valid_address() {
    let email = Email::parse("ana@clinic.example").unwrap();

    assert_eq!(email.as_str(), "ana@clinic.example");
}

#[test]
fn trims_surrounding_whitespace_and_lowercases() {
    let email = Email::parse("  Ana@Clinic.Example\n").unwrap();

    assert_eq!(email.as_str(), "ana@clinic.example");
}

#[test]
fn addresses_differing_only_by_case_are_equal() {
    assert_eq!(
        Email::parse("ANA@clinic.example"),
        Email::parse("ana@clinic.example"),
    );
}

#[test]
fn displays_as_the_normalized_address() {
    let email = Email::parse("Ana@clinic.example").unwrap();

    assert_eq!(email.to_string(), "ana@clinic.example");
}

#[test]
fn rejects_empty_input() {
    assert_eq!(Email::parse(""), Err(EmailError::Empty));
    assert_eq!(Email::parse("   "), Err(EmailError::Empty));
}

#[test]
fn accepts_an_address_of_exactly_the_maximum_length() {
    let address = format!(
        "{}@clinic.example",
        "a".repeat(254 - "@clinic.example".len())
    );

    assert_eq!(address.len(), 254);
    assert!(Email::parse(&address).is_ok());
}

#[test]
fn rejects_an_address_over_the_maximum_length() {
    let address = format!(
        "{}@clinic.example",
        "a".repeat(255 - "@clinic.example".len())
    );

    assert_eq!(address.len(), 255);
    assert_eq!(Email::parse(&address), Err(EmailError::TooLong));
}

#[test]
fn rejects_malformed_addresses() {
    let malformed = [
        "ana",
        "@clinic.example",
        "ana@",
        "ana@clinic",
        "ana@.example",
        "ana@clinic.",
        "ana@clinic..example",
        "ana@@clinic.example",
        "ana@cli@nic.example",
        "a na@clinic.example",
        "ana@clinic .example",
    ];

    for address in malformed {
        assert_eq!(
            Email::parse(address),
            Err(EmailError::InvalidFormat),
            "expected {address:?} to be rejected",
        );
    }
}

#[test]
fn error_has_a_readable_message() {
    assert_eq!(EmailError::Empty.to_string(), "email address is empty");
}

#[test]
fn error_implements_the_standard_error_trait() {
    fn assert_is_error<E: std::error::Error>(_: E) {}

    assert_is_error(EmailError::InvalidFormat);
}
