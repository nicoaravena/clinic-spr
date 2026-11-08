# appointment

What kind of appointment is booked, and where it is in its lifecycle.

- **Source:** `crates/domain/src/appointment.rs`
- **Introduced in:** [step 1.4](../steps/1.4-appointment-types.md)
- **API docs:** [`clinic_domain::appointment`][api]

## Public items

- `AppointmentType`: `new`, `name`, `duration`, `slot_starting_at`
- `AppointmentTypeError`: `EmptyName`, `NonPositiveDuration`
- `AppointmentStatus`: `Scheduled`, `Completed`, `Cancelled`, `NoShow`;
  `transition_to`

## Rules and how they are enforced

| Rule | How |
|---|---|
| Each type has a non-empty name (after trimming) | `EmptyName` |
| Each type has a positive duration | `NonPositiveDuration` |
| Only valid types exist | Private fields, `compile_fail` doctest |
| Only allowed status moves happen | `transition_to` returns `None` otherwise |

## Status transitions

```text
Scheduled --> Completed
          --> Cancelled
          --> NoShow
```

`Completed`, `Cancelled` and `NoShow` are final: they can't move anywhere.
`Scheduled -> Scheduled` is refused too.

## Notes

`slot_starting_at` returns `None` only if the end would fall outside the
dates chrono can represent.

[api]: ../../../target/doc/clinic_domain/appointment/index.html
