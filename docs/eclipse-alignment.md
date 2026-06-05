# Real Agent ↔ Eclipse Foundation alignment

Status: Informative

How Real Agent relates to the Eclipse Foundation's open-source AI ecosystem,
and what hosting/alignment would take. Saved as durable context (see also
`CLAUDE.md`).

## Positioning: a spec, not a runtime

Real Agent is a **specification + contracts** (what a conformant agent *is*:
identity, decision records, action/outcome, memory, governance). It is
deliberately runtime- and vendor-neutral (SPEC §8). That makes it
**complementary** to the Eclipse AI projects, not competitive.

| Eclipse project | What it is | Relationship to Real Agent |
|---|---|---|
| **Eclipse LMOS** | Platform that *orchestrates* specialized AI agents ("LM Operating System") | A runtime that could **implement / conform to** the Real Agent contracts; Real Agent gives LMOS agents a portable identity, decision-record, and governance contract |
| **Eclipse Theia AI** | Framework to build AI-enabled IDEs/tools | Could consume the `.agent` grammar (LSP) and render/edit agent definitions |
| **Langium / Langium AI** | TypeScript language workbench (Xtext successor) | Already used here — `reference/agent-lang/langium/` defines the `.agent` grammar |
| **Eclipse Graphene / Aidge / DL4J** | Model frameworks | Out of scope; Real Agent is above the model layer |

Short version: **LMOS runs agents; Real Agent says what a real agent must be.**
The two fit together.

## OSAID / open-principles alignment

The Foundation backs the **Open Source AI Definition (OSAID)** — transparency,
openness, responsible development. Real Agent already leans this way:

- Open spec (`SPEC.md`, `ONTOLOGY.md`) and machine-readable contracts
  (`proto/`, `schemas/*.schema.json`).
- Auditability and accountability are first-class (decision records §6,
  governance §4.8) — aligned with "responsible AI".
- Reference implementations are non-normative and swappable (no lock-in).

## Hosting-readiness checklist (if proposing to Eclipse)

| Item | Status | Note |
|---|---|---|
| Open governance doc | ✅ `GOVERNANCE.md` | exists |
| Lifecycle/spec maturity | ✅ Draft spec | versioned, conformance levels defined |
| Machine-readable contracts | ✅ proto + JSON Schema | |
| **Code license** | ⚠️ `LICENSE` is **CC BY 4.0** | CC BY is a *content* license; Eclipse expects **EPL-2.0** (or Apache-2.0) for **code**. The `reference/` code would likely need relicensing/dual-licensing for an Eclipse project. |
| Project proposal | ❌ not started | Eclipse requires a project proposal + creation review |
| Committers / diversity | ❌ | needs named committers beyond a single author |
| EF membership / IP due diligence | ❌ | Eclipse IP process (CQs), DCO/ECA sign-off |
| CI / build infra | ⚠️ none in repo | Eclipse expects reproducible builds; add CI |

## Suggested next steps (not yet actioned)

1. **License split**: keep docs/spec under CC BY 4.0, relicense `reference/` code
   under EPL-2.0 or Apache-2.0 (Eclipse-compatible).
2. **Add CI** (GitHub Actions) running `go build/vet`, JSON-Schema validation,
   and the Langium parse test — Eclipse expects automated builds.
3. **Draft an Eclipse project proposal** positioning Real Agent as the
   conformance spec that LMOS-style runtimes can implement.
4. **Engage LMOS**: explore mapping Real Agent contracts onto LMOS agents as a
   concrete interop demonstration.
