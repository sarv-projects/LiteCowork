use super::*;

use std::path::Path;

const OWNER: &str = "workspace-rename-owner";
const RUNTIME: &str = "workspace-rename-runtime";
const NOW: &str = "2026-11-10T12:00:00Z";

fn open_store(directory: &tempfile::TempDir) -> SqliteWorkspaceStore {
    fs::set_permissions(directory.path(), fs::Permissions::from_mode(0o700))
        .expect("private test data directory");
    SqliteWorkspaceStore::open(
        directory.path().join("state.sqlite3"),
        Arc::new(FileBlobStore::new(
            directory.path().join("blobs"),
            IpcReplayTestKeys,
        )),
        SqliteConfig {
            writer_queue_capacity: 8,
            busy_timeout: Duration::from_secs(2),
        },
    )
    .expect("real SQLite store opens")
}

fn create_workspace(store: &SqliteWorkspaceStore, workspace_id: &str) {
    WorkspaceService::new(store.clone())
        .create(CreateWorkspace {
            workspace_id: workspace_id.to_owned(),
            name: "Original name".to_owned(),
            owner_principal_id: OWNER.to_owned(),
            event: EventContext {
                event_id: format!("{workspace_id}-created"),
                origin_runtime_id: RUNTIME.to_owned(),
                hlc_timestamp: NOW.to_owned(),
                correlation_id: format!("{workspace_id}-created"),
                causation_id: None,
                recorded_at: NOW.to_owned(),
            },
        })
        .expect("owner Workspace commits");
}

fn renamed_event_count(directory: &tempfile::TempDir, workspace_id: &str) -> i64 {
    rusqlite::Connection::open(directory.path().join("state.sqlite3"))
        .expect("open event database")
        .query_row(
            "SELECT COUNT(*) FROM domain_events WHERE workspace_id = ?1 AND type = 'workspace.renamed.v1'",
            rusqlite::params![workspace_id],
            |row| row.get(0),
        )
        .expect("count renamed events")
}

fn router_for(store: &SqliteWorkspaceStore) -> Router {
    workspace_archive_routes()
        .route_layer(middleware::from_fn(authenticate))
        .with_state(WorkspaceArchiveApiState {
            store: store.clone(),
            principal_id: OWNER.to_owned(),
            runtime_id: RUNTIME.to_owned(),
        })
}

async fn rename_request(
    router: Router,
    workspace_id: &str,
    version: &str,
    key: &str,
    name: &str,
) -> Response {
    let mut request = Request::put(format!("/v1/workspaces/{workspace_id}/name"))
        .header("x-workspace-id", workspace_id)
        .header("if-match", version)
        .header("idempotency-key", key)
        .header("content-type", "application/json")
        .body(Body::from(json!({"name": name}).to_string()))
        .expect("rename request is valid HTTP");
    request.extensions_mut().insert(AuthenticatedLocalPeer);
    router
        .oneshot(request)
        .await
        .expect("rename route responds")
}

async fn error_code(response: Response) -> (Value, StatusCode) {
    let status = response.status();
    let body = to_bytes(response.into_body(), 8192)
        .await
        .expect("bounded error body");
    (
        serde_json::from_slice::<Value>(&body).expect("error JSON"),
        status,
    )
}

#[tokio::test]
async fn workspace_rename_commits_trimmed_name_and_replays_exact_key() {
    let directory = tempfile::tempdir().expect("temporary rename directory");
    let store = open_store(&directory);
    create_workspace(&store, "rename-ok");
    let router = router_for(&store);

    let first = rename_request(
        router.clone(),
        "rename-ok",
        "1",
        "rename-key-1",
        "  Renamed  ",
    )
    .await;
    assert_eq!(first.status(), StatusCode::OK);
    let first_body = to_bytes(first.into_body(), 16384)
        .await
        .expect("bounded body");
    let first_json: Value = serde_json::from_slice(&first_body).expect("Workspace JSON");
    assert_eq!(first_json["name"], "Renamed");
    assert_eq!(first_json["version"], 2);

    let replay = rename_request(router.clone(), "rename-ok", "1", "rename-key-1", "Renamed").await;
    assert_eq!(replay.status(), StatusCode::OK);
    let replay_body = to_bytes(replay.into_body(), 16384)
        .await
        .expect("bounded body");
    assert_eq!(
        serde_json::from_slice::<Value>(&replay_body).expect("Workspace JSON")["version"],
        2
    );
    assert_eq!(renamed_event_count(&directory, "rename-ok"), 1);

    let reused = rename_request(router, "rename-ok", "1", "rename-key-1", "Different").await;
    let (json, status) = error_code(reused).await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(json["code"], "CONFLICT");
    let stored = store.get_workspace("rename-ok").unwrap().unwrap();
    assert_eq!(stored.name, "Renamed");
    assert_eq!(stored.version, 2);
    assert_eq!(renamed_event_count(&directory, "rename-ok"), 1);
}

#[tokio::test]
async fn workspace_rename_stale_version_returns_current_name_without_writing() {
    let directory = tempfile::tempdir().expect("temporary rename directory");
    let store = open_store(&directory);
    create_workspace(&store, "rename-stale");
    let router = router_for(&store);
    assert_eq!(
        rename_request(router.clone(), "rename-stale", "1", "stale-key-1", "Latest")
            .await
            .status(),
        StatusCode::OK
    );

    let stale = rename_request(router, "rename-stale", "1", "stale-key-2", "Loser").await;
    let (json, status) = error_code(stale).await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(json["code"], "STALE_WORKSPACE_VERSION");
    assert_eq!(json["details"]["current_name"], "Latest");
    assert_eq!(json["details"]["current_version"], 2);
    assert_eq!(
        store.get_workspace("rename-stale").unwrap().unwrap().name,
        "Latest"
    );
    assert_eq!(renamed_event_count(&directory, "rename-stale"), 1);
}

#[tokio::test]
async fn workspace_rename_rejects_invalid_names_and_missing_headers_without_writes() {
    let directory = tempfile::tempdir().expect("temporary rename directory");
    let store = open_store(&directory);
    create_workspace(&store, "rename-invalid");
    let router = router_for(&store);

    for (index, name) in ["", "   ", "bad\u{0007}name", &"n".repeat(201)]
        .into_iter()
        .enumerate()
    {
        let response = rename_request(
            router.clone(),
            "rename-invalid",
            "1",
            &format!("invalid-key-{index}"),
            name,
        )
        .await;
        let (json, status) = error_code(response).await;
        assert_eq!(status, StatusCode::BAD_REQUEST, "name case {index}");
        assert_eq!(json["code"], "WORKSPACE_NAME_INVALID", "name case {index}");
    }

    let mut missing_version = Request::put("/v1/workspaces/rename-invalid/name")
        .header("x-workspace-id", "rename-invalid")
        .header("idempotency-key", "no-version")
        .header("content-type", "application/json")
        .body(Body::from(json!({"name": "X"}).to_string()))
        .expect("request is valid HTTP");
    missing_version
        .extensions_mut()
        .insert(AuthenticatedLocalPeer);
    let (json, status) = error_code(
        router
            .clone()
            .oneshot(missing_version)
            .await
            .expect("responds"),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(json["code"], "INVALID_ARGUMENT");

    let stored = store.get_workspace("rename-invalid").unwrap().unwrap();
    assert_eq!(stored.name, "Original name");
    assert_eq!(stored.version, 1);
    assert_eq!(renamed_event_count(&directory, "rename-invalid"), 0);
}

#[tokio::test]
async fn workspace_rename_is_rejected_for_archived_workspace() {
    let directory = tempfile::tempdir().expect("temporary rename directory");
    let store = open_store(&directory);
    create_workspace(&store, "rename-archived");
    let router = router_for(&store);
    let mut archive = Request::post("/v1/workspaces/rename-archived/archive")
        .header("x-workspace-id", "rename-archived")
        .header("if-match", "1")
        .header("idempotency-key", "archive-before-rename")
        .body(Body::empty())
        .expect("archive request is valid HTTP");
    archive.extensions_mut().insert(AuthenticatedLocalPeer);
    assert_eq!(
        router
            .clone()
            .oneshot(archive)
            .await
            .expect("responds")
            .status(),
        StatusCode::OK
    );

    let response = rename_request(
        router,
        "rename-archived",
        "2",
        "rename-archived-key",
        "Nope",
    )
    .await;
    let (json, status) = error_code(response).await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(json["code"], "WORKSPACE_ARCHIVED");
    assert_eq!(
        store
            .get_workspace("rename-archived")
            .unwrap()
            .unwrap()
            .name,
        "Original name"
    );
    assert_eq!(renamed_event_count(&directory, "rename-archived"), 0);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn workspace_rename_concurrent_requests_commit_exactly_one() {
    let directory = tempfile::tempdir().expect("temporary rename directory");
    let store = open_store(&directory);
    create_workspace(&store, "rename-race");
    let router = router_for(&store);

    let mut handles = Vec::new();
    for index in 0..4 {
        let router = router.clone();
        handles.push(tokio::spawn(async move {
            rename_request(
                router,
                "rename-race",
                "1",
                &format!("race-key-{index}"),
                &format!("Racer {index}"),
            )
            .await
            .status()
        }));
    }
    let mut ok = 0;
    let mut conflict = 0;
    for handle in handles {
        match handle.await.expect("task joins") {
            StatusCode::OK => ok += 1,
            StatusCode::CONFLICT => conflict += 1,
            other => panic!("unexpected status {other}"),
        }
    }
    assert_eq!(ok, 1, "exactly one rename commits");
    assert_eq!(conflict, 3);
    assert_eq!(
        store.get_workspace("rename-race").unwrap().unwrap().version,
        2
    );
    assert_eq!(renamed_event_count(&directory, "rename-race"), 1);
    let _ = Path::new("");
}
