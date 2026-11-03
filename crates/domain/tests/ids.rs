use std::collections::HashSet;

use clinic_domain::ids::{ClinicId, PractitionerId};

#[test]
fn id_exposes_the_value_it_was_created_with() {
    assert_eq!(ClinicId::new(7).value(), 7);
    assert_eq!(PractitionerId::new(9).value(), 9);
}

#[test]
fn ids_with_the_same_value_are_equal() {
    assert_eq!(ClinicId::new(1), ClinicId::new(1));
    assert_ne!(ClinicId::new(1), ClinicId::new(2));
}

#[test]
fn id_can_be_used_after_being_passed_by_value() {
    fn take(_: ClinicId) {}

    let id = ClinicId::new(1);
    take(id);

    assert_eq!(id.value(), 1);
}

#[test]
fn ids_can_be_stored_in_a_set() {
    let ids = HashSet::from([
        PractitionerId::new(1),
        PractitionerId::new(1),
        PractitionerId::new(2),
    ]);

    assert_eq!(ids.len(), 2);
}
