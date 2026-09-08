#!/usr/bin/env python3
"""Esquema compartido de marcadores de lectura por sesion (helper, no es hook).

Usado por track-siempre-reads.py (escribe la marca al leer un .md), por
gate-skill-siempre.py y enforce-essential-reads.py (consultan si ya se
leyo), y por reset-session-reads.py (borra el directorio de la sesion en
compact/resume/clear).

Marca = archivo vacio en <STATE_DIR>/<session_id>/<slug de la ruta>. La
existencia del archivo es la unica senal -- sin contenido que parsear.
"""

import os
import shutil
import time

STATE_DIR = ".agents/state/.session-reads"
SWEEP_DAYS = 3


def _slug(rel_path):
    return rel_path.lstrip(".").replace("/", "__")


def session_dir(repo_root, session_id):
    return os.path.join(repo_root, STATE_DIR, session_id)


def marker_path(repo_root, session_id, rel_path):
    return os.path.join(session_dir(repo_root, session_id), _slug(rel_path))


def mark_read(repo_root, session_id, rel_path):
    path = marker_path(repo_root, session_id, rel_path)
    os.makedirs(os.path.dirname(path), exist_ok=True)
    open(path, "w").close()


def already_read(repo_root, session_id, rel_path):
    return os.path.exists(marker_path(repo_root, session_id, rel_path))


def sweep_old(repo_root, max_age_days=SWEEP_DAYS):
    """Borra directorios de sesion con mas de max_age_days sin tocarse --
    red de seguridad para sesiones que nunca disparan SessionEnd."""
    base = os.path.join(repo_root, STATE_DIR)
    if not os.path.isdir(base):
        return
    cutoff = time.time() - max_age_days * 86400
    for name in os.listdir(base):
        path = os.path.join(base, name)
        try:
            if os.path.isdir(path) and os.path.getmtime(path) < cutoff:
                shutil.rmtree(path, ignore_errors=True)
        except OSError:
            continue
