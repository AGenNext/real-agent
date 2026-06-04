# Migrations

Versioned, idempotent schema evolution for the Real Agent SurrealDB runtime.

## How it works

- Each migration is a numbered file: `NNNN_description.surql`.
- `migrate.sh` applies pending migrations in filename order, exactly once each,
  recording applied versions in a `_migration` table.
- Re-running is safe: already-applied migrations are skipped. Write migration
  bodies idempotently too (`DEFINE … IF NOT EXISTS`, additive changes).

## Run

```sh
ENDPOINT=http://localhost:8000 USER=root PASS=root NS=real_agent DB=v1 \
  ./migrate.sh
```

## Add a migration

Create the next number and describe the change:

```
0002_add_cost_budget.surql
```

Keep migrations forward-only and additive where possible; the append-only
event log and change feeds preserve history regardless.
