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

pub mod model;
pub mod runtime;
pub mod store;

pub use model::*;
pub use runtime::{Runtime, RuntimeError, SUSPEND_THRESHOLD};
pub use store::{MemoryStore, Store};
