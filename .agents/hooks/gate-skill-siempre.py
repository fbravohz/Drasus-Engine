#!/usr/bin/env python3
"""Bloquea la invocacion de un skill si sus lecturas SIEMPRE propias
(alcance distinto de "Todos") no se leyeron aun en esta sesion.

Hook PreToolUse sobre el tool Skill. Las lecturas SIEMPRE con alcance
"Todos" ya llegan al contexto via session-start-siempre.py (SessionStart)
y no se exigen aqui -- este gate cubre solo el hueco que ese hook no
puede cubrir: lecturas SIEMPRE propias de un skill concreto, que
SessionStart no puede anticipar porque no sabe que skill se va a invocar.

Si no puede resolver el skill o su archivo, permite sin bloquear -- un
gate roto no debe tumbar la ejecucion normal.
"""

import json
import os
import re
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from _session_reads import already_read

# Formato ligero: "- [[ruta]] -> SIEMPRE -> <alcance>"
RE_LIGHT = re.compile(
    r"^\s*-\s*\[\[([^\]]+)\]\]\s*->\s*SIEMPRE\s*->\s*(.+?)\s*$", re.M
)
# Formato legado: tabla de 4 columnas con "SIEMPRE" en la 3a (disparador).
RE_LEGACY = re.compile(
    r"^\|\s*\[\[([^\]]+)\]\]\s*\|[^|]*\|[^|]*SIEMPRE[^|]*\|\s*([^|]+?)\s*\|\s*$",
    re.M,
)


def is_universal(alcance):
    return "todos" in alcance.lower()


def is_placeholder(path):
    """Rutas como 'docs/execution/<ID>.md' no son un archivo real -- son
    un patron que representa "el que este activo". Nunca resuelven a un
    Read literal, asi que exigirlas bloquearia el skill para siempre."""
    return "<" in path or ">" in path


def required_reads(skill_md_text):
    out = []
    for pattern in (RE_LIGHT, RE_LEGACY):
        for path, alcance in pattern.findall(skill_md_text):
            path = path.strip()
            if not is_universal(alcance) and not is_placeholder(path):
                out.append(path)
    return out


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

    if event.get("tool_name") != "Skill":
        return 0

    session_id = event.get("session_id")
    skill_name = (event.get("tool_input") or {}).get("skill")
    if not session_id or not skill_name:
        return 0

    skill_md = os.path.join(repo_root, ".agents", "skills", skill_name, "SKILL.md")
    if not os.path.isfile(skill_md):
        return 0

    with open(skill_md, encoding="utf-8") as fh:
        text = fh.read()

    missing = [
        path
        for path in required_reads(text)
        if not already_read(repo_root, session_id, path)
    ]
    if not missing:
        return 0

    listado = ", ".join(f"`{p}`" for p in missing)
    deny(
        f"Antes de usar el skill '{skill_name}' lee completo con Read: {listado} "
        "(lecturas SIEMPRE propias de este skill, aun no leidas en esta sesion). "
        "Luego vuelve a invocar el skill."
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
