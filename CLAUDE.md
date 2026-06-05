# Project memory — Real Agent

Notes for future sessions.

## Remembered context

- **Eclipse Foundation (open source AI)** is a candidate home / alignment target
  for this project. Relevant because the Eclipse Foundation offers
  vendor-neutral governance, legal/IP support, and hosts AI projects — and this
  repo already has `GOVERNANCE.md`, `LICENSE` (CC BY 4.0), and a spec posture
  suited to standardization.
  - Notably relevant Eclipse projects: **Eclipse LMOS** (multi-agent
    orchestration / "Language Model Operating System") and **Theia AI**.
  - The Foundation supports the **Open Source AI Definition (OSAID)** —
    transparency/openness principles that fit the Real Agent spec.
  - Possible future actions (not yet requested): compare Real Agent to LMOS,
    align governance with Eclipse conventions, or propose the project for
    Eclipse hosting.

## Repo orientation (for quick ramp-up)

- Spec: `README.md`, `SPEC.md`, `ONTOLOGY.md`, `CONTRACT.md`, `GOVERNANCE.md`,
  `LIFECYCLE.md`, `TRUST.md`; `proto/` (wire contract); `schemas/*.schema.json`
  (JSON Schema 2020-12 record contracts).
- Reference implementations (non-normative) under `reference/`:
  - `reference/surrealdb/` — SurrealDB memory store + Go package (`agentmem`),
    verified end-to-end against a live SurrealDB.
  - `reference/agent-lang/` — `.agent` DSL + `agentc` compiler → `agent.schema.json`.
- Docs: `docs/glossary.md`, `docs/schema-org-mapping.md`, `docs/agent-graph.md`,
  `docs/agent-language.md`.
