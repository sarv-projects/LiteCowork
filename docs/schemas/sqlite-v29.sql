-- Conversation durable-turn additions for Feature 001 (Conversation Session Supervisor).
-- Forward-only and additive: no existing Conversation, turn, message, or session row is
-- rewritten. This migration wires the durable dispatch outbox, single-flight admission,
-- the final-response commit identity fields, and the per-turn usage record.

-- 1. Durable dispatch intent (outbox). One row is inserted in the same transaction as
--    turn admission and as the Retry transition, keyed by (turn, retry ordinal). The
--    dispatcher claims UNCLAIMED -> CLAIMED by compare-and-swap, stamping the claiming
--    Runtime incarnation; settlement marks it SETTLED. A crash before the claim leaves an
--    UNCLAIMED row for the startup recovery sweep. The native handle is never recorded here.
CREATE TABLE conversation_dispatch_intents (
  turn_id TEXT NOT NULL,
  retry_ordinal INTEGER NOT NULL CHECK (retry_ordinal >= 0),
  conversation_id TEXT NOT NULL REFERENCES conversations(conversation_id),
  workspace_id TEXT NOT NULL REFERENCES workspaces(workspace_id),
  state TEXT NOT NULL CHECK (state IN ('UNCLAIMED', 'CLAIMED', 'SETTLED')),
  claimed_runtime_id TEXT,
  claimed_runtime_incarnation_id TEXT,
  created_at TEXT NOT NULL,
  claimed_at TEXT,
  settled_at TEXT,
  PRIMARY KEY (turn_id, retry_ordinal),
  FOREIGN KEY (conversation_id, turn_id)
    REFERENCES conversation_turns(conversation_id, turn_id),
  FOREIGN KEY (claimed_runtime_id, claimed_runtime_incarnation_id)
    REFERENCES runtime_incarnations(runtime_id, runtime_incarnation_id) DEFERRABLE INITIALLY DEFERRED,
  CHECK (
    (state = 'UNCLAIMED' AND claimed_runtime_id IS NULL AND claimed_runtime_incarnation_id IS NULL AND claimed_at IS NULL)
    OR (state = 'CLAIMED' AND claimed_runtime_id IS NOT NULL AND claimed_runtime_incarnation_id IS NOT NULL AND claimed_at IS NOT NULL AND settled_at IS NULL)
    OR (state = 'SETTLED')
  )
);
CREATE INDEX idx_conversation_dispatch_intents_unclaimed
  ON conversation_dispatch_intents(state, created_at) WHERE state = 'UNCLAIMED';

-- 2. Single-flight admission. At most one nonterminal turn may exist per Conversation.
--    The storage admission transaction rejects a concurrent submit with
--    CONVERSATION_TURN_IN_PROGRESS; this partial unique index is the durable authority
--    that makes two racing stale-snapshot submits collapse to one admission.
CREATE UNIQUE INDEX uq_conversation_single_flight_turn
  ON conversation_turns(conversation_id)
  WHERE status IN ('OPEN', 'RUNNING', 'WAITING_USER', 'WAITING_DEPENDENCY', 'CANCEL_REQUESTED');

-- 3. Final-response commit identity. The provider-response identifier observed on the
--    native final message is recorded on the committed AGENT message so the receipt
--    identity is derived from (turn_id, retry_ordinal, provider_response_id) rather than a
--    caller-generated message id. It is NULL for USER, CHANNEL, and SYSTEM_NOTICE rows and
--    for AGENT rows written before this migration.
ALTER TABLE conversation_messages ADD COLUMN provider_response_id TEXT;

-- 4. Per-turn usage record. One row is written in the settlement transaction (including
--    FAILED and CANCELLED with whatever was observed). PROVIDER_REPORTED carries the tokens
--    the harness reported; ESTIMATED_LOCAL carries a byte-derived estimate; UNKNOWN carries
--    neither. No cost is inferred and settlement is never blocked on missing telemetry. A
--    replay of the settlement writes no second row (one row per turn + retry ordinal).
CREATE TABLE conversation_usage_records (
  turn_id TEXT NOT NULL,
  retry_ordinal INTEGER NOT NULL CHECK (retry_ordinal >= 0),
  conversation_id TEXT NOT NULL REFERENCES conversations(conversation_id),
  workspace_id TEXT NOT NULL REFERENCES workspaces(workspace_id),
  attribution_source TEXT NOT NULL
    CHECK (attribution_source IN ('PROVIDER_REPORTED', 'ESTIMATED_LOCAL', 'UNKNOWN')),
  input_tokens INTEGER CHECK (input_tokens IS NULL OR input_tokens >= 0),
  output_tokens INTEGER CHECK (output_tokens IS NULL OR output_tokens >= 0),
  recorded_at TEXT NOT NULL,
  PRIMARY KEY (turn_id, retry_ordinal),
  FOREIGN KEY (conversation_id, turn_id)
    REFERENCES conversation_turns(conversation_id, turn_id),
  CHECK (
    (attribution_source = 'UNKNOWN' AND input_tokens IS NULL AND output_tokens IS NULL)
    OR (attribution_source <> 'UNKNOWN' AND input_tokens IS NOT NULL AND output_tokens IS NOT NULL)
  )
);
CREATE INDEX idx_conversation_usage_records_workspace
  ON conversation_usage_records(workspace_id, recorded_at);
