#!/usr/bin/env python3
"""Stream Kafka records into the Real Agent SurrealDB runtime.

A reference consumer: read JSON messages from a Kafka topic and write them into
a runtime table, converting record-link fields (e.g. ``agent``) to record IDs —
the streaming counterpart of ingest_jsonl.sh.

SurrealDB OSS has no built-in Kafka source, so ingestion runs in an external
consumer that reuses the same conversion logic and writes via the official SDK.

Run:
    pip install surrealdb confluent-kafka
    BOOTSTRAP=localhost:9092 TOPIC=agent.memory TABLE=semantic_memory \\
    LINKS=agent ENDPOINT=ws://localhost:8000/rpc python kafka_consumer.py

NOTE: not exercised in CI here (no broker available); the SurrealDB write path
is the same one verified by ingest_jsonl.sh.
"""
import json
import os

from confluent_kafka import Consumer  # type: ignore
from surrealdb import Surreal  # type: ignore


def main() -> None:
    table = os.environ.get("TABLE", "semantic_memory")
    links = [f for f in os.environ.get("LINKS", "").split(",") if f]
    endpoint = os.environ.get("ENDPOINT", "ws://localhost:8000/rpc")

    consumer = Consumer({
        "bootstrap.servers": os.environ.get("BOOTSTRAP", "localhost:9092"),
        "group.id": os.environ.get("GROUP", "real-agent-ingest"),
        "auto.offset.reset": "earliest",
    })
    consumer.subscribe([os.environ["TOPIC"]])

    # Build the link conversion once: ($r + { agent: type::record($r.agent) }).
    merge = ""
    if links:
        parts = ", ".join(f"{f}: type::record($r.{f})" for f in links)
        merge = f" + {{ {parts} }}"
    surql = f'CREATE type::table("{table}") CONTENT ($r{merge});'

    db = Surreal(endpoint)
    db.signin({"username": os.environ.get("USER", "root"),
               "password": os.environ.get("PASS", "root")})
    db.use(os.environ.get("NS", "real_agent"), os.environ.get("DB", "v1"))

    try:
        while True:
            msg = consumer.poll(1.0)
            if msg is None or msg.error():
                continue
            record = json.loads(msg.value())
            db.query(surql, {"r": record})  # same write path as ingest_jsonl.sh
            consumer.commit(msg)
    finally:
        consumer.close()
        db.close()


if __name__ == "__main__":
    main()
