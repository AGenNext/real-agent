-- Real Agent — reference memory store (SurrealDB / SurrealQL)
--
-- This is a REFERENCE implementation only. The Real Agent specification is
-- database-agnostic (see SPEC.md §8 Non-goals). This schema shows one concrete
-- way to satisfy the spec's memory and decision-record requirements:
--   - §4.1 Identity        -> agent
--   - §4.4 Decision        -> decision
--   - §4.5 Action          -> action
--   - §4.6 Outcome         -> outcome
--   - §4.7 Memory          -> memory_fact (semantic) + the above (episodic)
--   - §6   Decision record -> every decision row IS the accountable unit
--
-- Apply with:
--   surreal import --conn http://localhost:8000 --user root --pass root \
--     --ns real_agent --db memory schemas/memory.surql

USE NS real_agent DB memory;

-- Identity (SPEC §4.1)
DEFINE TABLE agent SCHEMAFULL;
DEFINE FIELD name       ON agent TYPE string;
DEFINE FIELD version    ON agent TYPE string;
DEFINE FIELD owner      ON agent TYPE string;
DEFINE FIELD tenant     ON agent TYPE option<string>;
DEFINE FIELD role       ON agent TYPE option<string>;
DEFINE FIELD authority  ON agent FLEXIBLE TYPE object DEFAULT {};
DEFINE FIELD lifecycle  ON agent TYPE string DEFAULT 'draft';
DEFINE FIELD created_at ON agent TYPE datetime DEFAULT time::now();

-- Decision (SPEC §4.4 / §6 — the accountable unit of agency)
DEFINE TABLE decision SCHEMAFULL;
DEFINE FIELD agent         ON decision TYPE record<agent>;
DEFINE FIELD objective_ref ON decision TYPE option<string>;
DEFINE FIELD context_ref   ON decision TYPE option<string>;
DEFINE FIELD alternatives  ON decision TYPE array DEFAULT [];
DEFINE FIELD selected      ON decision FLEXIBLE TYPE object;
DEFINE FIELD reasoning     ON decision TYPE option<string>;
DEFINE FIELD confidence    ON decision TYPE option<number>
    ASSERT $value == NONE OR ($value >= 0 AND $value <= 1);
DEFINE FIELD policy_result ON decision TYPE string
    ASSERT $value IN ['allow', 'deny', 'escalate', 'approve'];
DEFINE FIELD created_at    ON decision TYPE datetime DEFAULT time::now();

-- Action (SPEC §4.5)
DEFINE TABLE action SCHEMAFULL;
DEFINE FIELD decision      ON action TYPE record<decision>;
DEFINE FIELD action_type   ON action TYPE string;
DEFINE FIELD target        ON action TYPE string;
DEFINE FIELD params        ON action FLEXIBLE TYPE object DEFAULT {};
DEFINE FIELD authorization ON action TYPE string;
DEFINE FIELD status        ON action TYPE string DEFAULT 'pending'
    ASSERT $value IN ['pending', 'running', 'done', 'failed'];
DEFINE FIELD reversible    ON action TYPE bool DEFAULT false;
DEFINE FIELD created_at    ON action TYPE datetime DEFAULT time::now();

-- Outcome (SPEC §4.6)
DEFINE TABLE outcome SCHEMAFULL;
DEFINE FIELD action     ON outcome TYPE record<action>;
DEFINE FIELD result     ON outcome FLEXIBLE TYPE option<object>;
DEFINE FIELD status     ON outcome TYPE string
    ASSERT $value IN ['success', 'failure'];
DEFINE FIELD error      ON outcome TYPE option<string>;
DEFINE FIELD impact     ON outcome TYPE option<string>;
DEFINE FIELD follow_up  ON outcome TYPE option<string>;
DEFINE FIELD created_at ON outcome TYPE datetime DEFAULT time::now();

-- Semantic memory (SPEC §4.7). Record id is [agent_id, key] so writes upsert.
DEFINE TABLE memory_fact SCHEMAFULL;
DEFINE FIELD agent      ON memory_fact TYPE record<agent>;
DEFINE FIELD key        ON memory_fact TYPE string;
DEFINE FIELD value      ON memory_fact FLEXIBLE TYPE any;
DEFINE FIELD source     ON memory_fact TYPE option<string>;
DEFINE FIELD created_at ON memory_fact TYPE datetime DEFAULT time::now();

-- Audit-friendly lookups
DEFINE INDEX decision_agent  ON decision FIELDS agent;
DEFINE INDEX action_decision ON action   FIELDS decision;
DEFINE INDEX outcome_action  ON outcome  FIELDS action;
