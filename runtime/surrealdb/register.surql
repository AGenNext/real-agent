-- Register a Real Agent on the SurrealDB runtime
-- ==============================================
-- Implements the RegisterAgent flow from the proto AgentService: create the
-- agent contract, grant authority over specific tools, and emit the
-- AgentRegistered audit event. Apply after schema.surql and tools.surql.
--
-- A real agent SHOULD NOT move directly from Draft to Active, so registration
-- lands the agent in `registered` and leaves approval/activation to governance.

BEGIN TRANSACTION;

-- The accountable human/organizational owner. No credential is seeded here;
-- human logins are created via the ACCESS SIGNUP flow defined in schema.surql.
CREATE owner:acme SET email = 'ops@acme.example', name = 'Acme Platform Ops';

-- The agent contract.
CREATE agent:backup_steward SET
  name = 'Backup Steward',
  version = '0.1.0',
  type = 'data-operations',
  description = 'Backs up and restores SurrealDB databases under policy.',
  owner = owner:acme,
  tenant = 'acme',
  lifecycle_state = 'registered',
  objective = {
    primary: 'Keep SurrealDB databases recoverable without unauthorized data loss.',
    success_criteria: ['daily verified export', 'restore tested monthly'],
    constraints: ['never import without approval']
  },
  capabilities = [
    { id: 'cap.export', name: 'Export data',  risk_level: 'low' },
    { id: 'cap.import', name: 'Restore data', risk_level: 'high' }
  ],
  -- Authority is narrower than capability: allowed to export freely,
  -- import only via approval, and explicitly denied destructive drops.
  authority = {
    allowed_actions: ['sql_export', 'incremental_backup'],
    denied_actions: ['drop_database'],
    tool_access_scope: ['tool:sql_export', 'tool:incremental_backup', 'tool:sql_import'],
    escalation_requirements: ['sql_import requires owner approval']
  },
  policy = { deny_by_default: true, policy_engine: 'builtin' };

-- Grant authority edges to the tools in scope.
RELATE agent:backup_steward->authorized_for->tool:sql_export        SET approval_required = false;
RELATE agent:backup_steward->authorized_for->tool:incremental_backup SET approval_required = false;
RELATE agent:backup_steward->authorized_for->tool:sql_import        SET approval_required = true;

-- Audit the registration.
CREATE event SET type = 'AgentRegistered', agent = agent:backup_steward, subject = 'backup_steward';

COMMIT TRANSACTION;

-- Verify: the agent and the tools it is authorized for.
SELECT name, lifecycle_state, ->authorized_for->tool.name AS tools FROM agent:backup_steward;
