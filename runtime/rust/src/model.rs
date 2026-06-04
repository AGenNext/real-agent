//! Canonical Real Agent contract types. Vendor-neutral and dependency-free
//! (pure `std`), so the core carries no supply-chain surface.
//!
//! These mirror `proto/real_agent/v1` and `schemas/*.json` so the in-memory
//! domain model and the wire contract stay aligned. Serialization (serde, proto,
//! JSON) belongs in adapter crates, not in the core.

/// Milliseconds since the Unix epoch.
pub type Millis = u64;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LifecycleState {
    Draft,
    Registered,
    Approved,
    Active,
    Paused,
    Suspended,
    Revoked,
    Retired,
    Archived,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PolicyDecision {
    Allow,
    Deny,
    RequireApproval,
    RequireMoreContext,
    Escalate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApprovalStatus {
    NotRequired,
    Pending,
    Approved,
    Rejected,
    Expired,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionStatus {
    Pending,
    Queued,
    Running,
    Completed,
    Failed,
    Cancelled,
    RolledBack,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrustState {
    Unknown,
    Experimental,
    Trusted,
    Conditional,
    Probation,
    Suspended,
    Revoked,
}

impl TrustState {
    /// Trust state derived from a measured score in [0, 1].
    pub fn from_score(score: f64) -> TrustState {
        match score {
            s if s < 0.0 => TrustState::Unknown,
            s if s < 0.4 => TrustState::Probation,
            s if s < 0.7 => TrustState::Conditional,
            _ => TrustState::Trusted,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Identity {
    pub id: String,
    pub name: String,
    pub version: String,
    /// Maps to the contract's `type` field.
    pub kind: String,
    pub owner: String,
    pub tenant: String,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Objective {
    pub primary: String,
    pub success_criteria: Vec<String>,
    pub constraints: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Capability {
    pub id: String,
    pub name: String,
    pub risk_level: RiskLevel,
}

/// What the agent is *allowed* to do — distinct from capability.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Authority {
    pub allowed_tools: Vec<String>,
    pub denied_tools: Vec<String>,
    pub escalation_requirements: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Trust {
    pub score: f64,
    pub state: TrustState,
}

impl Default for Trust {
    fn default() -> Self {
        Trust { score: 0.0, state: TrustState::Unknown }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Tool {
    pub id: String,
    pub name: String,
    pub target: String,
    pub reversible: bool,
    pub approval_required: bool,
    pub risk_level: RiskLevel,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Agent {
    pub identity: Identity,
    pub lifecycle_state: LifecycleState,
    pub objective: Objective,
    pub capabilities: Vec<Capability>,
    pub authority: Authority,
    /// Deny-by-default governance: an action is allowed only if explicitly authorized.
    pub deny_by_default: bool,
    /// Security posture (authn/authz, allowed identity providers, classification).
    pub security: crate::security::Security,
    pub trust: Trust,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DecisionAlternative {
    pub id: String,
    pub description: String,
    pub risk_level: RiskLevel,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PolicyResult {
    pub decision: PolicyDecision,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Decision {
    pub id: String,
    pub agent_id: String,
    pub timestamp: Millis,
    pub objective: String,
    pub alternatives: Vec<DecisionAlternative>,
    pub selected: String,
    pub reasoning_summary: String,
    pub confidence: f64,
    pub policy_result: PolicyResult,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Action {
    pub id: String,
    pub decision_id: String,
    pub agent_id: String,
    pub tool_id: String,
    pub authorization: ApprovalStatus,
    pub status: ActionStatus,
    pub reversible: bool,
    pub started_at: Millis,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Outcome {
    pub id: String,
    pub action_id: String,
    pub agent_id: String,
    pub success: bool,
    pub observed_result: String,
    pub observed_at: Millis,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Evaluation {
    pub id: String,
    pub agent_id: String,
    pub trust_score: f64,
    pub evaluated_at: Millis,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventKind {
    AgentRegistered,
    AgentApproved,
    AgentActivated,
    AgentSuspended,
    AgentRetired,
    DecisionMade,
    ActionRequested,
    ActionAuthorized,
    ApprovalRequested,
    ActionExecuted,
    OutcomeRecorded,
    EvaluationRecorded,
    TrustUpdated,
}

/// Append-only audit record. The accountable unit of agency lives in the event log.
#[derive(Debug, Clone, PartialEq)]
pub struct Event {
    pub id: String,
    pub kind: EventKind,
    pub agent_id: String,
    pub subject: String,
    pub at: Millis,
}
