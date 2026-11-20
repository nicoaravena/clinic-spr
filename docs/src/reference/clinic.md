# clinic (binary)

The server, and the composition root: the one place that picks the
adapters, builds the router from [`clinic-api`](api.md) and serves it.

- **Source:** `crates/clinic/src/`
- **Introduced in:** the [interlude](../steps/interlude-running-the-clinic.md)
  as a scripted day; the server since [step 2.5](../steps/2.5-serving-http.md);
  names the adapter types for the generic router since
  [step 3.1](../steps/3.1-async-ports.md)

## Running it

```sh
cargo run -p clinic
```

```text
clinic listening on http://127.0.0.1:3000
```

The [walkthrough](../steps/2.5-serving-http.md#see-it-run) has curl
commands to try against it. Ctrl-C stops it.

## Configuration

| Variable | Default | Meaning |
|---|---|---|
| `CLINIC_ADDR` | `127.0.0.1:3000` | The IP address and port to listen on, such as `0.0.0.0:8080`. Port 0 picks a free port. A host name such as `localhost` isn't accepted. |

## Output

stdout is never written. stderr gets one line when the server is ready,
`clinic listening on http://<address>` (with the real port, even when
`CLINIC_ADDR` asked for port 0), and one when it stops, `clinic shutting
down`. A startup failure is one line starting with `clinic: `. A request
that fails on the server's side adds a line starting with `clinic-api: `
(see [api errors](api.md#errors)); the in-memory storage never does.

## Signals and exit codes

SIGINT (Ctrl-C) and SIGTERM (`kill`, `docker stop`) start a graceful
shutdown: the server stops accepting connections, lets the requests in
progress finish, and exits. On platforms without SIGTERM only Ctrl-C is
handled.

| Code | Meaning |
|---|---|
| 0 | Stopped after a signal |
| 1 | Couldn't start: `CLINIC_ADDR` doesn't parse, the address is in use, or the signal handlers couldn't be installed |

## Seed data

Storage is in memory, so the server starts from the same data every time,
and loses every booking when it stops:

| Clinic | Appointment types | Practitioners |
|---|---|---|
| 1 | Consultation (30 min), Follow-up (15 min) | 1: works 09:00-12:00 and 13:00-17:00 UTC on Monday 2026-03-02, nothing booked |
| 2 | Consultation (30 min) | none |

The seed is saved through the same async `ScheduleRepository::save` the
use cases call, so a seed that couldn't be saved stops the server from
starting.

Clinic 2 is there to show that one clinic can't reach another's data.
Other practitioners get hours through `PUT /practitioners/{id}/hours`
(step 2.6). M3 replaces the in-memory storage with Postgres.

## Tests

- `tests/walkthrough.rs` starts the binary, plays the curl walkthrough
  over a real socket, and compares the transcript with
  `tests/expected-output.txt`. Regenerate the transcript with
  `UPDATE_EXPECTED=1 cargo test -p clinic --test walkthrough`, and review
  its diff.
- `tests/server.rs` checks that SIGTERM stops the server with exit code 0
  and nothing on stdout, and that a bad or busy address exits with code 1.
  The SIGTERM test runs only on Unix.
- `tests/common/mod.rs` holds `Server`, which starts the binary on port 0,
  reads the real address from stderr, sends requests with hyper's client,
  and kills the process when dropped.
