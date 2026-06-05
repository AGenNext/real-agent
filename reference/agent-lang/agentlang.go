// Package agentlang parses the Real Agent language (.agent) — a small
// declarative DSL for defining an agent — and compiles it to a JSON document
// conforming to schemas/agent.schema.json.
//
// Grammar (informal):
//
//	agent <id> {
//	  name "..."  version <v>  type <t>  [description "..."]
//	  identity { owner <o> tenant <t> [issuer <i>] [subject <s>] lifecycle <state> }
//	  objective "<primary>" { [success "..."]* [constraint "..."]* }
//	  capability <id> { name "..." risk <low|medium|high|critical> [description "..."] }*
//	  action <id> { name "..." target "..." reversible <bool> approval <required|none> policy <required|none> }*
//	  policy { deny-by-default <bool> [ref "..."]* [engine "..."] }
//	  memory { enable <working|episodic|semantic|procedural>* retention "..." }
//	  evaluation { metric "..."* [test "..."] min-trust <0..1> }
//	}
package agentlang

import (
	"fmt"
	"strconv"
)

// Contract mirrors schemas/agent.schema.json.
type Contract struct {
	Agent        Agent        `json:"agent"`
	Identity     Identity     `json:"identity"`
	Objective    Objective    `json:"objective"`
	Capabilities []Capability `json:"capabilities"`
	Actions      []Action     `json:"actions"`
	Policy       Policy       `json:"policy"`
	Memory       Memory       `json:"memory"`
	Evaluation   Evaluation   `json:"evaluation"`
}

type Agent struct {
	ID          string `json:"id"`
	Name        string `json:"name"`
	Version     string `json:"version"`
	Type        string `json:"type"`
	Description string `json:"description,omitempty"`
}

type Identity struct {
	Owner          string `json:"owner"`
	Tenant         string `json:"tenant"`
	Issuer         string `json:"issuer,omitempty"`
	Subject        string `json:"subject,omitempty"`
	LifecycleState string `json:"lifecycle_state"`
}

type Objective struct {
	Primary         string   `json:"primary"`
	SuccessCriteria []string `json:"success_criteria,omitempty"`
	Constraints     []string `json:"constraints,omitempty"`
}

type Capability struct {
	ID           string `json:"id"`
	Name         string `json:"name"`
	Description  string `json:"description,omitempty"`
	InputSchema  string `json:"input_schema,omitempty"`
	OutputSchema string `json:"output_schema,omitempty"`
	RiskLevel    string `json:"risk_level"`
}

type Action struct {
	ID               string `json:"id"`
	Name             string `json:"name"`
	Target           string `json:"target"`
	Reversible       bool   `json:"reversible"`
	ApprovalRequired bool   `json:"approval_required"`
	PolicyRequired   bool   `json:"policy_required"`
}

type Policy struct {
	PolicyEngine  string   `json:"policy_engine,omitempty"`
	PolicyRefs    []string `json:"policy_refs,omitempty"`
	DenyByDefault bool     `json:"deny_by_default"`
}

type Memory struct {
	Working         bool   `json:"working"`
	Episodic        bool   `json:"episodic"`
	Semantic        bool   `json:"semantic"`
	Procedural      bool   `json:"procedural"`
	RetentionPolicy string `json:"retention_policy"`
}

type Evaluation struct {
	Metrics           []string `json:"metrics"`
	TestSuite         string   `json:"test_suite,omitempty"`
	MinimumTrustScore float64  `json:"minimum_trust_score"`
}

// --- tokenizer ---

func tokenize(src string) []string {
	var toks []string
	i, n := 0, len(src)
	isSpace := func(b byte) bool { return b == ' ' || b == '\t' || b == '\r' || b == '\n' }
	for i < n {
		switch ch := src[i]; {
		case isSpace(ch):
			i++
		case ch == '#':
			for i < n && src[i] != '\n' {
				i++
			}
		case ch == '{' || ch == '}':
			toks = append(toks, string(ch))
			i++
		case ch == '"':
			i++
			start := i
			for i < n && src[i] != '"' {
				i++
			}
			toks = append(toks, src[start:i])
			i++ // skip closing quote
		default:
			start := i
			for i < n && !isSpace(src[i]) && src[i] != '{' && src[i] != '}' && src[i] != '"' && src[i] != '#' {
				i++
			}
			toks = append(toks, src[start:i])
		}
	}
	return toks
}

// --- parser ---

type cur struct {
	t []string
	i int
}

func (c *cur) eof() bool { return c.i >= len(c.t) }
func (c *cur) peek() string {
	if c.eof() {
		return ""
	}
	return c.t[c.i]
}
func (c *cur) take() string { s := c.peek(); c.i++; return s }
func (c *cur) expect(tok string) error {
	if got := c.take(); got != tok {
		return fmt.Errorf("expected %q, got %q", tok, got)
	}
	return nil
}

func parseBool(s string) bool { return s == "true" || s == "yes" || s == "on" }
func required(s string) bool  { return s == "required" || s == "true" || s == "yes" }

// Parse compiles agent-language source into a Contract.
func Parse(src string) (*Contract, error) {
	c := &cur{t: tokenize(src)}
	out := &Contract{Capabilities: []Capability{}, Actions: []Action{}}
	out.Evaluation.Metrics = []string{}

	if err := c.expect("agent"); err != nil {
		return nil, err
	}
	out.Agent.ID = c.take()
	if err := c.expect("{"); err != nil {
		return nil, err
	}
	for !c.eof() && c.peek() != "}" {
		switch key := c.take(); key {
		case "name":
			out.Agent.Name = c.take()
		case "version":
			out.Agent.Version = c.take()
		case "type":
			out.Agent.Type = c.take()
		case "description":
			out.Agent.Description = c.take()
		case "identity":
			if err := c.block(func(k string) error {
				switch k {
				case "owner":
					out.Identity.Owner = c.take()
				case "tenant":
					out.Identity.Tenant = c.take()
				case "issuer":
					out.Identity.Issuer = c.take()
				case "subject":
					out.Identity.Subject = c.take()
				case "lifecycle":
					out.Identity.LifecycleState = c.take()
				default:
					return fmt.Errorf("identity: unknown key %q", k)
				}
				return nil
			}); err != nil {
				return nil, err
			}
		case "objective":
			out.Objective.Primary = c.take()
			if err := c.block(func(k string) error {
				switch k {
				case "success":
					out.Objective.SuccessCriteria = append(out.Objective.SuccessCriteria, c.take())
				case "constraint":
					out.Objective.Constraints = append(out.Objective.Constraints, c.take())
				default:
					return fmt.Errorf("objective: unknown key %q", k)
				}
				return nil
			}); err != nil {
				return nil, err
			}
		case "capability":
			cap := Capability{ID: c.take()}
			if err := c.block(func(k string) error {
				switch k {
				case "name":
					cap.Name = c.take()
				case "risk":
					cap.RiskLevel = c.take()
				case "description":
					cap.Description = c.take()
				case "input":
					cap.InputSchema = c.take()
				case "output":
					cap.OutputSchema = c.take()
				default:
					return fmt.Errorf("capability: unknown key %q", k)
				}
				return nil
			}); err != nil {
				return nil, err
			}
			out.Capabilities = append(out.Capabilities, cap)
		case "action":
			act := Action{ID: c.take()}
			if err := c.block(func(k string) error {
				switch k {
				case "name":
					act.Name = c.take()
				case "target":
					act.Target = c.take()
				case "reversible":
					act.Reversible = parseBool(c.take())
				case "approval":
					act.ApprovalRequired = required(c.take())
				case "policy":
					act.PolicyRequired = required(c.take())
				default:
					return fmt.Errorf("action: unknown key %q", k)
				}
				return nil
			}); err != nil {
				return nil, err
			}
			out.Actions = append(out.Actions, act)
		case "policy":
			if err := c.block(func(k string) error {
				switch k {
				case "deny-by-default":
					out.Policy.DenyByDefault = parseBool(c.take())
				case "ref":
					out.Policy.PolicyRefs = append(out.Policy.PolicyRefs, c.take())
				case "engine":
					out.Policy.PolicyEngine = c.take()
				default:
					return fmt.Errorf("policy: unknown key %q", k)
				}
				return nil
			}); err != nil {
				return nil, err
			}
		case "memory":
			if err := c.block(func(k string) error {
				switch k {
				case "enable":
					for {
						switch c.peek() {
						case "working":
							out.Memory.Working = true
						case "episodic":
							out.Memory.Episodic = true
						case "semantic":
							out.Memory.Semantic = true
						case "procedural":
							out.Memory.Procedural = true
						default:
							return nil
						}
						c.take()
					}
				case "retention":
					out.Memory.RetentionPolicy = c.take()
				default:
					return fmt.Errorf("memory: unknown key %q", k)
				}
				return nil
			}); err != nil {
				return nil, err
			}
		case "evaluation":
			if err := c.block(func(k string) error {
				switch k {
				case "metric":
					out.Evaluation.Metrics = append(out.Evaluation.Metrics, c.take())
				case "test":
					out.Evaluation.TestSuite = c.take()
				case "min-trust":
					f, err := strconv.ParseFloat(c.take(), 64)
					if err != nil {
						return fmt.Errorf("evaluation: min-trust: %w", err)
					}
					out.Evaluation.MinimumTrustScore = f
				default:
					return fmt.Errorf("evaluation: unknown key %q", k)
				}
				return nil
			}); err != nil {
				return nil, err
			}
		default:
			return nil, fmt.Errorf("unknown key %q", key)
		}
	}
	if err := c.expect("}"); err != nil {
		return nil, err
	}
	return out, nil
}

// block consumes `{ ... }`, calling fn for each key inside.
func (c *cur) block(fn func(key string) error) error {
	if err := c.expect("{"); err != nil {
		return err
	}
	for !c.eof() && c.peek() != "}" {
		if err := fn(c.take()); err != nil {
			return err
		}
	}
	return c.expect("}")
}
