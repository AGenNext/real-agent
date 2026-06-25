-- Migration 0001: searchable agent tags
-- Adds a free-form tag set to agents plus an index, so the control plane can
-- group and filter agents (e.g. by team, environment, or risk tier).
-- Idempotent: re-applying is safe (DEFINE overwrites, data update is a no-op).

DEFINE FIELD IF NOT EXISTS tags ON agent TYPE option<array<string>> DEFAULT [];
DEFINE INDEX IF NOT EXISTS agent_tags ON agent FIELDS tags;
