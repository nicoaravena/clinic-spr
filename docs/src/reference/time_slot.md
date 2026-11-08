# time_slot

A span of UTC time that always starts before it ends.

- **Source:** `crates/domain/src/time_slot.rs`
- **Introduced in:** [step 1.3](../steps/1.3-time-slot.md)
- **API docs:** [`clinic_domain::time_slot`][api]

## Public items

- `TimeSlot`: `new`, `start`, `end`, `overlaps`, `contains`
- `TimeSlotError`: `EndNotAfterStart`

## Rules and how they are enforced

| Rule | How |
|---|---|
| Start is strictly before end | `new` returns `EndNotAfterStart` otherwise |
| Times are UTC | Fields are `DateTime<Utc>` |
| Only valid slots exist | Private fields, `compile_fail` doctest |

## Semantics

Slots are half-open: the start is included and the end is not.

- `a.overlaps(b)` is true when they share any instant. Back-to-back slots
  (09:00-09:30 and 09:30-10:00) don't overlap. It is symmetric.
- `a.contains(b)` is true when `b` lies entirely inside `a`, edges included.
  A slot contains itself.

## Used by

- `AppointmentType::slot_starting_at` builds one.
- `WorkingHours` stores shifts as slots and uses `contains`.

[api]: ../../../target/doc/clinic_domain/time_slot/index.html
