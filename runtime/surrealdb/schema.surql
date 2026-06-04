-- Real Agent runtime on SurrealDB
-- ================================
-- This schema realizes the Real Agent contract (proto/real_agent/v1, schemas/*)
-- as a SurrealDB database. It maps the conceptual ontology directly onto
-- SurrealDB features:
--
--   ontology concept            SurrealDB feature
--   ----------------            -----------------
--   primitives (agent, ...)     schemafull tables
--   ontology relationships      graph edges (RELATE)
--   nested contract objects     FLEXIBLE objects / record links
--   append-only event model     DEFINE EVENT (triggers) + immutable event table
--   governance / authority      DEFINE ACCESS + table/field PERMISSIONS
--   derived current state       computed via graph traversal of events
--
-- Target: SurrealDB 3.x. Apply with:
--   surreal import --endpoint <url> --user <u> --pass <p> --ns real_agent --db v1 schema.surql

-- ---------------------------------------------------------------------------
-- Enumerated domains (kept in sync with proto/real_agent/v1/common.proto)
-- ---------------------------------------------------------------------------
-- SurrealQL has no native enum type; we enforce membership with ASSERT.

-- ---------------------------------------------------------------------------
-- Identity / authority access control (governance)
-- ---------------------------------------------------------------------------
-- A record-based access method gives every owner/operator a verifiable
-- identity. Accountability cannot be assigned to a model, so every governed
-- write is attributable to an authenticated owner.
DEFINE ACCESS owner ON DATABASE TYPE RECORD
  SIGNUP ( CREATE owner SET email = $email, name = $name, pass = crypto::argon2::generate($pass) )
  SIGNIN ( SELECT * FROM owner WHERE email = $email AND crypto::argon2::compare(pass, $pass) )
  DURATION FOR TOKEN 1h, FOR SESSION 8h;

DEFINE TABLE owner SCHEMAFULL;
DEFINE FIELD email ON owner TYPE string ASSERT string::is_email($value);
DEFINE FIELD name  ON owner TYPE string;
DEFINE FIELD pass  ON owner TYPE string PERMISSIONS FOR select NONE;
DEFINE INDEX owner_email ON owner FIELDS email UNIQUE;

-- ---------------------------------------------------------------------------
-- agent: the canonical AgentContract (identity + objective + posture)
-- ---------------------------------------------------------------------------
DEFINE TABLE agent SCHEMAFULL
  PERMISSIONS
    FOR select FULL
    FOR create, update, delete WHERE owner = $auth.id;

DEFINE FIELD name        ON agent TYPE string;
DEFINE FIELD version     ON agent TYPE string;
DEFINE FIELD type        ON agent TYPE string;
DEFINE FIELD description ON agent TYPE option<string>;
DEFINE FIELD owner       ON agent TYPE record<owner>;
DEFINE FIELD tenant      ON agent TYPE string;
DEFINE FIELD issuer      ON agent TYPE option<string>;
DEFINE FIELD subject     ON agent TYPE option<string>;

DEFINE FIELD lifecycle_state ON agent TYPE string
  DEFAULT 'draft'
  ASSERT $value IN ['draft','registered','approved','active','paused','suspended','revoked','retired','archived'];

DEFINE FIELD objective                  ON agent TYPE object;
DEFINE FIELD objective.primary          ON agent TYPE string;
DEFINE FIELD objective.success_criteria ON agent TYPE option<array<string>>;
DEFINE FIELD objective.constraints      ON agent TYPE option<array<string>>;

-- Capability = what the agent CAN do.
DEFINE FIELD capabilities             ON agent TYPE array<object>;
DEFINE FIELD capabilities[*].id        ON agent TYPE string;
DEFINE FIELD capabilities[*].name      ON agent TYPE string;
DEFINE FIELD capabilities[*].risk_level ON agent TYPE string
  ASSERT $value IN ['low','medium','high','critical'];

-- Authority = what the agent is ALLOWED to do (distinct from capability).
DEFINE FIELD authority                         ON agent TYPE option<object>;
DEFINE FIELD authority.allowed_actions          ON agent TYPE option<array<string>>;
DEFINE FIELD authority.denied_actions           ON agent TYPE option<array<string>>;
DEFINE FIELD authority.data_access_scope        ON agent TYPE option<array<string>>;
DEFINE FIELD authority.tool_access_scope        ON agent TYPE option<array<string>>;
DEFINE FIELD authority.financial_limit          ON agent TYPE option<string>;
DEFINE FIELD authority.operational_limits       ON agent TYPE option<array<string>>;
DEFINE FIELD authority.escalation_requirements  ON agent TYPE option<array<string>>;

DEFINE FIELD policy                 ON agent TYPE object;
DEFINE FIELD policy.deny_by_default ON agent TYPE bool DEFAULT true;
DEFINE FIELD policy.policy_engine   ON agent TYPE option<string>;
DEFINE FIELD policy.policy_refs     ON agent TYPE option<array<string>>;

DEFINE FIELD memory ON agent TYPE option<object>;
DEFINE FIELD trust  ON agent TYPE option<object> FLEXIBLE;

DEFINE FIELD created_at ON agent TYPE datetime DEFAULT time::now() READONLY;
DEFINE FIELD updated_at ON agent TYPE datetime DEFAULT time::now() VALUE time::now();

DEFINE INDEX agent_tenant ON agent FIELDS tenant;

-- ---------------------------------------------------------------------------
-- tool: a capability surfaced to agents (e.g. the SurrealDB CLI operations)
-- ---------------------------------------------------------------------------
DEFINE TABLE tool SCHEMAFULL
  PERMISSIONS FOR select FULL FOR create, update, delete WHERE $auth.id != NONE;
DEFINE FIELD name        ON tool TYPE string;
DEFINE FIELD description ON tool TYPE string;
DEFINE FIELD provider    ON tool TYPE string DEFAULT 'surrealdb';
DEFINE FIELD target      ON tool TYPE string;
DEFINE FIELD reversible  ON tool TYPE bool DEFAULT false;
DEFINE FIELD approval_required ON tool TYPE bool DEFAULT false;
DEFINE FIELD risk_level   ON tool TYPE string
  ASSERT $value IN ['low','medium','high','critical'];
DEFINE INDEX tool_name ON tool FIELDS name UNIQUE;

-- ---------------------------------------------------------------------------
-- decision: the atomic proof of agency
-- ---------------------------------------------------------------------------
DEFINE TABLE decision SCHEMAFULL;
DEFINE FIELD agent             ON decision TYPE record<agent>;
DEFINE FIELD timestamp         ON decision TYPE datetime DEFAULT time::now() READONLY;
DEFINE FIELD objective         ON decision TYPE string;
DEFINE FIELD context_refs      ON decision TYPE option<array<string>>;
DEFINE FIELD alternatives      ON decision TYPE array<object>;
DEFINE FIELD selected          ON decision TYPE string;
DEFINE FIELD reasoning_summary ON decision TYPE string;
DEFINE FIELD confidence        ON decision TYPE number ASSERT $value >= 0 AND $value <= 1;
DEFINE FIELD uncertainty       ON decision TYPE option<string>;
DEFINE FIELD policy_result     ON decision TYPE object;
DEFINE FIELD policy_result.decision ON decision TYPE string
  ASSERT $value IN ['allow','deny','require_approval','require_more_context','escalate'];
DEFINE FIELD approval_result   ON decision TYPE option<object> FLEXIBLE;
DEFINE FIELD expected_outcome  ON decision TYPE option<string>;

-- ---------------------------------------------------------------------------
-- action: the boundary between internal reasoning and external consequence
-- ---------------------------------------------------------------------------
DEFINE TABLE action SCHEMAFULL;
DEFINE FIELD decision      ON action TYPE record<decision>;
DEFINE FIELD agent         ON action TYPE record<agent>;
DEFINE FIELD tool          ON action TYPE option<record<tool>>;
DEFINE FIELD action_type   ON action TYPE string;
DEFINE FIELD target        ON action TYPE string;
DEFINE FIELD parameters    ON action TYPE option<object> FLEXIBLE;
DEFINE FIELD authorization ON action TYPE object;
DEFINE FIELD authorization.status ON action TYPE string
  ASSERT $value IN ['authorized','denied','approval_required'];
DEFINE FIELD status ON action TYPE string
  ASSERT $value IN ['pending','queued','running','completed','failed','cancelled','rolled_back'];
DEFINE FIELD reversible         ON action TYPE bool DEFAULT false;
DEFINE FIELD rollback_reference ON action TYPE option<string>;
DEFINE FIELD started_at         ON action TYPE option<datetime>;
DEFINE FIELD completed_at       ON action TYPE option<datetime>;

-- ---------------------------------------------------------------------------
-- outcome + evaluation
-- ---------------------------------------------------------------------------
DEFINE TABLE outcome SCHEMAFULL;
DEFINE FIELD action          ON outcome TYPE record<action>;
DEFINE FIELD agent           ON outcome TYPE record<agent>;
DEFINE FIELD success         ON outcome TYPE bool;
DEFINE FIELD observed_result ON outcome TYPE option<string>;
DEFINE FIELD impact          ON outcome TYPE option<string>;
DEFINE FIELD error           ON outcome TYPE option<string>;
DEFINE FIELD observed_at     ON outcome TYPE datetime DEFAULT time::now() READONLY;

DEFINE TABLE evaluation SCHEMAFULL;
DEFINE FIELD agent             ON evaluation TYPE record<agent>;
DEFINE FIELD period            ON evaluation TYPE option<string>;
DEFINE FIELD trust_score       ON evaluation TYPE number ASSERT $value >= 0 AND $value <= 1;
DEFINE FIELD policy_compliance ON evaluation TYPE option<number>;
DEFINE FIELD evidence_refs     ON evaluation TYPE option<array<string>>;
DEFINE FIELD evaluated_at      ON evaluation TYPE datetime DEFAULT time::now() READONLY;

-- ---------------------------------------------------------------------------
-- event: append-only audit log (derived-state principle)
-- ---------------------------------------------------------------------------
DEFINE TABLE event SCHEMAFULL
  PERMISSIONS FOR select FULL FOR update, delete NONE;  -- immutable
DEFINE FIELD type    ON event TYPE string
  ASSERT $value IN [
    'AgentRegistered','AgentApproved','AgentActivated','DecisionMade',
    'PolicyEvaluated','ApprovalRequested','ApprovalGranted','ApprovalRejected',
    'ActionRequested','ActionAuthorized','ActionExecuted','OutcomeRecorded',
    'EvaluationRecorded','TrustUpdated','AgentSuspended','AgentRetired'
  ];
DEFINE FIELD agent   ON event TYPE record<agent>;
DEFINE FIELD subject ON event TYPE option<string>;
DEFINE FIELD payload ON event TYPE option<object> FLEXIBLE;
DEFINE FIELD at      ON event TYPE datetime DEFAULT time::now() READONLY;
DEFINE INDEX event_agent_at ON event FIELDS agent, at;

-- ---------------------------------------------------------------------------
-- Triggers: every meaningful write emits an immutable audit event
-- ---------------------------------------------------------------------------
DEFINE EVENT decision_made ON TABLE decision WHEN $event = "CREATE" THEN {
  CREATE event SET type = 'DecisionMade', agent = $after.agent, subject = <string> $after.id;
};
DEFINE EVENT action_executed ON TABLE action WHEN $event = "UPDATE" AND $after.status = 'completed' THEN {
  CREATE event SET type = 'ActionExecuted', agent = $after.agent, subject = <string> $after.id;
};
DEFINE EVENT outcome_recorded ON TABLE outcome WHEN $event = "CREATE" THEN {
  CREATE event SET type = 'OutcomeRecorded', agent = $after.agent, subject = <string> $after.id;
};
DEFINE EVENT evaluation_recorded ON TABLE evaluation WHEN $event = "CREATE" THEN {
  CREATE event SET type = 'EvaluationRecorded', agent = $after.agent, subject = <string> $after.id,
    payload = { trust_score: $after.trust_score };
};
