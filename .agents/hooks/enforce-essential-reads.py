#!/usr/bin/env python3
"""Bloquea Write/Edit/Bash si una lectura forzosa declarada no se hizo aun.

Hook PreToolUse sobre Write|Edit|Bash. Complementa a gate-skill-siempre.py
(que gatilla solo al invocar el tool Skill, con lecturas SIEMPRE de
alcance fijo): este gate cubre un punto intermedio del propio flujo de un
skill -- "antes de escribir X, hace falta haber leido Y" -- sin depender
de que el usuario lo dispare con un prompt nuevo. No existe ningun hook
atado al razonamiento interno del agente, solo a llamadas de herramienta
(verificado contra code.claude.com/docs/en/hooks) -- por eso el gate se
ancla a la llamada de herramienta que ese punto del flujo produce, no al
"momento" en si.

Convencion declarativa (un bullet por regla, dentro de cualquier
.agents/skills/*/SKILL.md o .agents/knowledge/*.md):

    - Lectura forzosa: [[ruta/al/doc.md]] antes de `Write(patron/**)`

El patron dentro de backticks usa la misma sintaxis Tool(especificador)
que .claude/settings.json (docs/en/permissions) -- Write, Edit o Bash,
con * como comodin (fnmatch: * cruza "/" igual que en las reglas de
permisos de Claude Code). Si no puede resolver ninguna regla o el
archivo esta vacio, permite sin bloquear -- un gate roto no debe tumbar
la ejecucion normal.

Ver .agents/plans/haz-el-plan-de-agile-sketch.md, Seccion H3, subseccion
"Lecturas forzosas mid-flujo".
"""

import fnmatch
import json
import os
import re
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from _session_reads import already_read

RE_RULE = re.compile(
    r"^\s*-\s*Lectura forzosa:\s*\[\[([^\]]+)\]\]\s*antes de\s*`"
    r"(Write|Edit|Bash)\(([^)]*)\)`\s*$",
    re.M,
)


def source_files(repo_root):
    out = []
    skills_dir = os.path.join(repo_root, ".agents", "skills")
    if os.path.isdir(skills_dir):
        for name in sorted(os.listdir(skills_dir)):
            path = os.path.join(skills_dir, name, "SKILL.md")
            if os.path.isfile(path):
                out.append(path)
    knowledge_dir = os.path.join(repo_root, ".agents", "knowledge")
    if os.path.isdir(knowledge_dir):
        for name in sorted(os.listdir(knowledge_dir)):
            if name.endswith(".md"):
                out.append(os.path.join(knowledge_dir, name))
    return out


def load_rules(repo_root):
    rules = []
    for path in source_files(repo_root):
        try:
            with open(path, encoding="utf-8") as fh:
                text = fh.read()
        except OSError:
            continue
        for read_path, tool, pattern in RE_RULE.findall(text):
            rules.append((read_path.strip(), tool, pattern.strip()))
    return rules


def subject_for(tool_name, tool_input, repo_root):
    if tool_name in ("Write", "Edit"):
        file_path = (tool_input or {}).get("file_path")
        if not file_path:
            return None
        return os.path.relpath(file_path, repo_root)
    if tool_name == "Bash":
        return (tool_input or {}).get("command")
    return None


def deny(reason):
    print(json.dumps({
        "hookSpecificOutput": {
            "hookEventName": "PreToolUse",
            "permissionDecision": "deny",
            "permissionDecisionReason": reason,
        }
    }))


def main():
    repo_root = os.environ.get("CLAUDE_PROJECT_DIR") or os.getcwd()
    try:
        event = json.load(sys.stdin)
    except (json.JSONDecodeError, ValueError):
        return 0

    tool_name = event.get("tool_name")
    if tool_name not in ("Write", "Edit", "Bash"):
        return 0

    session_id = event.get("session_id")
    if not session_id:
        return 0

    subject = subject_for(tool_name, event.get("tool_input"), repo_root)
    if subject is None:
        return 0

    missing = []
    seen = set()
    for read_path, rule_tool, pattern in load_rules(repo_root):
        if rule_tool != tool_name:
            continue
        if not fnmatch.fnmatch(subject, pattern):
            continue
        if already_read(repo_root, session_id, read_path):
            continue
        if read_path not in seen:
            seen.add(read_path)
            missing.append(read_path)

    if not missing:
        return 0

    listado = ", ".join(f"[[{p}]]" for p in missing)
    deny(
        f"Antes de esto, lee completo con Read: {listado} (lectura forzosa "
        "declarada para este punto del flujo, aun no leida en esta sesion). "
        "Luego reintenta."
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
