# Real Agent glossary and taxonomy

Status: Informative

A definition for every term used across [README.md](../README.md),
[SPEC.md](../SPEC.md), [ONTOLOGY.md](../ONTOLOGY.md), and the record
[`schemas/`](../schemas). Organised as a taxonomy: each section groups related
terms, and every enumerated value is defined.

---

## 1. Foundational terms

| Term | Definition |
|---|---|
| **Agent** | The accountable actor within the agency graph. The governed carrier of agency — not the graph itself. |
| **Real Agent** | An autonomous decision-making entity that has identity, pursues an objective, observes context, decides, acts, records outcomes, updates memory, and operates under governance (SPEC §2). |
| **Agency** | The emergent property of relationships among identity, authority, objective, context, decision, action, outcome, memory, policy, trust, evaluation, and governance. Agency is a graph, not an object. |
| **Capability** | What an agent *can* do (a technical ability). |
| **Authority** | What an agent is *allowed* to do (a granted permission). A capability without authority does not imply permission. |
| **Decision record** | The atomic, accountable unit of agency. Every meaningful decision MUST produce one (SPEC §6). Without it, a system cannot be audited as an agent. |
| **Causal chain** | `Objective → Context → Decision → Action → Outcome → Evaluation → Trust → Governance adjustment`. |

## 2. The twelve canonical primitives

The ontology's twelve first-class concepts (ONTOLOGY "Canonical Primitives").

| Primitive | Question it answers | Definition |
|---|---|---|
| **Identity** | Who is acting? | Stable, unique designation of the actor: id, name, version, owner, tenant, type, authority scope. MUST NOT be inferred from a prompt or model name. |
| **Authority** | What may it do? | The granted operating boundary; the permission envelope for governed actions. |
| **Objective** | Why is it acting? | The desired state and success criteria the agent pursues. An agent without an objective is a tool. |
| **Context** | What does it know now? | Situational input: messages, documents, events, records, state, prior decisions. Should be traceable to source. |
| **Decision** | What did it choose? | The selection among alternatives, with reasoning, confidence, and policy result. Captures agency. |
| **Action** | What did it do? | The boundary between internal reasoning and external consequence; changes or requests change in the world. |
| **Outcome** | What happened? | The observed result of an action: success/failure, error, impact, follow-up. |
| **Memory** | What was retained? | Preserved experience and knowledge across the agent's life (see §5). |
| **Policy** | What rules constrain it? | Enforced allow/deny/escalate rules. In governed deployments, policy is enforcement, not advice. |
| **Trust** | How much confidence is justified? | A contextual measure of reliability/safety, updated by evaluation and outcomes. |
| **Evaluation** | How was it measured? | The process producing evidence (metrics, tests) that informs trust. |
| **Governance** | Who is accountable? | Control, review, and accountability — assigns a human/organizational owner, enforces policy, controls lifecycle. |

## 3. Lifecycle states

The states an agent moves through (`agent.schema.json` `identity.lifecycle_state`; SPEC §5).

| State | Definition |
|---|---|
| **draft** | Defined but not yet registered. |
| **registered** | Recorded in the governing system. |
| **approved** | Cleared by governance to operate. |
| **active** | Eligible to be invoked and to act. |
| **paused** | Temporarily halted, expected to resume. |
| **suspended** | Halted by governance/policy; actions disabled. |
| **revoked** | Authority withdrawn (e.g., after a trust breach). |
| **retired** | Decommissioned; audit records preserved. |
| **archived** | Cold-stored for historical/audit reference. |

## 4. Status and result enumerations

### Action status (`action.schema.json` `status`)
| Value | Definition |
|---|---|
| **pending** | Created, not yet queued. |
| **queued** | Awaiting execution. |
| **running** | Executing now. |
| **completed** | Finished successfully. |
| **failed** | Finished unsuccessfully. |
| **cancelled** | Stopped before completion by request. |
| **rolled_back** | Reversed after execution. |

### Outcome status (`outcome.schema.json` `status`)
| Value | Definition |
|---|---|
| **success** | The action achieved its intended result. |
| **failure** | The action did not achieve its result. |
| **partial** | Some but not all of the intended result was achieved. |

### Policy result (`decision.schema.json` `policy_result.decision`)
| Value | Definition |
|---|---|
| **allow** | The action may proceed. |
| **deny** | The action is forbidden. |
| **require_approval** | A human/governance approval is needed first. |
| **require_more_context** | Insufficient information to decide. |
| **escalate** | Raise to a higher authority for handling. |

### Authorization status (`action.schema.json` `authorization.status`)
| Value | Definition |
|---|---|
| **authorized** | Permission granted to perform the action. |
| **denied** | Permission refused. |
| **approval_required** | Permission pending an approval gate. |

### Risk level (`capabilities[].risk_level`)
| Value | Definition |
|---|---|
| **low** | Minimal potential for harm. |
| **medium** | Moderate, recoverable impact. |
| **high** | Serious impact; tighter controls warranted. |
| **critical** | Severe/irreversible impact; strict governance required. |

## 5. Memory taxonomy

The memory types an agent may maintain (`agent.schema.json` `memory`; SPEC §4.7).

| Type | Definition |
|---|---|
| **Working memory** | Current task state; transient scratch space for the active loop. |
| **Episodic memory** | Prior events, decisions, actions, and outcomes — the audit trail. In this repo, the `decision`/`action`/`outcome` records and their graph edges. |
| **Semantic memory** | Facts and knowledge the agent retains; the `memory_fact` table (key/value). |
| **Procedural memory** | Policies, playbooks, routines, and skills the agent follows. |
| **Retention policy** | The rule governing how long memory is kept and when it is purged (privacy/compliance). |

### Knowledge / retrieval terms (for the planned RAG layer)
| Term | Definition |
|---|---|
| **Knowledge** | Durable, retrievable semantic content an agent can draw on to reason. |
| **Embedding** | A fixed-length vector of numbers representing the meaning of a piece of content, produced by an embedding model. |
| **Vector** | An ordered list of floats; the in-database form of an embedding. |
| **RAG** | Retrieval-Augmented Generation: retrieve the most relevant stored knowledge (by vector similarity) and supply it as context before the model reasons. |
| **Semantic search** | Finding records by meaning (vector closeness) rather than exact keywords. |
| **kNN** | k-Nearest-Neighbours: the k vectors closest to a query vector under a distance metric. |
| **Cosine distance** | A similarity metric based on the angle between two vectors (smaller = more similar). |

## 6. Graph terms

Terms from the SurrealDB graph model ([`schemas/memory.surql`](../schemas/memory.surql); [agent-graph.md](./agent-graph.md)).

| Term | Definition |
|---|---|
| **Edge / relation** | A directed link between two records (`in → relation → out`). |
| **made** | Edge: `agent → decision`. |
| **triggered** | Edge: `decision → action`. |
| **produced** | Edge: `action → outcome`. |
| **recalls** | Edge: `agent → memory_fact`. |
| **Traversal** | Walking edges to gather a connected sub-graph (e.g., `TraceAgent`). |

## 7. Conformance levels

The capability ladder (SPEC §9).

| Level | Name | Definition |
|---|---|---|
| **0** | Assistant | Responds; no autonomous decision/action boundary. |
| **1** | Tool-using assistant | Calls tools; lacks stable identity, governance, decision records. |
| **2** | Agentic workflow | Structured steps and partial state; limited autonomy. |
| **3** | Real agent | Has identity, objective, context, decision, action, memory, and decision records. |
| **4** | Governed enterprise agent | Adds policy, approval, audit, lifecycle, risk, observability, evaluation. |
| **5** | Interoperable agent | Adds portable contracts, verifiable identity, cross-runtime interoperability, standardized evaluation. |

## 8. Event vocabulary

Canonical events; every meaningful state change should be expressible as one (ONTOLOGY "Event Model"). Append-only evidence — current state is *derived* from accumulated events.

`AgentRegistered`, `AgentApproved`, `AgentActivated`, `DecisionMade`,
`PolicyEvaluated`, `ApprovalRequested`, `ApprovalGranted`, `ApprovalRejected`,
`ActionRequested`, `ActionAuthorized`, `ActionExecuted`, `OutcomeRecorded`,
`EvaluationRecorded`, `TrustUpdated`, `AgentSuspended`, `AgentRetired`.

## 9. Document map

| Term | Definition |
|---|---|
| **Theory** | Explains *why* real agents exist (README, SPEC). |
| **Protocol** | Defines *how* records are exchanged (`proto/`, schemas). |
| **Ontology** | Defines *how the concepts relate* (ONTOLOGY.md). |
| **Contract** | The machine-readable agent definition (`agent.schema.json`). |
| **Reference implementation** | A non-normative example (`examples/go`, `schemas/memory.surql`). |
