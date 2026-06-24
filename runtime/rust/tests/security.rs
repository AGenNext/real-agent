//! Security-as-core tests: the acting principal is gated by the agent's posture.

use real_agent_runtime::*;

fn setup(security: Security) -> (Runtime<MemoryStore>, String, String) {
    let mut rt = Runtime::new(MemoryStore::new());
    rt.register_tool(Tool {
        id: "tool:export".into(),
        name: "export".into(),
        target: "t".into(),
        reversible: true,
        approval_required: false,
        risk_level: RiskLevel::Low,
    });
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
    rt.grant_tool(&id, "tool:export").unwrap();
    rt.set_security(&id, security).unwrap();
    (rt, id, "tool:export".to_string())
}

fn decide(rt: &mut Runtime<MemoryStore>, agent_id: &str) -> String {
    rt.record_decision(
        agent_id,
        "o",
        vec![],
        vec![],
        "s",
        "r",
        0.9,
        PolicyResult {
            decision: PolicyDecision::Allow,
            reason: "ok".into(),
        },
    )
    .unwrap()
    .id
}

#[test]
fn authentication_required_blocks_anonymous() {
    let sec = Security {
        authentication_required: true,
        ..Default::default()
    };
    let (mut rt, id, tool) = setup(sec);
    let d = decide(&mut rt, &id);
    // Anonymous request is denied.
    assert_eq!(
        rt.request_action(&d, &tool),
        Err(RuntimeError::SecurityDenied("authentication required"))
    );
    // An authenticated principal passes.
    let p = Principal::new("oidc", "user:ops");
    assert!(rt.request_action_as(&d, &tool, &p).is_ok());
}

#[test]
fn authorization_restricts_identity_provider() {
    let sec = Security {
        authentication_required: true,
        authorization_required: true,
        allowed_identity_providers: vec!["oidc".into()],
        ..Default::default()
    };
    let (mut rt, id, tool) = setup(sec);
    let d = decide(&mut rt, &id);
    // Wrong provider denied.
    assert_eq!(
        rt.request_action_as(&d, &tool, &Principal::new("apikey", "k1")),
        Err(RuntimeError::SecurityDenied(
            "identity provider not authorized"
        ))
    );
    // Allowed provider passes.
    assert!(rt
        .request_action_as(&d, &tool, &Principal::new("oidc", "user:ops"))
        .is_ok());
}

#[test]
fn open_posture_allows_anonymous_by_default() {
    let (mut rt, id, tool) = setup(Security::default());
    let d = decide(&mut rt, &id);
    assert!(rt.request_action(&d, &tool).is_ok());
}
