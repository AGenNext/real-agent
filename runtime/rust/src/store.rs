//! Persistence port (vendor-neutral) plus a built-in in-memory adapter.
//!
//! The runtime core depends only on the `Store` trait, never on a concrete
//! database. A SurrealDB adapter (using the SurrealDB Rust SDK) — or any other
//! backend — implements this trait without the core knowing or caring.

use std::collections::HashMap;

use crate::model::*;

/// Vendor-neutral persistence interface used by the runtime engine.
pub trait Store {
    fn put_agent(&mut self, agent: Agent);
    fn get_agent(&self, id: &str) -> Option<Agent>;

    fn put_tool(&mut self, tool: Tool);
    fn get_tool(&self, id: &str) -> Option<Tool>;

    fn put_decision(&mut self, decision: Decision);
    fn get_decision(&self, id: &str) -> Option<Decision>;

    fn put_action(&mut self, action: Action);
    fn get_action(&self, id: &str) -> Option<Action>;

    fn put_outcome(&mut self, outcome: Outcome);
    fn put_evaluation(&mut self, evaluation: Evaluation);

    /// Append an immutable audit event.
    fn append_event(&mut self, event: Event);
    /// Full audit log, in append order.
    fn events(&self) -> Vec<Event>;
}

/// Default in-memory adapter — zero external dependencies.
#[derive(Debug, Default)]
pub struct MemoryStore {
    agents: HashMap<String, Agent>,
    tools: HashMap<String, Tool>,
    decisions: HashMap<String, Decision>,
    actions: HashMap<String, Action>,
    outcomes: HashMap<String, Outcome>,
    evaluations: HashMap<String, Evaluation>,
    events: Vec<Event>,
}

impl MemoryStore {
    pub fn new() -> Self {
        MemoryStore::default()
    }
}

impl Store for MemoryStore {
    fn put_agent(&mut self, agent: Agent) {
        self.agents.insert(agent.identity.id.clone(), agent);
    }
    fn get_agent(&self, id: &str) -> Option<Agent> {
        self.agents.get(id).cloned()
    }

    fn put_tool(&mut self, tool: Tool) {
        self.tools.insert(tool.id.clone(), tool);
    }
    fn get_tool(&self, id: &str) -> Option<Tool> {
        self.tools.get(id).cloned()
    }

    fn put_decision(&mut self, decision: Decision) {
        self.decisions.insert(decision.id.clone(), decision);
    }
    fn get_decision(&self, id: &str) -> Option<Decision> {
        self.decisions.get(id).cloned()
    }

    fn put_action(&mut self, action: Action) {
        self.actions.insert(action.id.clone(), action);
    }
    fn get_action(&self, id: &str) -> Option<Action> {
        self.actions.get(id).cloned()
    }

    fn put_outcome(&mut self, outcome: Outcome) {
        self.outcomes.insert(outcome.id.clone(), outcome);
    }
    fn put_evaluation(&mut self, evaluation: Evaluation) {
        self.evaluations.insert(evaluation.id.clone(), evaluation);
    }

    fn append_event(&mut self, event: Event) {
        self.events.push(event);
    }
    fn events(&self) -> Vec<Event> {
        self.events.clone()
    }
}
