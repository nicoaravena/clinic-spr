//! Runs the real `clinic` binary and talks to it over HTTP.

// Each test file is its own crate and uses a different subset of these helpers.
#![allow(dead_code)]

use std::io::{BufRead, BufReader, Read};
use std::net::SocketAddr;
use std::process::{Child, ChildStderr, Command, ExitStatus, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use http_body_util::{BodyExt, Full};
use hyper::body::Bytes;
use hyper::{Request, StatusCode};
use hyper_util::rt::TokioIo;
use tokio::net::TcpStream;

/// What the binary writes to stderr once it's ready, followed by its address.
const LISTENING: &str = "clinic listening on http://";

/// How long the binary may take to stop after a signal.
const STOP_TIMEOUT: Duration = Duration::from_secs(10);

/// A running `clinic` binary, listening on a free port that the OS picked.
///
/// Dropping it kills the process, so a failing test doesn't leave a server behind.
pub(crate) struct Server {
    child: Child,
    stderr: BufReader<ChildStderr>,
    addr: SocketAddr,
}

/// How a server ended, from [`Server::stop`].
pub(crate) struct Stopped {
    pub(crate) status: ExitStatus,
    pub(crate) stdout: String,
    /// Everything written to stderr after the `listening` line.
    pub(crate) stderr: String,
}

impl Server {
    /// Starts the binary with `CLINIC_ADDR=127.0.0.1:0` and waits until it's listening.
    ///
    /// Port 0 asks the OS for any free port, so tests can run in parallel. The binary reports the
    /// port it got on stderr.
    pub(crate) fn start() -> Self {
        let mut child = Command::new(env!("CARGO_BIN_EXE_clinic"))
            .env("CLINIC_ADDR", "127.0.0.1:0")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("the clinic binary should start");
        let mut stderr = BufReader::new(child.stderr.take().unwrap());

        let mut line = String::new();
        stderr.read_line(&mut line).unwrap();
        let Some(addr) = line.trim_end().strip_prefix(LISTENING) else {
            let mut rest = String::new();
            stderr.read_to_string(&mut rest).unwrap();
            let _ = child.kill();
            panic!("expected {LISTENING:?}, the binary wrote:\n{line}{rest}");
        };
        let addr = addr.parse().expect("the address should parse");

        Self {
            child,
            stderr,
            addr,
        }
    }

    /// Sends one request and returns the status and the body as text, exactly as sent.
    ///
    /// Each request uses a new connection, as separate `curl` commands would.
    pub(crate) async fn request(
        &self,
        method: &str,
        path: &str,
        clinic: Option<&str>,
        body: Option<&str>,
    ) -> (StatusCode, String) {
        let stream = TcpStream::connect(self.addr).await.unwrap();
        let (mut sender, connection) = hyper::client::conn::http1::handshake(TokioIo::new(stream))
            .await
            .unwrap();
        // The connection is a future that moves bytes to and from the socket. It runs on its own
        // task while `sender` sends the request and waits for the response.
        tokio::spawn(connection);

        let mut request = Request::builder()
            .method(method)
            .uri(path)
            .header("host", self.addr.to_string());
        if let Some(clinic) = clinic {
            request = request.header("x-clinic-id", clinic);
        }
        let body = match body {
            Some(body) => {
                request = request.header("content-type", "application/json");
                Bytes::from(body.to_owned())
            }
            None => Bytes::new(),
        };

        let response = sender
            .send_request(request.body(Full::new(body)).unwrap())
            .await
            .unwrap();
        let status = response.status();
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        (status, String::from_utf8(bytes.to_vec()).unwrap())
    }

    /// Sends SIGTERM, as `kill` and `docker stop` do, and waits for the binary to exit.
    #[cfg(unix)]
    pub(crate) fn stop(mut self) -> Stopped {
        let killed = Command::new("kill")
            .args(["-TERM", &self.child.id().to_string()])
            .status()
            .unwrap();
        assert!(killed.success(), "kill -TERM failed: {killed}");

        let deadline = Instant::now() + STOP_TIMEOUT;
        let status = loop {
            if let Some(status) = self.child.try_wait().unwrap() {
                break status;
            }
            assert!(
                Instant::now() < deadline,
                "the binary didn't stop within {STOP_TIMEOUT:?}"
            );
            thread::sleep(Duration::from_millis(20));
        };

        let mut stdout = String::new();
        self.child
            .stdout
            .take()
            .unwrap()
            .read_to_string(&mut stdout)
            .unwrap();
        let mut stderr = String::new();
        self.stderr.read_to_string(&mut stderr).unwrap();

        Stopped {
            status,
            stdout,
            stderr,
        }
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        // Errors are ignored: after `stop` the process is already gone.
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}
