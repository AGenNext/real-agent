# Real Agent ↔ Google enterprise agent platforms

Status: Informative

How Real Agent relates to Google's enterprise agent stack — the
**Antigravity SDK**, the **Agent Development Kit (ADK)**, and the
**Gemini Enterprise Agent Platform** (formerly Vertex AI). Companion to
`eclipse-alignment.md`.

## Positioning: a spec vs. a vendor platform

Real Agent is a **specification + contracts** — what a conformant agent *is*
(identity, decision records, action/outcome, memory, governance) and how its
records serialize (`proto/`, `schemas/`). It is deliberately runtime- and
vendor-neutral (SPEC §8).

Google's stack is a **vendor implementation** — a concrete, hosted product
family for building, running, and governing agents on Google Cloud. The two sit
at **different layers**: Real Agent says what a real agent must be; Google
provides an SDK and a managed runtime to build and operate them. They are
**complementary**, not competitive — and Google's enterprise platform already
implements much of what Real Agent specifies, which makes it a strong candidate
to *conform* (see "Conformance angle" below).

Short version: **Google's platform runs agents; Real Agent says what a real
agent must be.**

## The Google stack, by layer

| Google product | What it is | Real Agent layer it touches |
|---|---|---|
| **Antigravity SDK** (Python/TS/Go) | Build-time SDK that abstracts the agentic loop (`Agent`, `Conversation`, `Step`, `ToolCall`); ships policies, triggers, hooks, sub-agents, MCP, multimodal | The **agent loop** + build-time wiring of decision/action/policy |
| **Agent Development Kit (ADK)** | Open-source framework; graph-based orchestration of sub-agent networks; MCP | Decision/orchestration layer; multi-agent composition |
| **Gemini Enterprise Agent Platform** (ex-Vertex AI) | Managed platform: **Agent Identity**, **Agent Registry**, **Agent Gateway**, **Agent Engine** runtime + **Memory Bank** | **Identity, Governance, Memory, lifecycle** — the enterprise controls |
| **Managed Agents API** | Hosted execution surface for agents | Runtime / deployment target |

## Capability mapping (Real Agent formula → Google primitive)

`Real Agent = Identity + Objective + Context + Decision + Action + Memory + Governance`

| Real Agent capability | Google primitive | Notes |
|---|---|---|
| **Identity** (who, owner, authority) | **Agent Identity** — unique cryptographic ID per agent, mapped to authorization policies | Strong match. Google's "auditable trail mapped back to authorization policies" is close to Real Agent's *authority ≠ capability* (C3). |
| **Objective** | `system_instructions` / agent config; ADK orchestration goals | Present, but objective is prompt/config, not a first-class versioned record as in the contract. |
| **Context** | `Conversation`, multimodal inputs, retrieval/grounding; `Memory Bank` for long-term context | Strong runtime context; not modeled as an explicit `Context` record. |
| **Decision** | The abstracted agentic loop; `Step` / `ToolCall`; ADK graph nodes | Steps exist, but there is no **append-only decision record** as the atomic unit of agency (Real Agent C1). |
| **Action** | `ToolCall` execution, in-process tools, MCP, sub-agents | Maps cleanly to Real Agent `Action`. |
| **Memory** | `step_history` on `Conversation`; **Memory Bank** (persistent, long-term) in Agent Engine | Strong. Memory Bank ≈ Real Agent durable memory; Real Agent additionally requires it be **append-only / outcome-updated**. |
| **Governance** | **Policies** (`deny` / `allow` / `ask_user`), nine **hooks**, **Agent Gateway**, **Agent Registry** | Strong policy + interception story; see below. |

## Enterprise controls mapping

Real Agent's enterprise list (policy, approval, audit, traceability, risk,
reversibility, observability, evaluation, lifecycle, IAM) against Google:

| Real Agent control | Google equivalent | Gap / note |
|---|---|---|
| Policy (deny-by-default) | SDK `policies`: `deny` / `allow` / `ask_user` | Matches *policy overrides intelligence*. Default posture is dev-configurable, not spec-mandated deny-by-default. |
| Approval / escalation | `ask_user` policy; user-interaction hook | Human-in-the-loop exists; no canonical `ApprovalStatus` record (approver/timestamp/decision). |
| Audit / traceability | Agent Identity "auditable trail"; hook points | Present at the platform level; Real Agent wants every state change as an **immutable event**. |
| Risk scoring | — | No first-class `RiskLevel` primitive surfaced. |
| Reversibility | — | Not a named primitive. |
| Observability | Nine **hooks** (session start/end, pre/post turn, pre/post tool call, tool-error recovery, user interaction, context compaction) | Rich interception surface — a natural place to emit Real Agent records. |
| Evaluation | ADK debug tooling; platform eval | Present operationally; not modeled as a `TrustState`-updating loop (C4). |
| Lifecycle | Agent Registry (governed, approved assets); Agent Engine long-running state | Registry ≈ governed catalog; no explicit `LifecycleState` machine with forbidden transitions. |
| Identity / access control | **Agent Identity** + Cloud OAuth + Agent Gateway | Strongest part of the Google story. |
| Suspension (kill switch) | Agent Gateway ("air traffic control") | Plausible enforcement point; not documented as a per-agent suspend primitive. |

## Where they diverge

- **Records vs. runtime.** Google models *steps and tool calls*; Real Agent
  models an **append-only Decision → Action → Outcome → Memory** graph where the
  Decision is the atomic, recorded unit of agency. Google's hooks are the obvious
  place to materialize those records, but the platform doesn't require them.
- **Vendor-neutral contract vs. product.** Real Agent's value is a portable
  proto/JSON-Schema contract that round-trips across vendors. Google's primitives
  are excellent but proprietary to GCP — agents don't carry a portable identity or
  decision record off-platform.
- **Trust as a governor.** Real Agent C4 makes **trust** modify autonomy and
  auto-suspend below a threshold. Google has evaluation and governance, but
  trust-driven autonomy is not a named primitive.

## Conformance angle

Mapping Google's platform onto `CONFORMANCE.md`:

| Level | Could Google's stack claim it? | What it would take |
|---|---|---|
| **C1** model-compatible | Close | Persist an append-only Decision/Action/Outcome graph (e.g. via hooks) instead of only `step_history`. |
| **C2** contract-compatible | Not today | Serialize records to the canonical proto / JSON Schema with canonical enums. |
| **C3** governed | Largely *capable* | Already has identity, policy, approval (`ask_user`), audit, registry, gateway-suspend — would need to bind them to the contract and enforce deny-by-default + governed lifecycle transitions. |
| **C4** trust-governed | Not today | Add continuous evaluation → contextual trust → autonomy bound by trust + auto-suspend. |

Net: **Google's Gemini Enterprise Agent Platform is one of the closest existing
implementations to a Real-Agent-conformant runtime at the C3 (governed) tier** —
strong on identity, policy, audit, and registry. The gap is representational:
emitting the canonical, append-only decision records (C1/C2) and the
trust-driven autonomy loop (C4). The Antigravity SDK's nine hook points are the
natural integration seam to close C1/C2.

## Sources

- [Antigravity SDK (GitHub)](https://github.com/google-antigravity/antigravity-sdk-python)
- [google-antigravity on PyPI](https://pypi.org/project/google-antigravity/)
- [Introducing Gemini Enterprise Agent Platform — Google Cloud Blog](https://cloud.google.com/blog/products/ai-machine-learning/introducing-gemini-enterprise-agent-platform)
- [Agent Platform overview — Google Cloud docs](https://docs.cloud.google.com/gemini-enterprise-agent-platform/overview)
- [Agent Development Kit (ADK) — Google Cloud docs](https://docs.cloud.google.com/gemini-enterprise-agent-platform/build/adk)
- [The new Gemini Enterprise: one platform for agent development — Google Cloud Blog](https://cloud.google.com/blog/products/ai-machine-learning/the-new-gemini-enterprise-one-platform-for-agent-development)
