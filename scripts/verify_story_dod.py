#!/usr/bin/env python3
from __future__ import annotations

import re
import sys
from pathlib import Path

import yaml


PROJECT_ROOT = Path(__file__).resolve().parents[1]
SPRINT_STATUS = PROJECT_ROOT / "docs/implementation-artifacts/sprint-status.yaml"
STORIES_DIR = PROJECT_ROOT / "docs/implementation-artifacts"


def _read_text(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def _extract_done_story_keys(sprint_status_text: str) -> list[str]:
    data = yaml.safe_load(sprint_status_text)
    development_status = data.get("development_status", {})
    done_story_keys: list[str] = []
    for key, value in development_status.items():
        if value != "done":
            continue
        if re.match(r"^\d+-\d+-", str(key)) is None:
            continue
        done_story_keys.append(str(key))
    return sorted(done_story_keys)


def _section_exists(text: str, heading_pattern: str) -> bool:
    return re.search(heading_pattern, text, re.M) is not None


def _agent_model_used_value(text: str) -> str | None:
    match = re.search(r"^###\s+Agent Model Used\s*\n\s*\n?([^\n]+)", text, re.M)
    if not match:
        return None
    return match.group(1).strip()


def main() -> int:
    sprint_status_text = _read_text(SPRINT_STATUS)
    done_story_keys = _extract_done_story_keys(sprint_status_text)

    errors: list[str] = []
    for key in done_story_keys:
        story_path = STORIES_DIR / f"{key}.md"
        if not story_path.exists():
            errors.append(f"[{key}] missing story file: {story_path}")
            continue

        text = _read_text(story_path)

        if not _section_exists(text, r"^##\s+Dev Agent Record\b"):
            errors.append(f"[{key}] missing section: ## Dev Agent Record")

        model_used = _agent_model_used_value(text)
        if model_used is None:
            errors.append(f"[{key}] missing section: ### Agent Model Used")
        elif model_used == "":
            errors.append(f"[{key}] empty Agent Model Used value")

        if not _section_exists(text, r"^##\s+Review Notes\b"):
            errors.append(f"[{key}] missing section: ## Review Notes")

        if not (
            _section_exists(text, r"^###\s+File List\b")
            or _section_exists(text, r"^##\s+File List\b")
        ):
            errors.append(f"[{key}] missing section: File List (##/### File List)")

    if errors:
        print("Story DoD verification failed:\n", file=sys.stderr)
        for e in errors:
            print(f"- {e}", file=sys.stderr)
        print(f"\nChecked {len(done_story_keys)} done stories from {SPRINT_STATUS}", file=sys.stderr)
        return 1

    print(f"OK: verified {len(done_story_keys)} done stories from {SPRINT_STATUS}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

