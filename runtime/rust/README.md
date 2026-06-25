# Real Agent runtime core (Rust)

A **vendor-neutral, dependency-free** implementation of the Real Agent contract.
Pure `std` — **zero third-party crates**, so the core has no SBOM / supply-chain
surface. Storage backends and serialization are adapters behind a trait, never
baked into the core.

## What it enforces

The engine drives the agent loop and records every step as an immutable event:

```text
Observe → Decide → Authorize → Act → Evaluate → Remember
```

- **Governed lifecycle** — `Draft → Registered → Approved → Active → …`, with
  forbidden transitions rejected (no `Draft → Active`, no resurrection from
  `Revoked`).
- **Authority ≠ capability** — `can_invoke` checks what an agent is *allowed* to
  do, independent of what it *can* do.
- **Deny-by-default policy** — an action without authorization is denied.
- **Approval gates** — approval-required tools can't execute until approved.
- **Trust-driven suspension** — a low evaluation auto-suspends an active agent.
- **Append-only audit log** — every meaningful step emits an `Event`.

## Architecture (ports & adapters)

```text
   Runtime<S: Store>          ← engine (governance), this crate
        │ depends on
        ▼
   trait Store                ← vendor-neutral persistence port
   ├── MemoryStore            ← built-in, zero-dependency adapter
   └── (SurrealDB SDK adapter, etc.)  ← lives in a separate crate
```

The SurrealDB Rust SDK — or any database — is used *as a tool*: an adapter crate
implements `Store`. The core never names a vendor.

## Use

```sh
cargo test                      # 10 tests, compiles in <1s, no network
cargo run --example agent_loop  # runs one loop, prints the audit log
```

```rust
use real_agent_runtime::*;

let mut rt = Runtime::new(MemoryStore::new());
let agent = rt.register_agent(/* identity */, /* objective */, vec![], Authority::default());
rt.transition(&agent.identity.id, LifecycleState::Approved)?;
rt.transition(&agent.identity.id, LifecycleState::Active)?;
```

## Status

Verified: `cargo test` passes 10/10 on a zero-dependency build.
