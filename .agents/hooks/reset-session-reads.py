#!/usr/bin/env python3
"""Resetea las marcas de lectura de la sesion en compact/resume/clear.

Hook SessionStart (matcher compact|resume|clear). Borra el directorio
completo de marcas de esta sesion -- .agents/state/.session-reads/<id>/
-- para que gate-skill-siempre.py y enforce-essential-reads.py vuelvan a
exigir la lectura real, en vez de confiar en una marca de antes de la
compactacion/reanudacion cuyo contenido puede ya no estar en el contexto
usable del agente.

Decision de diseno: session_id sobrevive a /compact (verificado
comparando la ruta del transcript pre-compactacion contra la del hook
SessionStart:compact disparado despues). Pero eso no basta para confiar
en la marca -- el proposito del mecanismo es
garantizar contenido *actualmente* en contexto, no "se leyo alguna vez" --
asi que la marca se borra de todas formas, sin importar si el id
sobrevivio o no. Sin esto habria que releer para que la garantia sea
real, que es exactamente el costo aceptado.

El barrido de sesiones abandonadas (SWEEP_DAYS) vive en el helper
compartido y tambien se invoca desde session-start-siempre.py -- ese
corre en las 5 razones de arranque, no solo compact/resume/clear, asi
que el barrido real es mas frecuente que solo esta ruta.

Ver .agents/plans/haz-el-plan-de-agile-sketch.md, Seccion H3, subseccion
"Lecturas forzosas mid-flujo".
"""

import json
import os
import shutil
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from _session_reads import session_dir, sweep_old


def main():
    repo_root = os.environ.get("CLAUDE_PROJECT_DIR") or os.getcwd()
    try:
        event = json.load(sys.stdin)
    except (json.JSONDecodeError, ValueError):
        return 0

    session_id = event.get("session_id")
    if session_id:
        shutil.rmtree(session_dir(repo_root, session_id), ignore_errors=True)

    sweep_old(repo_root)
    return 0


if __name__ == "__main__":
    sys.exit(main())
