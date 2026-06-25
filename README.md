# Real Agent

**Real Agent** is a canonical definition and design specification for building agents that are more than chatbots, prompts, workflows, or tool wrappers.

A real agent is an autonomous decision-making entity that observes state, reasons about objectives and constraints, chooses actions, executes them, learns from outcomes, and remains accountable for consequences.

## Why this exists

Most systems called agents today are incomplete. They are often one of the following:

- an LLM with a prompt
- a workflow with tool calls
- a chatbot with memory
- an automation script with natural language
- a planning loop without accountability

Those can be useful, but they are not sufficient for enterprise-grade agency.

A real agent needs identity, authority, objective, context, decision, action, memory, governance, and traceability.

## Canonical formula

```text
Real Agent = Identity + Objective + Context + Decision + Action + Memory + Governance
```

If an entity cannot observe, decide, act, remember, and be held accountable, it is not a real agent. It may be a tool, workflow, assistant, or automation.

## Minimum viable agent

The smallest complete agent has six capabilities:

| Capability | Meaning | Without it |
|---|---|---|
| Identity | Who the agent is, who owns it, and what authority it has | Function |
| Objective | What the agent is trying to achieve | Tool |
| Context | What the agent knows about current reality | Blind executor |
| Decision | How the agent chooses among possible actions | Automation |
| Action | How the agent changes the world | Advisor |
| Memory | How the agent preserves experience and outcomes | Stateless loop |

## Enterprise-grade agent

A production-grade agent adds the controls required for safe operation:

- policy
- approval
- audit
- traceability
- risk scoring
- reversibility
- observability
- evaluation
- lifecycle management
- identity and access control

This turns an AI feature into a governed digital worker.

## Design principle

Policy overrides intelligence.

The model may reason. The agent may plan. The runtime may execute. But policy defines what is allowed, denied, escalated, approved, reversed, logged, and audited.

## Agent loop

```text
Observe -> Contextualize -> Decide -> Authorize -> Act -> Evaluate -> Remember
```

Every pass through the loop must produce a decision record.

## Repository structure

```text
.
├── README.md
├── SPEC.md
├── ONTOLOGY.md
├── CONTRACT.md
├── GOVERNANCE.md
├── LIFECYCLE.md
├── TRUST.md
├── LICENSE
├── buf.yaml
├── buf.gen.yaml
├── proto/
│   └── real_agent/v1/
│       ├── agent.proto
│       └── common.proto
├── schemas/
│   ├── agent.schema.json
│   ├── decision.schema.json
│   └── action.schema.json
└── runtime/
    ├── rust/                      # vendor-neutral, zero-dependency runtime core
    │   ├── src/{lib,model,store,runtime,memory,security,context}.rs
    │   ├── examples/{agent_loop,demo}.rs
    │   ├── tests/*.rs
    │   └── Cargo.toml
    ├── rust-surrealdb/            # SurrealDB adapter implementing the core Store trait
    │   ├── src/lib.rs
    │   ├── tests/roundtrip.rs
    │   └── Cargo.toml
    └── surrealdb/
        ├── bootstrap.surql
        ├── schema.surql
        ├── tools.surql
        ├── memory.surql
        ├── register.surql
        ├── functions.surql
        ├── migrations/
        │   ├── 0001_agent_tags.surql
        │   └── README.md
        └── README.md
```

## Status

This repository is an early canonical specification. It is intended to become a foundation for agent design, agent governance, agent evaluation, and agent runtime interoperability.

## Ecosystem

Real Agent is the **specification** within the broader [AGenNext](https://github.com/AGenNext) platform — the model, the contract, the `AgentService` interface, and conformance levels. It defines *what* a real agent is; the agents themselves, and the surfaces around them, are **built in other repositories** that conform to this spec:

- [Agent-Platform](https://github.com/AGenNext/Agent-Platform) — where the agents are built (conforming to this spec)
- [Agent-MCPs](https://github.com/AGenNext/Agent-MCPs) — registry of MCP servers and integrations (the provider marketplace)
- [Agent-Communication-Protocol](https://github.com/AGenNext/Agent-Communication-Protocol) — vendor-neutral agent orchestration protocol
- [Agent-Context-Protocol](https://github.com/AGenNext/Agent-Context-Protocol) — context exchange between agents
- [agent-console](https://github.com/AGenNext/agent-console) — operator console UI
- [Agent-Runtime](https://github.com/AGenNext/Agent-Runtime) — a runtime surface
- [Agent-Vocabulary](https://github.com/AGenNext/Agent-Vocabulary) · [Agent-Concepts](https://github.com/AGenNext/Agent-Concepts) — the definitional layer

The `runtime/` directory here is a **non-normative reference** implementation — proof the spec is buildable and verifiable, not the product. Vendors need only satisfy [`CONFORMANCE.md`](CONFORMANCE.md), not use this code.

This repo deliberately stays in its lane: the model and the core. UIs, marketplaces, and orchestration live in their own repositories — and align with this one by satisfying [`CONFORMANCE.md`](CONFORMANCE.md), which defines what it means to be **Real Agent compatible** (levels C1–C4).

## License

See [LICENSE](LICENSE).
