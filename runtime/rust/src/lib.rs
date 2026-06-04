//! # Real Agent runtime core
//!
//! A vendor-neutral, dependency-free (pure `std`) implementation of the Real
//! Agent contract. No SBOM/supply-chain surface in the core; storage backends
//! (e.g. a SurrealDB SDK adapter) plug in behind the [`Store`] trait.
//!
//! The engine drives the agent loop and records every step as an immutable
//! event:
//!
//! ```text
//! Observe -> Decide -> Authorize -> Act -> Evaluate -> Remember
//! ```
//!
//! and enforces the governance the spec requires: a governed lifecycle,
//! authority distinct from capability, deny-by-default policy, approval gates,
//! trust-driven suspension, and an append-only audit log.
//!
//! # The twelve primitives (one real-world model)
//!
//! The core reflects the canonical ontology in full — no more, no less:
//!
//! | Primitive  | In the core |
//! |------------|-------------|
//! | Identity   | [`Identity`] on every [`Agent`] |
//! | Authority  | [`Authority`] — what the agent is *allowed* to do |
//! | Objective  | [`Objective`] |
//! | Context    | [`ContextItem`] via `Runtime::observe` |
//! | Decision   | [`Decision`] (grounded in `context_refs`) |
//! | Action     | [`Action`] (authorized, gated, then executed) |
//! | Outcome    | [`Outcome`] |
//! | Memory     | [`memory`] — semantic, episodic, procedural, working |
//! | Policy     | [`PolicyResult`] + deny-by-default on the [`Agent`] |
//! | Trust      | [`Trust`] / [`TrustState`], updated by evaluation |
//! | Evaluation | [`Evaluation`] |
//! | Governance | the [`Runtime`] engine + immutable [`Event`] log |
//!
//! Security ([`security`]) is treated as core alongside governance, not as an
//! adapter.

pub mod context;
pub mod memory;
pub mod model;
pub mod runtime;
pub mod security;
pub mod store;

pub use context::ContextItem;
pub use memory::{
    cosine, Classification, ProceduralKind, ProceduralRecord, Recall, SemanticRecord, WorkingMemory,
};
pub use model::*;
pub use runtime::{Runtime, RuntimeError, SUSPEND_THRESHOLD};
pub use security::{Principal, Security};
pub use store::{MemoryStore, Store};
