# Real Agent runtime — SurrealDB adapter

Implements the core's [`Store`] trait using the **SurrealDB Rust SDK**. The
vendor dependency lives *only* here; the core (`real-agent-runtime`) stays
zero-dependency. This proves the vendor-neutral design — swap the adapter,
keep the engine.

```rust
use real_agent_runtime::Runtime;
use real_agent_surrealdb::SurrealStore;

let rt = Runtime::new(SurrealStore::memory()?); // embedded; or wire a remote SDK
```

The sync `Store` port is bridged to the async SDK via a private Tokio runtime;
each entity is persisted as a JSON document with scalar index columns.

## Verify

```sh
cargo test   # round-trips the full agent loop through embedded SurrealDB
```
