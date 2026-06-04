# Real Agent Ontology

Version: 0.1.0  
Status: Draft for public review

## Purpose

The Real Agent Ontology defines the canonical concepts and relationships required to describe agency in a machine-readable, implementation-neutral way.

The theory explains why real agents exist.  
The protocol defines how records are exchanged.  
The ontology defines how the concepts relate.

## Core Thesis

Agency is not a single object. Agency is a graph.

A real agent is the governed carrier of agency, but agency itself emerges from relationships among identity, authority, objective, context, decision, action, outcome, memory, policy, trust, evaluation, and governance.

## Top-Level Graph

```text
Agent
  HAS_IDENTITY Identity
  HAS_AUTHORITY Authority
  PURSUES_OBJECTIVE Objective
  OBSERVES_CONTEXT Context
  MAKES_DECISION Decision
  PERFORMS_ACTION Action
  PRODUCES_OUTCOME Outcome
  UPDATES_MEMORY Memory
  OPERATES_UNDER Policy
  HAS_TRUST Trust
  RECEIVES_EVALUATION Evaluation
  IS_GOVERNED_BY Governance
```

## Canonical Primitives

The ontology defines twelve first-class primitives.

| Primitive | Question | Role |
|---|---|---|
| Identity | Who is acting? | Establishes stable actor identity. |
| Authority | What is it permitted to do? | Defines granted operating boundary. |
| Objective | Why is it acting? | Defines purpose and desired state. |
| Context | What does it know now? | Provides situational awareness. |
| Decision | What did it choose? | Captures agency. |
| Action | What did it do? | Changes or requests change in the world. |
| Outcome | What happened? | Captures consequence. |
| Memory | What was retained? | Preserves experience and knowledge. |
| Policy | What rules constrain it? | Enforces allowed and denied behavior. |
| Trust | How much confidence is justified? | Measures reliability and safety. |
| Evaluation | How was it measured? | Produces evidence for trust. |
| Governance | Who is accountable? | Defines control, review, and accountability. |

## Causal Chain

The central causal chain of real agency is:

```text
Objective
  -> Context
  -> Decision
  -> Action
  -> Outcome
  -> Evaluation
  -> Trust
  -> Governance Adjustment
```

This means trust is not static. Trust changes as outcomes and evaluations accumulate.

## Decision as the Atomic Unit

A Decision is the atomic unit of observable agency.

```text
Agent MAKES Decision
Decision SELECTS Alternative
Decision IS_CONSTRAINED_BY Policy
Decision MAY_REQUIRE Approval
Decision CAUSES Action
```

A system that does not produce decision records cannot be reliably audited as an agent.

## Action Boundary

Action is the boundary between internal reasoning and external consequence.

```text
Action
  IS_CAUSED_BY Decision
  TARGETS Resource
  REQUIRES Authorization
  HAS_STATUS ActionStatus
  MAY_BE_REVERSIBLE
  PRODUCES Outcome
```

Actions must be governed because actions modify, request modification, or influence the state of the world.

## Authority and Capability

Authority and capability are distinct.

```text
Capability = what the agent can do
Authority = what the agent is allowed to do
```

Ontology rule:

```text
Agent MAY_HAVE Capability
Agent MUST_HAVE Authority before performing governed Action
```

A capability without authority must not imply permission.

## Policy Relationships

```text
Policy GOVERNS Agent
Policy CONSTRAINS Decision
Policy AUTHORIZES_OR_DENIES Action
Policy MAY_REQUIRE Approval
Policy MAY_TRIGGER Suspension
```

Policy is not advisory in governed deployments. Policy is enforcement.

## Trust Relationships

```text
Trust IS_UPDATED_BY Evaluation
Trust IS_INFORMED_BY Outcome
Trust MODIFIES Authority
Trust MAY_TRIGGER LifecycleTransition
```

Trust is contextual. An agent may be trusted for one capability and untrusted for another.

## Lifecycle Relationships

```text
Agent HAS_LIFECYCLE_STATE LifecycleState
LifecycleState CONTROLS InvocationEligibility
LifecycleTransition REQUIRES GovernanceEvent
Suspension DISABLES Action
Retirement PRESERVES AuditRecord
```

An agent should not move from Draft to Active without registration and approval.

## Governance Relationships

```text
Governance OWNS Accountability
Governance DEFINES ReviewProcess
Governance ASSIGNS Owner
Governance ENFORCES Policy
Governance CONTROLS Lifecycle
Governance REQUIRES Audit
```

Accountability cannot be assigned to a model. A real agent must have a human or organizational owner.

## Event Model

Every meaningful state change should be expressible as an event.

Canonical events include:

```text
AgentRegistered
AgentApproved
AgentActivated
DecisionMade
PolicyEvaluated
ApprovalRequested
ApprovalGranted
ApprovalRejected
ActionRequested
ActionAuthorized
ActionExecuted
OutcomeRecorded
EvaluationRecorded
TrustUpdated
AgentSuspended
AgentRetired
```

## Derived State Principle

The ontology prefers append-only evidence over destructive mutation.

```text
Events accumulate.
Records preserve evidence.
Current state is derived.
```

This supports auditability, replay, evaluation, and compliance.

## Minimum Graph for a Real Agent

A minimum real agent graph must include:

```text
Agent HAS_IDENTITY Identity
Agent PURSUES_OBJECTIVE Objective
Agent MAKES_DECISION Decision
Decision CAUSES Action
Action PRODUCES Outcome
Outcome UPDATES_MEMORY Memory
Agent OPERATES_UNDER Governance
```

If any of these are absent, the system may still be useful, but it is not a complete real agent.

## Enterprise Graph Extension

An enterprise real agent graph should also include:

```text
Agent HAS_AUTHORITY Authority
Agent OPERATES_UNDER Policy
Decision HAS_POLICY_RESULT PolicyResult
Decision HAS_APPROVAL_RESULT ApprovalResult
Action HAS_AUTHORIZATION_RESULT AuthorizationResult
Outcome FEEDS Evaluation
Evaluation UPDATES Trust
Trust CONTROLS AutonomyLevel
Governance CONTROLS LifecycleState
```

## Conformance Implication

A system may claim Real Agent compatibility only when it can represent and preserve the minimum graph.

A system may claim governed Real Agent compatibility only when it can represent policy, approval, authority, evaluation, trust, audit, and lifecycle relationships.

## Summary

The Real Agent Ontology defines agency as a governed graph of identity, authority, objective, context, decision, action, outcome, memory, policy, trust, evaluation, and governance.

The agent is not the graph itself. The agent is the accountable actor within the graph.

Agency becomes real when decisions cause actions, actions produce outcomes, outcomes update trust, and governance remains accountable for the entire chain.
