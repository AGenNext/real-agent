-- Real Agent memory on SurrealDB
-- ==============================
-- Realizes the spec's Memory primitive (proto AgentMemory: working, episodic,
-- semantic, procedural) using SurrealDB's multi-model features in one engine:
--
--   memory type   SurrealDB feature
--   -----------   -----------------
--   semantic      vector index (HNSW) + full-text search (BM25)
--   episodic      change feed (time-travel over decisions/actions/outcomes)
--   procedural    schemafull policy/playbook records
--   working       ephemeral per-agent task state
--
-- Memory MUST be governed by retention/classification/access (spec 4.7), so
-- every row carries an owning agent, a classification, and a retention horizon.
-- Apply after schema.surql.

-- ---------------------------------------------------------------------------
-- Semantic memory: facts/knowledge, retrievable by meaning and by text
-- ---------------------------------------------------------------------------
DEFINE ANALYZER doc TOKENIZERS blank,class FILTERS lowercase,ascii;

DEFINE TABLE semantic_memory SCHEMAFULL
  PERMISSIONS FOR select FULL FOR create, update, delete WHERE $auth.id != NONE;
DEFINE FIELD agent          ON semantic_memory TYPE record<agent>;
DEFINE FIELD content        ON semantic_memory TYPE string;
DEFINE FIELD embedding      ON semantic_memory TYPE option<array<float>>;
DEFINE FIELD classification ON semantic_memory TYPE string DEFAULT 'internal'
  ASSERT $value IN ['public','internal','confidential','restricted'];
DEFINE FIELD source         ON semantic_memory TYPE option<string>;
DEFINE FIELD retain_until    ON semantic_memory TYPE option<datetime>;
DEFINE FIELD created_at      ON semantic_memory TYPE datetime DEFAULT time::now() READONLY;

-- Vector similarity (semantic recall) via an HNSW index. Dimension matches a
-- 384-d embedder; adjust DIMENSION to your model.
DEFINE INDEX semantic_vec ON semantic_memory FIELDS embedding HNSW DIMENSION 384 DIST COSINE;
-- Full-text recall over the same content.
DEFINE INDEX semantic_text ON semantic_memory FIELDS content FULLTEXT ANALYZER doc BM25 HIGHLIGHTS;

-- ---------------------------------------------------------------------------
-- Procedural memory: policies, playbooks, routines, skills
-- ---------------------------------------------------------------------------
DEFINE TABLE procedural_memory SCHEMAFULL;
DEFINE FIELD agent       ON procedural_memory TYPE record<agent>;
DEFINE FIELD kind        ON procedural_memory TYPE string
  ASSERT $value IN ['policy','playbook','routine','skill'];
DEFINE FIELD name        ON procedural_memory TYPE string;
DEFINE FIELD definition  ON procedural_memory TYPE object FLEXIBLE;
DEFINE FIELD version     ON procedural_memory TYPE string DEFAULT '0.1.0';
DEFINE INDEX procedural_name ON procedural_memory FIELDS agent, name, version UNIQUE;

-- ---------------------------------------------------------------------------
-- Working memory: ephemeral current-task state (one row per agent task)
-- ---------------------------------------------------------------------------
DEFINE TABLE working_memory SCHEMAFULL;
DEFINE FIELD agent      ON working_memory TYPE record<agent>;
DEFINE FIELD task       ON working_memory TYPE string;
DEFINE FIELD state      ON working_memory TYPE object FLEXIBLE;
DEFINE FIELD expires_at ON working_memory TYPE datetime DEFAULT time::now() + 1h;
DEFINE FIELD updated_at ON working_memory TYPE datetime DEFAULT time::now() VALUE time::now();

-- ---------------------------------------------------------------------------
-- Episodic memory: change feeds give time-travel over the experience record.
-- A change feed retains a window of mutations so memory can be replayed
-- (supports the spec's derived-state / append-only evidence principle).
-- ---------------------------------------------------------------------------
DEFINE TABLE episodic_memory SCHEMAFULL CHANGEFEED 30d;
DEFINE FIELD agent    ON episodic_memory TYPE record<agent>;
DEFINE FIELD decision ON episodic_memory TYPE option<record<decision>>;
DEFINE FIELD action   ON episodic_memory TYPE option<record<action>>;
DEFINE FIELD outcome  ON episodic_memory TYPE option<record<outcome>>;
DEFINE FIELD summary  ON episodic_memory TYPE string;
DEFINE FIELD at       ON episodic_memory TYPE datetime DEFAULT time::now() READONLY;

-- ---------------------------------------------------------------------------
-- Live monitoring (run interactively against a connection):
--   LIVE SELECT * FROM event;                     -- audit stream
--   LIVE SELECT * FROM evaluation WHERE trust_score < 0.6;  -- trust alerts
-- Time-travel over episodic memory:
--   SELECT * FROM episodic_memory VERSION d"2026-01-01T00:00:00Z";
-- Semantic recall:
--   SELECT content, vector::similarity::cosine(embedding, $q) AS score
--     FROM semantic_memory WHERE embedding <|5,COSINE|> $q;
-- Hybrid recall (RAG) — fuse vector + full-text with reciprocal rank fusion
-- (verified on 3.0.1):
--   LET $vs = (SELECT id FROM semantic_memory WHERE embedding <|5|> $q);
--   LET $ft = (SELECT id, search::score(1) AS score
--                FROM semantic_memory WHERE content @1@ $text ORDER BY score DESC LIMIT 5);
--   search::rrf([$vs, $ft], 5, 60);
-- ---------------------------------------------------------------------------
