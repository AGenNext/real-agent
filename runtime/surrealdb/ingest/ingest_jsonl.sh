#!/usr/bin/env bash
# JSON Lines -> Real Agent runtime ingester (injection-safe)
# =========================================================
# Bulk-loads a .jsonl file (one JSON object per line) into a runtime table.
#
# SECURITY: record data is sent to SurrealDB's RPC endpoint as BOUND PARAMETERS
# (a `$rows` array variable), never concatenated into the query string. Hostile
# content in the data (quotes, `; DROP …`, etc.) is stored verbatim and cannot
# alter the statement. The only operator-supplied tokens placed in the query are
# the table name and link-field names, which are validated as identifiers below.
#
# Record-link fields (e.g. `agent`) arrive as strings like "agent:backup_steward"
# and SurrealDB 3.x does not coerce strings into record<> fields, so name them as
# link fields and they are converted with type::record().
#
# Usage:
#   ENDPOINT=http://localhost:8000 USER=root PASS=root NS=real_agent DB=v1 \
#     ./ingest_jsonl.sh <table> <file.jsonl> [linkField1,linkField2,...]
#
# Requires: curl, jq.
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

command -v jq >/dev/null   || { echo "error: jq is required" >&2; exit 2; }
command -v curl >/dev/null || { echo "error: curl is required" >&2; exit 2; }

is_ident() { [[ "$1" =~ ^[A-Za-z_][A-Za-z0-9_]*$ ]]; }
is_ident "$TABLE" || { echo "error: invalid table name '$TABLE'" >&2; exit 2; }

# Build the (static) link-conversion clause from validated field names.
merge=""
if [ -n "$LINKS" ]; then
  parts=""
  IFS=',' read -ra fields <<< "$LINKS"
  for f in "${fields[@]}"; do
    is_ident "$f" || { echo "error: invalid link field '$f'" >&2; exit 2; }
    parts="${parts:+$parts, }$f: type::record(\$r.$f)"
  done
  merge=" + { $parts }"
fi
SQL="FOR \$r IN \$rows { CREATE type::table(\$tb) CONTENT (\$r${merge}); };"

post_batch() { # $1 = JSON array of records
  local rows="$1" resp status
  resp="$(jq -cn --arg sql "$SQL" --arg tb "$TABLE" --argjson rows "$rows" \
    '{method:"query",params:[$sql,{tb:$tb,rows:$rows}]}' \
    | curl -fsS -X POST "$ENDPOINT/rpc" -u "$USER:$PASS" \
        -H "surreal-ns: $NS" -H "surreal-db: $DB" \
        -H 'Accept: application/json' -H 'Content-Type: application/json' --data-binary @-)"
  status="$(jq -r '.result[0].status // .error.message // "ERR"' <<<"$resp")"
  if [ "$status" != "OK" ]; then
    echo "error: batch failed: $resp" >&2; exit 1
  fi
}

# Slurp valid JSON lines into an array and post in batches of $BATCH.
total="$(jq -c 'select(length>0)' "$FILE" | jq -s '.' | jq length)"
jq -c 'select(length>0)' "$FILE" | jq -s -c "_nwise($BATCH)" | while IFS= read -r batch; do
  post_batch "$batch"
done

echo "ingested $total record(s) into $TABLE"
