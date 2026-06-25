# Providers

Built-in, **configurable** providers for the runtime's pluggable slots (the core's
ports). These are sensible defaults you override per slot — the full provider
**catalog is supplied by the marketplace** and is out of scope here.

## Slots

| Slot | Default | Options | Port |
|---|---|---|---|
| `store` | `memory` | `memory`, `surrealdb` | core `Store` trait |
| `identity` | `none` | `none`, `oidc`, `saml` | principal authentication |
| `embedding` | `none` | `none`, `openai`, `local` | semantic-memory vectors |
| `policy` | `builtin` (deny-by-default) | `builtin`, `external` | policy engine |

## Files

- `providers.json` — the default configuration (in-memory store, no auth, builtin policy).
- `providers.surrealdb.example.json` — an example override (SurrealDB store + OIDC + local embeddings).

Both validate against [`schemas/providers.schema.json`](../schemas/providers.schema.json).

## Override

Copy `providers.json`, change only the slots you need, and point the runtime at it.
Unspecified slots fall back to their defaults; the `embedding.dimension` must match
the store's vector index.
