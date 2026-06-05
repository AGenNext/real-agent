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
surreal sql -e http://localhost:8000 -u root -p change-me-please \
  --ns real_agent --db memory < ../../schemas/memory.surql

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

## Graph traversal

The agent loop is also modelled as SurrealDB graph edges, so it can be walked
as a chain (SPEC §4.3 traceability, §6 audit):

```
agent ->made-> decision ->triggered-> action ->produced-> outcome
```

- `Store.Relate(in, out, edge, data)` creates an edge (`made`/`triggered`/`produced`/`recalls`).
- `Store.TraceAgent(id)` walks the edges and returns every reachable decision,
  action, and outcome.

Equivalent SurrealQL:

```surql
SELECT ->made->decision->triggered->action->produced->outcome AS chain
FROM agent:agent-demo-001;
```

## Memory layers (incl. RAG)

The store covers the four memory types from SPEC §4.7:

| Layer | Backing | Store method |
|-------|---------|--------------|
| Working | `working_memory` | `SetWorkingMemory(agentID, task, state)` |
| Episodic | `decision`/`action`/`outcome` + edges | `RecordDecision` / `RecordAction` / `RecordOutcome` |
| Semantic (key/value) | `memory_fact` | `RememberFact` |
| Semantic (knowledge/RAG) | `knowledge` + HNSW vector index | `RememberKnowledge` / `SearchKnowledge` |
| Procedural | `procedure` | `RememberProcedure` |

### RAG (retrieve-augment)

Embeddings are produced by your own model (provider-agnostic — pass the vector
in). Store knowledge, then retrieve the nearest chunks for a query:

```go
// ingest
store.RememberKnowledge("agent-demo-001", agentmem.Knowledge{
    Content:   "Traefik is the ingress controller in this cluster.",
    Embedding: embed("Traefik is the ingress controller in this cluster."), // []float64
    Source:    "runbook",
})

// retrieve top-3 for a query embedding (cosine kNN over the HNSW index)
hits, _ := store.SearchKnowledge(embed("which ingress?"), 3)
for _, h := range hits {
    fmt.Printf("%.3f  %s\n", h.Distance, h.Content)
}
```

> The `knowledge` index `DIMENSION` (1536) must match your embedding model;
> edit it in `schemas/memory.surql` if you use a different one.
