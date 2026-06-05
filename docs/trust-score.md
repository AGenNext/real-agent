# Trust score

Status: Informative · Reference: `reference/surrealdb` (`ComputeTrust`)

## Principle: calculated, never inferred

Trust in the Real Agent model is **always calculated from recorded evidence —
never asserted, guessed, or inferred.** Per the ontology:

```
Outcome  FEEDS     Evaluation
Evaluation UPDATES Trust
Trust    IS_INFORMED_BY Outcome
```

So a trust score is a *function of an agent's outcome history*, derived by
querying ("searching") the agent's graph — not a number a human or model types
in. A manually set value would be an opinion, not trust.

## Formula

For an agent with outcomes `o_1 … o_N`:

```
            Σ wᵢ + α
trust  =  ───────────────
           N + α + β
```

| Symbol | Meaning | Value |
|---|---|---|
| `wᵢ` | weight of outcome *i* | `success = 1.0`, `partial = 0.5`, `failure = 0.0` |
| `N` | number of outcomes considered | from the graph walk |
| `α` | prior pseudo-successes | `1` |
| `β` | prior pseudo-failures | `1` |

`α = β = 1` is **Laplace smoothing**: with no evidence the score is the neutral
prior `α/(α+β) = 0.5`, and small samples are pulled toward it so three lucky
successes don't read as certainty.

## How it's computed

`ComputeTrust` runs a single SurrealQL graph traversal from the agent to every
produced outcome and reads the statuses:

```surql
SELECT ->made->decision->triggered->action->produced->outcome.status
       AS statuses
FROM agent:⟨id⟩;
```

Then applies the formula. `UpdateTrustFromOutcomes(team, agent)` computes the
score and records it on the `team ->trusts-> agent` edge **together with the
source outcome ids** it was calculated from:

```
trusts { score: 0.6, sources: [outcome:…, outcome:…, outcome:…], context: "computed from 3 outcome(s)" }
```

Because the edge stores its own evidence, any recorded trust is **anchored to
its sources** — reproducible and auditable, never inferred.

## Worked examples

| Outcomes | Raw success rate | **Trust (smoothed)** |
|---|---|---|
| none | — | 0.500 (neutral prior) |
| 1 × success | 1.000 | 0.667 |
| 2 × success, 1 × failure | 0.667 | **0.600** |
| 9 × success, 1 × failure | 0.900 | 0.833 |
| 100 × success | 1.000 | 0.990 |

Note how the smoothed score only approaches 1.0 with sustained evidence — trust
is *earned*, and recalculated every time, never inferred.

## Tuning

- Raise `α`/`β` to demand more evidence before trust moves.
- Outcome weights could be extended (e.g. weight by impact or recency); any
  change keeps the core property: **the score is computed from outcomes.**
