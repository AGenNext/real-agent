//! The Real Agent runtime engine: governs lifecycle, authority, policy,
//! approval, action execution, outcomes, trust, and the append-only audit log.
//!
//! Generic over any [`Store`]; the core never names a concrete backend.

use std::error::Error;
use std::fmt;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::context::ContextItem;
use crate::memory::*;
use crate::model::*;
use crate::security::{Principal, Security};
use crate::store::Store;

/// Trust at or below this score suspends an active agent.
pub const SUSPEND_THRESHOLD: f64 = 0.4;

#[derive(Debug, Clone, PartialEq)]
pub enum RuntimeError {
    UnknownAgent(String),
    UnknownTool(String),
    UnknownDecision(String),
    UnknownAction(String),
    /// Lifecycle transition is not permitted (e.g. Draft -> Active).
    ForbiddenTransition(LifecycleState, LifecycleState),
    /// Agent is not Active and so may not decide or act.
    NotActive(LifecycleState),
    /// Tool is outside the agent's authority (capability is not permission).
    NotAuthorized(String),
    /// Policy denied the action (deny-by-default with no authorization).
    PolicyDenied(String),
    /// Action requires approval that has not been granted.
    ApprovalRequired(String),
    /// The acting principal failed the agent's security posture.
    SecurityDenied(&'static str),
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use RuntimeError::*;
        match self {
            UnknownAgent(id) => write!(f, "unknown agent: {id}"),
            UnknownTool(id) => write!(f, "unknown tool: {id}"),
            UnknownDecision(id) => write!(f, "unknown decision: {id}"),
            UnknownAction(id) => write!(f, "unknown action: {id}"),
            ForbiddenTransition(from, to) => {
                write!(f, "forbidden lifecycle transition: {from:?} -> {to:?}")
            }
            NotActive(s) => write!(f, "agent not active (state: {s:?})"),
            NotAuthorized(t) => write!(f, "agent not authorized for tool: {t}"),
            PolicyDenied(t) => write!(f, "policy denied action on tool: {t}"),
            ApprovalRequired(t) => write!(f, "approval required for tool: {t}"),
            SecurityDenied(why) => write!(f, "security denied: {why}"),
        }
    }
}

impl Error for RuntimeError {}

type Result<T> = std::result::Result<T, RuntimeError>;

/// Is a lifecycle transition allowed? Encodes the spec's required and forbidden
/// transitions (no Draft->Active, no resurrection from Revoked, no Suspended->Active
/// without re-approval).
fn transition_allowed(from: LifecycleState, to: LifecycleState) -> bool {
    use LifecycleState::*;
    matches!(
        (from, to),
        (Draft, Registered)
            | (Registered, Approved)
            | (Approved, Active)
            | (Active, Paused)
            | (Paused, Active)
            | (Active, Suspended)
            | (Suspended, Approved)
            | (Active, Retired)
            | (Suspended, Retired)
            | (Retired, Archived)
            | (Active, Revoked)
            | (Suspended, Revoked)
    )
}

pub struct Runtime<S: Store> {
    store: S,
    seq: u64,
}

impl<S: Store> Runtime<S> {
    pub fn new(store: S) -> Self {
        Runtime { store, seq: 0 }
    }

    /// Borrow the underlying store (e.g. to read the audit log).
    pub fn store(&self) -> &S {
        &self.store
    }

    fn now(&self) -> Millis {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0)
    }

    fn next_id(&mut self, prefix: &str) -> String {
        self.seq += 1;
        format!("{prefix}:{}", self.seq)
    }

    fn emit(&mut self, kind: EventKind, agent_id: &str, subject: &str) {
        let id = self.next_id("event");
        let at = self.now();
        self.store.append_event(Event {
            id,
            kind,
            agent_id: agent_id.to_string(),
            subject: subject.to_string(),
            at,
        });
    }

    pub fn register_tool(&mut self, tool: Tool) {
        self.store.put_tool(tool);
    }

    /// Register an agent. Lands in `Registered` (never directly `Active`).
    pub fn register_agent(
        &mut self,
        identity: Identity,
        objective: Objective,
        capabilities: Vec<Capability>,
        authority: Authority,
    ) -> Agent {
        let agent = Agent {
            identity,
            lifecycle_state: LifecycleState::Registered,
            objective,
            capabilities,
            authority,
            deny_by_default: true,
            security: Security::default(),
            trust: Trust::default(),
        };
        let id = agent.identity.id.clone();
        self.store.put_agent(agent.clone());
        self.emit(EventKind::AgentRegistered, &id, &id);
        agent
    }

    pub fn agent(&self, id: &str) -> Result<Agent> {
        self.store
            .get_agent(id)
            .ok_or_else(|| RuntimeError::UnknownAgent(id.to_string()))
    }

    /// Move an agent through a governed lifecycle transition.
    pub fn transition(&mut self, agent_id: &str, to: LifecycleState) -> Result<()> {
        let mut agent = self.agent(agent_id)?;
        let from = agent.lifecycle_state;
        if !transition_allowed(from, to) {
            return Err(RuntimeError::ForbiddenTransition(from, to));
        }
        agent.lifecycle_state = to;
        self.store.put_agent(agent);
        if let Some(kind) = match to {
            LifecycleState::Approved => Some(EventKind::AgentApproved),
            LifecycleState::Active => Some(EventKind::AgentActivated),
            LifecycleState::Suspended => Some(EventKind::AgentSuspended),
            LifecycleState::Retired => Some(EventKind::AgentRetired),
            _ => None,
        } {
            self.emit(kind, agent_id, agent_id);
        }
        Ok(())
    }

    /// Grant the agent authority to use a tool.
    pub fn grant_tool(&mut self, agent_id: &str, tool_id: &str) -> Result<()> {
        if self.store.get_tool(tool_id).is_none() {
            return Err(RuntimeError::UnknownTool(tool_id.to_string()));
        }
        let mut agent = self.agent(agent_id)?;
        if !agent.authority.allowed_tools.iter().any(|t| t == tool_id) {
            agent.authority.allowed_tools.push(tool_id.to_string());
        }
        self.store.put_agent(agent);
        Ok(())
    }

    /// Set the agent's security posture (authn/authz, allowed identity providers).
    pub fn set_security(&mut self, agent_id: &str, security: Security) -> Result<()> {
        let mut agent = self.agent(agent_id)?;
        agent.security = security;
        self.store.put_agent(agent);
        Ok(())
    }

    /// Authority check: is the agent *allowed* to use this tool?
    pub fn can_invoke(&self, agent_id: &str, tool_id: &str) -> bool {
        match self.store.get_agent(agent_id) {
            Some(a) => {
                !a.authority.denied_tools.iter().any(|t| t == tool_id)
                    && a.authority.allowed_tools.iter().any(|t| t == tool_id)
            }
            None => false,
        }
    }

    pub fn needs_approval(&self, agent_id: &str, tool_id: &str) -> Result<bool> {
        let _ = self.agent(agent_id)?;
        let tool = self
            .store
            .get_tool(tool_id)
            .ok_or_else(|| RuntimeError::UnknownTool(tool_id.to_string()))?;
        Ok(tool.approval_required)
    }

    /// Observe a piece of context (the "Observe" step). Traceable to its source.
    pub fn observe(&mut self, agent_id: &str, source: &str, content: &str) -> Result<ContextItem> {
        let _ = self.agent(agent_id)?;
        let id = self.next_id("context");
        let item = ContextItem {
            id,
            agent_id: agent_id.to_string(),
            source: source.to_string(),
            content: content.to_string(),
            observed_at: self.now(),
        };
        self.store.put_context(item.clone());
        Ok(item)
    }

    /// The agent's observed context.
    pub fn context(&self, agent_id: &str) -> Vec<ContextItem> {
        self.store.context_for(agent_id)
    }

    /// Record a decision grounded in observed context. Requires the agent Active.
    #[allow(clippy::too_many_arguments)]
    pub fn record_decision(
        &mut self,
        agent_id: &str,
        objective: &str,
        context_refs: Vec<String>,
        alternatives: Vec<DecisionAlternative>,
        selected: &str,
        reasoning: &str,
        confidence: f64,
        policy: PolicyResult,
    ) -> Result<Decision> {
        let agent = self.agent(agent_id)?;
        if agent.lifecycle_state != LifecycleState::Active {
            return Err(RuntimeError::NotActive(agent.lifecycle_state));
        }
        let id = self.next_id("decision");
        let decision = Decision {
            id: id.clone(),
            agent_id: agent_id.to_string(),
            timestamp: self.now(),
            objective: objective.to_string(),
            context_refs,
            alternatives,
            selected: selected.to_string(),
            reasoning_summary: reasoning.to_string(),
            confidence,
            policy_result: policy,
        };
        self.store.put_decision(decision.clone());
        self.emit(EventKind::DecisionMade, agent_id, &id);
        Ok(decision)
    }

    /// Request an action arising from a decision, as an anonymous principal.
    pub fn request_action(&mut self, decision_id: &str, tool_id: &str) -> Result<Action> {
        self.request_action_as(decision_id, tool_id, &Principal::anonymous())
    }

    /// Request an action on behalf of `principal`. Enforces, in order: agent
    /// Active, the agent's security posture (authn/authz), authority (capability
    /// is not permission), deny-by-default policy, and approval gating.
    /// Authorized actions are recorded but not yet executed.
    pub fn request_action_as(
        &mut self,
        decision_id: &str,
        tool_id: &str,
        principal: &Principal,
    ) -> Result<Action> {
        let decision = self
            .store
            .get_decision(decision_id)
            .ok_or_else(|| RuntimeError::UnknownDecision(decision_id.to_string()))?;
        let agent_id = decision.agent_id.clone();
        let agent = self.agent(&agent_id)?;
        if agent.lifecycle_state != LifecycleState::Active {
            return Err(RuntimeError::NotActive(agent.lifecycle_state));
        }
        // Security is core: the principal must satisfy the agent's posture.
        agent
            .security
            .check(principal)
            .map_err(RuntimeError::SecurityDenied)?;
        let tool = self
            .store
            .get_tool(tool_id)
            .ok_or_else(|| RuntimeError::UnknownTool(tool_id.to_string()))?;

        if !self.can_invoke(&agent_id, tool_id) {
            // deny-by-default: no authority => denied.
            if agent.deny_by_default {
                return Err(RuntimeError::PolicyDenied(tool_id.to_string()));
            }
            return Err(RuntimeError::NotAuthorized(tool_id.to_string()));
        }

        let authorization = if tool.approval_required {
            ApprovalStatus::Pending
        } else {
            ApprovalStatus::NotRequired
        };
        let status = ActionStatus::Pending;

        let id = self.next_id("action");
        let action = Action {
            id: id.clone(),
            decision_id: decision_id.to_string(),
            agent_id: agent_id.clone(),
            tool_id: tool_id.to_string(),
            authorization,
            status,
            reversible: tool.reversible,
            started_at: self.now(),
        };
        self.store.put_action(action.clone());
        self.emit(EventKind::ActionRequested, &agent_id, &id);
        if authorization == ApprovalStatus::Pending {
            self.emit(EventKind::ApprovalRequested, &agent_id, &id);
        } else {
            self.emit(EventKind::ActionAuthorized, &agent_id, &id);
        }
        Ok(action)
    }

    /// Approve a pending action (records the approver in the audit log).
    pub fn approve_action(&mut self, action_id: &str, approver: &str) -> Result<()> {
        let mut action = self
            .store
            .get_action(action_id)
            .ok_or_else(|| RuntimeError::UnknownAction(action_id.to_string()))?;
        action.authorization = ApprovalStatus::Approved;
        let agent_id = action.agent_id.clone();
        self.store.put_action(action);
        self.emit(EventKind::ActionAuthorized, &agent_id, approver);
        Ok(())
    }

    /// Execute an action. Only authorized (or approved) actions run.
    pub fn execute_action(&mut self, action_id: &str) -> Result<Action> {
        let mut action = self
            .store
            .get_action(action_id)
            .ok_or_else(|| RuntimeError::UnknownAction(action_id.to_string()))?;
        if action.authorization == ApprovalStatus::Pending
            || action.authorization == ApprovalStatus::Rejected
        {
            return Err(RuntimeError::ApprovalRequired(action.tool_id.clone()));
        }
        action.status = ActionStatus::Completed;
        let agent_id = action.agent_id.clone();
        let id = action.id.clone();
        self.store.put_action(action.clone());
        self.emit(EventKind::ActionExecuted, &agent_id, &id);
        Ok(action)
    }

    pub fn record_outcome(
        &mut self,
        action_id: &str,
        success: bool,
        observed_result: &str,
    ) -> Result<Outcome> {
        let action = self
            .store
            .get_action(action_id)
            .ok_or_else(|| RuntimeError::UnknownAction(action_id.to_string()))?;
        let id = self.next_id("outcome");
        let outcome = Outcome {
            id: id.clone(),
            action_id: action_id.to_string(),
            agent_id: action.agent_id.clone(),
            success,
            observed_result: observed_result.to_string(),
            observed_at: self.now(),
        };
        self.store.put_outcome(outcome.clone());
        self.emit(EventKind::OutcomeRecorded, &action.agent_id, &id);
        Ok(outcome)
    }

    /// Record an evaluation, update the agent's trust, and auto-suspend if trust
    /// falls to the safety threshold.
    pub fn record_evaluation(&mut self, agent_id: &str, trust_score: f64) -> Result<Evaluation> {
        let mut agent = self.agent(agent_id)?;
        let id = self.next_id("evaluation");
        let evaluation = Evaluation {
            id: id.clone(),
            agent_id: agent_id.to_string(),
            trust_score,
            evaluated_at: self.now(),
        };
        self.store.put_evaluation(evaluation.clone());
        self.emit(EventKind::EvaluationRecorded, agent_id, &id);

        agent.trust = Trust {
            score: trust_score,
            state: TrustState::from_score(trust_score),
        };
        let active = agent.lifecycle_state == LifecycleState::Active;
        self.store.put_agent(agent);
        self.emit(EventKind::TrustUpdated, agent_id, &id);

        if active && trust_score <= SUSPEND_THRESHOLD {
            self.transition(agent_id, LifecycleState::Suspended)?;
        }
        Ok(evaluation)
    }

    // --- Memory primitive ---------------------------------------------------

    /// Store a semantic (knowledge) memory for an agent.
    pub fn remember_semantic(
        &mut self,
        agent_id: &str,
        content: &str,
        embedding: Vec<f32>,
        classification: Classification,
    ) -> Result<SemanticRecord> {
        let _ = self.agent(agent_id)?;
        let id = self.next_id("semantic");
        let record = SemanticRecord {
            id,
            agent_id: agent_id.to_string(),
            content: content.to_string(),
            embedding,
            classification,
            at: self.now(),
        };
        self.store.put_semantic(record.clone());
        Ok(record)
    }

    /// Recall the top-`k` semantic memories most similar to a query embedding
    /// (exact cosine KNN; knowledge retrieval for grounding/RAG).
    pub fn recall_semantic(&self, agent_id: &str, query: &[f32], k: usize) -> Vec<Recall> {
        let mut hits: Vec<Recall> = self
            .store
            .semantic_for(agent_id)
            .into_iter()
            .map(|record| {
                let score = cosine(&record.embedding, query);
                Recall { record, score }
            })
            .collect();
        hits.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        hits.truncate(k);
        hits
    }

    /// Store a procedural memory (policy/playbook/routine/skill).
    pub fn remember_procedural(
        &mut self,
        agent_id: &str,
        kind: ProceduralKind,
        name: &str,
        body: &str,
    ) -> Result<ProceduralRecord> {
        let _ = self.agent(agent_id)?;
        let id = self.next_id("procedural");
        let record = ProceduralRecord {
            id,
            agent_id: agent_id.to_string(),
            kind,
            name: name.to_string(),
            body: body.to_string(),
        };
        self.store.put_procedural(record.clone());
        Ok(record)
    }

    pub fn procedural(&self, agent_id: &str) -> Vec<ProceduralRecord> {
        self.store.procedural_for(agent_id)
    }

    /// Set the agent's ephemeral working state for a task.
    pub fn set_working(
        &mut self,
        agent_id: &str,
        task: &str,
        state: &str,
        ttl_ms: Millis,
    ) -> Result<()> {
        let _ = self.agent(agent_id)?;
        let expires_at = self.now() + ttl_ms;
        self.store.set_working(WorkingMemory {
            agent_id: agent_id.to_string(),
            task: task.to_string(),
            state: state.to_string(),
            expires_at,
        });
        Ok(())
    }

    pub fn get_working(&self, agent_id: &str, task: &str) -> Option<WorkingMemory> {
        self.store.get_working(agent_id, task)
    }

    /// Episodic memory: the agent's slice of the append-only event log.
    pub fn episodic(&self, agent_id: &str) -> Vec<Event> {
        self.store
            .events()
            .into_iter()
            .filter(|e| e.agent_id == agent_id)
            .collect()
    }
}
