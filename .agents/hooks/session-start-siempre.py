#!/usr/bin/env python3
"""Inyecta las lecturas SIEMPRE universales al inicio de la sesion.

Hook SessionStart: se dispara una sola vez por razon de arranque
(startup/resume/clear/compact/fork), antes de que el modelo procese
el primer turno. Imprime por stdout el contenido completo de los dos
documentos que CLAUDE.md declara "los dos unicos documentos SIEMPRE,
pares entre si" -- gobernanza y prohibiciones absolutas -- para que
lleguen al contexto sin depender de que el agente decida leerlos.

No cubre lecturas SIEMPRE especificas de un skill (ej. writing-efficiency.md
para context-engineer): SessionStart no sabe que skill se va a invocar.
Esas siguen dependiendo del paso explicito de Pipeline/Filtro de arranque
de cada skill (ver CONTEXT-DEBT-011).

De paso, barre marcas de sesion abandonadas (.agents/state/.session-reads/,
ver _session_reads.sweep_old) -- corre aqui porque este hook dispara en
las 5 razones de arranque, no solo compact/resume/clear como
reset-session-reads.py, asi que el barrido real es mas frecuente.
"""

import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from _session_reads import sweep_old

SIEMPRE_UNIVERSAL = (
    ".agents/knowledge/governance.md",
    ".agents/knowledge/hard-prohibitions.md",
)


def main():
    repo_root = os.environ.get("CLAUDE_PROJECT_DIR") or os.getcwd()
    sweep_old(repo_root)

    print(
        "OBLIGATORIO -- lecturas SIEMPRE universales, ya cargadas por el hook "
        "SessionStart (.agents/hooks/session-start-siempre.py). No hace falta "
        "releerlas con Read: declara su lectura igual que si las hubieras "
        "abierto tu mismo."
    )

    for rel_path in SIEMPRE_UNIVERSAL:
        path = os.path.join(repo_root, rel_path)
        try:
            with open(path, encoding="utf-8") as fh:
                content = fh.read()
        except OSError:
            print(
                f"\n[session-start-siempre.py] no pude leer {rel_path} -- "
                "revisa la ruta.",
                file=sys.stderr,
            )
            continue
        print(f"\n--- {rel_path} ---\n")
        print(content)

    return 0


if __name__ == "__main__":
    sys.exit(main())
