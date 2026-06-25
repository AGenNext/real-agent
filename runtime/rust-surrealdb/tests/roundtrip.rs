//! The same agent loop the core tests run, but persisted through the SurrealDB
//! adapter — proving the vendor-neutral `Store` design against a real database.

use real_agent_runtime::*;
use real_agent_surrealdb::SurrealStore;

fn allow() -> PolicyResult {
    PolicyResult {
        decision: PolicyDecision::Allow,
        reason: "ok".into(),
    }
}

#[test]
fn agent_loop_persists_through_surrealdb() {
    let store = SurrealStore::memory().expect("embedded surrealdb");
    let mut rt = Runtime::new(store);

    rt.register_tool(Tool {
        id: "tool:export".into(),
        name: "export".into(),
        target: "surreal".into(),
        reversible: true,
        approval_required: false,
        risk_level: RiskLevel::Low,
    });
    let agent = rt.register_agent(
        Identity {
            id: "agent:steward".into(),
            name: "Backup Steward".into(),
            version: "0.1.0".into(),
            kind: "data-ops".into(),
            owner: "owner:acme".into(),
            tenant: "acme".into(),
        },
        Objective {
            primary: "Keep data recoverable".into(),
            ..Default::default()
        },
        vec![Capability {
            id: "cap.export".into(),
            name: "Export".into(),
            risk_level: RiskLevel::Low,
        }],
        Authority::default(),
    );
    let id = agent.identity.id.clone();

    // Agent round-trips through SurrealDB with its contract intact.
    let loaded = rt.agent(&id).unwrap();
    assert_eq!(loaded.identity.tenant, "acme");
    assert_eq!(loaded.lifecycle_state, LifecycleState::Registered);
    assert_eq!(loaded.capabilities[0].risk_level, RiskLevel::Low);

    rt.transition(&id, LifecycleState::Approved).unwrap();
    rt.transition(&id, LifecycleState::Active).unwrap();
    rt.grant_tool(&id, "tool:export").unwrap();

    let d = rt
        .record_decision(
            &id,
            "back up",
            vec![],
            vec![],
            "export",
            "nightly",
            0.95,
            allow(),
        )
        .unwrap();
    let a = rt.request_action(&d.id, "tool:export").unwrap();
    rt.execute_action(&a.id).unwrap();
    rt.record_outcome(&a.id, true, "exported").unwrap();
    rt.record_evaluation(&id, 0.9).unwrap();

    // State and trust persisted correctly.
    assert_eq!(
        rt.agent(&id).unwrap().lifecycle_state,
        LifecycleState::Active
    );
    assert_eq!(rt.agent(&id).unwrap().trust.state, TrustState::Trusted);

    // The full audit chain is on SurrealDB, in order.
    let kinds: Vec<EventKind> = rt.store().events().iter().map(|e| e.kind).collect();
    for k in [
        EventKind::AgentRegistered,
        EventKind::DecisionMade,
        EventKind::ActionExecuted,
        EventKind::OutcomeRecorded,
        EventKind::TrustUpdated,
    ] {
        assert!(kinds.contains(&k), "missing {k:?}");
    }
    assert_eq!(kinds.first(), Some(&EventKind::AgentRegistered));

    // Memory persists and recalls through the adapter.
    rt.remember_semantic(
        &id,
        "restore drill recovered the cluster",
        vec![0.0, 1.0, 0.0],
        Classification::Internal,
    )
    .unwrap();
    let hits = rt.recall_semantic(&id, &[0.0, 1.0, 0.0], 1);
    assert_eq!(
        hits[0].record.content,
        "restore drill recovered the cluster"
    );
}
