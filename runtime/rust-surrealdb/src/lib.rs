//! SurrealDB adapter for the Real Agent core's `Store` trait.
//!
//! The vendor (SurrealDB Rust SDK) is isolated to this crate. The core
//! (`real_agent_runtime`) stays dependency-free; here we bridge its sync `Store`
//! port to the async SDK via a private Tokio runtime, persisting each entity as
//! a JSON document. This proves the vendor-neutral design: swap the adapter,
//! keep the engine.

use real_agent_runtime::*;
use serde_json::{json, Value};
use surrealdb::engine::local::{Db, Mem};
use surrealdb::Surreal;
use tokio::runtime::Runtime as Tokio;

pub struct SurrealStore {
    db: Surreal<Db>,
    rt: Tokio,
    ord: std::cell::Cell<u64>,
}

impl SurrealStore {
    /// An embedded in-memory SurrealDB instance (no external server).
    #[allow(clippy::result_large_err)] // surrealdb::Error is the SDK's own type
    pub fn memory() -> surrealdb::Result<Self> {
        let rt = Tokio::new().expect("tokio runtime");
        let db = rt.block_on(async {
            let db = Surreal::new::<Mem>(()).await?;
            db.use_ns("real_agent").use_db("v1").await?;
            Ok::<_, surrealdb::Error>(db)
        })?;
        Ok(SurrealStore {
            db,
            rt,
            ord: std::cell::Cell::new(0),
        })
    }

    /// Store `doc` (the full entity JSON) as an opaque `data` string plus plain
    /// scalar index columns. Binding only primitives avoids SurrealDB's binder
    /// rejecting `serde_json::Value`, and the record's own id never collides
    /// with the entity's logical id (which lives inside `data`).
    fn upsert(&self, table: &str, key: &str, doc: Value, ord: i64) {
        let agent_id = gs(&doc, "agent_id");
        let action_id = gs(&doc, "action_id");
        let data = doc.to_string();
        let q = "UPSERT type::thing($tb, $k) SET k = $k, data = $data, \
                 agent_id = $aid, action_id = $acid, ord = $ord";
        let _ = self.rt.block_on(async {
            self.db
                .query(q)
                .bind(("tb", table.to_string()))
                .bind(("k", key.to_string()))
                .bind(("data", data))
                .bind(("aid", agent_id))
                .bind(("acid", action_id))
                .bind(("ord", ord))
                .await
        });
    }

    fn parse_rows(rows: Vec<Value>) -> Vec<Value> {
        rows.iter()
            .filter_map(|r| r.get("data").and_then(|d| d.as_str()))
            .filter_map(|s| serde_json::from_str(s).ok())
            .collect()
    }

    fn get_one(&self, table: &str, key: &str) -> Option<Value> {
        self.rt
            .block_on(async {
                let mut r = self
                    .db
                    .query("SELECT data FROM type::thing($tb, $k)")
                    .bind(("tb", table.to_string()))
                    .bind(("k", key.to_string()))
                    .await
                    .ok()?;
                let rows: Vec<Value> = r.take(0).ok()?;
                Some(Self::parse_rows(rows))
            })
            .and_then(|v| v.into_iter().next())
    }

    fn select_where(&self, table: &str, field: &str, val: &str, order: bool) -> Vec<Value> {
        // `table` and `field` are internal constants, never user input.
        // Backtick-quote the table name so reserved words (e.g. `event`) parse.
        let q = if order {
            format!("SELECT data FROM `{table}` WHERE {field} = $val ORDER BY ord")
        } else {
            format!("SELECT data FROM `{table}` WHERE {field} = $val")
        };
        self.rt
            .block_on(async {
                let mut r = self.db.query(q).bind(("val", val.to_string())).await.ok()?;
                let rows: Vec<Value> = r.take(0).ok()?;
                Some(Self::parse_rows(rows))
            })
            .unwrap_or_default()
    }

    fn select_all(&self, table: &str) -> Vec<Value> {
        self.rt
            .block_on(async {
                let mut r = self
                    .db
                    .query(format!("SELECT data, ord FROM `{table}`"))
                    .await
                    .ok()?;
                let mut rows: Vec<Value> = r.take(0).ok()?;
                rows.sort_by_key(|v| v.get("ord").and_then(|o| o.as_i64()).unwrap_or(0));
                Some(Self::parse_rows(rows))
            })
            .unwrap_or_default()
    }
}

// --- value helpers ----------------------------------------------------------
fn gs(v: &Value, k: &str) -> String {
    v.get(k)
        .and_then(|x| x.as_str())
        .unwrap_or_default()
        .to_string()
}
fn gb(v: &Value, k: &str) -> bool {
    v.get(k).and_then(|x| x.as_bool()).unwrap_or(false)
}
fn gf(v: &Value, k: &str) -> f64 {
    v.get(k).and_then(|x| x.as_f64()).unwrap_or(0.0)
}
fn gu(v: &Value, k: &str) -> u64 {
    v.get(k).and_then(|x| x.as_u64()).unwrap_or(0)
}
fn ga(v: &Value, k: &str) -> Vec<String> {
    v.get(k)
        .and_then(|x| x.as_array())
        .map(|a| {
            a.iter()
                .filter_map(|e| e.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default()
}
fn gfa(v: &Value, k: &str) -> Vec<f32> {
    v.get(k)
        .and_then(|x| x.as_array())
        .map(|a| {
            a.iter()
                .filter_map(|e| e.as_f64().map(|f| f as f32))
                .collect()
        })
        .unwrap_or_default()
}

// --- enum string mapping ----------------------------------------------------
fn risk_s(r: RiskLevel) -> &'static str {
    match r {
        RiskLevel::Low => "low",
        RiskLevel::Medium => "medium",
        RiskLevel::High => "high",
        RiskLevel::Critical => "critical",
    }
}
fn risk_p(s: &str) -> RiskLevel {
    match s {
        "medium" => RiskLevel::Medium,
        "high" => RiskLevel::High,
        "critical" => RiskLevel::Critical,
        _ => RiskLevel::Low,
    }
}
fn life_s(l: LifecycleState) -> &'static str {
    use LifecycleState::*;
    match l {
        Draft => "draft",
        Registered => "registered",
        Approved => "approved",
        Active => "active",
        Paused => "paused",
        Suspended => "suspended",
        Revoked => "revoked",
        Retired => "retired",
        Archived => "archived",
    }
}
fn life_p(s: &str) -> LifecycleState {
    use LifecycleState::*;
    match s {
        "registered" => Registered,
        "approved" => Approved,
        "active" => Active,
        "paused" => Paused,
        "suspended" => Suspended,
        "revoked" => Revoked,
        "retired" => Retired,
        "archived" => Archived,
        _ => Draft,
    }
}
fn appr_s(a: ApprovalStatus) -> &'static str {
    use ApprovalStatus::*;
    match a {
        NotRequired => "not_required",
        Pending => "pending",
        Approved => "approved",
        Rejected => "rejected",
        Expired => "expired",
    }
}
fn appr_p(s: &str) -> ApprovalStatus {
    use ApprovalStatus::*;
    match s {
        "pending" => Pending,
        "approved" => Approved,
        "rejected" => Rejected,
        "expired" => Expired,
        _ => NotRequired,
    }
}
fn act_s(a: ActionStatus) -> &'static str {
    use ActionStatus::*;
    match a {
        Pending => "pending",
        Queued => "queued",
        Running => "running",
        Completed => "completed",
        Failed => "failed",
        Cancelled => "cancelled",
        RolledBack => "rolled_back",
    }
}
fn act_p(s: &str) -> ActionStatus {
    use ActionStatus::*;
    match s {
        "queued" => Queued,
        "running" => Running,
        "completed" => Completed,
        "failed" => Failed,
        "cancelled" => Cancelled,
        "rolled_back" => RolledBack,
        _ => Pending,
    }
}
fn trust_s(t: TrustState) -> &'static str {
    use TrustState::*;
    match t {
        Unknown => "unknown",
        Experimental => "experimental",
        Trusted => "trusted",
        Conditional => "conditional",
        Probation => "probation",
        Suspended => "suspended",
        Revoked => "revoked",
    }
}
fn trust_p(s: &str) -> TrustState {
    use TrustState::*;
    match s {
        "experimental" => Experimental,
        "trusted" => Trusted,
        "conditional" => Conditional,
        "probation" => Probation,
        "suspended" => Suspended,
        "revoked" => Revoked,
        _ => Unknown,
    }
}
fn pol_s(p: PolicyDecision) -> &'static str {
    use PolicyDecision::*;
    match p {
        Allow => "allow",
        Deny => "deny",
        RequireApproval => "require_approval",
        RequireMoreContext => "require_more_context",
        Escalate => "escalate",
    }
}
fn pol_p(s: &str) -> PolicyDecision {
    use PolicyDecision::*;
    match s {
        "deny" => Deny,
        "require_approval" => RequireApproval,
        "require_more_context" => RequireMoreContext,
        "escalate" => Escalate,
        _ => Allow,
    }
}
fn class_s(c: Classification) -> &'static str {
    match c {
        Classification::Public => "public",
        Classification::Internal => "internal",
        Classification::Confidential => "confidential",
        Classification::Restricted => "restricted",
    }
}
fn class_p(s: &str) -> Classification {
    match s {
        "public" => Classification::Public,
        "confidential" => Classification::Confidential,
        "restricted" => Classification::Restricted,
        _ => Classification::Internal,
    }
}
fn proc_s(k: ProceduralKind) -> &'static str {
    match k {
        ProceduralKind::Policy => "policy",
        ProceduralKind::Playbook => "playbook",
        ProceduralKind::Routine => "routine",
        ProceduralKind::Skill => "skill",
    }
}
fn proc_p(s: &str) -> ProceduralKind {
    match s {
        "policy" => ProceduralKind::Policy,
        "routine" => ProceduralKind::Routine,
        "skill" => ProceduralKind::Skill,
        _ => ProceduralKind::Playbook,
    }
}
fn evk_s(k: EventKind) -> &'static str {
    use EventKind::*;
    match k {
        AgentRegistered => "AgentRegistered",
        AgentApproved => "AgentApproved",
        AgentActivated => "AgentActivated",
        AgentSuspended => "AgentSuspended",
        AgentRetired => "AgentRetired",
        DecisionMade => "DecisionMade",
        ActionRequested => "ActionRequested",
        ActionAuthorized => "ActionAuthorized",
        ApprovalRequested => "ApprovalRequested",
        ActionExecuted => "ActionExecuted",
        OutcomeRecorded => "OutcomeRecorded",
        EvaluationRecorded => "EvaluationRecorded",
        TrustUpdated => "TrustUpdated",
    }
}
fn evk_p(s: &str) -> EventKind {
    use EventKind::*;
    match s {
        "AgentApproved" => AgentApproved,
        "AgentActivated" => AgentActivated,
        "AgentSuspended" => AgentSuspended,
        "AgentRetired" => AgentRetired,
        "DecisionMade" => DecisionMade,
        "ActionRequested" => ActionRequested,
        "ActionAuthorized" => ActionAuthorized,
        "ApprovalRequested" => ApprovalRequested,
        "ActionExecuted" => ActionExecuted,
        "OutcomeRecorded" => OutcomeRecorded,
        "EvaluationRecorded" => EvaluationRecorded,
        "TrustUpdated" => TrustUpdated,
        _ => AgentRegistered,
    }
}

// --- entity (de)serialization ----------------------------------------------
fn agent_to(a: &Agent) -> Value {
    json!({
        "id": a.identity.id, "name": a.identity.name, "version": a.identity.version,
        "kind": a.identity.kind, "owner": a.identity.owner, "tenant": a.identity.tenant,
        "lifecycle_state": life_s(a.lifecycle_state),
        "objective": { "primary": a.objective.primary, "success_criteria": a.objective.success_criteria, "constraints": a.objective.constraints },
        "capabilities": a.capabilities.iter().map(|c| json!({"id":c.id,"name":c.name,"risk_level":risk_s(c.risk_level)})).collect::<Vec<_>>(),
        "authority": { "allowed_tools": a.authority.allowed_tools, "denied_tools": a.authority.denied_tools, "escalation_requirements": a.authority.escalation_requirements },
        "deny_by_default": a.deny_by_default,
        "security": { "authentication_required": a.security.authentication_required, "authorization_required": a.security.authorization_required, "allowed_identity_providers": a.security.allowed_identity_providers },
        "trust": { "score": a.trust.score, "state": trust_s(a.trust.state) },
    })
}
fn agent_from(v: &Value) -> Agent {
    let obj = v.get("objective").cloned().unwrap_or(json!({}));
    let auth = v.get("authority").cloned().unwrap_or(json!({}));
    let sec = v.get("security").cloned().unwrap_or(json!({}));
    let tr = v.get("trust").cloned().unwrap_or(json!({}));
    Agent {
        identity: Identity {
            id: gs(v, "id"),
            name: gs(v, "name"),
            version: gs(v, "version"),
            kind: gs(v, "kind"),
            owner: gs(v, "owner"),
            tenant: gs(v, "tenant"),
        },
        lifecycle_state: life_p(&gs(v, "lifecycle_state")),
        objective: Objective {
            primary: gs(&obj, "primary"),
            success_criteria: ga(&obj, "success_criteria"),
            constraints: ga(&obj, "constraints"),
        },
        capabilities: v
            .get("capabilities")
            .and_then(|x| x.as_array())
            .map(|a| {
                a.iter()
                    .map(|c| Capability {
                        id: gs(c, "id"),
                        name: gs(c, "name"),
                        risk_level: risk_p(&gs(c, "risk_level")),
                    })
                    .collect()
            })
            .unwrap_or_default(),
        authority: Authority {
            allowed_tools: ga(&auth, "allowed_tools"),
            denied_tools: ga(&auth, "denied_tools"),
            escalation_requirements: ga(&auth, "escalation_requirements"),
        },
        deny_by_default: gb(v, "deny_by_default"),
        security: Security {
            authentication_required: gb(&sec, "authentication_required"),
            authorization_required: gb(&sec, "authorization_required"),
            allowed_identity_providers: ga(&sec, "allowed_identity_providers"),
            ..Default::default()
        },
        trust: Trust {
            score: gf(&tr, "score"),
            state: trust_p(&gs(&tr, "state")),
        },
    }
}
fn tool_to(t: &Tool) -> Value {
    json!({"id":t.id,"name":t.name,"target":t.target,"reversible":t.reversible,"approval_required":t.approval_required,"risk_level":risk_s(t.risk_level)})
}
fn tool_from(v: &Value) -> Tool {
    Tool {
        id: gs(v, "id"),
        name: gs(v, "name"),
        target: gs(v, "target"),
        reversible: gb(v, "reversible"),
        approval_required: gb(v, "approval_required"),
        risk_level: risk_p(&gs(v, "risk_level")),
    }
}
fn decision_to(d: &Decision) -> Value {
    json!({
        "id": d.id, "agent_id": d.agent_id, "timestamp": d.timestamp, "objective": d.objective,
        "context_refs": d.context_refs,
        "alternatives": d.alternatives.iter().map(|a| json!({"id":a.id,"description":a.description,"risk_level":risk_s(a.risk_level)})).collect::<Vec<_>>(),
        "selected": d.selected, "reasoning_summary": d.reasoning_summary, "confidence": d.confidence,
        "policy_result": { "decision": pol_s(d.policy_result.decision), "reason": d.policy_result.reason },
    })
}
fn decision_from(v: &Value) -> Decision {
    let pr = v.get("policy_result").cloned().unwrap_or(json!({}));
    Decision {
        id: gs(v, "id"),
        agent_id: gs(v, "agent_id"),
        timestamp: gu(v, "timestamp"),
        objective: gs(v, "objective"),
        context_refs: ga(v, "context_refs"),
        alternatives: v
            .get("alternatives")
            .and_then(|x| x.as_array())
            .map(|a| {
                a.iter()
                    .map(|e| DecisionAlternative {
                        id: gs(e, "id"),
                        description: gs(e, "description"),
                        risk_level: risk_p(&gs(e, "risk_level")),
                    })
                    .collect()
            })
            .unwrap_or_default(),
        selected: gs(v, "selected"),
        reasoning_summary: gs(v, "reasoning_summary"),
        confidence: gf(v, "confidence"),
        policy_result: PolicyResult {
            decision: pol_p(&gs(&pr, "decision")),
            reason: gs(&pr, "reason"),
        },
    }
}
fn action_to(a: &Action) -> Value {
    json!({"id":a.id,"decision_id":a.decision_id,"agent_id":a.agent_id,"tool_id":a.tool_id,"authorization":appr_s(a.authorization),"status":act_s(a.status),"reversible":a.reversible,"started_at":a.started_at})
}
fn action_from(v: &Value) -> Action {
    Action {
        id: gs(v, "id"),
        decision_id: gs(v, "decision_id"),
        agent_id: gs(v, "agent_id"),
        tool_id: gs(v, "tool_id"),
        authorization: appr_p(&gs(v, "authorization")),
        status: act_p(&gs(v, "status")),
        reversible: gb(v, "reversible"),
        started_at: gu(v, "started_at"),
    }
}
fn outcome_from(v: &Value) -> Outcome {
    Outcome {
        id: gs(v, "id"),
        action_id: gs(v, "action_id"),
        agent_id: gs(v, "agent_id"),
        success: gb(v, "success"),
        observed_result: gs(v, "observed_result"),
        observed_at: gu(v, "observed_at"),
    }
}
fn context_from(v: &Value) -> ContextItem {
    ContextItem {
        id: gs(v, "id"),
        agent_id: gs(v, "agent_id"),
        source: gs(v, "source"),
        content: gs(v, "content"),
        observed_at: gu(v, "observed_at"),
    }
}
fn semantic_from(v: &Value) -> SemanticRecord {
    SemanticRecord {
        id: gs(v, "id"),
        agent_id: gs(v, "agent_id"),
        content: gs(v, "content"),
        embedding: gfa(v, "embedding"),
        classification: class_p(&gs(v, "classification")),
        at: gu(v, "at"),
    }
}
fn procedural_from(v: &Value) -> ProceduralRecord {
    ProceduralRecord {
        id: gs(v, "id"),
        agent_id: gs(v, "agent_id"),
        kind: proc_p(&gs(v, "kind")),
        name: gs(v, "name"),
        body: gs(v, "body"),
    }
}
fn working_from(v: &Value) -> WorkingMemory {
    WorkingMemory {
        agent_id: gs(v, "agent_id"),
        task: gs(v, "task"),
        state: gs(v, "state"),
        expires_at: gu(v, "expires_at"),
    }
}
fn event_from(v: &Value) -> Event {
    Event {
        id: gs(v, "id"),
        kind: evk_p(&gs(v, "kind")),
        agent_id: gs(v, "agent_id"),
        subject: gs(v, "subject"),
        at: gu(v, "at"),
    }
}

impl Store for SurrealStore {
    fn put_agent(&mut self, a: Agent) {
        self.upsert("agent", &a.identity.id, agent_to(&a), 0);
    }
    fn get_agent(&self, id: &str) -> Option<Agent> {
        self.get_one("agent", id).map(|v| agent_from(&v))
    }
    fn put_tool(&mut self, t: Tool) {
        self.upsert("tool", &t.id, tool_to(&t), 0);
    }
    fn get_tool(&self, id: &str) -> Option<Tool> {
        self.get_one("tool", id).map(|v| tool_from(&v))
    }
    fn put_decision(&mut self, d: Decision) {
        self.upsert("decision", &d.id, decision_to(&d), 0);
    }
    fn get_decision(&self, id: &str) -> Option<Decision> {
        self.get_one("decision", id).map(|v| decision_from(&v))
    }
    fn put_action(&mut self, a: Action) {
        self.upsert("action", &a.id, action_to(&a), 0);
    }
    fn get_action(&self, id: &str) -> Option<Action> {
        self.get_one("action", id).map(|v| action_from(&v))
    }
    fn put_outcome(&mut self, o: Outcome) {
        let doc = json!({"id":o.id,"action_id":o.action_id,"agent_id":o.agent_id,"success":o.success,"observed_result":o.observed_result,"observed_at":o.observed_at});
        self.upsert("outcome", &o.id, doc, 0);
    }
    fn get_outcomes(&self, action_id: &str) -> Vec<Outcome> {
        self.select_where("outcome", "action_id", action_id, false)
            .iter()
            .map(outcome_from)
            .collect()
    }
    fn put_evaluation(&mut self, e: Evaluation) {
        let doc = json!({"id":e.id,"agent_id":e.agent_id,"trust_score":e.trust_score,"evaluated_at":e.evaluated_at});
        self.upsert("evaluation", &e.id, doc, 0);
    }
    fn put_context(&mut self, c: ContextItem) {
        let doc = json!({"id":c.id,"agent_id":c.agent_id,"source":c.source,"content":c.content,"observed_at":c.observed_at});
        self.upsert("context", &c.id, doc, c.observed_at as i64);
    }
    fn context_for(&self, agent_id: &str) -> Vec<ContextItem> {
        self.select_where("context", "agent_id", agent_id, true)
            .iter()
            .map(context_from)
            .collect()
    }
    fn put_semantic(&mut self, r: SemanticRecord) {
        let doc = json!({"id":r.id,"agent_id":r.agent_id,"content":r.content,"embedding":r.embedding,"classification":class_s(r.classification),"at":r.at});
        self.upsert("semantic", &r.id, doc, 0);
    }
    fn semantic_for(&self, agent_id: &str) -> Vec<SemanticRecord> {
        self.select_where("semantic", "agent_id", agent_id, false)
            .iter()
            .map(semantic_from)
            .collect()
    }
    fn put_procedural(&mut self, r: ProceduralRecord) {
        let doc = json!({"id":r.id,"agent_id":r.agent_id,"kind":proc_s(r.kind),"name":r.name,"body":r.body});
        self.upsert("procedural", &r.id, doc, 0);
    }
    fn procedural_for(&self, agent_id: &str) -> Vec<ProceduralRecord> {
        self.select_where("procedural", "agent_id", agent_id, false)
            .iter()
            .map(procedural_from)
            .collect()
    }
    fn set_working(&mut self, w: WorkingMemory) {
        let key = format!("{}|{}", w.agent_id, w.task);
        let doc =
            json!({"agent_id":w.agent_id,"task":w.task,"state":w.state,"expires_at":w.expires_at});
        self.upsert("working", &key, doc, 0);
    }
    fn get_working(&self, agent_id: &str, task: &str) -> Option<WorkingMemory> {
        self.get_one("working", &format!("{agent_id}|{task}"))
            .map(|v| working_from(&v))
    }
    fn append_event(&mut self, e: Event) {
        self.ord.set(self.ord.get() + 1);
        let ord = self.ord.get() as i64;
        let doc = json!({"id":e.id,"kind":evk_s(e.kind),"agent_id":e.agent_id,"subject":e.subject,"at":e.at});
        self.upsert("event", &e.id, doc, ord);
    }
    fn events(&self) -> Vec<Event> {
        self.select_all("event").iter().map(event_from).collect()
    }
}
