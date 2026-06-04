-- SurrealDB tooling registered as Real Agent tools
-- ================================================
-- "Use [the SurrealDB tooling] as tools": each operation the SurrealDB CLI /
-- ecosystem provides is registered as a governed `tool` an agent may be
-- granted authority to invoke. Apply after schema.surql.

-- Graph edge tables (the ontology relationships) -----------------------------
DEFINE TABLE makes_decision SCHEMAFULL TYPE RELATION FROM agent TO decision;
DEFINE TABLE causes         SCHEMAFULL TYPE RELATION FROM decision TO action;
DEFINE TABLE produces       SCHEMAFULL TYPE RELATION FROM action TO outcome;
DEFINE TABLE feeds          SCHEMAFULL TYPE RELATION FROM outcome TO evaluation;
DEFINE TABLE updates_trust  SCHEMAFULL TYPE RELATION FROM evaluation TO agent;
DEFINE TABLE authorized_for SCHEMAFULL TYPE RELATION FROM agent TO tool;

-- The SurrealDB CLI / tooling surface, as tools ------------------------------
CREATE tool:sql_export SET
  name = 'sql_export',
  description = 'Export all data as SurrealQL (scopes, tables, fields, events, indexes, data) for backup.',
  target = 'surreal export',
  reversible = true,          -- a read-only export changes nothing
  approval_required = false,
  risk_level = 'low';

CREATE tool:sql_import SET
  name = 'sql_import',
  description = 'Import SurrealQL to restore a database from backup. Mutates the target database.',
  target = 'surreal import',
  reversible = false,
  approval_required = true,    -- restore overwrites state: gate it
  risk_level = 'high';

CREATE tool:incremental_backup SET
  name = 'incremental_backup',
  description = 'Export raw binary data with incremental backups for efficient cluster backup.',
  target = 'surreal export --binary',
  reversible = true,
  approval_required = false,
  risk_level = 'low';

CREATE tool:start_instance SET
  name = 'start_instance',
  description = 'Start a SurrealDB instance or cluster.',
  target = 'surreal start',
  reversible = true,
  approval_required = true,    -- provisioning infrastructure
  risk_level = 'medium';

CREATE tool:ide_language_support SET
  name = 'ide_language_support',
  description = 'SurrealQL syntax highlighting (VS Code TextMate grammar). Developer-time aid.',
  target = 'editor',
  reversible = true,
  approval_required = false,
  risk_level = 'low';

CREATE tool:language_server SET
  name = 'language_server',
  description = 'SurrealQL Language Server: completion and error highlighting (future).',
  target = 'lsp',
  reversible = true,
  approval_required = false,
  risk_level = 'low';
