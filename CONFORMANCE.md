# Real Agent Conformance

Version: 0.1.0
Status: Draft

This document defines what it means for a system to be **Real Agent compatible**.
It is the standard the rest of the AGenNext ecosystem (platform, blueprints,
runtimes, registries) conforms *to* — `real-agent` defines the model; conformance
is how other repositories align with it.

Conformance is assessed against the artifacts in this repository:

- the ontology (`ONTOLOGY.md`),
- the contract (`proto/real_agent/v1`, `schemas/*.json`, `vocab/real-agent.jsonld`),
- governance (`GOVERNANCE.md`), lifecycle (`LIFECYCLE.md`), trust (`TRUST.md`).

## Conformance levels

| Level | Name | Requirement |
|---|---|---|
| **C0** | Non-conformant | Cannot represent the minimum agent graph. |
| **C1** | Model-compatible | Represents and preserves the **minimum graph** (below). |
| **C2** | Contract-compatible | Exchanges records using the canonical contract (proto **or** JSON Schema), field-for-field. |
| **C3** | Governed | Adds policy, approval, authority, audit, and lifecycle (Governance ≥ G3). |
| **C4** | Trust-governed | Adds continuous evaluation and trust-driven autonomy/suspension (Governance G5). |

A system states its level as a claim; the checklists below let a reviewer verify it.

## C1 — minimum graph

A system is model-compatible when it can represent and preserve:

```text
Agent      HAS_IDENTITY     Identity
Agent      PURSUES          Objective
Agent      OBSERVES         Context
Agent      MAKES            Decision
Decision   CAUSES           Action
Action     PRODUCES         Outcome
Outcome    UPDATES          Memory
Agent      OPERATES_UNDER   Governance
```

- Every **Decision** is recorded (decisions are the atomic unit of agency).
- Every **Action** is caused by a decision and produces an outcome.
- Records are **append-only**; current state is derived, not destructively mutated.

If any of these cannot be represented, the system is C0 — it may be a tool,
workflow, or assistant, but not a real agent.

## C2 — contract compatibility

In addition to C1, the system serializes its records to the canonical contract:

- **Identity, Authority, Objective, Capability** per `AgentContract` / `agent.schema.json`.
- **Decision, Action, Outcome, Evaluation** per their proto messages / JSON Schemas.
- Enumerations use the canonical values (`LifecycleState`, `RiskLevel`,
  `PolicyDecision`, `ApprovalStatus`, `ActionStatus`, `TrustState`).
- Field names match the proto/schema exactly (the two encodings must round-trip).

Verify with: `buf build` / `buf lint` on the proto, and JSON Schema validation of
emitted records.

## C3 — governed

In addition to C2 (see `GOVERNANCE.md` for G-levels):

- **Authority ≠ capability** — a capability without granted authority does not imply permission.
- **Deny-by-default** — an action runs only if inside the contract, authorized, policy-allowed, approved where required, and auditable.
- **Approval gates** on high-risk actions, recorded with approver, timestamp, and decision.
- **Lifecycle** is governed; forbidden transitions (e.g. `Draft → Active`) are rejected.
- **Audit** — every meaningful state change emits an immutable event.
- **Suspension** — the agent can be suspended (kill switch).
- Maps to **Governance Level ≥ G3**.

## C4 — trust-governed

In addition to C3:

- Continuous **evaluation** produces evidence; **trust** is updated from outcomes.
- Trust **modifies autonomy**; agents **auto-suspend** below a safety threshold.
- Trust is **contextual** (trusted for one capability is not trusted for another).
- Maps to **Governance Level G5**.

## Reference

The `runtime/rust` core in this repository is a C4 reference: it enforces the
governed lifecycle, authority, deny-by-default policy, approval gates, security,
trust-driven suspension, and an append-only audit log — verifiably, with a
vendor-neutral `Store` port. A conforming system need not use this code, only
satisfy the checklist for its claimed level.

## Self-assessment checklist

```text
[ ] C1  minimum graph representable and append-only
[ ] C2  records validate against proto AND json schema; enums canonical
[ ] C3  authority, deny-by-default, approval, governed lifecycle, audit, suspension
[ ] C4  evaluation + trust update; autonomy bounded by trust; auto-suspend
```
