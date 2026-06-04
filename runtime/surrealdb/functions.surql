-- Real Agent runtime — governed functions and derived state
-- =========================================================
-- Encodes spec rules as reusable SurrealQL functions and computed tables, so
-- governance checks and rollups live in the database, not scattered in clients.
-- Apply after schema.surql, tools.surql, register.surql.

-- Authority check: is this agent ALLOWED to invoke this tool? (capability is not
-- permission — this answers permission via the authorized_for graph edges.)
DEFINE FUNCTION fn::can_invoke($agent: record<agent>, $tool: record<tool>) -> bool {
  LET $allowed = (SELECT VALUE ->authorized_for->tool FROM ONLY $agent);
  RETURN $tool IN $allowed;
};

-- Does invoking this tool require approval for this agent? Combines the tool's
-- own default with any per-edge override.
DEFINE FUNCTION fn::needs_approval($agent: record<agent>, $tool: record<tool>) -> bool {
  LET $edge = (SELECT VALUE approval_required FROM authorized_for WHERE in = $agent AND out = $tool)[0];
  RETURN $edge ?? $tool.approval_required;
};

-- Latest measured trust score for an agent (derived from evaluation records).
DEFINE FUNCTION fn::trust_score($agent: record<agent>) -> option<number> {
  LET $latest = (SELECT trust_score, evaluated_at FROM evaluation WHERE agent = $agent ORDER BY evaluated_at DESC LIMIT 1)[0];
  RETURN $latest.trust_score;
};

-- Derived state: per-agent activity rollup over the append-only event log.
-- A computed table re-derives current state from accumulated events (the
-- ontology's derived-state principle), rather than mutating a counter.
DEFINE TABLE agent_activity TYPE NORMAL SCHEMAFULL AS
  SELECT agent, count() AS events, time::max(at) AS last_event_at
  FROM event GROUP BY agent;
