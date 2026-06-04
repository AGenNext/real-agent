//! Runs one full pass of the agent loop on the in-memory store and prints the
//! resulting audit log. Build on the vendor-neutral core alone:
//!
//!   cargo run --example agent_loop

use real_agent_runtime::*;

fn main() {
    let mut rt = Runtime::new(MemoryStore::new());

    rt.register_tool(Tool {
        id: "tool:export".into(),
        name: "sql_export".into(),
        target: "surreal export".into(),
        reversible: true,
        approval_required: false,
        risk_level: RiskLevel::Low,
    });

    let agent = rt.register_agent(
        Identity {
            id: "agent:backup_steward".into(),
            name: "Backup Steward".into(),
            version: "0.1.0".into(),
            kind: "data-operations".into(),
            owner: "owner:acme".into(),
            tenant: "acme".into(),
        },
        Objective { primary: "Keep databases recoverable".into(), ..Default::default() },
        vec![Capability { id: "cap.export".into(), name: "Export".into(), risk_level: RiskLevel::Low }],
        Authority::default(),
    );
    let id = agent.identity.id.clone();

    // Governed activation, then authorize the tool.
    rt.transition(&id, LifecycleState::Approved).unwrap();
    rt.transition(&id, LifecycleState::Active).unwrap();
    rt.grant_tool(&id, "tool:export").unwrap();

    // Observe -> Decide -> Authorize -> Act -> Evaluate -> Remember.
    let ctx = rt.observe(&id, "scheduler", "nightly backup window open, billing db idle").unwrap();
    let decision = rt
        .record_decision(
            &id,
            "Back up the billing database",
            vec![ctx.id.clone()],
            vec![DecisionAlternative {
                id: "alt.export".into(),
                description: "Run a SurrealQL export".into(),
                risk_level: RiskLevel::Low,
            }],
            "alt.export",
            "Nightly window, low risk, reversible",
            0.95,
            PolicyResult { decision: PolicyDecision::Allow, reason: "within authority".into() },
        )
        .unwrap();

    let action = rt.request_action(&decision.id, "tool:export").unwrap();
    rt.execute_action(&action.id).unwrap();
    rt.record_outcome(&action.id, true, "export completed, 1.2GB").unwrap();
    rt.record_evaluation(&id, 0.9).unwrap();

    println!("audit log:");
    for e in rt.store().events() {
        println!("  {:>5}  {:<20?}  {}", e.at % 100000, e.kind, e.subject);
    }
    let a = rt.agent(&id).unwrap();
    println!("agent state: {:?}, trust: {:?} ({:.2})", a.lifecycle_state, a.trust.state, a.trust.score);
}
