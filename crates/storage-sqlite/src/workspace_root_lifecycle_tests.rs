use super::workspace_root_scan_tests::{
    INCARNATION, NOW, OWNER, ROOT, RUNTIME, WORKSPACE, database_path, fixture,
};
use super::*;
use domain_workspace::{
    ChangeWorkspaceRootStatus, EventContext, RevokeWorkspaceRoot, WorkspaceRootService,
};
use storage_core::WorkspaceRootStatusAction;

fn event(id: &str) -> EventContext {
    EventContext {
        event_id: id.to_owned(),
        origin_runtime_id: RUNTIME.to_owned(),
        hlc_timestamp: NOW.to_owned(),
        correlation_id: format!("correlation-{id}"),
        causation_id: None,
        recorded_at: NOW.to_owned(),
    }
}

fn change(
    action: WorkspaceRootStatusAction,
    request_id: &str,
    expected_version: u64,
) -> ChangeWorkspaceRootStatus {
    ChangeWorkspaceRootStatus {
        workspace_id: WORKSPACE.to_owned(),
        workspace_root_id: ROOT.to_owned(),
        principal_id: OWNER.to_owned(),
        request_id: request_id.to_owned(),
        expected_version,
        action,
        runtime_id: None,
        runtime_incarnation_id: None,
        event: event(&format!("event-{request_id}")),
    }
}

fn count(connection: &Connection, sql: &str, param: &str) -> i64 {
    connection
        .query_row(sql, [param], |row| row.get(0))
        .expect("count rows")
}

#[test]
fn workspace_root_lifecycle_pause_hides_root_and_stale_version_loses() {
    let fixture = fixture();
    let service = WorkspaceRootService::new(fixture.store.clone());
    let paused = service
        .change_root_status(change(WorkspaceRootStatusAction::Pause, "pause-1", 1))
        .expect("pause commits");
    assert_eq!(paused.root.status, "PAUSED");
    assert_eq!(paused.root.version, 2);

    // AC-004: a non-ACTIVE root is not a scan candidate.
    assert!(
        fixture
            .store
            .get_workspace_root_scan_candidate(OWNER, WORKSPACE, ROOT, RUNTIME, INCARNATION)
            .expect("candidate lookup")
            .is_none()
    );

    // Second pause from the same stale version loses with a version conflict.
    let error = service
        .change_root_status(change(WorkspaceRootStatusAction::Pause, "pause-2", 1))
        .expect_err("stale pause must lose");
    assert!(matches!(
        error,
        StoreError::Conflict {
            expected: Some(1),
            actual: Some(2)
        }
    ));
}

#[test]
fn workspace_root_lifecycle_revoke_deletes_private_bindings_and_is_terminal() {
    let fixture = fixture();
    let service = WorkspaceRootService::new(fixture.store.clone());
    let connection = Connection::open(database_path(&fixture.directory)).expect("inspect SQLite");
    assert_eq!(
        count(
            &connection,
            "SELECT COUNT(*) FROM resource_location_bindings WHERE location_id=?1",
            "location-scan-test"
        ),
        1
    );
    let revoked = service
        .revoke_root(RevokeWorkspaceRoot {
            workspace_id: WORKSPACE.to_owned(),
            workspace_root_id: ROOT.to_owned(),
            principal_id: OWNER.to_owned(),
            request_id: "revoke-1".to_owned(),
            expected_version: 1,
            event: event("event-revoke-1"),
        })
        .expect("revoke commits");
    assert_eq!(revoked.root.status, "REVOKED");
    for table in ["resource_location_bindings", "file_identity_bindings"] {
        assert_eq!(
            count(
                &connection,
                &format!("SELECT COUNT(*) FROM {table} WHERE location_id=?1"),
                "location-scan-test"
            ),
            0,
            "{table} must be deleted on revoke"
        );
    }

    // AC-005: a REVOKED root never returns to ACTIVE; resume and pause are refused.
    let mut resume = change(WorkspaceRootStatusAction::Resume, "resume-after-revoke", 2);
    resume.runtime_id = Some(RUNTIME.to_owned());
    resume.runtime_incarnation_id = Some(INCARNATION.to_owned());
    assert!(
        WorkspaceRootService::new(fixture.store.clone())
            .change_root_status(resume)
            .is_err()
    );
    assert!(
        service
            .change_root_status(change(WorkspaceRootStatusAction::Pause, "pause-revoked", 2))
            .is_err()
    );
    let root = fixture
        .store
        .get_workspace_root(WORKSPACE, ROOT)
        .expect("read root")
        .expect("root exists");
    assert_eq!(root.status, "REVOKED");
    assert!(
        fixture
            .store
            .get_workspace_root_scan_candidate(OWNER, WORKSPACE, ROOT, RUNTIME, INCARNATION)
            .expect("candidate lookup")
            .is_none()
    );
}

#[test]
fn workspace_root_lifecycle_same_key_replays_and_foreign_principal_is_not_found() {
    let fixture = fixture();
    let service = WorkspaceRootService::new(fixture.store.clone());
    let first = service
        .change_root_status(change(WorkspaceRootStatusAction::Pause, "pause-replay", 1))
        .expect("first pause");
    let replay = service
        .change_root_status(change(WorkspaceRootStatusAction::Pause, "pause-replay", 1))
        .expect("identical replay");
    assert_eq!(first.root.version, replay.root.version);
    assert_eq!(replay.root.status, "PAUSED");

    // Changed payload (different action) under the same key must not replay.
    assert!(
        service
            .change_root_status(change(
                WorkspaceRootStatusAction::Revoke,
                "pause-replay",
                1
            ))
            .is_err()
    );
    let mut foreign = change(WorkspaceRootStatusAction::Pause, "foreign-pause", 2);
    foreign.principal_id = "foreign-owner".to_owned();
    assert_eq!(
        service
            .change_root_status(foreign)
            .expect_err("foreign principal"),
        StoreError::NotFound
    );
}
