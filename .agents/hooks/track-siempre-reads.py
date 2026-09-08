#!/usr/bin/env python3
"""Marca en disco cuando el agente lee un documento .md del repo.

Hook PostToolUse sobre Read. Alimenta a dos gates que consultan la misma
marca: gate-skill-siempre.py (lecturas SIEMPRE de un skill, PreToolUse
sobre Skill) y enforce-essential-reads.py (lecturas forzosas mid-flujo,
PreToolUse sobre Write/Edit/Bash). Alcance amplio a proposito -- no solo
.agents/knowledge/ -- porque una lectura forzosa puede apuntar a
cualquier .md del repo (una plantilla, un SKILL.md, un doc de docs/).
"""

import json
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from _session_reads import mark_read


def main():
    repo_root = os.environ.get("CLAUDE_PROJECT_DIR") or os.getcwd()
    try:
        event = json.load(sys.stdin)
    except (json.JSONDecodeError, ValueError):
        return 0

    session_id = event.get("session_id")
    file_path = (event.get("tool_input") or {}).get("file_path")
    if not session_id or not file_path:
        return 0
    if not file_path.endswith(".md"):
        return 0

    rel_path = os.path.relpath(file_path, repo_root)
    mark_read(repo_root, session_id, rel_path)
    return 0


if __name__ == "__main__":
    sys.exit(main())
