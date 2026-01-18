#!/usr/bin/env bash
set -euo pipefail

if ! command -v sqlite3 >/dev/null 2>&1; then
  echo "sqlite3 not found. Please install sqlite3 first." >&2
  exit 1
fi

DB_PATH="${DB_PATH:-data/prompt_faster.db}"

if [[ ! -f "$DB_PATH" ]]; then
  echo "DB file not found: $DB_PATH" >&2
  echo "Set DB_PATH=/path/to/dbfile" >&2
  exit 1
fi

journal_mode=$(sqlite3 "$DB_PATH" "PRAGMA journal_mode;")
journal_mode=$(echo "$journal_mode" | tr '[:upper:]' '[:lower:]')

synchronous=$(sqlite3 "$DB_PATH" "PRAGMA synchronous;")
synchronous=$(echo "$synchronous" | tr '[:upper:]' '[:lower:]')

if [[ "$journal_mode" != "wal" ]]; then
  echo "journal_mode is not WAL: $journal_mode" >&2
  exit 1
fi

if [[ "$synchronous" != "full" && "$synchronous" != "2" ]]; then
  echo "synchronous is not FULL (2): $synchronous" >&2
  exit 1
fi

has_table=$(sqlite3 "$DB_PATH" "SELECT name FROM sqlite_master WHERE type='table' AND name='checkpoints';")
if [[ "$has_table" != "checkpoints" ]]; then
  echo "checkpoints table not found" >&2
  exit 1
fi

idx_task=$(sqlite3 "$DB_PATH" "SELECT name FROM sqlite_master WHERE type='index' AND name='idx_checkpoints_task_id';")
idx_created=$(sqlite3 "$DB_PATH" "SELECT name FROM sqlite_master WHERE type='index' AND name='idx_checkpoints_created_at';")

if [[ "$idx_task" != "idx_checkpoints_task_id" ]]; then
  echo "index idx_checkpoints_task_id not found" >&2
  exit 1
fi

if [[ "$idx_created" != "idx_checkpoints_created_at" ]]; then
  echo "index idx_checkpoints_created_at not found" >&2
  exit 1
fi

echo "OK: WAL + FULL synchronous and checkpoints schema/indexes verified."
