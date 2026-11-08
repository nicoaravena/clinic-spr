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

## Building the book

You need [mdBook](https://rust-lang.github.io/mdBook/), installed once with:

```sh
cargo install mdbook --locked
```

Then, from the repository root:

```sh
cargo doc --no-deps          # API docs, written to target/doc/
mdbook build docs --open     # the book, written to docs/book/
```

The reference pages link to the API docs with relative file paths, so those
links work when the book is opened from `docs/book/` after `cargo doc` has
run. `mdbook serve docs --open` rebuilds on every save, which is handy while
editing, but it can't serve the API docs, so those links break there.

`docs/book/` is build output and is not committed.
