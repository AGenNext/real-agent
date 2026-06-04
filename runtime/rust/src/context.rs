//! The Context primitive: what the agent observes about current reality before
//! it decides. Context SHOULD be traceable to its source (spec 4.3), so each
//! item records where it came from. Vendor-neutral, dependency-free.

use crate::model::Millis;

/// A single observation the agent reasons over. Decisions reference these by id,
/// closing the model's chain: Objective -> Context -> Decision -> Action -> ...
#[derive(Debug, Clone, PartialEq)]
pub struct ContextItem {
    pub id: String,
    pub agent_id: String,
    /// Where the observation came from (message, document, event, sensor, …).
    pub source: String,
    pub content: String,
    pub observed_at: Millis,
}
