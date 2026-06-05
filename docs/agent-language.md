# Real Agent Language (`.agent`)

Status: Informative · Reference implementation: [`reference/agent-lang/`](../reference/agent-lang)

A small declarative DSL for **defining** a Real Agent. A `.agent` file compiles
to a JSON document that conforms to [`schemas/agent.schema.json`](../schemas/agent.schema.json),
so the language is just an ergonomic surface over the canonical contract — it
adds no new semantics.

## Why a language

The agent contract is a nested JSON object. Hand-writing JSON is noisy and easy
to get wrong. The `.agent` syntax is line-oriented, comment-friendly, and maps
one-to-one onto the contract, then `agentc` compiles and the result is validated
against the JSON Schema.

```
.agent  ──agentc──▶  agent.json  ──validate──▶  agent.schema.json
```

## Grammar

Whitespace-insensitive; `#` starts a comment to end of line; strings are
double-quoted. `*` = repeatable, `[]` = optional.

```
agent <id> {
  name "<name>"
  version <semver>
  type <type>
  [description "<text>"]

  identity {
    owner <owner>
    tenant <tenant>
    [issuer <issuer>]
    [subject <subject>]
    lifecycle <draft|registered|approved|active|paused|suspended|revoked|retired|archived>
  }

  objective "<primary>" {
    [success "<criterion>"]*
    [constraint "<constraint>"]*
  }

  capability <id> {
    name "<name>"
    risk <low|medium|high|critical>
    [description "<text>"]
    [input "<schema-ref>"]
    [output "<schema-ref>"]
  }*

  action <id> {
    name "<name>"
    target "<target>"
    reversible <true|false>
    approval <required|none>
    policy <required|none>
  }*

  policy {
    deny-by-default <true|false>
    [ref "<policy-ref>"]*
    [engine "<engine>"]
  }

  memory {
    enable <working|episodic|semantic|procedural>*
    retention "<duration>"
  }

  evaluation {
    [metric "<metric>"]*
    [test "<suite-ref>"]
    min-trust <0..1>
  }
}
```

## Mapping to `agent.schema.json`

| Language | Contract field |
|---|---|
| `agent <id>` / `name` / `version` / `type` | `agent.id/name/version/type` |
| `identity { … }` | `identity.{owner,tenant,issuer,subject,lifecycle_state}` |
| `objective "x" { success/constraint }` | `objective.{primary,success_criteria[],constraints[]}` |
| `capability <id> { name, risk }` | `capabilities[].{id,name,risk_level,…}` |
| `action <id> { …, approval required }` | `actions[].{id,name,target,reversible,approval_required,policy_required}` |
| `policy { deny-by-default, ref }` | `policy.{deny_by_default,policy_refs[],policy_engine}` |
| `memory { enable …, retention }` | `memory.{working,episodic,semantic,procedural,retention_policy}` |
| `evaluation { metric, min-trust }` | `evaluation.{metrics[],test_suite,minimum_trust_score}` |

`approval required` → `approval_required: true`; `approval none` → `false`.

## Usage

```bash
cd reference/agent-lang
go run ./cmd/agentc examples/cluster-janitor.agent > agent.json
# validate (any JSON Schema 2020-12 validator)
```

## Relationship to `proto/`

This DSL targets the **JSON Schema** contract (`agent.schema.json`). The
protobuf contract in `proto/` is the wire format for runtime record exchange;
the agent language is an *authoring* format for the static agent definition.
Both describe the same Real Agent — one for defining it, one for transporting
its records.
