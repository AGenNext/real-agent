package agentlang

import (
	"encoding/json"
	"os"
	"testing"
)

func TestParseExample(t *testing.T) {
	src, err := os.ReadFile("examples/cluster-janitor.agent")
	if err != nil {
		t.Fatal(err)
	}
	c, err := Parse(string(src))
	if err != nil {
		t.Fatalf("parse: %v", err)
	}
	if c.Agent.ID != "agent-demo-001" {
		t.Errorf("id = %q, want agent-demo-001", c.Agent.ID)
	}
	if c.Agent.Name != "Cluster Janitor" {
		t.Errorf("name = %q", c.Agent.Name)
	}
	if c.Agent.Version != "0.1.0" {
		t.Errorf("version = %q", c.Agent.Version)
	}
	if len(c.Capabilities) != 2 {
		t.Errorf("capabilities = %d, want 2", len(c.Capabilities))
	}
	if len(c.Actions) != 1 {
		t.Errorf("actions = %d, want 1", len(c.Actions))
	}
	if !c.Actions[0].ApprovalRequired || !c.Actions[0].PolicyRequired {
		t.Errorf("action approval/policy flags not set: %+v", c.Actions[0])
	}
	if !c.Memory.Working || !c.Memory.Episodic || !c.Memory.Semantic || !c.Memory.Procedural {
		t.Errorf("memory flags not all set: %+v", c.Memory)
	}
	if c.Evaluation.MinimumTrustScore != 0.8 {
		t.Errorf("min-trust = %v, want 0.8", c.Evaluation.MinimumTrustScore)
	}
}

func TestCompiledJSONHasRequiredFields(t *testing.T) {
	src, err := os.ReadFile("examples/cluster-janitor.agent")
	if err != nil {
		t.Fatal(err)
	}
	c, err := Parse(string(src))
	if err != nil {
		t.Fatal(err)
	}
	b, err := json.Marshal(c)
	if err != nil {
		t.Fatal(err)
	}
	var m map[string]any
	if err := json.Unmarshal(b, &m); err != nil {
		t.Fatal(err)
	}
	for _, k := range []string{"agent", "identity", "objective", "capabilities", "actions", "policy", "memory", "evaluation"} {
		if _, ok := m[k]; !ok {
			t.Errorf("compiled JSON missing required top-level key %q", k)
		}
	}
}

func TestParseRejectsBadInput(t *testing.T) {
	if _, err := Parse("notagent { }"); err == nil {
		t.Error("expected an error for input not starting with 'agent'")
	}
}
