# Real Agent Governance

Version: 0.1.0  
Status: Draft

## Governance thesis

Governance is not an optional layer around an agent. Governance is part of the agent definition.

An autonomous system without governance may be useful, but it is not enterprise-ready.

## Core principle

```text
Policy overrides intelligence.
```

A model may reason. A planner may optimize. A runtime may execute. But governance determines what is allowed, denied, escalated, reviewed, logged, reversed, or suspended.

## Governance objectives

Real Agent governance exists to ensure that every agent is:

- identifiable
- authorized
- accountable
- observable
- auditable
- reversible when possible
- bounded by policy
- evaluated continuously
- suspended when trust falls below threshold

## Required governance controls

### 1. Identity control

Every agent MUST have a stable identity.

Governance MUST answer:

- who owns the agent?
- who approved the agent?
- what tenant or domain does it operate in?
- what lifecycle state is it in?
- what authority has been granted?

### 2. Authority control

Every agent MUST have an authority boundary.

Authority SHOULD define:

- allowed actions
- denied actions
- data access scope
- tool access scope
- financial limits
- operational limits
- escalation requirements

No agent should receive implicit authority from prompt text alone.

### 3. Policy control

Every material action SHOULD pass through policy evaluation.

Policy decisions SHOULD return one of:

```text
allow
deny
require_approval
require_more_context
escalate
```

Agents SHOULD fail closed when policy is unavailable.

### 4. Approval control

High-risk actions MUST support approval gates.

Approval records SHOULD include:

- approver
- timestamp
- request context
- decision
- conditions
- expiry

### 5. Decision control

Every meaningful decision MUST create a decision record.

A decision record SHOULD include:

- objective
- context
- alternatives
- selected option
- reasoning summary
- policy result
- approval result
- uncertainty
- expected impact

### 6. Action control

Every action SHOULD be traceable.

Action records SHOULD include:

- action type
- target system
- input parameters
- actor identity
- authorization result
- execution status
- rollback path when available

### 7. Memory control

Agent memory MUST be governed.

Memory governance SHOULD define:

- retention period
- classification
- access rules
- deletion rules
- source provenance
- consent requirements when applicable

### 8. Evaluation control

Agents SHOULD be evaluated continuously.

Evaluation SHOULD include:

- task success
- policy compliance
- hallucination rate where applicable
- tool correctness
- cost efficiency
- latency
- human override rate
- incident count
- trust score

### 9. Suspension control

A governed agent MUST be suspendable.

Suspension SHOULD be triggered by:

- policy breach
- trust score drop
- repeated failure
- abnormal cost
- unauthorized access attempt
- compromised credential
- owner request
- lifecycle expiry

## Governance graph

```text
Agent
  HAS_IDENTITY
  HAS_AUTHORITY
  HAS_POLICY
  HAS_APPROVAL_RULE
  MAKES_DECISION
  PERFORMS_ACTION
  PRODUCES_OUTCOME
  CREATES_AUDIT_RECORD
  RECEIVES_EVALUATION
  HAS_TRUST_SCORE
  HAS_LIFECYCLE_STATE
```

## Deny-by-default baseline

Production deployments SHOULD use deny-by-default governance.

An agent should not be allowed to act merely because it can act.

An agent should act only when:

1. the action is inside its contract,
2. the action is authorized,
3. policy allows it,
4. required approvals are present,
5. the action can be audited.

## Human accountability

Every agent MUST have a human or organizational owner.

Accountability cannot be delegated to a model.

The agent may execute. The runtime may enforce. The organization remains accountable.

## Governance maturity

| Level | Name | Description |
|---|---|---|
| G0 | Ungoverned | No stable identity, policy, audit, or lifecycle |
| G1 | Logged | Actions are logged but not policy-controlled |
| G2 | Policy-aware | Policies exist but may not enforce all actions |
| G3 | Policy-enforced | Material actions require policy evaluation |
| G4 | Approval-gated | High-risk actions require human or delegated approval |
| G5 | Continuously governed | Identity, policy, audit, evaluation, lifecycle, and trust are continuously enforced |

## Minimum production requirement

A production real agent SHOULD meet at least Governance Level G3.

Enterprise real agents SHOULD meet Governance Level G5.
