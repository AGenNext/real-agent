# Real Agent graph map

Status: Informative

Visual companion to [ONTOLOGY.md](../ONTOLOGY.md), the record
[`schemas/`](../schemas), and [schema-org-mapping.md](./schema-org-mapping.md).
Diagrams render inline on GitHub.

## The agent loop

The causal chain the spec requires — every pass produces a decision record
(SPEC §6) — modelled as the SurrealDB graph edges in
[`reference/surrealdb/memory.surql`](../reference/surrealdb/memory.surql).

```mermaid
graph LR
  A([Agent]):::id
  D[Decision]:::core
  Act[Action]:::core
  O[Outcome]:::core
  M[(Memory fact)]:::mem

  A -->|made| D
  D -->|triggered| Act
  Act -->|produced| O
  A -.->|recalls| M
  O -.->|updates| M

  classDef id fill:#dbeafe,stroke:#2563eb;
  classDef core fill:#dcfce7,stroke:#16a34a;
  classDef mem fill:#fef9c3,stroke:#ca8a04;
```

## Governed extension

The enterprise graph adds the accountability primitives (SPEC §4.8,
ONTOLOGY "Enterprise Graph Extension"). schema.org has **no** equivalent for
these — they stay in the native contract.

```mermaid
graph TD
  D[Decision] -->|has_policy_result| PR[PolicyResult]
  D -->|may_require| Ap[ApprovalResult]
  Act[Action] -->|has_authorization| Az[AuthorizationResult]
  O[Outcome] -->|feeds| Ev[Evaluation]
  Ev -->|updates| Tr[Trust]
  Tr -->|modifies| Au[Authority]
  Pol[Policy] -->|constrains| D
  Pol -->|authorizes_or_denies| Act
  Gov[Governance] -->|controls| LS[LifecycleState]

  classDef gov fill:#fee2e2,stroke:#dc2626;
  class PR,Ap,Az,Pol,Gov,Tr,Au,LS,Ev gov;
```

## schema.org overlay

Where the loop maps onto [schema.org](https://schema.org) (export/interop
view only — see [schema-org-mapping.md](./schema-org-mapping.md)).

```mermaid
graph LR
  subgraph Real Agent
    D[Decision]
    Act[Action]
    O[Outcome]
  end
  subgraph schema.org
    CA[ChooseAction]
    SA[Action]
    RES["Action.result + actionStatus"]
  end
  D -.->|maps to| CA
  Act -.->|maps to| SA
  O -.->|maps to| RES
```
