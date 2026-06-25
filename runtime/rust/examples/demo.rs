//! A narrated tour of the runtime core: governance, security, memory, trust.
//!   cargo run --example demo

use real_agent_runtime::*;

fn main() {
    let mut rt = Runtime::new(MemoryStore::new());

    let low = |id: &str, approval: bool| Tool {
        id: id.into(),
        name: id.into(),
        target: "surreal".into(),
        reversible: true,
        approval_required: approval,
        risk_level: RiskLevel::Low,
    };
    rt.register_tool(low("tool:export", false));
    rt.register_tool(low("tool:import", true)); // import requires approval

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
            primary: "Keep databases recoverable".into(),
            ..Default::default()
        },
        vec![],
        Authority::default(),
    );
    let id = agent.identity.id.clone();
    println!(
        "1. Registered '{}' -> state {:?}",
        id, agent.lifecycle_state
    );

    // Governance: cannot act before activation.
    let early = rt.record_decision(&id, "x", vec![], vec![], "s", "r", 0.9, allow());
    println!("2. Decide while Registered -> {}", show(&early));

    // Governed activation.
    rt.transition(&id, LifecycleState::Approved).unwrap();
    rt.transition(&id, LifecycleState::Active).unwrap();
    println!(
        "3. Lifecycle: Registered -> Approved -> Active (Draft->Active is rejected by design)"
    );

    // Deny-by-default: acting on a tool the agent has no authority for.
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
    let denied = rt.request_action(&d.id, "tool:export");
    println!("4. Action without authority -> {}", show(&denied));

    // Grant authority, then the same action is allowed and executes.
    rt.grant_tool(&id, "tool:export").unwrap();
    let a = rt.request_action(&d.id, "tool:export").unwrap();
    rt.execute_action(&a.id).unwrap();
    rt.record_outcome(&a.id, true, "exported 1.2GB").unwrap();
    println!("5. Granted authority -> action {} executed", a.id);

    // Approval gate: a high-risk tool can't run until approved.
    rt.grant_tool(&id, "tool:import").unwrap();
    let d2 = rt
        .record_decision(
            &id,
            "restore",
            vec![],
            vec![],
            "import",
            "drill",
            0.8,
            allow(),
        )
        .unwrap();
    let pending = rt.request_action(&d2.id, "tool:import").unwrap();
    let blocked = rt.execute_action(&pending.id);
    println!(
        "6. Approval-gated import: execute before approval -> {}",
        show(&blocked)
    );
    rt.approve_action(&pending.id, "owner:acme").unwrap();
    rt.execute_action(&pending.id).unwrap();
    println!("   ...after owner approval -> executed");

    // Security: require an authenticated principal from an allowed provider.
    rt.set_security(
        &id,
        Security {
            authentication_required: true,
            authorization_required: true,
            allowed_identity_providers: vec!["oidc".into()],
            ..Default::default()
        },
    )
    .unwrap();
    let d3 = rt
        .record_decision(
            &id,
            "back up",
            vec![],
            vec![],
            "export",
            "again",
            0.9,
            allow(),
        )
        .unwrap();
    let anon = rt.request_action(&d3.id, "tool:export");
    println!("7. Action as anonymous (authn required) -> {}", show(&anon));
    let ok = rt.request_action_as(&d3.id, "tool:export", &Principal::new("oidc", "user:ops"));
    println!("   ...as oidc principal -> {}", show(&ok));

    // Memory: store knowledge, recall by similarity.
    rt.remember_semantic(
        &id,
        "billing export ran clean",
        vec![1.0, 0.0, 0.0],
        Classification::Internal,
    )
    .unwrap();
    rt.remember_semantic(
        &id,
        "restore drill recovered the cluster",
        vec![0.0, 1.0, 0.0],
        Classification::Internal,
    )
    .unwrap();
    let hits = rt.recall_semantic(&id, &[0.1, 0.95, 0.0], 1);
    println!(
        "8. Memory recall for a 'restore' query -> \"{}\" (score {:.2})",
        hits[0].record.content, hits[0].score
    );

    // Trust: a poor evaluation auto-suspends the agent.
    rt.record_evaluation(&id, 0.2).unwrap();
    println!(
        "9. Evaluation 0.20 (<= {:.2}) -> agent now {:?}",
        SUSPEND_THRESHOLD,
        rt.agent(&id).unwrap().lifecycle_state
    );

    println!(
        "\nAudit log ({} events, append-only):",
        rt.store().events().len()
    );
    for e in rt.store().events() {
        println!("   {:<20?} {}", e.kind, e.subject);
    }
}

fn allow() -> PolicyResult {
    PolicyResult {
        decision: PolicyDecision::Allow,
        reason: "within policy".into(),
    }
}

fn show<T>(r: &Result<T, RuntimeError>) -> String {
    match r {
        Ok(_) => "OK".into(),
        Err(e) => format!("blocked: {e}"),
    }
}
