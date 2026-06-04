# Real Agent runtime on SurrealDB

A reference runtime that realizes the Real Agent contract (`proto/real_agent/v1`,
`schemas/*.json`) on [SurrealDB](https://surrealdb.com). The conceptual ontology
maps directly onto SurrealDB's feature set — the agent graph *is* a SurrealDB graph.

## Feature mapping

| Real Agent concept | SurrealDB feature |
|---|---|
| Canonical primitives (agent, decision, action, outcome, evaluation) | Schemafull tables (`DEFINE TABLE … SCHEMAFULL`) |
| Ontology relationships (MAKES_DECISION, CAUSES, PRODUCES, FEEDS, UPDATES_TRUST) | Graph edges (`DEFINE TABLE … TYPE RELATION`, `RELATE`) |
| Capability vs. Authority | `agent.capabilities` (can do) vs. `agent.authority` + `authorized_for` edges (allowed) |
| Nested contract objects (objective, policy, trust) | `FLEXIBLE` / typed `object` fields, record links |
| Append-only event model / derived state | Immutable `event` table (`FOR update, delete NONE`) |
| Audit on every meaningful write | `DEFINE EVENT` triggers emitting `event` records |
| Governance / accountable owner | `DEFINE ACCESS … TYPE RECORD` + table/field `PERMISSIONS` |
| Multi-tenancy | `DEFINE NAMESPACE` per tenant + `DEFINE DATABASE`; agents carry `tenant` |
| Governed logic in the database | `DEFINE FUNCTION` (`fn::can_invoke`, `fn::needs_approval`, `fn::trust_score`) |
| Derived state | Computed table `agent_activity AS SELECT … GROUP BY` over the event log |
| Tools available to an agent | `tool` table, seeded from the SurrealDB CLI tooling |
| Memory — semantic | Vector index (`HNSW`) + full-text search (`FULLTEXT … BM25`) |
| Memory — episodic | Change feed (`CHANGEFEED`) time-travel over experience |
| Memory — procedural / working | Schemafull policy/playbook + ephemeral task-state tables |
| Continuous monitoring of trust | Live queries: `LIVE SELECT * FROM event` / `FROM evaluation` |

## Files

| File | Purpose |
|---|---|
| `bootstrap.surql` | `DEFINE NAMESPACE`/`DATABASE` for multi-tenant provisioning |
| `schema.surql` | Tables, fields, constraints, access control, audit triggers |
| `tools.surql` | Graph edge tables + SurrealDB CLI tooling registered as `tool` records |
| `memory.surql` | Memory primitive: semantic (vector + full-text), episodic (change feed), procedural, working |
| `register.surql` | Example `RegisterAgent` flow (owner → contract → authority → audit) |
| `functions.surql` | Governance functions (`fn::can_invoke`, `fn::needs_approval`, `fn::trust_score`) + `agent_activity` derived table |
| `migrations/` | Versioned, idempotent schema evolution + runner (`migrate.sh`) |

## Apply

```sh
surreal start --user root --pass root memory &           # or a persisted path
for f in bootstrap schema tools memory register functions; do
  surreal import --endpoint http://localhost:8000 --user root --pass root \
    --ns real_agent --db v1 runtime/surrealdb/$f.surql
done
```

## Tools registered from the SurrealDB tooling

`tools.surql` turns each SurrealDB CLI / ecosystem capability into a governed
agent tool, with risk and approval defaults aligned to the spec (mutating
operations are gated):

- `sql_export` — export data as SurrealQL (low risk, no approval)
- `sql_import` — restore from SurrealQL backup (high risk, **approval required**)
- `incremental_backup` — raw binary / incremental backup (low risk)
- `start_instance` — start an instance or cluster (medium risk, approval required)
- `ide_language_support`, `language_server` — developer-time SurrealQL aids

## Status

Reference runtime, version 0.1.0. SurrealQL targets **SurrealDB 3.x** and is
verified end-to-end on 3.0.1 (all four files import; graph traversal, HNSW
vector KNN, BM25 full-text, change feeds, migrations, and audit triggers all
exercised). No UI is included — pair with SurrealDB's own
[Surrealist](https://surrealdb.com/surrealist) for table views, querying, and
graph visualisation, or build a dedicated console.
