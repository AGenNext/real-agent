// Package agentmem is a reference Go implementation of the Real Agent memory
// store, backed by SurrealDB (see schemas/memory.surql).
//
// It satisfies the spec's memory and decision-record requirements:
//   - SPEC §4.1 Identity        -> UpsertAgent
//   - SPEC §4.4 / §6 Decision   -> RecordDecision (the accountable unit)
//   - SPEC §4.5 Action          -> RecordAction
//   - SPEC §4.6 Outcome         -> RecordOutcome
//   - SPEC §4.7 Memory          -> RememberFact (semantic) + the above (episodic)
//
// Reference only: the spec remains database-agnostic (SPEC §8).
package agentmem

import (
	"fmt"

	surrealdb "github.com/surrealdb/surrealdb.go"
	"github.com/surrealdb/surrealdb.go/pkg/models"
)

// Store is a SurrealDB-backed agent memory store.
type Store struct {
	db *surrealdb.DB
}

// Config holds the connection settings for a SurrealDB instance.
type Config struct {
	Endpoint  string // e.g. ws://localhost:8000/rpc
	Namespace string
	Database  string
	Username  string
	Password  string
}

// Open connects to SurrealDB, signs in as root, and selects the namespace and
// database, returning a ready-to-use Store.
func Open(cfg Config) (*Store, error) {
	db, err := surrealdb.New(cfg.Endpoint)
	if err != nil {
		return nil, fmt.Errorf("agentmem: connect: %w", err)
	}
	if _, err := db.SignIn(&surrealdb.Auth{
		Username: cfg.Username,
		Password: cfg.Password,
	}); err != nil {
		return nil, fmt.Errorf("agentmem: signin: %w", err)
	}
	if err := db.Use(cfg.Namespace, cfg.Database); err != nil {
		return nil, fmt.Errorf("agentmem: use ns/db: %w", err)
	}
	return &Store{db: db}, nil
}

// Close terminates the SurrealDB connection.
func (s *Store) Close() error { return s.db.Close() }

// nullStr maps an empty Go string to nil so option<string> fields stay unset.
func nullStr(s string) any {
	if s == "" {
		return nil
	}
	return s
}

// record is the minimal shape returned by Create/Upsert: just the record id.
type record struct {
	ID models.RecordID `json:"id"`
}

// Agent mirrors the agent table (SPEC §4.1 Identity).
type Agent struct {
	Name      string         `json:"name"`
	Version   string         `json:"version"`
	Owner     string         `json:"owner"`
	Tenant    string         `json:"tenant,omitempty"`
	Role      string         `json:"role,omitempty"`
	Authority map[string]any `json:"authority,omitempty"`
	Lifecycle string         `json:"lifecycle"`
}

// UpsertAgent inserts or updates an agent identity at the stable id agent:⟨id⟩.
func (s *Store) UpsertAgent(id string, a Agent) (models.RecordID, error) {
	if a.Lifecycle == "" {
		a.Lifecycle = "draft"
	}
	rid := models.NewRecordID("agent", id)
	res, err := surrealdb.Upsert[record](s.db, rid, a)
	if err != nil {
		return models.RecordID{}, fmt.Errorf("agentmem: upsert agent: %w", err)
	}
	return res.ID, nil
}

// Decision mirrors the decision table (SPEC §4.4 / §6).
type Decision struct {
	Agent        models.RecordID `json:"agent"`
	ObjectiveRef string          `json:"objective_ref,omitempty"`
	ContextRef   string          `json:"context_ref,omitempty"`
	Alternatives []any           `json:"alternatives"`
	Selected     map[string]any  `json:"selected"`
	Reasoning    string          `json:"reasoning,omitempty"`
	Confidence   *float64        `json:"confidence,omitempty"`
	PolicyResult string          `json:"policy_result"` // allow | deny | escalate | approve
}

// RecordDecision persists a decision record and returns its generated id.
func (s *Store) RecordDecision(d Decision) (models.RecordID, error) {
	if d.Alternatives == nil {
		d.Alternatives = []any{}
	}
	res, err := surrealdb.Create[record](s.db, models.Table("decision"), d)
	if err != nil {
		return models.RecordID{}, fmt.Errorf("agentmem: record decision: %w", err)
	}
	return res.ID, nil
}

// Action mirrors the action table (SPEC §4.5).
type Action struct {
	Decision      models.RecordID `json:"decision"`
	ActionType    string          `json:"action_type"`
	Target        string          `json:"target"`
	Params        map[string]any  `json:"params,omitempty"`
	Authorization string          `json:"authorization"`
	Status        string          `json:"status"` // pending | running | done | failed
	Reversible    bool            `json:"reversible"`
}

// RecordAction persists an action and returns its generated id.
func (s *Store) RecordAction(a Action) (models.RecordID, error) {
	if a.Status == "" {
		a.Status = "pending"
	}
	res, err := surrealdb.Create[record](s.db, models.Table("action"), a)
	if err != nil {
		return models.RecordID{}, fmt.Errorf("agentmem: record action: %w", err)
	}
	return res.ID, nil
}

// Outcome mirrors the outcome table (SPEC §4.6).
type Outcome struct {
	Action   models.RecordID `json:"action"`
	Result   map[string]any  `json:"result,omitempty"`
	Status   string          `json:"status"` // success | failure
	Error    string          `json:"error,omitempty"`
	Impact   string          `json:"impact,omitempty"`
	FollowUp string          `json:"follow_up,omitempty"`
}

// RecordOutcome persists the result of an action and returns its generated id.
func (s *Store) RecordOutcome(o Outcome) (models.RecordID, error) {
	res, err := surrealdb.Create[record](s.db, models.Table("outcome"), o)
	if err != nil {
		return models.RecordID{}, fmt.Errorf("agentmem: record outcome: %w", err)
	}
	return res.ID, nil
}

// RememberFact upserts a semantic memory fact for an agent (SPEC §4.7). The
// fact is keyed by [agent_id, key] so repeated writes update in place.
func (s *Store) RememberFact(agentID, key string, value any, source string) error {
	rid := models.NewRecordID("memory_fact", []any{agentID, key})
	_, err := surrealdb.Upsert[record](s.db, rid, map[string]any{
		"agent":  models.NewRecordID("agent", agentID),
		"key":    key,
		"value":  value,
		"source": source,
	})
	if err != nil {
		return fmt.Errorf("agentmem: remember fact: %w", err)
	}
	return nil
}

// Graph edge names connecting the agent loop (see schemas/memory.surql).
const (
	EdgeMade      = "made"      // agent    -> decision
	EdgeTriggered = "triggered" // decision -> action
	EdgeProduced  = "produced"  // action   -> outcome
	EdgeRecalls   = "recalls"   // agent    -> memory_fact
)

// Relate creates a graph edge `in -> relation -> out`, optionally carrying edge
// metadata (SPEC §4.3 traceability). Returns the edge's record id.
func (s *Store) Relate(in, out models.RecordID, relation string, data map[string]any) (models.RecordID, error) {
	rel := surrealdb.Relationship{
		In:       in,
		Out:      out,
		Relation: models.Table(relation),
		Data:     data,
	}
	if err := surrealdb.Relate(s.db, &rel); err != nil {
		return models.RecordID{}, fmt.Errorf("agentmem: relate %s: %w", relation, err)
	}
	if rel.ID != nil {
		return *rel.ID, nil
	}
	return models.RecordID{}, nil
}

// AgentTrace is a flattened view of an agent's decision graph, gathered by
// walking the made/triggered/produced edges.
type AgentTrace struct {
	Decisions []models.RecordID `json:"decisions"`
	Actions   []models.RecordID `json:"actions"`
	Outcomes  []models.RecordID `json:"outcomes"`
}

// TraceAgent walks the graph from an agent and returns every decision, action,
// and outcome reachable through the made/triggered/produced edges (SPEC §6).
func (s *Store) TraceAgent(agentID string) (*AgentTrace, error) {
	rid := models.NewRecordID("agent", agentID)
	res, err := surrealdb.Query[[]AgentTrace](s.db, `
		SELECT
			->made->decision AS decisions,
			->made->decision->triggered->action AS actions,
			->made->decision->triggered->action->produced->outcome AS outcomes
		FROM $agent`,
		map[string]any{"agent": rid})
	if err != nil {
		return nil, fmt.Errorf("agentmem: trace agent: %w", err)
	}
	if res == nil || len(*res) == 0 || len((*res)[0].Result) == 0 {
		return &AgentTrace{}, nil
	}
	trace := (*res)[0].Result[0]
	return &trace, nil
}

// ---------------------------------------------------------------------------
// Memory layers (SPEC §4.7): working, procedural, and knowledge (RAG).
// Episodic memory is the decision/action/outcome chain above; semantic
// key/value memory is RememberFact.
// ---------------------------------------------------------------------------

// SetWorkingMemory upserts transient task state, keyed by [agentID, task].
func (s *Store) SetWorkingMemory(agentID, task string, state map[string]any) error {
	rid := models.NewRecordID("working_memory", []any{agentID, task})
	_, err := surrealdb.Upsert[record](s.db, rid, map[string]any{
		"agent": models.NewRecordID("agent", agentID),
		"task":  task,
		"state": state,
	})
	if err != nil {
		return fmt.Errorf("agentmem: set working memory: %w", err)
	}
	return nil
}

// RememberProcedure stores a named, ordered playbook (procedural memory).
func (s *Store) RememberProcedure(agentID, name string, steps []string) (models.RecordID, error) {
	res, err := surrealdb.Create[record](s.db, models.Table("procedure"), map[string]any{
		"agent": models.NewRecordID("agent", agentID),
		"name":  name,
		"steps": steps,
	})
	if err != nil {
		return models.RecordID{}, fmt.Errorf("agentmem: remember procedure: %w", err)
	}
	return res.ID, nil
}

// Knowledge is a unit of retrievable semantic memory for RAG. Embedding is the
// vector produced by an embedding model; its length must match the knowledge
// index DIMENSION (see schemas/memory.surql).
type Knowledge struct {
	Content   string         `json:"content"`
	Embedding []float64      `json:"embedding"`
	Metadata  map[string]any `json:"metadata,omitempty"`
	Source    string         `json:"source,omitempty"`
}

// RememberKnowledge stores a knowledge chunk and its embedding for later
// retrieval (SPEC §4.7 semantic memory).
func (s *Store) RememberKnowledge(agentID string, k Knowledge) (models.RecordID, error) {
	res, err := surrealdb.Create[record](s.db, models.Table("knowledge"), map[string]any{
		"agent":     models.NewRecordID("agent", agentID),
		"content":   k.Content,
		"embedding": k.Embedding,
		"metadata":  k.Metadata,
		"source":    nullStr(k.Source),
	})
	if err != nil {
		return models.RecordID{}, fmt.Errorf("agentmem: remember knowledge: %w", err)
	}
	return res.ID, nil
}

// KnowledgeHit is one result of a similarity search, ordered nearest-first.
type KnowledgeHit struct {
	ID       models.RecordID `json:"id"`
	Content  string          `json:"content"`
	Distance float64         `json:"distance"`
}

// SearchKnowledge returns the k nearest knowledge chunks to the query embedding
// by cosine distance, using the HNSW vector index — the retrieval half of RAG.
func (s *Store) SearchKnowledge(query []float64, k int) ([]KnowledgeHit, error) {
	// k and the HNSW ef-search size are literals in the operator <|k,ef|>.
	sql := fmt.Sprintf(
		`SELECT id, content, vector::distance::knn() AS distance
		 FROM knowledge WHERE embedding <|%d,40|> $q
		 ORDER BY distance`, k)
	res, err := surrealdb.Query[[]KnowledgeHit](s.db, sql, map[string]any{"q": query})
	if err != nil {
		return nil, fmt.Errorf("agentmem: search knowledge: %w", err)
	}
	if res == nil || len(*res) == 0 {
		return nil, nil
	}
	return (*res)[0].Result, nil
}
