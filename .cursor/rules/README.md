# Agent rules

Project rules that steer AI coding assistants toward idiomatic SurrealDB
patterns for this repo — correct SurrealQL, sensible SDK usage, and consistent
vector/index handling — so generated code matches how SurrealDB actually behaves.

| File | Scope |
|---|---|
| `surrealql.mdc` | SurrealQL schema/query/graph/transaction/access conventions (always applied to `.surql`) |
| `vector-and-fulltext.mdc` | Vector (HNSW) + full-text (BM25) + change-feed patterns |
| `sdk-javascript.mdc` | Official `surrealdb` JS/TS SDK usage |
| `sdk-python.mdc` | Official `surrealdb` Python SDK usage |

These are committed as living project context. Cursor loads `.cursor/rules/`
automatically; for Zed, OpenCode, or other editors, copy or symlink the files
into the location that tool expects. The canonical upstream copies live in the
SurrealDB docs repo under `public/integrations/agent-rules/`; the rules here are
adapted to this runtime and verified against SurrealDB 3.0.1.
