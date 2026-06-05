-- Real Agent — reference memory store (PostgreSQL)
--
-- This is a REFERENCE implementation only. The Real Agent specification is
-- database-agnostic (see SPEC.md §8 Non-goals). This schema shows one concrete
-- way to satisfy the spec's memory and decision-record requirements:
--   - §4.4 Decision        -> decisions
--   - §4.5 Action          -> actions
--   - §4.6 Outcome         -> outcomes
--   - §4.7 Memory          -> all of the above (episodic) + memory_facts (semantic)
--   - §6   Decision record -> every decision row IS the accountable unit
--
-- Apply with:
--   kubectl -n database exec -i postgres-0 -- psql -U app -d appdb < schemas/memory.sql

BEGIN;

-- Agents (SPEC §4.1 Identity)
CREATE TABLE IF NOT EXISTS agents (
    id            TEXT PRIMARY KEY,          -- stable unique identifier
    name          TEXT NOT NULL,
    version       TEXT NOT NULL,
    owner         TEXT NOT NULL,
    tenant        TEXT,                       -- operating boundary
    role          TEXT,
    authority     JSONB NOT NULL DEFAULT '{}',-- authority scope
    lifecycle     TEXT NOT NULL DEFAULT 'draft', -- SPEC §5
    created_at    TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Decisions (SPEC §4.4 / §6 — the accountable unit of agency)
CREATE TABLE IF NOT EXISTS decisions (
    id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    agent_id      TEXT NOT NULL REFERENCES agents(id),
    objective_ref TEXT,
    context_ref   TEXT,
    alternatives  JSONB NOT NULL DEFAULT '[]',
    selected      JSONB NOT NULL,
    reasoning     TEXT,
    confidence    NUMERIC(4,3) CHECK (confidence BETWEEN 0 AND 1),
    policy_result TEXT NOT NULL,              -- allow | deny | escalate | approve
    created_at    TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Actions (SPEC §4.5)
CREATE TABLE IF NOT EXISTS actions (
    id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    decision_id   UUID NOT NULL REFERENCES decisions(id),
    action_type   TEXT NOT NULL,
    target        TEXT NOT NULL,              -- target system or resource
    params        JSONB NOT NULL DEFAULT '{}',
    authorization TEXT NOT NULL,              -- authorization result
    status        TEXT NOT NULL DEFAULT 'pending', -- pending|running|done|failed
    reversible    BOOLEAN NOT NULL DEFAULT false,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Outcomes (SPEC §4.6)
CREATE TABLE IF NOT EXISTS outcomes (
    id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    action_id     UUID NOT NULL REFERENCES actions(id),
    result        JSONB,
    status        TEXT NOT NULL,              -- success | failure
    error         TEXT,
    impact        TEXT,
    follow_up     TEXT,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Semantic memory (SPEC §4.7 — facts/knowledge the agent retains)
CREATE TABLE IF NOT EXISTS memory_facts (
    id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    agent_id      TEXT NOT NULL REFERENCES agents(id),
    key           TEXT NOT NULL,
    value         JSONB NOT NULL,
    source        TEXT,                       -- traceable to source (SPEC §4.3)
    created_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (agent_id, key)
);

-- Audit-friendly lookups
CREATE INDEX IF NOT EXISTS idx_decisions_agent ON decisions(agent_id, created_at);
CREATE INDEX IF NOT EXISTS idx_actions_decision ON actions(decision_id);
CREATE INDEX IF NOT EXISTS idx_outcomes_action  ON outcomes(action_id);

COMMIT;
