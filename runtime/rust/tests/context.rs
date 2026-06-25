//! Context primitive: observations are traceable and ground decisions.

use real_agent_runtime::*;

fn active(rt: &mut Runtime<MemoryStore>) -> String {
    let a = rt.register_agent(
        Identity {
            id: "agent:a".into(),
            name: "A".into(),
            version: "0.1.0".into(),
            kind: "t".into(),
            owner: "o".into(),
            tenant: "t".into(),
        },
        Objective::default(),
        vec![],
        Authority::default(),
    );
    let id = a.identity.id.clone();
    rt.transition(&id, LifecycleState::Approved).unwrap();
    rt.transition(&id, LifecycleState::Active).unwrap();
    id
}

#[test]
fn context_is_traceable_to_source() {
    let mut rt = Runtime::new(MemoryStore::new());
    let id = active(&mut rt);
    let item = rt
        .observe(&id, "email", "customer reports failed export")
        .unwrap();
    assert_eq!(item.source, "email");
    let all = rt.context(&id);
    assert_eq!(all.len(), 1);
    assert_eq!(all[0].content, "customer reports failed export");
}

#[test]
fn decision_is_grounded_in_observed_context() {
    let mut rt = Runtime::new(MemoryStore::new());
    let id = active(&mut rt);
    let c1 = rt.observe(&id, "scheduler", "backup window open").unwrap();
    let c2 = rt.observe(&id, "monitor", "db idle").unwrap();
    let d = rt
        .record_decision(
            &id,
            "back up",
            vec![c1.id.clone(), c2.id.clone()],
            vec![],
            "export",
            "low risk in window",
            0.95,
            PolicyResult {
                decision: PolicyDecision::Allow,
                reason: "ok".into(),
            },
        )
        .unwrap();
    // The decision references exactly the context it was grounded in.
    assert_eq!(d.context_refs, vec![c1.id, c2.id]);
}

#[test]
fn context_is_scoped_per_agent() {
    let mut rt = Runtime::new(MemoryStore::new());
    let id = active(&mut rt);
    rt.observe(&id, "src", "obs").unwrap();
    assert!(rt.context("agent:other").is_empty());
}
