#!/usr/bin/env bash
# Real Agent runtime — SurrealDB migration runner
# ===============================================
# Applies numbered *.surql migrations in this directory in order, exactly once
# each, tracking applied versions in a `_migration` table. Idempotent: already
# applied migrations are skipped, so it is safe to run on every deploy.
#
# Usage:
#   ENDPOINT=http://localhost:8000 USER=root PASS=root NS=real_agent DB=v1 \
#     ./migrate.sh
#
# Requires the `surreal` CLI (3.x) on PATH.
set -euo pipefail

ENDPOINT="${ENDPOINT:-http://localhost:8000}"
USER="${USER:-root}"
PASS="${PASS:-root}"
NS="${NS:-real_agent}"
DB="${DB:-v1}"
DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

command -v jq >/dev/null || { echo "error: jq is required" >&2; exit 2; }

run_sql() {
  surreal sql --endpoint "$ENDPOINT" --user "$USER" --pass "$PASS" --ns "$NS" --db "$DB" --json <<<"$1"
}

# Ensure the tracking table exists.
run_sql "DEFINE TABLE IF NOT EXISTS _migration SCHEMAFULL;
         DEFINE FIELD IF NOT EXISTS applied_at ON _migration TYPE datetime DEFAULT time::now() READONLY;" >/dev/null

# Applied versions as a JSON array of record-id strings (exact membership test
# below — no fragile substring matching).
applied="$(run_sql 'SELECT VALUE id FROM _migration;')"

for f in "$DIR"/[0-9]*.surql; do
  [ -e "$f" ] || continue
  version="$(basename "$f" .surql)"
  if jq -e --arg id "_migration:$version" 'flatten | index($id) != null' >/dev/null 2>&1 <<<"$applied"; then
    echo "skip   $version (already applied)"
    continue
  fi
  echo "apply  $version"
  surreal import --endpoint "$ENDPOINT" --user "$USER" --pass "$PASS" --ns "$NS" --db "$DB" "$f" >/dev/null
  run_sql "CREATE _migration:\`${version}\`;" >/dev/null
done

echo "migrations up to date"
