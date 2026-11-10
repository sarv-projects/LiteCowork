-- Conversation co-edit and take-control records (specs/012 T05). Additive only.
-- A control event is immutable provenance for an owner take-over; an owner side note is a
-- bounded owner message correlated to a turn. Neither is ever a conversation_messages row,
-- so neither can enter a native prompt, and neither creates a turn.

CREATE TABLE conversation_control_events (
  control_event_id TEXT PRIMARY KEY,
  workspace_id TEXT NOT NULL REFERENCES workspaces(workspace_id),
  conversation_id TEXT NOT NULL REFERENCES conversations(conversation_id),
  turn_id TEXT NOT NULL REFERENCES conversation_turns(turn_id),
  kind TEXT NOT NULL CHECK (kind IN ('OWNER_TOOK_CONTROL', 'OWNER_RELEASED_CONTROL')),
  occurred_at TEXT NOT NULL,
  UNIQUE(turn_id, kind)
);
CREATE INDEX idx_conversation_control_events_conversation
  ON conversation_control_events(conversation_id, occurred_at);

CREATE TABLE conversation_owner_side_notes (
  side_note_id TEXT PRIMARY KEY,
  workspace_id TEXT NOT NULL REFERENCES workspaces(workspace_id),
  conversation_id TEXT NOT NULL REFERENCES conversations(conversation_id),
  correlated_turn_id TEXT NOT NULL REFERENCES conversation_turns(turn_id),
  body TEXT NOT NULL CHECK (length(body) BETWEEN 1 AND 4000),
  created_at TEXT NOT NULL
);
CREATE INDEX idx_conversation_owner_side_notes_turn
  ON conversation_owner_side_notes(conversation_id, correlated_turn_id, created_at);

CREATE TRIGGER conversation_control_event_immutable_update
BEFORE UPDATE ON conversation_control_events
BEGIN
  SELECT RAISE(ABORT, 'CONVERSATION_CONTROL_EVENT_IMMUTABLE');
END;

CREATE TRIGGER conversation_control_event_immutable_delete
BEFORE DELETE ON conversation_control_events
BEGIN
  SELECT RAISE(ABORT, 'CONVERSATION_CONTROL_EVENT_IMMUTABLE');
END;

CREATE TRIGGER conversation_owner_side_note_immutable_update
BEFORE UPDATE ON conversation_owner_side_notes
BEGIN
  SELECT RAISE(ABORT, 'CONVERSATION_SIDE_NOTE_IMMUTABLE');
END;
