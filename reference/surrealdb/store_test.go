package agentmem

import (
	"os"
	"strings"
	"testing"

	surrealdb "github.com/surrealdb/surrealdb.go"
)

// testStore opens a store against a live SurrealDB and loads a fresh schema.
// It skips (not fails) when SURREAL_ENDPOINT is unset, so `go test ./...` is
// green without a database and runs the full suite in CI where one is provided.
func testStore(t *testing.T) *Store {
	t.Helper()
	endpoint := os.Getenv("SURREAL_ENDPOINT")
	if endpoint == "" {
		t.Skip("SURREAL_ENDPOINT not set; skipping SurrealDB integration test")
	}
	s, err := Open(Config{
		Endpoint: endpoint, Namespace: "test_real_agent", Database: "test_memory",
		Username: "root", Password: "root",
	})
	if err != nil {
		t.Fatalf("open: %v", err)
	}
	// Fresh database, then apply the schema (minus its USE line — Open already
	// selected the namespace/database).
	_, _ = surrealdb.Query[any](s.db, "REMOVE DATABASE test_memory;", map[string]any{})
	raw, err := os.ReadFile("memory.surql")
	if err != nil {
		t.Fatalf("read schema: %v", err)
	}
	var b strings.Builder
	for _, line := range strings.Split(string(raw), "\n") {
		if strings.HasPrefix(strings.TrimSpace(line), "USE ") {
			continue
		}
		b.WriteString(line)
		b.WriteByte('\n')
	}
	if _, err := surrealdb.Query[any](s.db, b.String(), map[string]any{}); err != nil {
		t.Fatalf("apply schema: %v", err)
	}
	return s
}

func TestAgentLoopAndGraph(t *testing.T) {
	s := testStore(t)
	defer s.Close()

	a, err := s.UpsertAgent("a1", Agent{Name: "A", Version: "0.1.0", Owner: "o", Lifecycle: "active"})
	if err != nil {
		t.Fatal(err)
	}
	d, err := s.RecordDecision(Decision{Agent: a, Selected: map[string]any{"x": 1}, PolicyResult: "allow"})
	if err != nil {
		t.Fatal(err)
	}
	if _, err := s.Relate(a, d, EdgeMade, nil); err != nil {
		t.Fatal(err)
	}
	ac, err := s.RecordAction(Action{Decision: d, ActionType: "t", Target: "x", Authorization: "granted", Status: "done"})
	if err != nil {
		t.Fatal(err)
	}
	if _, err := s.Relate(d, ac, EdgeTriggered, nil); err != nil {
		t.Fatal(err)
	}
	o, err := s.RecordOutcome(Outcome{Action: ac, Status: "success"})
	if err != nil {
		t.Fatal(err)
	}
	if _, err := s.Relate(ac, o, EdgeProduced, nil); err != nil {
		t.Fatal(err)
	}
	trace, err := s.TraceAgent("a1")
	if err != nil {
		t.Fatal(err)
	}
	if len(trace.Decisions) != 1 || len(trace.Actions) != 1 || len(trace.Outcomes) != 1 {
		t.Errorf("trace = %d/%d/%d, want 1/1/1", len(trace.Decisions), len(trace.Actions), len(trace.Outcomes))
	}
}

func TestTrustIsCalculatedAndAnchored(t *testing.T) {
	s := testStore(t)
	defer s.Close()

	a, err := s.UpsertAgent("janitor", Agent{Name: "j", Version: "0.1.0", Owner: "o", Lifecycle: "active"})
	if err != nil {
		t.Fatal(err)
	}
	// 2 success + 1 failure
	for _, status := range []string{"success", "success", "failure"} {
		d, _ := s.RecordDecision(Decision{Agent: a, Selected: map[string]any{"x": 1}, PolicyResult: "allow"})
		s.Relate(a, d, EdgeMade, nil)
		ac, _ := s.RecordAction(Action{Decision: d, ActionType: "t", Target: "x", Authorization: "granted", Status: "done"})
		s.Relate(d, ac, EdgeTriggered, nil)
		o, _ := s.RecordOutcome(Outcome{Action: ac, Status: status})
		s.Relate(ac, o, EdgeProduced, nil)
	}
	score, sources, err := s.ComputeTrust("janitor")
	if err != nil {
		t.Fatal(err)
	}
	if len(sources) != 3 {
		t.Errorf("sources = %d, want 3", len(sources))
	}
	// Laplace-smoothed: (2 + 1) / (3 + 2) = 0.6
	if score < 0.59 || score > 0.61 {
		t.Errorf("trust = %.3f, want ~0.600", score)
	}
}

func TestRAGSearch(t *testing.T) {
	s := testStore(t)
	defer s.Close()

	if _, err := s.UpsertAgent("a1", Agent{Name: "A", Version: "0.1.0", Owner: "o", Lifecycle: "active"}); err != nil {
		t.Fatal(err)
	}
	emb := make([]float64, 1536)
	emb[0] = 1
	if _, err := s.RememberKnowledge("a1", Knowledge{Content: "hello", Embedding: emb}); err != nil {
		t.Fatal(err)
	}
	hits, err := s.SearchKnowledge(emb, 1)
	if err != nil {
		t.Fatal(err)
	}
	if len(hits) < 1 || hits[0].Content != "hello" {
		t.Errorf("RAG hits = %+v, want first content 'hello'", hits)
	}
}

func TestOrganisationBackedTeam(t *testing.T) {
	s := testStore(t)
	defer s.Close()

	if _, err := s.UpsertOrganization("org", "Org", true, ""); err != nil {
		t.Fatal(err)
	}
	if _, err := s.UpsertTeam("tm", "Team", "org", ""); err != nil {
		t.Fatal(err)
	}
	if _, err := s.BackTeam("org", "tm"); err != nil {
		t.Fatal(err)
	}
	if _, err := s.UpsertAgent("m1", Agent{Name: "m1", Version: "0.1.0", Owner: "org", Lifecycle: "active"}); err != nil {
		t.Fatal(err)
	}
	if _, err := s.AddTeamMember("tm", "m1"); err != nil {
		t.Fatal(err)
	}
	roster, err := s.TeamRoster("tm")
	if err != nil {
		t.Fatal(err)
	}
	if len(roster) != 1 {
		t.Errorf("roster = %d, want 1", len(roster))
	}
}
