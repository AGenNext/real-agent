# Real Agent → schema.org mapping

Status: Informative

This document maps the Real Agent ontology (see [ONTOLOGY.md](../ONTOLOGY.md))
and record schemas (see [`schemas/`](../schemas)) onto the [schema.org](https://schema.org)
vocabulary and its [DataType](https://schema.org/DataType) primitives.

It is **reference only**. The Real Agent specification is deliberately
vocabulary-agnostic (SPEC.md §8 Non-goals); nothing here changes the JSON
Schemas or the wire contract. The goal is interoperability: showing how Real
Agent records can be expressed as schema.org / JSON-LD structured data for
systems that consume it (search, knowledge graphs, agent registries).

## Why schema.org fits

schema.org already models **actions** as a first-class graph: an
[`Action`](https://schema.org/Action) has an `agent`, a `target`, an
`actionStatus`, start/end times, and a `result`. That lines up almost exactly
with the Real Agent causal chain `Decision → Action → Outcome`, so the mapping
is mostly a matter of naming, not reshaping.

## Primitive → schema.org type

| Real Agent primitive | schema.org type | Notes |
|---|---|---|
| Agent | [`SoftwareApplication`](https://schema.org/SoftwareApplication) / [`Agent`-as-`Thing`] | The acting entity. `agent.type` selects the concrete type. |
| Identity → owner | [`Organization`](https://schema.org/Organization) or [`Person`](https://schema.org/Person) | Accountability rests with a human/org, never a model. |
| Objective | [`Action.object`](https://schema.org/object) / no exact type | Closest: a goal/desired state; often modelled as text. |
| Context | [`Action.instrument`](https://schema.org/instrument) / [`CreativeWork`](https://schema.org/CreativeWork) refs | Context items are cited works/records. |
| **Decision** | [`ChooseAction`](https://schema.org/ChooseAction) | `actionOption` = alternatives; `result` = selected. |
| **Action** | [`Action`](https://schema.org/Action) | `agent`, `target` ([`EntryPoint`](https://schema.org/EntryPoint)), `actionStatus`, `startTime`, `endTime`. |
| **Outcome** | [`Action.result`](https://schema.org/result) + [`actionStatus`](https://schema.org/actionStatus) | Success/failure carried by the status; `error` as text. |
| Memory (fact) | [`PropertyValue`](https://schema.org/PropertyValue) | `name`/`value` pair; semantic memory. |
| Policy | [`DigitalDocument`](https://schema.org/DigitalDocument) ref | Policies referenced by URL. |
| Trust / Evaluation | [`Rating`](https://schema.org/Rating) | `ratingValue` in `[0,1]` ≈ trust score. |

## Field → schema.org DataType

The six schema.org [DataTypes](https://schema.org/DataType) are
[`Text`](https://schema.org/Text), [`Number`](https://schema.org/Number)
(with [`Integer`](https://schema.org/Integer) / [`Float`](https://schema.org/Float)),
[`Boolean`](https://schema.org/Boolean), [`Date`](https://schema.org/Date),
[`DateTime`](https://schema.org/DateTime), and [`Time`](https://schema.org/Time).
[`URL`](https://schema.org/URL) is a subtype of `Text`.

### agent.schema.json

| Field | JSON type | schema.org DataType |
|---|---|---|
| `agent.id` | string | `Text` (or `URL`) |
| `agent.name` | string | `Text` |
| `agent.version` | string | `Text` |
| `agent.type` | string | `Text` |
| `identity.owner` | string | `Text` / `Organization` ref |
| `identity.lifecycle_state` | string enum | `Text` |
| `evaluation.minimum_trust_score` | number 0–1 | `Float` |
| `memory.working` … | boolean | `Boolean` |

### decision.schema.json (→ `ChooseAction`)

| Field | JSON type | schema.org |
|---|---|---|
| `id` | string | `Text` / `URL` |
| `timestamp` | date-time | `DateTime` |
| `objective` | string | `Text` (`object`) |
| `context_refs[]` | string | `URL` |
| `alternatives[]` | object | `actionOption` |
| `selected` | string | `result` (`Text`/ref) |
| `confidence` | number 0–1 | `Float` |
| `policy_result.decision` | string enum | `Text` |

### action.schema.json (→ `Action`)

| Field | JSON type | schema.org |
|---|---|---|
| `id` | string | `Text` / `URL` |
| `action_type` | string | `Text` |
| `target` | string | `target` → [`EntryPoint`](https://schema.org/EntryPoint) |
| `status` | string enum | [`actionStatus`](https://schema.org/actionStatus) (see below) |
| `reversible` | boolean | `Boolean` |
| `started_at` | date-time | `startTime` (`DateTime`) |
| `completed_at` | date-time | `endTime` (`DateTime`) |

### outcome.schema.json (→ `Action.result` + status)

| Field | JSON type | schema.org |
|---|---|---|
| `id` | string | `Text` / `URL` |
| `status` | string enum | `actionStatus` |
| `result` | object | `result` ([`Thing`](https://schema.org/Thing)) |
| `error.message` | string | `Text` |
| `impact` | string | `Text` |
| `observed_at` | date-time | `DateTime` |

## Status → actionStatus

Real Agent `status` enums map onto schema.org
[`ActionStatusType`](https://schema.org/ActionStatusType):

| Real Agent | schema.org |
|---|---|
| `pending`, `queued` | [`PotentialActionStatus`](https://schema.org/PotentialActionStatus) |
| `running` | [`ActiveActionStatus`](https://schema.org/ActiveActionStatus) |
| `completed` / outcome `success` | [`CompletedActionStatus`](https://schema.org/CompletedActionStatus) |
| `failed` / outcome `failure` | [`FailedActionStatus`](https://schema.org/FailedActionStatus) |

## Example: an Action + Outcome as JSON-LD

```json
{
  "@context": "https://schema.org",
  "@type": "Action",
  "@id": "https://real-agent.dev/action/act-001",
  "actionStatus": "CompletedActionStatus",
  "agent": {
    "@type": "SoftwareApplication",
    "@id": "https://real-agent.dev/agent/agent-demo-001",
    "name": "Cluster Janitor"
  },
  "target": {
    "@type": "EntryPoint",
    "urlTemplate": "kube-system/pods?phase=Succeeded"
  },
  "startTime": "2026-06-05T01:00:00Z",
  "endTime": "2026-06-05T01:00:02Z",
  "result": {
    "@type": "Thing",
    "name": "2 completed pods removed from kube-system"
  }
}
```

## Caveats

- schema.org has **no native concept of governance, authority, policy, or
  trust**. Those Real Agent primitives have no faithful schema.org type and
  should stay in the native contract; only the action/outcome surface maps
  cleanly.
- schema.org `Action` is descriptive, not normative — it cannot express the
  MUST/MUST NOT requirements of decision records (SPEC §6). Use schema.org as
  an **export/interoperability view**, not as the source of truth.
- `DataType` distinctions (`Integer` vs `Float`) are finer than JSON's single
  `number`; pick the subtype based on the field's documented range.
