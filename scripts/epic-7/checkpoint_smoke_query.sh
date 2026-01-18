#!/usr/bin/env bash
set -euo pipefail

if ! command -v sqlite3 >/dev/null 2>&1; then
  echo "sqlite3 not found. Please install sqlite3 first." >&2
  exit 1
fi

DB_PATH="${DB_PATH:-data/prompt_faster.db}"
TASK_ID="${TASK_ID:-}"
LIMIT="${LIMIT:-10}"

if [[ -z "$TASK_ID" ]]; then
  echo "TASK_ID is required" >&2
  exit 1
fi

if [[ ! -f "$DB_PATH" ]]; then
  echo "DB file not found: $DB_PATH" >&2
  echo "Set DB_PATH=/path/to/dbfile" >&2
  exit 1
fi

echo "Latest checkpoints for task_id=$TASK_ID (limit=$LIMIT)"
sqlite3 -header -column "$DB_PATH" "SELECT id, iteration, created_at, checksum FROM checkpoints WHERE task_id='$TASK_ID' ORDER BY created_at DESC LIMIT $LIMIT;"

missing_checksum=$(sqlite3 "$DB_PATH" "SELECT COUNT(*) FROM checkpoints WHERE task_id='$TASK_ID' AND (checksum IS NULL OR checksum='');")
if [[ "$missing_checksum" != "0" ]]; then
  echo "Found checkpoints with missing checksum: $missing_checksum" >&2
  exit 1
fi

count=$(sqlite3 "$DB_PATH" "SELECT COUNT(*) FROM checkpoints WHERE task_id='$TASK_ID';")
if [[ "$count" == "0" ]]; then
  echo "No checkpoints found for task_id=$TASK_ID" >&2
  exit 1
fi

echo "OK: checkpoint query returned $count rows with checksum present."
