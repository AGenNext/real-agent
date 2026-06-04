# Migrations

Ordered, idempotent schema evolution for the SurrealDB runtime — applied with
the native `surreal import`, no extra tooling.

## How it works

- Each migration is a numbered file: `NNNN_description.surql`.
- Write bodies idempotently (`DEFINE … IF NOT EXISTS`, additive changes) so
  re-importing is always safe.
- Apply in filename order:

```sh
for m in runtime/surrealdb/migrations/[0-9]*.surql; do
  surreal import --endpoint http://localhost:8000 --user root --pass root \
    --ns real_agent --db v1 "$m"
done
```

## Add a migration

Create the next number and describe the change, e.g. `0002_add_cost_budget.surql`.
Keep migrations forward-only and additive; the append-only event log and change
feeds preserve history regardless.
