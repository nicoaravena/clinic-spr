# email

An email address that has been checked and normalized.

- **Source:** `crates/domain/src/email.rs`
- **Introduced in:** [step 1.2](../steps/1.2-ids-and-email.md)
- **API docs:** [`clinic_domain::email`][api]

## Public items

- `Email`: `parse`, `as_str`, `Display`
- `EmailError`: `Empty`, `TooLong`, `InvalidFormat`

## Rules and how they are enforced

| Rule | How |
|---|---|
| Stored trimmed and lowercased | `parse` normalizes before checking |
| Not empty | `EmailError::Empty` |
| At most 254 bytes | `EmailError::TooLong` |
| Shaped like `local@domain.tld`: one `@`, no whitespace, a dot in the domain, no empty domain labels | `EmailError::InvalidFormat` |
| Only valid emails exist | The field is private, so `parse` is the only constructor |

Because the address is normalized, two emails that differ only in case or
surrounding spaces are equal.

## Notes

The format check is deliberately basic, not full RFC 5322. Delivery is the
only real proof an address works.

[api]: ../../../target/doc/clinic_domain/email/index.html
