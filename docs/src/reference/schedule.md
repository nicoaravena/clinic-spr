# schedule

One practitioner's working hours and the appointments booked in them.

- **Source:** `crates/domain/src/schedule.rs`
- **Introduced in:** [step 1.6](../steps/1.6-schedule.md)
- **API docs:** [`clinic_domain::schedule`][api]

## Public items

- `Schedule`: `new`, `appointments`, `appointments_in`, `book`
- `BookingError`: `OutOfRange`, `OutsideWorkingHours`, `Overlaps { existing }`

## Rules and how they are enforced

| Rule | How |
|---|---|
| An appointment fits inside a single shift | `book` returns `OutsideWorkingHours` otherwise |
| Appointments don't overlap | `book` returns `Overlaps` otherwise |
| Appointments are stored sorted by start | `book` inserts at the sorted position |
| Appointments are added only through `book` | Private fields, `compile_fail` doctest; `Appointment` has a `pub(crate)` constructor |

## Semantics

`book(patient, start, appointment_type)` builds the slot from the type's duration,
then checks, in this order, and stops at the first failure:

1. The slot ends within chrono's range (`OutOfRange`).
2. A single shift contains it (`OutsideWorkingHours`).
3. It doesn't overlap a booked appointment (`Overlaps`, with that
   appointment's slot).

On success it returns a reference to the new `Scheduled` appointment. On
failure the schedule is unchanged. Back-to-back appointments are allowed,
because slots are half-open.

`appointments_in(range)` returns an iterator over the appointments that
overlap `range`, sorted by start (added in
[step 2.2](../steps/2.2-practitioner-agenda.md)). It borrows the schedule
and copies nothing. Overlap is half-open, as for `TimeSlot::overlaps`: an
appointment ending exactly at `range.start()` or starting exactly at
`range.end()` is left out, and one straddling either edge is included. It
relies on appointments being sorted and non-overlapping to stop at the end
of the range.

## Not yet

- Cancelled appointments still block their slot; there is no cancel yet.
- No way to rebuild a schedule from stored appointments. The in-memory
  adapter of step 2.1 stores whole `Schedule` values, so this is needed only
  when the database arrives in M3.

[api]: ../../../target/doc/clinic_domain/schedule/index.html
