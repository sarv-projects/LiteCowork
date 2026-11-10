-- Conversation history management lifecycle state (specs/014 T02, T03). Additive only.
-- A Conversation with no row here is ACTIVE. The side table keeps the immutable
-- v1 conversations table and every existing aggregate snapshot unchanged.
-- The title is never copied here; renames write only the conversations row.

CREATE TABLE conversation_history_state (
  conversation_id TEXT PRIMARY KEY REFERENCES conversations(conversation_id),
  workspace_id TEXT NOT NULL REFERENCES workspaces(workspace_id),
  status TEXT NOT NULL CHECK (status IN ('ACTIVE', 'ARCHIVED', 'DELETION_PENDING', 'DELETED')),
  archived_at TEXT,
  deleted_at TEXT,
  updated_at TEXT NOT NULL,
  CHECK ((status = 'ARCHIVED') = (archived_at IS NOT NULL) OR status IN ('DELETION_PENDING', 'DELETED'))
);
CREATE INDEX idx_conversation_history_state_workspace_status
  ON conversation_history_state(workspace_id, status);
