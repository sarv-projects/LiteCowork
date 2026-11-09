-- Conversation durable turn: single-flight admission, dispatch outbox, final-response
-- commit identity and per-turn usage records (specs/001 DATA-MODEL, B-01, B-04, B-10).
-- Additive only. Single-flight is enforced by triggers instead of a partial unique index
-- so that a database holding legacy duplicate nonterminal turns still migrates; the
-- triggers only constrain new admissions and re-openings.

CREATE TRIGGER conversation_turn_single_flight_insert
BEFORE INSERT ON conversation_turns
WHEN NEW.status IN ('OPEN', 'RUNNING', 'WAITING_USER', 'WAITING_DEPENDENCY', 'CANCEL_REQUESTED')
  AND EXISTS (
    SELECT 1 FROM conversation_turns other
    WHERE other.conversation_id = NEW.conversation_id
      AND other.status IN ('OPEN', 'RUNNING', 'WAITING_USER', 'WAITING_DEPENDENCY', 'CANCEL_REQUESTED')
  )
BEGIN
  SELECT RAISE(ABORT, 'CONVERSATION_TURN_IN_PROGRESS');
END;

CREATE TRIGGER conversation_turn_single_flight_reopen
BEFORE UPDATE OF status ON conversation_turns
WHEN OLD.status IN ('COMPLETED', 'FAILED', 'CANCELLED')
  AND NEW.status IN ('OPEN', 'RUNNING', 'WAITING_USER', 'WAITING_DEPENDENCY', 'CANCEL_REQUESTED')
  AND EXISTS (
    SELECT 1 FROM conversation_turns other
    WHERE other.conversation_id = NEW.conversation_id
      AND other.turn_id <> NEW.turn_id
      AND other.status IN ('OPEN', 'RUNNING', 'WAITING_USER', 'WAITING_DEPENDENCY', 'CANCEL_REQUESTED')
  )
BEGIN
  SELECT RAISE(ABORT, 'CONVERSATION_TURN_IN_PROGRESS');
END;

CREATE TABLE conversation_dispatch_intents (
  turn_id TEXT NOT NULL REFERENCES conversation_turns(turn_id),
  retry_ordinal INTEGER NOT NULL CHECK (retry_ordinal >= 0),
  conversation_id TEXT NOT NULL REFERENCES conversations(conversation_id),
  state TEXT NOT NULL CHECK (state IN ('UNCLAIMED', 'CLAIMED', 'SETTLED')),
  claimed_by_runtime_incarnation TEXT,
  claimed_at TEXT,
  settled_at TEXT,
  created_at TEXT NOT NULL,
  version INTEGER NOT NULL DEFAULT 1 CHECK (version >= 1),
  PRIMARY KEY(turn_id, retry_ordinal),
  CHECK ((state = 'UNCLAIMED' AND claimed_by_runtime_incarnation IS NULL AND claimed_at IS NULL AND settled_at IS NULL)
      OR (state = 'CLAIMED' AND claimed_by_runtime_incarnation IS NOT NULL AND claimed_at IS NOT NULL AND settled_at IS NULL)
      OR (state = 'SETTLED' AND settled_at IS NOT NULL))
);
CREATE INDEX idx_conversation_dispatch_intents_state
  ON conversation_dispatch_intents(state, created_at);

CREATE TRIGGER conversation_dispatch_intent_state_guard
BEFORE UPDATE ON conversation_dispatch_intents
WHEN NEW.turn_id <> OLD.turn_id
  OR NEW.retry_ordinal <> OLD.retry_ordinal
  OR NEW.conversation_id <> OLD.conversation_id
  OR NEW.created_at <> OLD.created_at
  OR NOT (
    (OLD.state = 'UNCLAIMED' AND NEW.state IN ('CLAIMED', 'SETTLED'))
    OR (OLD.state = 'CLAIMED' AND NEW.state IN ('CLAIMED', 'UNCLAIMED', 'SETTLED'))
  )
BEGIN
  SELECT RAISE(ABORT, 'CONVERSATION_DISPATCH_INTENT_TRANSITION_INVALID');
END;

CREATE TRIGGER conversation_dispatch_intent_no_delete
BEFORE DELETE ON conversation_dispatch_intents
BEGIN
  SELECT RAISE(ABORT, 'CONVERSATION_DISPATCH_INTENT_IMMUTABLE');
END;

-- Existing OPEN turns that never reached an AgentSession are owed a dispatch.
INSERT INTO conversation_dispatch_intents (turn_id, retry_ordinal, conversation_id, state, created_at)
SELECT turn_id, retry_ordinal, conversation_id, 'UNCLAIMED', created_at
FROM conversation_turns
WHERE status = 'OPEN' AND agent_session_id IS NULL;

CREATE TABLE conversation_response_commits (
  turn_id TEXT NOT NULL REFERENCES conversation_turns(turn_id),
  retry_ordinal INTEGER NOT NULL CHECK (retry_ordinal >= 0),
  provider_response_id TEXT NOT NULL CHECK (length(provider_response_id) BETWEEN 1 AND 256),
  content_digest TEXT NOT NULL CHECK (length(content_digest) BETWEEN 1 AND 128),
  message_id TEXT NOT NULL REFERENCES conversation_messages(message_id),
  receipt_request_id TEXT NOT NULL CHECK (length(receipt_request_id) BETWEEN 1 AND 256),
  created_at TEXT NOT NULL,
  PRIMARY KEY(turn_id, retry_ordinal),
  UNIQUE(message_id)
);

CREATE TRIGGER conversation_response_commit_immutable_update
BEFORE UPDATE ON conversation_response_commits
BEGIN
  SELECT RAISE(ABORT, 'CONVERSATION_RESPONSE_COMMIT_IMMUTABLE');
END;

CREATE TRIGGER conversation_response_commit_immutable_delete
BEFORE DELETE ON conversation_response_commits
BEGIN
  SELECT RAISE(ABORT, 'CONVERSATION_RESPONSE_COMMIT_IMMUTABLE');
END;

CREATE TABLE conversation_turn_usage_records (
  usage_id TEXT PRIMARY KEY,
  turn_id TEXT NOT NULL REFERENCES conversation_turns(turn_id),
  retry_ordinal INTEGER NOT NULL CHECK (retry_ordinal >= 0),
  conversation_id TEXT NOT NULL REFERENCES conversations(conversation_id),
  attribution_source TEXT NOT NULL
    CHECK (attribution_source IN ('PROVIDER_REPORTED', 'ESTIMATED_LOCAL', 'UNKNOWN')),
  input_tokens INTEGER CHECK (input_tokens IS NULL OR input_tokens >= 0),
  output_tokens INTEGER CHECK (output_tokens IS NULL OR output_tokens >= 0),
  created_at TEXT NOT NULL,
  UNIQUE(turn_id, retry_ordinal),
  CHECK (attribution_source <> 'UNKNOWN' OR (input_tokens IS NULL AND output_tokens IS NULL))
);

CREATE TRIGGER conversation_turn_usage_record_immutable_update
BEFORE UPDATE ON conversation_turn_usage_records
BEGIN
  SELECT RAISE(ABORT, 'CONVERSATION_USAGE_RECORD_IMMUTABLE');
END;

CREATE TRIGGER conversation_turn_usage_record_immutable_delete
BEFORE DELETE ON conversation_turn_usage_records
BEGIN
  SELECT RAISE(ABORT, 'CONVERSATION_USAGE_RECORD_IMMUTABLE');
END;
