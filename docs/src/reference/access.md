# access

Who may do what in a clinic, proved by holding a capability.

- **Source:** `crates/domain/src/access.rs`
- **Introduced in:** [step 1.7](../steps/1.7-roles-and-access.md)
- **API docs:** [`clinic_domain::access`][api]

## Public items

- `Role`: `Practitioner(PractitionerId)`, `Receptionist`, `Administrator`;
  `grant`
- `Capability` trait: `NAME`, `grant`
- `ClinicalAccess`: `practitioner`
- `ScheduleAccess`
- `AuditAccess`
- `AccessDenied`: `capability`

## Who gets what

| Capability | Practitioner | Receptionist | Administrator |
|---|---|---|---|
| `ClinicalAccess` (clinical content) | yes | no | no |
| `ScheduleAccess` (schedules) | yes | yes | yes |
| `AuditAccess` (the audit log) | no | no | yes |

## Rules and how they are enforced

| Rule | How |
|---|---|
| Only practitioners touch clinical content | `ClinicalAccess::grant` matches only `Role::Practitioner` |
| A capability can't be forged | Private fields, `compile_fail` doctest; only `Capability::grant` builds one |
| A practitioner role always names the practitioner | `Role::Practitioner` carries a `PractitionerId` |
| Every role is decided for every capability | Each `grant` is an exhaustive `match`, with no `_` arm |

## Semantics

`role.grant::<C>()` returns `Ok(C)` if the role is allowed `C`, or
`Err(AccessDenied)` naming `C::NAME`. `C::grant(&role)` makes the same
decision and returns an `Option`. `ClinicalAccess` remembers which
practitioner it was granted to.

The domain trusts the `Role` it is given. Authenticating the user and
choosing their role happen outside the domain (M4).

## Not yet

- Nothing requires a capability yet. `ClinicalNote` (step 1.8) will take
  `&ClinicalAccess`, and `Schedule::book` may take `&ScheduleAccess` once
  there is a caller (M2/M4).
- Support staff are a separate principal, arriving in M6.

[api]: ../../../target/doc/clinic_domain/access/index.html
