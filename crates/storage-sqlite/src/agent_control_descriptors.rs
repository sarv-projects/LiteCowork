use super::*;
use storage_core::{
    AgentControlDescriptorRecord, AgentDescriptorGate, AgentDescriptorStore,
    validate_agent_control_descriptor_record,
};

impl AgentDescriptorStore for SqliteWorkspaceStore {
    fn put_agent_control_descriptor(
        &self,
        mut record: AgentControlDescriptorRecord,
        expected_prior_digest: Option<String>,
    ) -> Result<(), StoreError> {
        validate_agent_control_descriptor_record(&record).map_err(StoreError::Invalid)?;
        record.observed_at = canonicalize_utc_timestamp(&record.observed_at)?;
        record.expires_at = canonicalize_utc_timestamp(&record.expires_at)?;
        if record.observed_at >= record.expires_at {
            return Err(StoreError::Invalid(
                "AgentControlDescriptor freshness window is invalid".to_owned(),
            ));
        }
        self.run_descriptor_operation(move |connection| {
            let transaction = connection
                .transaction_with_behavior(TransactionBehavior::Immediate)
                .map_err(map_database_error)?;
            let prior: Option<String> = transaction
                .query_row(
                    "SELECT descriptor_digest FROM agent_control_descriptors
                     WHERE registry_agent_id = ?1 AND runtime_id = ?2
                       AND runtime_incarnation_id = ?3 AND module_version = ?4
                     ORDER BY observed_at DESC, rowid DESC LIMIT 1",
                    params![
                        record.registry_agent_id,
                        record.runtime_id,
                        record.runtime_incarnation_id,
                        record.module_version
                    ],
                    |row| row.get(0),
                )
                .optional()
                .map_err(map_database_error)?;
            if prior != expected_prior_digest {
                return Err(StoreError::Conflict {
                    expected: None,
                    actual: None,
                });
            }
            transaction
                .execute(
                    "INSERT INTO agent_control_descriptors(
                       descriptor_digest, registry_agent_id, runtime_id, runtime_incarnation_id,
                       module_id, module_version, descriptor_api_version, descriptor_json,
                       observed_at, expires_at
                     ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                    params![
                        record.descriptor_digest,
                        record.registry_agent_id,
                        record.runtime_id,
                        record.runtime_incarnation_id,
                        record.module_id,
                        record.module_version,
                        i64::from(record.descriptor_api_version),
                        record.descriptor_json,
                        record.observed_at,
                        record.expires_at,
                    ],
                )
                .map_err(map_database_error)?;
            // Keep the stream bounded: only the newest revisions can ever be current.
            transaction
                .execute(
                    "DELETE FROM agent_control_descriptors
                     WHERE registry_agent_id = ?1 AND runtime_id = ?2
                       AND runtime_incarnation_id = ?3 AND module_version = ?4
                       AND descriptor_digest <> ?5 AND expires_at <= ?6",
                    params![
                        record.registry_agent_id,
                        record.runtime_id,
                        record.runtime_incarnation_id,
                        record.module_version,
                        record.descriptor_digest,
                        record.observed_at,
                    ],
                )
                .map_err(map_database_error)?;
            transaction.commit().map_err(map_database_error)
        })
    }

    fn latest_agent_control_descriptor(
        &self,
        gate: &AgentDescriptorGate,
    ) -> Result<Option<AgentControlDescriptorRecord>, StoreError> {
        let gate = gate.clone();
        self.run_descriptor_operation(move |connection| latest_descriptor(connection, &gate))
    }

    fn current_agent_control_descriptor(
        &self,
        gate: &AgentDescriptorGate,
        now: &str,
    ) -> Result<Option<AgentControlDescriptorRecord>, StoreError> {
        let now = canonicalize_utc_timestamp(now)?;
        let gate = gate.clone();
        self.run_descriptor_operation(move |connection| {
            latest_descriptor(connection, &gate)
                .map(|row| row.filter(|record| record.expires_at.as_str() > now.as_str()))
        })
    }
}

impl SqliteWorkspaceStore {
    fn run_descriptor_operation<T: Send + 'static>(
        &self,
        operation: impl FnOnce(&mut Connection) -> Result<T, StoreError> + Send + 'static,
    ) -> Result<T, StoreError> {
        let (reply, receive) = mpsc::channel();
        self.execute_command(
            Command::Operation {
                operation: Box::new(move |connection| {
                    let _ = reply.send(operation(connection));
                }),
            },
            receive,
        )
    }
}

fn latest_descriptor(
    connection: &Connection,
    gate: &AgentDescriptorGate,
) -> Result<Option<AgentControlDescriptorRecord>, StoreError> {
    connection
        .query_row(
            "SELECT registry_agent_id, runtime_id, runtime_incarnation_id, module_id,
                    module_version, descriptor_api_version, descriptor_digest,
                    descriptor_json, observed_at, expires_at
             FROM agent_control_descriptors
             WHERE registry_agent_id = ?1 AND runtime_id = ?2
               AND runtime_incarnation_id = ?3 AND module_version = ?4
             ORDER BY observed_at DESC, rowid DESC LIMIT 1",
            params![
                gate.registry_agent_id,
                gate.runtime_id,
                gate.runtime_incarnation_id,
                gate.module_version
            ],
            |row| {
                Ok(AgentControlDescriptorRecord {
                    registry_agent_id: row.get(0)?,
                    runtime_id: row.get(1)?,
                    runtime_incarnation_id: row.get(2)?,
                    module_id: row.get(3)?,
                    module_version: row.get(4)?,
                    descriptor_api_version: u32::try_from(row.get::<_, i64>(5)?)
                        .unwrap_or(u32::MAX),
                    descriptor_digest: row.get(6)?,
                    descriptor_json: row.get(7)?,
                    observed_at: row.get(8)?,
                    expires_at: row.get(9)?,
                })
            },
        )
        .optional()
        .map_err(map_database_error)
}

/// True when `descriptor_revision` is the latest fresh revision of the gate's stream.
/// Used by the enable gate inside its own transaction so a refresh committed between
/// the UI read and the enable can never enable against the older revision.
pub(super) fn descriptor_revision_is_current(
    connection: &Connection,
    gate: &AgentDescriptorGate,
    descriptor_revision: &str,
    now: &str,
) -> Result<bool, StoreError> {
    let now = canonicalize_utc_timestamp(now)?;
    Ok(latest_descriptor(connection, gate)?.is_some_and(|record| {
        record.expires_at.as_str() > now.as_str() && record.descriptor_digest == descriptor_revision
    }))
}
