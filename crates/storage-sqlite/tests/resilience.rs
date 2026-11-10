//! Release resilience cases against the shipped `storage-sqlite` code (019-T18).
//!
//! These run natively on every OS the crate builds on; none is `cfg`-skipped. They use the
//! agent registry cache as the durable write because it is a public, self-contained store
//! operation. Cases that need the daemon (failed migration leaves DEGRADED, single-instance
//! lock) belong to `apps/litecoworkd` and are not claimed here.

use std::sync::Arc;

use rusqlite::Connection;
use serde_json::json;
use storage_core::{
    AgentRegistryCacheEntry, AgentRegistryCacheSnapshot, AgentRegistryCacheStore, BlobPurpose,
    StoreError,
};
use storage_sqlite::{
    FileBlobStore, SqliteConfig, SqliteWorkspaceStore, WorkspaceBlobKey, WorkspaceBlobKeyProvider,
};
use zeroize::Zeroizing;

struct TestKeys;

impl WorkspaceBlobKeyProvider for TestKeys {
    fn current_key(
        &self,
        _workspace_id: &str,
        _purpose: BlobPurpose,
    ) -> Result<WorkspaceBlobKey, StoreError> {
        Ok(WorkspaceBlobKey {
            version: 1,
            bytes: Zeroizing::new([13; 32]),
        })
    }

    fn key_by_version(
        &self,
        workspace_id: &str,
        purpose: BlobPurpose,
        version: u32,
    ) -> Result<WorkspaceBlobKey, StoreError> {
        if version != 1 {
            return Err(StoreError::Blob("unknown test key version".to_owned()));
        }
        self.current_key(workspace_id, purpose)
    }
}

fn snapshot(revision: &str) -> AgentRegistryCacheSnapshot {
    AgentRegistryCacheSnapshot {
        source_registry: "https://registry.example.test/index.json".to_owned(),
        source_revision: Some(revision.to_owned()),
        observed_at: "2026-10-19T10:00:00Z".to_owned(),
        expires_at: Some("2026-10-19T10:15:00Z".to_owned()),
        entries: vec![AgentRegistryCacheEntry {
            registry_agent_id: "agent-a".to_owned(),
            metadata: json!({"name":"agent-a","distribution":{"npx":{"package":"safe-package"}}}),
        }],
    }
}

fn open_store(state: &std::path::Path) -> Result<SqliteWorkspaceStore, StoreError> {
    std::fs::create_dir_all(state).expect("state directory");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(state, std::fs::Permissions::from_mode(0o700))
            .expect("private state directory");
    }
    let blobs = Arc::new(FileBlobStore::new(state.join("blobs"), TestKeys));
    SqliteWorkspaceStore::open(
        state.join("litecowork.sqlite3"),
        blobs,
        SqliteConfig::default(),
    )
}

fn integrity_check(database: &std::path::Path) -> String {
    let connection = Connection::open(database).expect("open for integrity check");
    connection
        .query_row("PRAGMA integrity_check", [], |row| row.get(0))
        .expect("integrity_check")
}

#[test]
fn committed_write_survives_restart_and_integrity_is_ok() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let state = directory.path().join("state");
    let committed = snapshot("committed");
    {
        let store = open_store(&state).expect("first open");
        store
            .replace_agent_registry_snapshot(committed.clone())
            .expect("write commits");
    }
    let reopened = open_store(&state).expect("reopen after restart");
    assert_eq!(
        reopened
            .read_agent_registry_snapshot(&committed.source_registry)
            .expect("read after restart"),
        Some(committed)
    );
    drop(reopened);
    assert_eq!(integrity_check(&state.join("litecowork.sqlite3")), "ok");
}

#[test]
fn uncommitted_foreign_transaction_is_rolled_back_by_restart() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let state = directory.path().join("state");
    let committed = snapshot("committed");
    {
        let store = open_store(&state).expect("first open");
        store
            .replace_agent_registry_snapshot(committed.clone())
            .expect("write commits");
    }
    // A second connection starts a write and the "process" ends before commit.
    {
        let connection = Connection::open(state.join("litecowork.sqlite3")).expect("raw open");
        connection
            .execute_batch("BEGIN IMMEDIATE; DELETE FROM agent_registry_cache;")
            .expect("uncommitted delete");
        // Dropping without COMMIT models the crash: SQLite rolls the transaction back.
    }
    let reopened = open_store(&state).expect("reopen after crash");
    assert_eq!(
        reopened
            .read_agent_registry_snapshot(&committed.source_registry)
            .expect("read after crash"),
        Some(committed)
    );
    drop(reopened);
    assert_eq!(integrity_check(&state.join("litecowork.sqlite3")), "ok");
}

#[test]
fn page_capped_database_refuses_a_write_and_stays_consistent() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let state = directory.path().join("state");
    {
        let store = open_store(&state).expect("first open");
        store
            .replace_agent_registry_snapshot(snapshot("before-full"))
            .expect("write commits");
    }
    let database = state.join("litecowork.sqlite3");
    let connection = Connection::open(&database).expect("raw open");
    let pages: i64 = connection
        .query_row("PRAGMA page_count", [], |row| row.get(0))
        .expect("page count");
    connection
        .pragma_update(None, "max_page_count", pages)
        .expect("cap pages");
    let result = connection.execute(
        "INSERT INTO agent_registry_cache (registry_agent_id, source_registry, source_revision, metadata_json, observed_at, expires_at) \
         VALUES ('agent-full', 'https://full.example.test', 'r', json_object('pad', ?1), '2026-10-19T10:00:00Z', '2026-10-19T10:15:00Z')",
        [&"x".repeat(1024 * 1024)],
    );
    // The refusal must be the storage-full error, not a schema or syntax mistake.
    let error = result.expect_err("a write beyond the page cap must be refused");
    assert!(
        error.to_string().to_lowercase().contains("full"),
        "unexpected error: {error}"
    );
    drop(connection);
    assert_eq!(integrity_check(&database), "ok");
    let reopened = open_store(&state).expect("reopen after full disk");
    assert!(
        reopened
            .read_agent_registry_snapshot("https://full.example.test")
            .expect("read")
            .is_none()
    );
}
