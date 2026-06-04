//! End-to-end tests for the Real Agent runtime core.

use real_agent_runtime::*;

fn tool(id: &str, approval: bool) -> Tool {
    Tool {
        id: id.to_string(),
        name: id.to_string(),
        target: "test".into(),
        reversible: true,
        approval_required: approval,
        risk_level: RiskLevel::Low,
    }
}

fn identity(id: &str) -> Identity {
    Identity {
        id: id.to_string(),
        name: "Backup Steward".into(),
        version: "0.1.0".into(),
        kind: "data-operations".into(),
        owner: "owner:acme".into(),
        tenant: "acme".into(),
    }
}

fn policy_allow() -> PolicyResult {
    PolicyResult { decision: PolicyDecision::Allow, reason: "ok".into() }
}

/// Build a runtime with one registered+activated agent and a granted tool.
fn active_agent_with_tool(approval: bool) -> (Runtime<MemoryStore>, String, String) {
    let mut rt = Runtime::new(MemoryStore::new());
    rt.register_tool(tool("tool:export", approval));
    let a = rt.register_agent(identity("agent:a"), Objective::default(), vec![], Authority::default());
    let id = a.identity.id.clone();
    rt.transition(&id, LifecycleState::Approved).unwrap();
    rt.transition(&id, LifecycleState::Active).unwrap();
    rt.grant_tool(&id, "tool:export").unwrap();
    (rt, id, "tool:export".to_string())
}

#[test]
fn registers_in_registered_state_not_active() {
    let mut rt = Runtime::new(MemoryStore::new());
    let a = rt.register_agent(identity("agent:a"), Objective::default(), vec![], Authority::default());
    assert_eq!(a.lifecycle_state, LifecycleState::Registered);
}

#[test]
fn forbids_draft_to_active_and_resurrection() {
    let mut rt = Runtime::new(MemoryStore::new());
    rt.register_agent(identity("agent:a"), Objective::default(), vec![], Authority::default());
    // Registered -> Active is not allowed (must go through Approved).
    assert_eq!(
        rt.transition("agent:a", LifecycleState::Active),
        Err(RuntimeError::ForbiddenTransition(LifecycleState::Registered, LifecycleState::Active))
    );
    rt.transition("agent:a", LifecycleState::Approved).unwrap();
    rt.transition("agent:a", LifecycleState::Active).unwrap();
    rt.transition("agent:a", LifecycleState::Revoked).unwrap();
    // Revoked cannot return to Active.
    assert!(matches!(
        rt.transition("agent:a", LifecycleState::Active),
        Err(RuntimeError::ForbiddenTransition(LifecycleState::Revoked, LifecycleState::Active))
    ));
}

#[test]
fn capability_is_not_permission() {
    let (rt, id, _) = active_agent_with_tool(false);
    assert!(rt.can_invoke(&id, "tool:export"));
    assert!(!rt.can_invoke(&id, "tool:unknown"));
}

#[test]
fn deny_by_default_blocks_ungranted_tool() {
    let mut rt = Runtime::new(MemoryStore::new());
    rt.register_tool(tool("tool:export", false));
    rt.register_agent(identity("agent:a"), Objective::default(), vec![], Authority::default());
    rt.transition("agent:a", LifecycleState::Approved).unwrap();
    rt.transition("agent:a", LifecycleState::Active).unwrap();
    let d = rt
        .record_decision("agent:a", "obj", vec![], vec![], "alt", "why", 0.9, policy_allow())
        .unwrap();
    // Tool not granted -> deny-by-default denies it.
    assert_eq!(
        rt.request_action(&d.id, "tool:export"),
        Err(RuntimeError::PolicyDenied("tool:export".into()))
    );
}

#[test]
fn happy_path_decide_act_execute_outcome() {
    let (mut rt, id, tool_id) = active_agent_with_tool(false);
    let d = rt
        .record_decision(&id, "back up db", vec![], vec![], "export", "nightly", 0.95, policy_allow())
        .unwrap();
    let action = rt.request_action(&d.id, &tool_id).unwrap();
    assert_eq!(action.authorization, ApprovalStatus::NotRequired);
    let executed = rt.execute_action(&action.id).unwrap();
    assert_eq!(executed.status, ActionStatus::Completed);
    let outcome = rt.record_outcome(&action.id, true, "exported").unwrap();
    assert!(outcome.success);
}

#[test]
fn approval_gate_blocks_execution_until_approved() {
    let (mut rt, id, tool_id) = active_agent_with_tool(true); // approval required
    let d = rt
        .record_decision(&id, "restore", vec![], vec![], "import", "drill", 0.8, policy_allow())
        .unwrap();
    let action = rt.request_action(&d.id, &tool_id).unwrap();
    assert_eq!(action.authorization, ApprovalStatus::Pending);
    // Cannot execute while approval is pending.
    assert_eq!(
        rt.execute_action(&action.id),
        Err(RuntimeError::ApprovalRequired(tool_id.clone()))
    );
    rt.approve_action(&action.id, "owner:acme").unwrap();
    let executed = rt.execute_action(&action.id).unwrap();
    assert_eq!(executed.status, ActionStatus::Completed);
}

#[test]
fn inactive_agent_cannot_decide() {
    let mut rt = Runtime::new(MemoryStore::new());
    rt.register_agent(identity("agent:a"), Objective::default(), vec![], Authority::default());
    // Still Registered, not Active.
    assert!(matches!(
        rt.record_decision("agent:a", "o", vec![], vec![], "s", "r", 0.5, policy_allow()),
        Err(RuntimeError::NotActive(LifecycleState::Registered))
    ));
}

#[test]
fn low_trust_auto_suspends_active_agent() {
    let (mut rt, id, _) = active_agent_with_tool(false);
    rt.record_evaluation(&id, 0.2).unwrap(); // <= SUSPEND_THRESHOLD
    assert_eq!(rt.agent(&id).unwrap().lifecycle_state, LifecycleState::Suspended);
    assert_eq!(rt.agent(&id).unwrap().trust.state, TrustState::Probation);
}

#[test]
fn good_trust_keeps_agent_active() {
    let (mut rt, id, _) = active_agent_with_tool(false);
    rt.record_evaluation(&id, 0.85).unwrap();
    assert_eq!(rt.agent(&id).unwrap().lifecycle_state, LifecycleState::Active);
    assert_eq!(rt.agent(&id).unwrap().trust.state, TrustState::Trusted);
}

#[test]
fn audit_log_captures_the_loop() {
    let (mut rt, id, tool_id) = active_agent_with_tool(false);
    let d = rt
        .record_decision(&id, "o", vec![], vec![], "s", "r", 0.9, policy_allow())
        .unwrap();
    let a = rt.request_action(&d.id, &tool_id).unwrap();
    rt.execute_action(&a.id).unwrap();
    rt.record_outcome(&a.id, true, "done").unwrap();
    rt.record_evaluation(&id, 0.9).unwrap();

    let kinds: Vec<EventKind> = rt.store().events().iter().map(|e| e.kind).collect();
    for expected in [
        EventKind::AgentRegistered,
        EventKind::AgentApproved,
        EventKind::AgentActivated,
        EventKind::DecisionMade,
        EventKind::ActionRequested,
        EventKind::ActionAuthorized,
        EventKind::ActionExecuted,
        EventKind::OutcomeRecorded,
        EventKind::EvaluationRecorded,
        EventKind::TrustUpdated,
    ] {
        assert!(kinds.contains(&expected), "missing audit event: {expected:?}");
    }
}
