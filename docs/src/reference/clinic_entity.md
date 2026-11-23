# clinic (module)

A clinic as the platform knows it: its id, the slug it's known by, and
whether it may be used. Not to be confused with the
[clinic binary](clinic.md).

- **Source:** `crates/domain/src/clinic.rs`
- **Introduced in:** [step 3.3](../steps/3.3-db-crate.md)
- **API docs:** [`clinic_domain::clinic`][api]

## Public items

- `Slug`: `parse`, `as_str`, `Display`
- `SlugError`: `Empty`, `TooLong`, `InvalidFormat`
- `ClinicStatus`: `Active`, `Suspended`
- `Clinic`: `new`, `id`, `slug`, `status`

## Rules and how they are enforced

| Rule | How |
|---|---|
| A slug is groups of `a-z` and `0-9` joined by single hyphens | `Slug::parse` returns `InvalidFormat`; private field |
| A slug is 1 to 63 bytes | `Slug::parse` returns `Empty` or `TooLong` |
| Slugs differing only by case or surrounding spaces are the same slug | `Slug::parse` trims and lowercases first |
| A `Clinic` is built only from checked parts | Private fields; `compile_fail` doctest (E0451); `new` takes a `Slug`, not a string |

63 bytes is the limit of a DNS label, so a slug could name a subdomain.

## Notes

The database spells the status `'active'` or `'suspended'`; that mapping
lives in the [db crate](db.md), not here.

[api]: ../../../target/doc/clinic_domain/clinic/index.html
