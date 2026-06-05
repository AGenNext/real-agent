# Real Agent Language — reference compiler

A tiny declarative DSL (`.agent`) for defining a Real Agent, with a compiler
that emits JSON conforming to [`schemas/agent.schema.json`](../../schemas/agent.schema.json).

Language spec: [`docs/agent-language.md`](../../docs/agent-language.md).

## Layout

| File | Purpose |
|------|---------|
| `agentlang.go` | tokenizer + parser → `Contract` (mirrors `agent.schema.json`) |
| `cmd/agentc/` | CLI: `.agent` → JSON on stdout |
| `examples/cluster-janitor.agent` | worked example |

## Use

```bash
go run ./cmd/agentc examples/cluster-janitor.agent
```

Compile and validate in one step:

```bash
go run ./cmd/agentc examples/cluster-janitor.agent > /tmp/agent.json
python3 -c 'import json,sys; from jsonschema import Draft202012Validator; \
  Draft202012Validator(json.load(open("../../schemas/agent.schema.json"))).validate(json.load(open("/tmp/agent.json"))); \
  print("valid")'
```

The example compiles to a document that validates against the agent contract.
