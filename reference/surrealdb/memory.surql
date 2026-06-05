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
DEFINE FIELD authority  ON agent TYPE option<object> FLEXIBLE;
DEFINE FIELD lifecycle  ON agent TYPE string DEFAULT 'draft';
DEFINE FIELD created_at ON agent TYPE datetime DEFAULT ALWAYS time::now();

-- Decision (SPEC §4.4 / §6 — the accountable unit of agency)
DEFINE TABLE decision SCHEMAFULL;
DEFINE FIELD agent         ON decision TYPE record<agent>;
DEFINE FIELD objective_ref ON decision TYPE option<string>;
DEFINE FIELD context_ref   ON decision TYPE option<string>;
DEFINE FIELD alternatives  ON decision TYPE array DEFAULT [];
DEFINE FIELD selected      ON decision TYPE object FLEXIBLE;
DEFINE FIELD reasoning     ON decision TYPE option<string>;
DEFINE FIELD confidence    ON decision TYPE option<number>
    ASSERT $value == NONE OR ($value >= 0 AND $value <= 1);
DEFINE FIELD policy_result ON decision TYPE string
    ASSERT $value IN ['allow', 'deny', 'escalate', 'approve'];
DEFINE FIELD created_at    ON decision TYPE datetime DEFAULT ALWAYS time::now();

-- Action (SPEC §4.5)
DEFINE TABLE action SCHEMAFULL;
DEFINE FIELD decision      ON action TYPE record<decision>;
DEFINE FIELD action_type   ON action TYPE string;
DEFINE FIELD target        ON action TYPE string;
DEFINE FIELD params        ON action TYPE option<object> FLEXIBLE;
DEFINE FIELD authorization ON action TYPE string;
DEFINE FIELD status        ON action TYPE string DEFAULT 'pending'
    ASSERT $value IN ['pending', 'running', 'done', 'failed'];
DEFINE FIELD reversible    ON action TYPE bool DEFAULT false;
DEFINE FIELD created_at    ON action TYPE datetime DEFAULT ALWAYS time::now();

-- Outcome (SPEC §4.6)
DEFINE TABLE outcome SCHEMAFULL;
DEFINE FIELD action     ON outcome TYPE record<action>;
DEFINE FIELD result     ON outcome TYPE option<object> FLEXIBLE;
DEFINE FIELD status     ON outcome TYPE string
    ASSERT $value IN ['success', 'failure'];
DEFINE FIELD error      ON outcome TYPE option<string>;
DEFINE FIELD impact     ON outcome TYPE option<string>;
DEFINE FIELD follow_up  ON outcome TYPE option<string>;
DEFINE FIELD created_at ON outcome TYPE datetime DEFAULT ALWAYS time::now();

-- Semantic memory (SPEC §4.7). Record id is [agent_id, key] so writes upsert.
DEFINE TABLE memory_fact SCHEMAFULL;
DEFINE FIELD agent      ON memory_fact TYPE record<agent>;
DEFINE FIELD key        ON memory_fact TYPE string;
DEFINE FIELD value      ON memory_fact TYPE any;
DEFINE FIELD source     ON memory_fact TYPE option<string>;
DEFINE FIELD created_at ON memory_fact TYPE datetime DEFAULT ALWAYS time::now();

-- Audit-friendly lookups
DEFINE INDEX decision_agent  ON decision FIELDS agent;
DEFINE INDEX action_decision ON action   FIELDS decision;
DEFINE INDEX outcome_action  ON outcome  FIELDS action;

-- Graph edges (SPEC §4.3 traceability, §6 audit chains). These make the agent
-- loop traversable as a graph: agent ->made-> decision ->triggered-> action
-- ->produced-> outcome. Query with arrow syntax, e.g.
--   SELECT ->made->decision->triggered->action->produced->outcome AS chain
--   FROM agent:⟨id⟩;
DEFINE TABLE made      TYPE RELATION FROM agent    TO decision;
DEFINE TABLE triggered TYPE RELATION FROM decision TO action;
DEFINE TABLE produced  TYPE RELATION FROM action   TO outcome;
DEFINE TABLE recalls   TYPE RELATION FROM agent    TO memory_fact;

-- ---------------------------------------------------------------------------
-- Memory layers (SPEC §4.7). The four memory types, built layer by layer:
--   working    -> working_memory  (transient current-task state)
--   episodic   -> decision/action/outcome (above)
--   semantic   -> memory_fact (above, key/value) + knowledge (vector/RAG)
--   procedural -> procedure      (playbooks / skills / routines)
-- ---------------------------------------------------------------------------

-- Working memory: transient state for the active task, keyed [agent, task].
DEFINE TABLE working_memory SCHEMAFULL;
DEFINE FIELD agent      ON working_memory TYPE record<agent>;
DEFINE FIELD task       ON working_memory TYPE string;
DEFINE FIELD state      ON working_memory TYPE option<object> FLEXIBLE;
DEFINE FIELD updated_at ON working_memory TYPE datetime DEFAULT ALWAYS time::now();

-- Procedural memory: named, ordered playbooks the agent can follow.
DEFINE TABLE procedure SCHEMAFULL;
DEFINE FIELD agent      ON procedure TYPE record<agent>;
DEFINE FIELD name       ON procedure TYPE string;
DEFINE FIELD steps      ON procedure TYPE array DEFAULT [];
DEFINE FIELD created_at ON procedure TYPE datetime DEFAULT ALWAYS time::now();

-- Knowledge: retrievable semantic memory for RAG. `embedding` holds the vector
-- produced by an embedding model; the HNSW index enables k-NN similarity search.
-- DIMENSION must match the embedding model (1536 = OpenAI text-embedding-3-small).
DEFINE TABLE knowledge SCHEMAFULL;
DEFINE FIELD agent      ON knowledge TYPE record<agent>;
DEFINE FIELD content    ON knowledge TYPE string;
DEFINE FIELD embedding  ON knowledge TYPE array<float>;
DEFINE FIELD metadata   ON knowledge TYPE option<object> FLEXIBLE;
DEFINE FIELD source     ON knowledge TYPE option<string>;
DEFINE FIELD created_at ON knowledge TYPE datetime DEFAULT ALWAYS time::now();
DEFINE INDEX knowledge_vec ON knowledge
    FIELDS embedding HNSW DIMENSION 1536 DIST COSINE TYPE F32;

-- ---------------------------------------------------------------------------
-- Organisation-backed, trustworthy agent teams.
-- A vendor-neutral organisation backs teams; a team has member agents; trust
-- is recorded per (team, agent). Multiple agents, one runtime, governed home.
--   organization -backs->  team
--   team         -member-> agent
--   team         -trusts-> agent   (edge fields: score, context)
-- ---------------------------------------------------------------------------
DEFINE TABLE organization SCHEMAFULL;
DEFINE FIELD name           ON organization TYPE string;
DEFINE FIELD vendor_neutral ON organization TYPE bool DEFAULT true;
DEFINE FIELD governance     ON organization TYPE option<string>;
DEFINE FIELD created_at     ON organization TYPE datetime DEFAULT ALWAYS time::now();

DEFINE TABLE team SCHEMAFULL;
DEFINE FIELD name         ON team TYPE string;
DEFINE FIELD organization ON team TYPE record<organization>;
DEFINE FIELD objective    ON team TYPE option<string>;
DEFINE FIELD created_at   ON team TYPE datetime DEFAULT ALWAYS time::now();

-- Graph edges. Relations are schemaless so `trusts` can carry score/context.
DEFINE TABLE backs  TYPE RELATION FROM organization TO team;
DEFINE TABLE member TYPE RELATION FROM team         TO agent;
DEFINE TABLE trusts TYPE RELATION FROM team         TO agent;

DEFINE INDEX team_org ON team FIELDS organization;
