# Real Agent Trust

Version: 0.1.0  
Status: Draft

## Purpose

Trust is the measurable confidence that an agent can operate safely, correctly, and accountably within a defined boundary.

Trust is not a feeling. Trust is an evaluated property.

## Trust principle

```text
Trust must be earned, measured, recorded, and continuously re-evaluated.
```

An agent MUST NOT be considered trustworthy merely because it uses a powerful model, has a well-written prompt, or has access to tools.

## Trust dimensions

A real agent SHOULD be evaluated across six trust dimensions.

### 1. Identity trust

Can the agent identity be verified?

Signals:

- stable identifier
- owner declared
- issuer declared
- lifecycle state known
- contract available
- authority boundary declared

### 2. Capability trust

Can the agent perform the capability it claims?

Signals:

- capability tests
- benchmark results
- tool success rate
- task completion rate
- regression results
- versioned skill records

### 3. Policy trust

Does the agent follow policy?

Signals:

- policy pass rate
- denied action attempts
- approval compliance
- escalation compliance
- policy breach count

### 4. Behavioral trust

Does the agent behave predictably within its operating context?

Signals:

- repeated failure rate
- abnormal action patterns
- drift from objective
- unauthorized tool attempt rate
- human override rate

### 5. Operational trust

Can the agent operate reliably in production?

Signals:

- uptime
- latency
- error rate
- retry rate
- cost variance
- incident count

### 6. Outcome trust

Do agent actions produce desired outcomes?

Signals:

- outcome success rate
- business impact
- rollback rate
- customer/user satisfaction
- negative impact count

## Trust score

A Real Agent trust score MAY be represented as a number between 0 and 1.

```text
0.0 = no trust
1.0 = maximum measured trust
```

A deployment SHOULD define thresholds for:

- activation
- autonomy
- approval requirement
- probation
- suspension
- retirement

## Trust states

| State | Description |
|---|---|
| Unknown | Not enough evidence exists. |
| Experimental | Agent is under limited test. |
| Trusted | Agent meets required trust threshold. |
| Conditional | Agent may operate with restrictions. |
| Probation | Agent has trust concerns but may continue under observation. |
| Suspended | Agent is blocked from acting. |
| Revoked | Agent has lost trust permanently or until re-certification. |

## Trust graph

```text
Agent
  HAS_TRUST_SCORE
  HAS_TRUST_STATE
  HAS_EVALUATION
  HAS_POLICY_RESULT
  HAS_DECISION_RECORD
  HAS_ACTION_RECORD
  HAS_OUTCOME_RECORD
```

## Trust requirements

A governed agent SHOULD:

1. expose a current trust score,
2. expose trust dimensions,
3. retain evidence behind the score,
4. update trust after meaningful outcomes,
5. lower autonomy when trust drops,
6. suspend when trust falls below safety threshold.

## Trust is contextual

Trust is scoped.

An agent may be trusted for one capability and not trusted for another.

Example:

```text
Trusted to summarize invoices.
Not trusted to approve payments.
```

## Minimum enterprise requirement

An enterprise real agent SHOULD NOT be activated without:

- identity trust
- policy trust
- capability trust
- operational monitoring
- suspension threshold
