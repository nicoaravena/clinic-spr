# Introduction

This book goes with the clinic-spr repository. The goal is to learn Rust,
and the project is a multi-tenant clinic platform (see the repository
`README.md` for the problem it solves). Work happens in small steps that
follow the [Roadmap](roadmap.md).

## How to read it

The book has two parts. Both are meant to be used.

- **Learning journal.** One page per roadmap step: what was built, the Rust
  concepts it introduced, the design choices worth questioning, and what the
  tests pin down. Read these in order to revise.
- **Reference.** One page per domain module: its purpose, the rules it
  enforces, and how it enforces them. Each page links to the API docs that
  rustdoc generates from the doc comments in the source, which hold the
  exact signatures.

The [Glossary](glossary.md) defines both domain terms (shift, slot) and Rust
terms (newtype, invariant).

## Running the clinic

From the repository root:

```sh
cargo run -p clinic
```

It starts the HTTP server on `127.0.0.1:3000` with two demo clinics in
memory (see [clinic](reference/clinic.md)). Ctrl-C stops it. The
[walkthrough in step 2.5](steps/2.5-serving-http.md#see-it-run) lists curl
commands to try against it: bookings that succeed and are refused,
practitioners' agendas, and, from step 2.6, setting a practitioner's weekly
hours. Every step extends that walkthrough. Its full
transcript is in `crates/clinic/tests/expected-output.txt`, and a test
replays it against the real binary on every `cargo test`.

Until step 2.5, the same command printed a scripted day instead. The
[interlude](steps/interlude-running-the-clinic.md) and the step pages up to
2.3 show its output.

## Running the database

From step 3.3, part of the test suite runs against Postgres, which runs in a
container. Once:

```sh
cp .env.example .env    # DATABASE_URL for the container
```

Then, before `cargo test`:

```sh
docker compose up -d --wait
```

The database tests fail rather than skip when Postgres isn't running. With
`.env` present, `sqlx::query!` also checks its SQL against the database
while compiling, so a build needs it too. `SQLX_OFFLINE=true` makes it use
the committed `.sqlx/` cache instead. See [db](reference/db.md) for the
sqlx-cli commands.

`docker compose down` stops it; add `-v` to delete its data as well.

## Building the book

You need [mdBook](https://rust-lang.github.io/mdBook/), installed once with:

```sh
cargo install mdbook --locked
```

Then, from the repository root:

```sh
cargo doc --no-deps          # API docs, written to target/doc/
mdbook build docs --open     # the book, written to public/
```

The reference pages link to the API docs with relative file paths, so those
links work when the book is opened from `public/` after `cargo doc` has
run. `mdbook serve docs --open` rebuilds on every save, which is handy while
editing, but it can't serve the API docs, so those links break there.

`public/` is build output and is not committed.
