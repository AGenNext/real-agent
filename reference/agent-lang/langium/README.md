# Agent language — Langium grammar

A formal [Langium](https://langium.org) grammar for the `.agent` DSL, side-by-side
with the hand-written Go parser in [`../`](../). Both parse the same `.agent` files;
this one additionally gives you a generated parser, AST types, and LSP scaffolding.

## Build & test

```bash
npm install
npm run langium:generate     # generate parser/AST into src/generated
npm test                     # parse ../examples/cluster-janitor.agent, expect 0 errors
```

Verified: the example parses with **0 lexer / 0 parser errors** and the AST matches
(agent id, identity, 2 capabilities, 1 action, memory flags, evaluation).

## Files
- `src/agent.langium` — the grammar
- `src/parse-test.ts` — parses the example and checks for errors
- `langium-config.json` — Langium codegen config
