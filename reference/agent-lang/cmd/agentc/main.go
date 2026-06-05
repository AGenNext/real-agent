// Command agentc compiles a Real Agent language file (.agent) into a JSON
// document conforming to schemas/agent.schema.json.
//
// Usage:
//
//	agentc path/to/agent.agent           # prints JSON to stdout
package main

import (
	"encoding/json"
	"fmt"
	"os"

	agentlang "github.com/AGenNext/real-agent/reference/agent-lang"
)

func main() {
	if len(os.Args) != 2 {
		fmt.Fprintln(os.Stderr, "usage: agentc <file.agent>")
		os.Exit(2)
	}
	src, err := os.ReadFile(os.Args[1])
	if err != nil {
		fmt.Fprintln(os.Stderr, err)
		os.Exit(1)
	}
	contract, err := agentlang.Parse(string(src))
	if err != nil {
		fmt.Fprintln(os.Stderr, "parse error:", err)
		os.Exit(1)
	}
	out, err := json.MarshalIndent(contract, "", "  ")
	if err != nil {
		fmt.Fprintln(os.Stderr, err)
		os.Exit(1)
	}
	fmt.Println(string(out))
}
