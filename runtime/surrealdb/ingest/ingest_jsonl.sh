#!/usr/bin/env bash
# JSON Lines -> Real Agent runtime ingester
# =========================================
# Bulk-loads a .jsonl file (one JSON object per line) into a runtime table.
# Record-link fields (e.g. `agent`) arrive as strings like "agent:backup_steward"
# and SurrealDB 3.x does NOT coerce a string into a record<> field, so name them
# as link fields and they are converted with type::record().
#
# Usage:
#   ENDPOINT=http://localhost:8000 USER=root PASS=root NS=real_agent DB=v1 \
#     ./ingest_jsonl.sh <table> <file.jsonl> [linkField1,linkField2,...]
#
# Example:
#   ./ingest_jsonl.sh semantic_memory examples/semantic_memory.jsonl agent
#
# Requires the `surreal` CLI (3.x) on PATH.
set -euo pipefail

TABLE="${1:?usage: ingest_jsonl.sh <table> <file.jsonl> [linkFields]}"
FILE="${2:?usage: ingest_jsonl.sh <table> <file.jsonl> [linkFields]}"
LINKS="${3:-}"
ENDPOINT="${ENDPOINT:-http://localhost:8000}"
USER="${USER:-root}"
PASS="${PASS:-root}"
NS="${NS:-real_agent}"
DB="${DB:-v1}"
BATCH="${BATCH:-500}"

# Build the link-conversion merge: " + { agent: type::record($r.agent), ... }"
merge=""
if [ -n "$LINKS" ]; then
  parts=""
  IFS=',' read -ra fields <<< "$LINKS"
  for f in "${fields[@]}"; do parts="${parts:+$parts, }$f: type::record(\$r.$f)"; done
  merge=" + { $parts }"
fi

ingest_batch() {
  local rows="$1"
  [ -z "$rows" ] && return 0
  printf 'LET $rows = [ %s ]; FOR $r IN $rows { CREATE type::table("%s") CONTENT ($r%s); };' \
    "$rows" "$TABLE" "$merge" \
    | surreal sql --endpoint "$ENDPOINT" --user "$USER" --pass "$PASS" --ns "$NS" --db "$DB" >/dev/null
}

count=0; buf=""
while IFS= read -r line || [ -n "$line" ]; do
  [ -z "${line//[[:space:]]/}" ] && continue
  buf="${buf:+$buf,}$line"
  count=$((count + 1))
  if [ $((count % BATCH)) -eq 0 ]; then ingest_batch "$buf"; buf=""; fi
done < "$FILE"
ingest_batch "$buf"

echo "ingested $count record(s) into $TABLE"
