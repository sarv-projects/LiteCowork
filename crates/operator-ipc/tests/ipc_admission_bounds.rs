#![cfg(unix)]

use operator_ipc::{
    DEFAULT_MAX_IN_FLIGHT_BODY_BYTES, InFlightBodyBudget, LogicalHeader, MAX_BODY_BYTES,
    PROTOCOL_VERSION, ProtocolError, RequestFrame, RequestHeader, ResponseFrame, ResponseHeader,
    unix::{MAX_CONCURRENT_CONNECTIONS, UnixEndpoint},
};
use std::{
    fs,
    os::unix::fs::PermissionsExt,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

struct PrivateTempDir(PathBuf);

impl PrivateTempDir {
    fn new() -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock is after the Unix epoch")
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "litecowork-ipc-bounds-{}-{nonce}",
            std::process::id()
        ));
        fs::create_dir(&path).expect("private test directory is created");
        fs::set_permissions(&path, fs::Permissions::from_mode(0o700))
            .expect("directory is owner-only");
        Self(path)
    }
}

impl Drop for PrivateTempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn request_header(id: &str, body_length: u64) -> RequestHeader {
    RequestHeader {
        protocol_version: PROTOCOL_VERSION,
        request_id: id.to_owned(),
        method: "POST".to_owned(),
        path_and_query: "/v1/echo".to_owned(),
        headers: vec![LogicalHeader {
            name: "Content-Type".to_owned(),
            value: "application/octet-stream".to_owned(),
        }],
        body_length,
    }
}

fn response_header(id: &str, body_length: u64) -> ResponseHeader {
    ResponseHeader {
        protocol_version: PROTOCOL_VERSION,
        request_id: id.to_owned(),
        status: 200,
        headers: vec![LogicalHeader {
            name: "Content-Type".to_owned(),
            value: "application/octet-stream".to_owned(),
        }],
        body_length,
    }
}

#[test]
fn ipc_admission_bounds_combined_bodies_over_the_in_flight_limit_are_refused() {
    let budget = InFlightBodyBudget::new(DEFAULT_MAX_IN_FLIGHT_BODY_BYTES).expect("budget");
    // 100 MiB (the per-frame maximum) plus 28 MiB fills the 128 MiB budget exactly.
    let first = budget.reserve_bytes(MAX_BODY_BYTES).expect("first body");
    let rest = DEFAULT_MAX_IN_FLIGHT_BODY_BYTES - MAX_BODY_BYTES;
    let second = budget
        .reserve_bytes(rest)
        .expect("second body fits exactly");
    assert!(matches!(
        budget.reserve_bytes(1),
        Err(ProtocolError::CapacityExceeded)
    ));
    // A single body above the per-frame limit is a limit error, not a capacity wait.
    assert!(matches!(
        budget.reserve_bytes(MAX_BODY_BYTES + 1),
        Err(ProtocolError::LimitExceeded)
    ));
    drop(first);
    let again = budget
        .reserve_bytes(1)
        .expect("released bytes are reusable");
    drop((second, again));
    budget
        .reserve_bytes(MAX_BODY_BYTES)
        .expect("all capacity returns after every permit drops");
}

#[test]
fn ipc_admission_bounds_wire_request_is_refused_before_body_is_read_when_budget_is_full() {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("runtime");
    runtime.block_on(async {
        let directory = PrivateTempDir::new();
        let endpoint = UnixEndpoint::derive(&directory.0).expect("endpoint derives");
        let listener = endpoint.bind().expect("binds");
        let budget = InFlightBodyBudget::new(1024 * 1024).expect("small budget");
        let held = budget.reserve_bytes(1024 * 1024).expect("budget is full");

        // The client declares a body that cannot fit; it sends only the header.
        let client_budget = InFlightBodyBudget::new(4 * 1024 * 1024).expect("client budget");
        let mut client = endpoint.connect(|_| true).await.expect("connects");
        let mut server = listener.accept(|_| true).await.expect("accepted");
        let frame = RequestFrame::new(
            request_header("too-big", 512 * 1024),
            vec![7_u8; 512 * 1024],
            &client_budget,
        )
        .expect("client frame");
        // The client write may fail once the server drops the refused exchange.
        let writer = tokio::spawn(async move {
            let _ = client.write_request(&frame).await;
        });
        let refused = server.read_request(&budget).await;
        drop(server);
        let _ = writer.await;
        assert!(
            refused.is_err(),
            "a body that cannot be reserved must not be admitted"
        );
        drop(held);
    });
}

#[test]
fn ipc_admission_bounds_eight_concurrent_peers_get_uninterleaved_correlated_responses() {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("runtime");
    runtime.block_on(async {
        let directory = PrivateTempDir::new();
        let endpoint = UnixEndpoint::derive(&directory.0).expect("endpoint derives");
        let listener = endpoint.bind().expect("binds");
        let budget = InFlightBodyBudget::new(DEFAULT_MAX_IN_FLIGHT_BODY_BYTES).expect("budget");

        let mut clients = Vec::new();
        for index in 0..MAX_CONCURRENT_CONNECTIONS {
            let endpoint = endpoint.clone();
            let budget = std::sync::Arc::clone(&budget);
            clients.push(tokio::spawn(async move {
                let id = format!("peer-{index}");
                let payload = vec![index as u8 + 1; 256 * 1024 + index];
                let mut connection = endpoint.connect(|_| true).await.expect("client connects");
                let request = RequestFrame::new(
                    request_header(&id, payload.len() as u64),
                    payload.clone(),
                    &budget,
                )
                .expect("request frame");
                connection.write_request(&request).await.expect("writes");
                let response = connection.read_response(&budget).await.expect("reads");
                assert_eq!(response.header.request_id, id);
                assert_eq!(response.body(), payload.as_slice());
            }));
        }

        let mut servers = Vec::new();
        for _ in 0..MAX_CONCURRENT_CONNECTIONS {
            let mut connection = listener.accept(|_| true).await.expect("accepted");
            let budget = std::sync::Arc::clone(&budget);
            servers.push(tokio::spawn(async move {
                let request = connection.read_request(&budget).await.expect("request");
                let id = request.header.request_id.clone();
                let body = request.body().to_vec();
                let response =
                    ResponseFrame::new(response_header(&id, body.len() as u64), body, &budget)
                        .expect("response frame");
                connection
                    .write_response(&response)
                    .await
                    .expect("responds");
                // Hold the slot until the client has consumed the response.
                let _ = connection.wait_for_stream_disconnect().await;
            }));
        }
        for client in clients {
            client.await.expect("client task");
        }
        for server in servers {
            server.await.expect("server task");
        }
    });
}
