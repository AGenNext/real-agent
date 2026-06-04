# Real Agent Contract

Version: 0.1.0  
Status: Draft

## Purpose

The Real Agent Contract defines the minimum machine-readable agreement an agent must expose before it can be trusted, invoked, governed, evaluated, or certified.

A contract is not an implementation detail. It is the boundary between an agent and the world.

## Contract principle

Every real agent MUST declare:

```text
Who am I?
What do I own?
What can I do?
What must I not do?
What do I need?
What do I produce?
How am I governed?
How am I evaluated?
```

## Canonical contract fields

```yaml
agent:
  id: string
  name: string
  version: string
  type: string
  description: string

identity:
  owner: string
  tenant: string
  issuer: string
  subject: string
  lifecycle_state: string

objective:
  primary: string
  success_criteria: []
  constraints: []

capabilities:
  - id: string
    name: string
    description: string
    input_schema: string
    output_schema: string
    risk_level: low | medium | high | critical

inputs:
  - type: string
    source: string
    required: boolean

outputs:
  - type: string
    destination: string
    approval_required: boolean

actions:
  - id: string
    name: string
    target: string
    reversible: boolean
    approval_required: boolean
    policy_required: boolean

memory:
  working: boolean
  episodic: boolean
  semantic: boolean
  procedural: boolean
  retention_policy: string

policy:
  policy_engine: string
  policy_refs: []
  deny_by_default: boolean

security:
  authn_required: boolean
  authz_required: boolean
  secret_access: []
  data_classification: []

evaluation:
  metrics: []
  test_suite: string
  minimum_trust_score: number

observability:
  traces: boolean
  logs: boolean
  metrics: boolean
  audit: boolean
```

## Contract rules

1. An agent MUST have a stable identifier.
2. An agent MUST declare its owner.
3. An agent MUST declare its authority boundary.
4. An agent MUST declare the actions it can perform.
5. An agent MUST declare which actions require approval.
6. An agent MUST declare the policy engine or policy source controlling it.
7. An agent MUST declare memory behavior and retention policy.
8. An agent SHOULD declare evaluation metrics.
9. An agent SHOULD declare risk level per capability.
10. An agent MUST fail closed when contract validation fails.

## Runtime use

A runtime SHOULD load and validate an agent contract before activation.

A control plane SHOULD use the contract to enforce:

- discovery
- registration
- approval
- invocation
- authorization
- monitoring
- evaluation
- suspension
- retirement

## Human-readable test

A valid contract should allow a human reviewer to answer:

- who owns this agent?
- what is it allowed to do?
- what is it forbidden to do?
- what data can it access?
- what actions need approval?
- how are decisions recorded?
- how is failure handled?
- how can it be suspended?

If these cannot be answered from the contract, the agent is not ready for production.
