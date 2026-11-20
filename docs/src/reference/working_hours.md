# working_hours

The dated shifts a practitioner works, in UTC.

- **Source:** `crates/domain/src/working_hours.rs`
- **Introduced in:** [step 1.5](../steps/1.5-working-hours.md);
  `replace_from` in [step 2.6](../steps/2.6-weekly-hours-endpoint.md)
- **API docs:** [`clinic_domain::working_hours`][api]

## Public items

- `WorkingHours`: `new`, `shifts`, `contains`, `replace_from`
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

`replace_from(cut, replacement)` returns new hours: these hours before
`cut`, then `replacement`'s hours from `cut` on. A shift of either that
straddles `cut` is cut short there, and if the two pieces left meet exactly
at `cut` they become one shift. It can't fail, and it changes neither
input.

| Hours | Replacement | Cut | Result |
|---|---|---|---|
| Mon 09-13, Tue 09-13 | Tue 14-18 | Tue 00:00 | Mon 09-13, Tue 14-18 |
| Mon 20:00-Tue 02:00 | Tue 09-13 | Tue 01:00 | Mon 20:00-Tue 01:00, Tue 09-13 |
| Mon 20:00-Tue 02:00 | Mon 23:00-Tue 04:00 | Tue 01:00 | Mon 20:00-Tue 04:00 |

## Where shifts come from

The domain only knows concrete UTC shifts. They are built from a weekly
template in a local time zone, including daylight saving changes, outside
the domain: `WeeklyHours::working_hours` in the
[app crate](app.md#weekly-hours), since
[step 2.3](../steps/2.3-weekly-hours.md).

[api]: ../../../target/doc/clinic_domain/working_hours/index.html
