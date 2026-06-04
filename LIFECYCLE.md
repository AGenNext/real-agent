# Real Agent Lifecycle

Version: 0.1.0  
Status: Draft

## Purpose

A real agent must have a governed lifecycle. Lifecycle defines whether an agent is being designed, reviewed, approved, operating, paused, suspended, retired, or archived.

Without lifecycle management, agents become unmanaged automation.

## Canonical lifecycle states

```text
Draft -> Registered -> Approved -> Active -> Paused -> Suspended -> Retired -> Archived
```

## State definitions

| State | Meaning |
|---|---|
| Draft | The agent is being designed and is not invokable in production. |
| Registered | The agent has a stable identity and contract but is not approved for active use. |
| Approved | The agent has passed review and may be activated. |
| Active | The agent may observe, decide, and act within its authority boundary. |
| Paused | The agent is temporarily prevented from acting but may retain state. |
| Suspended | The agent is blocked due to risk, incident, policy breach, or trust failure. |
| Revoked | The agent has permanently lost authority. |
| Retired | The agent is no longer used but may remain available for audit. |
| Archived | The agent is preserved only for historical, legal, or compliance reasons. |

## Required transitions

A production runtime SHOULD enforce explicit transitions.

```text
Draft -> Registered
Registered -> Approved
Approved -> Active
Active -> Paused
Paused -> Active
Active -> Suspended
Suspended -> Active
Active -> Retired
Retired -> Archived
```

## Forbidden transitions

A real agent SHOULD NOT move directly from Draft to Active.

A suspended agent SHOULD NOT become Active without review.

A revoked agent SHOULD NOT become Active.

## Activation requirements

Before activation, an agent SHOULD have:

- valid contract
- stable identity
- assigned owner
- declared authority
- declared actions
- policy configuration
- audit enabled
- evaluation baseline
- suspension mechanism

## Suspension triggers

An agent SHOULD be suspended when:

- policy breach is detected
- trust score falls below threshold
- abnormal behavior is observed
- unauthorized access is attempted
- repeated failures occur
- cost exceeds boundary
- owner revokes approval
- credentials are compromised

## Retirement requirements

Before retirement, the system SHOULD preserve:

- final contract
- final lifecycle state
- decision records
- action records
- evaluation records
- audit records
- memory retention decision

## Lifecycle graph

```text
Agent
  HAS_LIFECYCLE_STATE
  HAS_OWNER
  HAS_CONTRACT
  HAS_AUTHORITY
  HAS_EVALUATION
  HAS_AUDIT_TRAIL
```
