//! The clinic server, and the composition root: the one place that picks the adapters, builds the
//! router from `clinic_api` and serves it.
//!
//! Configuration comes from the environment:
//!
//! | Variable | Default | Meaning |
//! |---|---|---|
//! | `CLINIC_ADDR` | `127.0.0.1:3000` | The IP address and port to listen on. Port 0 picks a free one. |
//!
//! Once it's ready, the server writes `clinic listening on http://<address>` to stderr. stdout is
//! never written. Ctrl-C (SIGINT) or SIGTERM stops it gracefully: it stops accepting connections,
//! lets the requests in progress finish, and exits.
//!
//! Exit codes: 0 after a graceful shutdown, 1 if the server couldn't start.

mod seed;

use std::env::{self, VarError};
use std::error::Error;
use std::io;
use std::net::SocketAddr;
use std::process::ExitCode;

use tokio::net::TcpListener;

/// Where the server listens when `CLINIC_ADDR` isn't set.
const DEFAULT_ADDR: &str = "127.0.0.1:3000";

#[tokio::main]
async fn main() -> ExitCode {
    match run().await {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("clinic: {error}");
            ExitCode::FAILURE
        }
    }
}

/// Wires the app together and serves it until a shutdown signal arrives.
async fn run() -> Result<(), Box<dyn Error>> {
    let addr = listen_addr()?;
    let shutdown =
        shutdown_signal().map_err(|error| format!("can't listen for shutdown signals: {error}"))?;
    let state = seed::demo_state()?;

    let listener = TcpListener::bind(addr)
        .await
        .map_err(|error| format!("can't listen on {addr}: {error}"))?;
    eprintln!("clinic listening on http://{}", listener.local_addr()?);

    axum::serve(listener, clinic_api::router(state))
        .with_graceful_shutdown(shutdown)
        .await?;
    Ok(())
}

/// The address from `CLINIC_ADDR`, or [`DEFAULT_ADDR`] if it isn't set.
fn listen_addr() -> Result<SocketAddr, String> {
    let value = match env::var("CLINIC_ADDR") {
        Ok(value) => value,
        Err(VarError::NotPresent) => DEFAULT_ADDR.to_owned(),
        Err(error @ VarError::NotUnicode(_)) => return Err(format!("CLINIC_ADDR: {error}")),
    };
    value.parse().map_err(|error| {
        format!("CLINIC_ADDR {value:?} isn't an address such as {DEFAULT_ADDR}: {error}")
    })
}

/// A future that finishes when the process is asked to stop: on SIGINT (Ctrl-C) or SIGTERM (what
/// `kill` and `docker stop` send).
///
/// The handlers are installed right away, not when the future is first polled, so a failure to
/// install them stops the server from starting instead of going unnoticed.
#[cfg(unix)]
fn shutdown_signal() -> io::Result<impl Future<Output = ()>> {
    use tokio::signal::unix::{SignalKind, signal};

    let mut interrupt = signal(SignalKind::interrupt())?;
    let mut terminate = signal(SignalKind::terminate())?;

    Ok(async move {
        tokio::select! {
            _ = interrupt.recv() => {}
            _ = terminate.recv() => {}
        }
        eprintln!("clinic shutting down");
    })
}

/// A future that finishes on Ctrl-C. Other platforms have no SIGTERM.
#[cfg(not(unix))]
fn shutdown_signal() -> io::Result<impl Future<Output = ()>> {
    Ok(async {
        if let Err(error) = tokio::signal::ctrl_c().await {
            // Without a handler nothing can ask the server to stop, so it keeps serving.
            eprintln!("clinic: can't listen for Ctrl-C: {error}");
            std::future::pending::<()>().await;
        }
        eprintln!("clinic shutting down");
    })
}
