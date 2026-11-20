# How step pages work

Each roadmap step gets one page, written when the step is finished. Every
page has the same sections:

1. **Goal.** What the step built, and its row in the roadmap.
2. **What changed.** Files and types added, with links to the reference.
3. **Rust concepts.** Each concept, shown with code from this repository
   and a link to the matching chapter of
   [The Rust Book](https://doc.rust-lang.org/book/).
4. **Design choices worth questioning.** Decisions that could have gone
   another way, and why they went this way.
5. **Tests.** What the tests pin down.
6. **See it run.** What the step added to the running clinic: up to step
   2.3, new lines in the scripted day printed by `cargo run -p clinic`; from
   step 2.5, new requests in the curl walkthrough and their answers.
7. **Commits.** Where to find the step in `git log`.

Pages for steps 1.1 to 1.7 were written before the runnable clinic existed,
so they have no **See it run** section. The
[interlude](interlude-running-the-clinic.md) shows the whole day as it was
when the clinic binary arrived.

Code snippets are copied from the source when the step was written. If they
differ from the source later, the source is right.
