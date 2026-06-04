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
├── CONTRACT.md
├── GOVERNANCE.md
├── LIFECYCLE.md
├── SECURITY.md
├── schemas/
│   ├── agent.schema.json
│   ├── decision.schema.json
│   └── action.schema.json
├── examples/
│   ├── minimal-agent.json
│   └── enterprise-agent.json
└── docs/
    ├── architecture.md
    ├── glossary.md
    └── evaluation.md
```

## Status

This repository is an early canonical specification. It is intended to become a foundation for agent design, agent governance, agent evaluation, and agent runtime interoperability.

## License

See [LICENSE](LICENSE).
