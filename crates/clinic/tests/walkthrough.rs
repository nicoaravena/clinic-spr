//! The curl walkthrough: a fixed sequence of requests against the real binary, compared with a
//! golden transcript.
//!
//! Each step is written out as the `curl` command that sends the same request, so the transcript
//! can be replayed by hand against `cargo run -p clinic`. Update the transcript when a step changes
//! what the clinic does:
//!
//! `UPDATE_EXPECTED=1 cargo test -p clinic --test walkthrough`

mod common;

use std::fmt::Write;
use std::fs;

use common::Server;

/// The exact transcript of a run.
const EXPECTED: &str = include_str!("expected-output.txt");

/// One request in the walkthrough.
struct Step {
    /// What the step shows, written as a `#` comment above the command.
    about: &'static str,
    clinic: Option<&'static str>,
    request: Request,
}

enum Request {
    Get(&'static str),
    Post(&'static str, &'static str),
}

const STEPS: &[Step] = &[
    Step {
        about: "[2.4] Book a 30-minute consultation at 10:00 UTC",
        clinic: Some("1"),
        request: Request::Post(
            "/appointments",
            r#"{"practitioner_id":1,"patient_id":1,"appointment_type":"Consultation","start":"2026-03-02T10:00:00Z"}"#,
        ),
    },
    Step {
        about: "[2.4] The same request again: the slot is taken",
        clinic: Some("1"),
        request: Request::Post(
            "/appointments",
            r#"{"practitioner_id":1,"patient_id":1,"appointment_type":"Consultation","start":"2026-03-02T10:00:00Z"}"#,
        ),
    },
    Step {
        about: "[2.4] A follow-up at 11:50 would run past the end of the 09:00-12:00 shift",
        clinic: Some("1"),
        request: Request::Post(
            "/appointments",
            r#"{"practitioner_id":1,"patient_id":2,"appointment_type":"Follow-up","start":"2026-03-02T11:50:00Z"}"#,
        ),
    },
    Step {
        about: "[2.4] A follow-up at 10:00 in Santiago (-03:00) is stored as 13:00 UTC",
        clinic: Some("1"),
        request: Request::Post(
            "/appointments",
            r#"{"practitioner_id":1,"patient_id":2,"appointment_type":"Follow-up","start":"2026-03-02T10:00:00-03:00"}"#,
        ),
    },
    Step {
        about: "[2.4] Clinic 2 can't book into clinic 1's practitioner",
        clinic: Some("2"),
        request: Request::Post(
            "/appointments",
            r#"{"practitioner_id":1,"patient_id":3,"appointment_type":"Consultation","start":"2026-03-02T11:00:00Z"}"#,
        ),
    },
    Step {
        about: "[2.4] Every request must name its clinic",
        clinic: None,
        request: Request::Post(
            "/appointments",
            r#"{"practitioner_id":1,"patient_id":1,"appointment_type":"Consultation","start":"2026-03-02T11:00:00Z"}"#,
        ),
    },
    Step {
        about: "[2.5] Practitioner 1's agenda for the working day",
        clinic: Some("1"),
        request: Request::Get(
            "/practitioners/1/agenda?from=2026-03-02T09:00:00Z&to=2026-03-02T17:00:00Z",
        ),
    },
    Step {
        about: "[2.5] Nothing is booked over lunch",
        clinic: Some("1"),
        request: Request::Get(
            "/practitioners/1/agenda?from=2026-03-02T12:00:00Z&to=2026-03-02T13:00:00Z",
        ),
    },
    Step {
        about: "[2.5] Clinic 2 can't read clinic 1's agenda",
        clinic: Some("2"),
        request: Request::Get(
            "/practitioners/1/agenda?from=2026-03-02T09:00:00Z&to=2026-03-02T17:00:00Z",
        ),
    },
    Step {
        about: "[2.5] A range must end after it starts",
        clinic: Some("1"),
        request: Request::Get(
            "/practitioners/1/agenda?from=2026-03-02T17:00:00Z&to=2026-03-02T09:00:00Z",
        ),
    },
];

/// The `curl` command that sends the same request to a server on the default address.
fn curl(step: &Step) -> String {
    let mut command = String::from("curl -s");
    match step.request {
        Request::Get(path) => write!(command, " 'localhost:3000{path}'").unwrap(),
        Request::Post(path, _) => write!(command, " localhost:3000{path}").unwrap(),
    }
    if let Some(clinic) = step.clinic {
        write!(command, " -H 'X-Clinic-Id: {clinic}'").unwrap();
    }
    if let Request::Post(_, body) = step.request {
        write!(command, " -H 'Content-Type: application/json' -d '{body}'").unwrap();
    }
    command
}

#[tokio::test]
async fn the_walkthrough_gives_the_expected_transcript() {
    let server = Server::start();
    let mut transcript = String::new();

    for step in STEPS {
        let (status, body) = match step.request {
            Request::Get(path) => server.request("GET", path, step.clinic, None).await,
            Request::Post(path, body) => {
                server.request("POST", path, step.clinic, Some(body)).await
            }
        };
        writeln!(transcript, "# {}", step.about).unwrap();
        writeln!(transcript, "$ {}", curl(step)).unwrap();
        writeln!(transcript, "{status}").unwrap();
        writeln!(transcript, "{body}\n").unwrap();
    }

    if std::env::var_os("UPDATE_EXPECTED").is_some() {
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/expected-output.txt");
        fs::write(path, &transcript).unwrap();
        return;
    }
    assert_eq!(transcript, EXPECTED);
}
