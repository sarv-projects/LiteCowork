-- Conversation turn activity observations (specs/001 T25, AC-029). Additive only.
-- Rows are REPORTED by the native agent: they are observations of what the harness said
-- it did, never LiteCowork Effects, receipts or grants. They never contain file paths,
-- command lines, tool arguments, tool output or reasoning text.

CREATE TABLE conversation_turn_activity_records (
  activity_id TEXT PRIMARY KEY,
  turn_id TEXT NOT NULL REFERENCES conversation_turns(turn_id),
  retry_ordinal INTEGER NOT NULL CHECK (retry_ordinal >= 0),
  conversation_id TEXT NOT NULL REFERENCES conversations(conversation_id),
  sequence INTEGER NOT NULL CHECK (sequence >= 1),
  kind TEXT NOT NULL
    CHECK (kind IN ('TOOL', 'FILE', 'PERMISSION', 'TODO', 'STEP', 'CONTEXT')),
  status TEXT NOT NULL
    CHECK (status IN ('STARTED', 'PROGRESS', 'SUCCEEDED', 'FAILED', 'ASKED', 'UPDATED', 'ENDED', 'REDUCED')),
  call_ref TEXT CHECK (call_ref IS NULL OR length(call_ref) BETWEEN 1 AND 128),
  label TEXT CHECK (label IS NULL OR length(label) <= 128),
  attribution TEXT NOT NULL DEFAULT 'REPORTED' CHECK (attribution = 'REPORTED'),
  created_at TEXT NOT NULL,
  UNIQUE(turn_id, retry_ordinal, sequence)
);
CREATE INDEX idx_conversation_turn_activity_turn
  ON conversation_turn_activity_records(conversation_id, turn_id, retry_ordinal, sequence);

CREATE TRIGGER conversation_turn_activity_record_immutable_update
BEFORE UPDATE ON conversation_turn_activity_records
BEGIN
  SELECT RAISE(ABORT, 'CONVERSATION_ACTIVITY_RECORD_IMMUTABLE');
END;
