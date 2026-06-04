-- Real Agent runtime — namespace/database bootstrap
-- =================================================
-- SurrealDB is multi-tenant: a `namespace` isolates an organisation/team, and a
-- `database` holds the tables. This maps the spec's tenancy model — each agent
-- carries a `tenant`, and namespaces give hard isolation between tenants.
--
-- DEFINE NAMESPACE/DATABASE require ROOT-level auth. When you import the other
-- files with `--ns real_agent --db v1` the context already exists; run this
-- explicitly only when provisioning a fresh instance.

DEFINE NAMESPACE IF NOT EXISTS real_agent COMMENT "Real Agent runtime tenant root";
DEFINE DATABASE IF NOT EXISTS v1 COMMENT "Real Agent contract v1";
