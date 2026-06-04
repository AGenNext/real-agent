# Ingestion

Load data into the Real Agent runtime from files and streams.

The key constraint: SurrealDB 3.x does **not** coerce a JSON string like
`"agent:backup_steward"` into a `record<agent>` field. So any link field must be
converted with `type::record(...)`. Both ingesters below do this for the link
fields you name.

**Security:** record data is passed to SurrealDB as **bound parameters** (a
`$rows` array variable over the RPC endpoint), never concatenated into the query
string. Hostile content in the data cannot alter the statement — verified by
ingesting a record whose value contained `"]; REMOVE TABLE …` and confirming the
table was untouched and the string stored verbatim. Only the table and link-field
names (operator-supplied) appear in the query text, and both are validated as
identifiers.

## JSON Lines — `ingest_jsonl.sh` (verified on 3.0.1)

One JSON object per line, batched and posted as bound `$rows` parameters.

```sh
ENDPOINT=http://localhost:8000 USER=root PASS=root NS=real_agent DB=v1 \
  ./ingest_jsonl.sh semantic_memory examples/semantic_memory.jsonl agent
# -> ingested 3 record(s) into semantic_memory
```

- Arg 1: target table. Arg 2: `.jsonl` file. Arg 3: comma-separated link fields.
- `BATCH` (default 500) controls records per statement.

## Kafka — `kafka_consumer.py` (reference pattern)

SurrealDB OSS has no native Kafka source, so streaming ingestion runs as an
external consumer that reuses the same conversion and write path:

```sh
pip install surrealdb confluent-kafka
BOOTSTRAP=localhost:9092 TOPIC=agent.memory TABLE=semantic_memory LINKS=agent \
  ENDPOINT=ws://localhost:8000/rpc python kafka_consumer.py
```

Each message is a JSON object; link fields are converted via `type::record()`
exactly as in the JSONL path. Not exercised here (no broker in the sandbox) — the
SurrealDB write is the same statement verified by `ingest_jsonl.sh`.
