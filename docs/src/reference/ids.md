# ids

Typed identifiers, so the compiler rejects an ID of the wrong kind.

- **Source:** `crates/domain/src/ids.rs`
- **Introduced in:** [step 1.2](../steps/1.2-ids-and-email.md)
- **API docs:** [`clinic_domain::ids`][api]

## Public items

- `ClinicId`: `new`, `value`
- `PractitionerId`: `new`, `value`

## Rules and how they are enforced

| Rule | How |
|---|---|
| A `ClinicId` can't be used where a `PractitionerId` is expected, or the reverse | Separate newtypes; `compile_fail` doctest |
| IDs are cheap to pass around | `Copy` |
| IDs work as map and set keys | `Eq` + `Hash` |

Any `u64` is accepted, so there is no error type.

## Notes

`value()` takes `self` rather than `&self`, since copying a `u64` is as
cheap as borrowing it.

[api]: ../../../target/doc/clinic_domain/ids/index.html
