# working_hours

The dated shifts a practitioner works, in UTC.

- **Source:** `crates/domain/src/working_hours.rs`
- **Introduced in:** [step 1.5](../steps/1.5-working-hours.md)
- **API docs:** [`clinic_domain::working_hours`][api]

## Public items

- `WorkingHours`: `new`, `shifts`, `contains`
- `WorkingHoursError`: `ShiftsNotSeparated`

## Rules and how they are enforced

| Rule | How |
|---|---|
| Shifts are stored sorted by start | `new` sorts them |
| Shifts don't overlap or touch | `new` returns `ShiftsNotSeparated` otherwise |
| Only valid hours exist | Private field, `compile_fail` doctest |

An empty list of shifts is valid and accepts no slot.

## Semantics

`contains(slot)` is true when a single shift fully contains the slot. A
slot that spans two shifts doesn't fit, even across a gap of one minute.
That is why touching shifts are rejected: 09-12 and 12-17 must be written
as 09-17.

## Where shifts come from

The domain only knows concrete UTC shifts. Building them from a weekly
template in a local time zone, including daylight saving changes, happens
outside the domain (roadmap M2).

[api]: ../../../target/doc/clinic_domain/working_hours/index.html
