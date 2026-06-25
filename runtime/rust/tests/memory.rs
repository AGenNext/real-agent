//! Tests for the Memory primitive.

use real_agent_runtime::*;

fn identity(id: &str) -> Identity {
    Identity {
        id: id.to_string(),
        name: "A".into(),
        version: "0.1.0".into(),
        kind: "t".into(),
        owner: "o".into(),
        tenant: "t".into(),
    }
}

fn agent(rt: &mut Runtime<MemoryStore>) -> String {
    let a = rt.register_agent(
        identity("agent:a"),
        Objective::default(),
        vec![],
        Authority::default(),
    );
    a.identity.id
}

#[test]
fn semantic_recall_ranks_by_cosine_similarity() {
    let mut rt = Runtime::new(MemoryStore::new());
    let id = agent(&mut rt);
    rt.remember_semantic(
        &id,
        "billing export ok",
        vec![1.0, 0.0, 0.0],
        Classification::Internal,
    )
    .unwrap();
    rt.remember_semantic(
        &id,
        "restore drill ok",
        vec![0.0, 1.0, 0.0],
        Classification::Internal,
    )
    .unwrap();
    rt.remember_semantic(
        &id,
        "policy note",
        vec![0.0, 0.0, 1.0],
        Classification::Confidential,
    )
    .unwrap();

    // Query closest to the "restore" vector.
    let hits = rt.recall_semantic(&id, &[0.1, 0.9, 0.0], 2);
    assert_eq!(hits.len(), 2);
    assert_eq!(hits[0].record.content, "restore drill ok");
    assert!(hits[0].score > hits[1].score);
}

#[test]
fn recall_is_scoped_per_agent() {
    let mut rt = Runtime::new(MemoryStore::new());
    let a = rt.register_agent(
        identity("agent:a"),
        Objective::default(),
        vec![],
        Authority::default(),
    );
    let b = rt.register_agent(
        Identity {
            id: "agent:b".into(),
            ..identity("agent:b")
        },
        Objective::default(),
        vec![],
        Authority::default(),
    );
    rt.remember_semantic(
        &a.identity.id,
        "a-knowledge",
        vec![1.0, 0.0],
        Classification::Internal,
    )
    .unwrap();
    let hits = rt.recall_semantic(&b.identity.id, &[1.0, 0.0], 5);
    assert!(hits.is_empty(), "agent b must not see agent a's memory");
}

#[test]
fn cosine_handles_degenerate_inputs() {
    assert_eq!(cosine(&[], &[]), 0.0);
    assert_eq!(cosine(&[1.0, 2.0], &[1.0]), 0.0); // length mismatch
    assert_eq!(cosine(&[0.0, 0.0], &[1.0, 1.0]), 0.0); // zero magnitude
    assert!((cosine(&[1.0, 0.0], &[1.0, 0.0]) - 1.0).abs() < 1e-6);
}

#[test]
fn procedural_and_working_memory_roundtrip() {
    let mut rt = Runtime::new(MemoryStore::new());
    let id = agent(&mut rt);
    rt.remember_procedural(
        &id,
        ProceduralKind::Playbook,
        "nightly-backup",
        "1. export 2. verify",
    )
    .unwrap();
    assert_eq!(rt.procedural(&id).len(), 1);
    assert_eq!(rt.procedural(&id)[0].kind, ProceduralKind::Playbook);

    rt.set_working(&id, "task-1", "step=2", 3_600_000).unwrap();
    let w = rt.get_working(&id, "task-1").unwrap();
    assert_eq!(w.state, "step=2");
    assert!(rt.get_working(&id, "missing").is_none());
}

#[test]
fn episodic_memory_is_the_agents_event_slice() {
    let mut rt = Runtime::new(MemoryStore::new());
    let id = agent(&mut rt);
    rt.transition(&id, LifecycleState::Approved).unwrap();
    rt.transition(&id, LifecycleState::Active).unwrap();
    let ep = rt.episodic(&id);
    let kinds: Vec<EventKind> = ep.iter().map(|e| e.kind).collect();
    assert!(kinds.contains(&EventKind::AgentRegistered));
    assert!(kinds.contains(&EventKind::AgentActivated));
    assert!(ep.iter().all(|e| e.agent_id == id));
}
