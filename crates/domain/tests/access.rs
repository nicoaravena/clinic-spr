use clinic_domain::access::{
    AccessDenied, AuditAccess, Capability, ClinicalAccess, Role, ScheduleAccess,
};
use clinic_domain::ids::PractitionerId;

fn all_roles() -> [Role; 3] {
    [
        Role::Practitioner(PractitionerId::new(7)),
        Role::Receptionist,
        Role::Administrator,
    ]
}

fn is_granted<C: Capability>(role: &Role) -> bool {
    role.grant::<C>().is_ok()
}

#[test]
fn only_practitioners_get_clinical_access() {
    for role in all_roles() {
        assert_eq!(
            is_granted::<ClinicalAccess>(&role),
            matches!(role, Role::Practitioner(_)),
            "unexpected clinical access for {role:?}",
        );
    }
}

#[test]
fn every_role_gets_schedule_access() {
    for role in all_roles() {
        assert!(
            is_granted::<ScheduleAccess>(&role),
            "expected schedule access for {role:?}",
        );
    }
}

#[test]
fn only_administrators_get_audit_access() {
    for role in all_roles() {
        assert_eq!(
            is_granted::<AuditAccess>(&role),
            role == Role::Administrator,
            "unexpected audit access for {role:?}",
        );
    }
}

#[test]
fn clinical_access_carries_the_practitioners_id() {
    let access = Role::Practitioner(PractitionerId::new(7))
        .grant::<ClinicalAccess>()
        .unwrap();

    assert_eq!(access.practitioner(), PractitionerId::new(7));
}

#[test]
fn the_trait_grants_the_same_as_the_role() {
    assert!(ClinicalAccess::grant(&Role::Receptionist).is_none());
    assert!(ClinicalAccess::grant(&Role::Practitioner(PractitionerId::new(7))).is_some());
}

#[test]
fn a_refusal_names_the_capability() {
    let denied = Role::Receptionist.grant::<ClinicalAccess>().unwrap_err();

    assert_eq!(denied.capability(), ClinicalAccess::NAME);
}

#[test]
fn error_has_a_readable_message() {
    let denied = Role::Administrator.grant::<ClinicalAccess>().unwrap_err();

    assert_eq!(denied.to_string(), "role may not access clinical content");
}

#[test]
fn error_implements_the_standard_error_trait() {
    fn assert_is_error<E: std::error::Error>(_: E) {}

    let denied: AccessDenied = Role::Receptionist.grant::<AuditAccess>().unwrap_err();
    assert_is_error(denied);
}
