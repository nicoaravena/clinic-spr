//! How the binary starts and stops.

mod common;

use std::process::Command;

#[cfg(unix)]
#[tokio::test]
async fn sigterm_stops_the_server_cleanly() {
    use common::Server;
    use hyper::StatusCode;

    let server = Server::start();
    let (status, _) = server
        .request(
            "GET",
            "/practitioners/1/agenda?from=2026-03-02T09:00:00Z&to=2026-03-02T17:00:00Z",
            Some("1"),
            None,
        )
        .await;
    assert_eq!(status, StatusCode::OK, "the server should be serving");

    let stopped = server.stop();

    assert!(
        stopped.status.success(),
        "exit status {}, stderr:\n{}",
        stopped.status,
        stopped.stderr,
    );
    assert_eq!(stopped.stderr, "clinic shutting down\n");
    assert_eq!(stopped.stdout, "", "stdout is kept for output, not logs");
}

#[test]
fn an_address_that_does_not_parse_fails_to_start_with_exit_code_1() {
    let output = Command::new(env!("CARGO_BIN_EXE_clinic"))
        .env("CLINIC_ADDR", "nope")
        .output()
        .expect("the clinic binary should start");
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert_eq!(output.status.code(), Some(1), "stderr:\n{stderr}");
    assert!(
        stderr.starts_with("clinic: CLINIC_ADDR \"nope\""),
        "stderr:\n{stderr}"
    );
    assert!(!stderr.contains("listening"), "stderr:\n{stderr}");
}

#[test]
fn an_address_already_in_use_fails_to_start_with_exit_code_1() {
    let taken = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = taken.local_addr().unwrap().to_string();

    let output = Command::new(env!("CARGO_BIN_EXE_clinic"))
        .env("CLINIC_ADDR", &addr)
        .output()
        .expect("the clinic binary should start");
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert_eq!(output.status.code(), Some(1), "stderr:\n{stderr}");
    assert!(
        stderr.starts_with(&format!("clinic: can't listen on {addr}")),
        "stderr:\n{stderr}"
    );
}
