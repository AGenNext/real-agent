# Real Agent — SurrealDB memory store (Go reference)

A small, reference Go implementation of the Real Agent memory and
decision-record requirements, backed by SurrealDB.

> Reference only. The spec is database-agnostic (SPEC.md §8). This is one
> concrete way to satisfy §4.4–§4.7 and §6, not a required implementation.

## Layout

| File | Purpose |
|------|---------|
| `store.go` | `agentmem` package — typed store: agents, decisions, actions, outcomes, memory facts |
| `cmd/demo/main.go` | Runs one full agent loop (Decide → Act → Outcome → Memory) |
| `../../schemas/memory.surql` | SurrealQL schema the store writes against |
| `../../deploy/surrealdb.yaml` | SurrealDB deployment for k3s |

## Run

```bash
# 1. Deploy SurrealDB via the official Helm chart (edit the password first).
#    k3s's Helm controller reconciles this HelmChart resource for you.
kubectl apply -f ../../deploy/surrealdb.yaml
kubectl -n database wait --for=condition=ready pod \
  -l app.kubernetes.io/name=surrealdb --timeout=180s

# 2. Load the schema
kubectl -n database port-forward svc/surrealdb 8000:8000 &
surreal import --conn http://localhost:8000 --user root --pass change-me-please \
  --ns real_agent --db memory ../../schemas/memory.surql

# 3. Run the demo
export SURREAL_ENDPOINT="ws://localhost:8000/rpc"
export SURREAL_USER="root"
export SURREAL_PASS="change-me-please"
go run ./cmd/demo
```

Expected output:

```
decision decision:⟨id⟩ -> action action:⟨id⟩ -> outcome outcome:⟨id⟩ recorded; memory updated
```

## Mapping to the spec

| Spec | Table | Store method |
|------|-------|--------------|
| §4.1 Identity | `agent` | `UpsertAgent` |
| §4.4 / §6 Decision | `decision` | `RecordDecision` |
| §4.5 Action | `action` | `RecordAction` |
| §4.6 Outcome | `outcome` | `RecordOutcome` |
| §4.7 Memory | `memory_fact` | `RememberFact` |
