//! The Memory primitive (spec 4.7), vendor-neutral and dependency-free.
//!
//! - semantic: facts/knowledge with an embedding; recalled by cosine similarity
//!   (a naive exact KNN in pure `std` — no vector-DB dependency in the core);
//! - procedural: policies, playbooks, routines, skills;
//! - working: ephemeral current-task state;
//! - episodic: derived from the append-only event log (see `Store::events`).

use crate::model::Millis;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Classification {
    Public,
    Internal,
    Confidential,
    Restricted,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SemanticRecord {
    pub id: String,
    pub agent_id: String,
    pub content: String,
    pub embedding: Vec<f32>,
    pub classification: Classification,
    pub at: Millis,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProceduralKind {
    Policy,
    Playbook,
    Routine,
    Skill,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProceduralRecord {
    pub id: String,
    pub agent_id: String,
    pub kind: ProceduralKind,
    pub name: String,
    pub body: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct WorkingMemory {
    pub agent_id: String,
    pub task: String,
    pub state: String,
    pub expires_at: Millis,
}

/// Cosine similarity of two equal-length vectors. Returns 0.0 for mismatched or
/// zero-magnitude inputs (so recall degrades gracefully rather than panicking).
pub fn cosine(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }
    let mut dot = 0.0f32;
    let mut na = 0.0f32;
    let mut nb = 0.0f32;
    for i in 0..a.len() {
        dot += a[i] * b[i];
        na += a[i] * a[i];
        nb += b[i] * b[i];
    }
    if na == 0.0 || nb == 0.0 {
        return 0.0;
    }
    dot / (na.sqrt() * nb.sqrt())
}

/// A semantic-recall hit: the record plus its similarity to the query.
#[derive(Debug, Clone, PartialEq)]
pub struct Recall {
    pub record: SemanticRecord,
    pub score: f32,
}
