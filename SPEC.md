# Real Agent Specification

Version: 0.1.0  
Status: Draft

## 1. Purpose

This specification defines the minimum and production-grade requirements for a real agent.

It is intentionally independent of any specific LLM, workflow engine, orchestration framework, database, cloud provider, or runtime.

## 2. Definition

A real agent is an autonomous decision-making entity that:

1. has identity,
2. pursues an objective,
3. observes context,
4. makes decisions,
5. performs actions,
6. records outcomes,
7. updates memory,
8. operates under governance.

## 3. Normative language

The key words **MUST**, **MUST NOT**, **SHOULD**, **SHOULD NOT**, and **MAY** are to be interpreted as requirement levels.

## 4. Core requirements

A conforming real agent MUST include the following components.

### 4.1 Identity

An agent MUST have a stable identity.

Identity MUST include:

- unique identifier
- name
- version
- owner
- tenant or operating boundary
- type or role
- authority scope

Identity MUST NOT be inferred only from a prompt, runtime session, or model name.

### 4.2 Objective

An agent MUST have one or more declared objectives.

An objective SHOULD define:

- desired state
- success criteria
- priority
- constraints
- time boundary

An agent without an objective is a tool, not an agent.

### 4.3 Context

An agent MUST operate over context.

Context MAY include:

- messages
- documents
- events
- records
- sensor input
- system state
- user input
- prior decisions

Context SHOULD be traceable to source.

### 4.4 Decision

An agent MUST be able to choose among alternatives.

A decision MUST include:

- decision identifier
- timestamp
- objective reference
- context reference
- alternatives considered
- selected option
- reasoning summary
- confidence or uncertainty
- policy result

A system that only executes predetermined steps without decision capability is automation, not a real agent.

### 4.5 Action

An agent MUST be able to perform or request actions.

An action MUST include:

- action identifier
- action type
- target system or resource
- input parameters
- authorization result
- execution status
- reversibility flag
- audit trail

An agent that only recommends but cannot act is an advisor, not a real agent.

### 4.6 Outcome

An agent MUST capture the result of an action.

An outcome SHOULD include:

- observed result
- success or failure status
- error details when applicable
- impact
- follow-up requirement

### 4.7 Memory

An agent MUST preserve relevant experience.

Memory MAY include:

- episodic memory: prior events, decisions, actions, outcomes
- semantic memory: facts and knowledge
- procedural memory: policies, playbooks, routines, skills
- working memory: current task state

Memory MUST be governed by privacy, retention, and access policies.

### 4.8 Governance

An agent MUST operate under governance.

Governance SHOULD include:

- identity and access control
- policy enforcement
- approval gates
- audit logs
- risk scoring
- monitoring
- evaluation
- lifecycle state
- kill switch or suspension mechanism

Policy MUST override model reasoning and autonomous planning.

## 5. Agent lifecycle

A real agent SHOULD support the following lifecycle states:

```text
Draft -> Registered -> Approved -> Active -> Suspended -> Retired -> Archived
```

Enterprise deployments MAY extend this with probation, revoked, compromised, or alumni states.

## 6. Decision record requirement

Every meaningful decision MUST produce a decision record.

A decision record is the accountable unit of agency.

Without decision records, there is no reliable auditability, evaluation, or trust.

## 7. Interoperability

A real agent SHOULD expose machine-readable contracts for:

- identity
- capabilities
- objectives
- permissions
- inputs
- outputs
- actions
- policies
- evaluation metrics

## 8. Non-goals

This specification does not define:

- a specific LLM interface
- a specific memory database
- a specific orchestration engine
- a specific user interface
- a specific cloud runtime

## 9. Conformance levels

### Level 0: Assistant

Can respond but has no autonomous decision/action boundary.

### Level 1: Tool-using assistant

Can call tools but lacks stable identity, governance, and decision records.

### Level 2: Agentic workflow

Has structured steps, tools, and partial state but limited autonomy.

### Level 3: Real agent

Has identity, objective, context, decision, action, memory, and decision records.

### Level 4: Governed enterprise agent

Adds policy, approval, audit, lifecycle, risk, observability, and evaluation.

### Level 5: Interoperable agent

Adds portable contracts, verifiable identity, cross-runtime interoperability, and standardized evaluation.
