// Command demo runs one full pass of the Real Agent loop against SurrealDB:
// Decide -> Act -> record Outcome -> update Memory (SPEC §6 agent loop).
//
// Usage:
//
//	export SURREAL_ENDPOINT="ws://localhost:8000/rpc"
//	export SURREAL_USER="root"
//	export SURREAL_PASS="root"
//	go run ./cmd/demo
//
// Inside the k3s cluster the endpoint is
// ws://surrealdb.database.svc.cluster.local:8000/rpc. To reach it from your
// workstation, port-forward first:
//
//	kubectl -n database port-forward svc/surrealdb 8000:8000
package main

import (
	"fmt"
	"log"
	"os"

	agentmem "github.com/AGenNext/real-agent/examples/go"
)

func env(key, fallback string) string {
	if v := os.Getenv(key); v != "" {
		return v
	}
	return fallback
}

func main() {
	store, err := agentmem.Open(agentmem.Config{
		Endpoint:  env("SURREAL_ENDPOINT", "ws://localhost:8000/rpc"),
		Namespace: env("SURREAL_NS", "real_agent"),
		Database:  env("SURREAL_DB", "memory"),
		Username:  env("SURREAL_USER", "root"),
		Password:  env("SURREAL_PASS", "root"),
	})
	if err != nil {
		log.Fatal(err)
	}
	defer store.Close()

	const agentID = "agent-demo-001"
	agentRID, err := store.UpsertAgent(agentID, agentmem.Agent{
		Name:      "Cluster Janitor",
		Version:   "0.1.0",
		Owner:     "thefractionalpm",
		Role:      "maintenance",
		Lifecycle: "active",
	})
	if err != nil {
		log.Fatal(err)
	}

	confidence := 0.82
	decisionRID, err := store.RecordDecision(agentmem.Decision{
		Agent:        agentRID,
		ObjectiveRef: "obj-keep-cluster-clean",
		Alternatives: []any{"delete completed pods", "leave as-is"},
		Selected:     map[string]any{"option": "delete completed pods"},
		Reasoning:    "Completed helm-install jobs hold no resources but clutter pod listings",
		Confidence:   &confidence,
		PolicyResult: "allow",
	})
	if err != nil {
		log.Fatal(err)
	}

	actionRID, err := store.RecordAction(agentmem.Action{
		Decision:      decisionRID,
		ActionType:    "kubectl.delete",
		Target:        "kube-system/pods?phase=Succeeded",
		Params:        map[string]any{"fieldSelector": "status.phase=Succeeded"},
		Authorization: "granted",
		Status:        "done",
		Reversible:    false,
	})
	if err != nil {
		log.Fatal(err)
	}

	outcomeRID, err := store.RecordOutcome(agentmem.Outcome{
		Action: actionRID,
		Status: "success",
		Result: map[string]any{"deleted": 2},
		Impact: "2 completed pods removed from kube-system",
	})
	if err != nil {
		log.Fatal(err)
	}

	if err := store.RememberFact(agentID, "cluster.ingress",
		"traefik", "kubectl get pods -A"); err != nil {
		log.Fatal(err)
	}

	// Wire the graph edges so the loop is traversable end to end.
	if _, err := store.Relate(agentRID, decisionRID, agentmem.EdgeMade, nil); err != nil {
		log.Fatal(err)
	}
	if _, err := store.Relate(decisionRID, actionRID, agentmem.EdgeTriggered, nil); err != nil {
		log.Fatal(err)
	}
	if _, err := store.Relate(actionRID, outcomeRID, agentmem.EdgeProduced, nil); err != nil {
		log.Fatal(err)
	}

	fmt.Printf("decision %s -> action %s -> outcome %s recorded; memory updated\n",
		decisionRID, actionRID, outcomeRID)

	// Walk the graph back from the agent to verify the chain.
	trace, err := store.TraceAgent(agentID)
	if err != nil {
		log.Fatal(err)
	}
	fmt.Printf("graph trace for %s: %d decision(s), %d action(s), %d outcome(s)\n",
		agentID, len(trace.Decisions), len(trace.Actions), len(trace.Outcomes))
}
