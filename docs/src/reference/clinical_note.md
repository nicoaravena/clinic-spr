# clinical_note

Clinical notes that are never overwritten: a correction adds a new version
and keeps the old.

- **Source:** `crates/domain/src/clinical_note.rs`
- **Introduced in:** [step 1.8](../steps/1.8-clinical-notes.md)
- **API docs:** [`clinic_domain::clinical_note`][api]

## Public items

- `ClinicalNote`: `write`, `patient`, `current`, `history`, `correct`
- `NoteVersion`: `text`, `author`, `written_at`
- `NoteError`: `EmptyText`, `Unchanged`, `BeforeLatest { latest }`,
  `NotAuthor { author }`

## Rules and how they are enforced

| Rule | How |
|---|---|
| Only practitioners read or write clinical content | `write`, `current`, `history` and `correct` take `&ClinicalAccess` |
| The author is the practitioner who holds the access | Taken from `ClinicalAccess::practitioner`, never a parameter |
| A note is never overwritten | `correct` only appends; no method gives out `&mut NoteVersion` |
| A note always has at least one version | The `original` field; `corrections` may be empty |
| Notes and versions can't be built or changed from outside | Private fields, `compile_fail` doctest |
| A version can't be held while the note is corrected | Borrow checker, `compile_fail` doctest (E0502) |
| Only the author corrects a note | `correct` returns `NotAuthor` otherwise |
| Versions are in time order | `correct` returns `BeforeLatest` for an earlier time |

## Semantics

`write(access, patient, text, at)` trims `text` and refuses it if nothing
is left (`EmptyText`).

`correct(access, text, at)` checks, in this order, and stops at the first
failure:

1. The holder of `access` wrote the note (`NotAuthor`, with the author).
2. The trimmed text isn't empty (`EmptyText`).
3. It differs from the current text (`Unchanged`).
4. `at` isn't before the current version (`BeforeLatest`, with its time).
   The same instant is allowed.

On success it returns the new version. On failure the note is unchanged.

`current` returns the latest version. `history` iterates over every
version, oldest first; the first is the original. Any practitioner can read
a note. `patient` needs no access.

## Not yet

- Reads aren't recorded. The audit log arrives in M5, and the access token
  already says who is reading.
- Notes aren't stored; they need a repository (M2) and a table (M3).
- No deletion. How it fits with records that are never overwritten is
  decided in M7.
- A note isn't linked to an appointment.

[api]: ../../../target/doc/clinic_domain/clinical_note/index.html
