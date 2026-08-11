#!/usr/bin/env python3
"""Valida el formato canonico de los documentos normativos de .agents/.

Dos modos de uso:

  1. Hook PostToolUse (sin argumentos): lee el JSON del evento por stdin,
     saca la ruta de tool_input.file_path y valida ese archivo. Sale con
     codigo 2 si encuentra una violacion dura, lo que bloquea la accion.

  2. Auditoria (--audit [ruta]): recorre el arbol y reporta todo, sin
     bloquear nada. Sale con 1 si hubo violaciones duras.

La norma que implementa vive en .agents/templates/template-instruction-formats.md
y .agents/templates/template-knowledge.md. Las reglas objetivas se validan aqui;
las que exigen juicio quedan como aviso.
"""

import json
import os
import re
import sys

# ---------------------------------------------------------------------------
# Alcance
# ---------------------------------------------------------------------------

# Solo los documentos normativos. state/, sessions/, plans/ y memory/ son
# bitacora, ledgers y datos — no reglas, y no se les exige este formato.
SCOPE_DIRS = (".agents/skills/", ".agents/knowledge/", ".agents/templates/")

# Archivos de indice y stubs de compatibilidad: no declaran reglas propias,
# no necesitan Slug ni Tamano. base.md solo sobrevive para que los planes y
# ordenes ya cerradas que lo citan no queden con un enlace muerto.
NO_SLUG_REQUIRED = {
    ".agents/knowledge/README.md",
    ".agents/knowledge/base.md",
}

# El stub de mudanza necesita nombrarse a si mismo para cumplir su funcion.
STUB_FILES = {".agents/knowledge/base.md"}

# Las plantillas exhiben ejemplos de todo lo que prohiben — se eximen de las
# reglas que sus propios ejemplos dispararian.
TEMPLATE_FILES = {
    ".agents/templates/template-instruction-formats.md",
    ".agents/templates/template-knowledge.md",
    ".agents/templates/template-session-report.md",
}

# writing-efficiency.md cita "base.md" dentro de un ejemplo de mal encabezado.
BASE_MD_ALLOWED = {".agents/knowledge/writing-efficiency.md"}

# El campo Tamano no puede cuadrar al caracter: escribir la cifra corregida
# cambia el propio conteo cuando cruza un orden de magnitud. Dos caracteres de
# holgura absorben esa oscilacion sin tolerar un campo realmente desfasado.
SIZE_TOLERANCE = 2

# ---------------------------------------------------------------------------
# Utilidades
# ---------------------------------------------------------------------------

FENCE_RE = re.compile(r"^\s*(`{3,}|~{3,})")


def strip_code_blocks(lines):
    """Devuelve (numero_de_linea, texto) solo de las lineas fuera de un bloque
    de codigo. Un ejemplo dentro de ``` no es una violacion del documento."""
    out = []
    fence = None
    for i, line in enumerate(lines, start=1):
        m = FENCE_RE.match(line)
        if m:
            marker = m.group(1)
            if fence is None:
                fence = marker[0] * 3
                continue
            if marker.startswith(fence):
                fence = None
                continue
        if fence is None:
            out.append((i, line))
    return out


def expected_slug(rel_path):
    """SKILL.md hereda el slug de su carpeta; el resto, de su nombre."""
    base = os.path.basename(rel_path)
    if base == "SKILL.md":
        stem = os.path.basename(os.path.dirname(rel_path))
    else:
        stem = base[:-3]
    return stem.upper()


def in_scope(rel_path):
    return rel_path.endswith(".md") and rel_path.startswith(SCOPE_DIRS)


# ---------------------------------------------------------------------------
# Reglas
# ---------------------------------------------------------------------------

RE_SLUG = re.compile(r"^Slug ID:\s*`([^`]+)`", re.M)
RE_SIZE = re.compile(r"^Tama\u00f1o:\s*`(\d+)\s*caracteres", re.M)
RE_ID3 = re.compile(r"\b[A-Z][A-Z0-9]*(?:-[A-Z0-9]+)*-\d+\.\d+\.\d+\b")
RE_NA = re.compile(
    r"^\*\*(Precedencia|Filtro de verificaci\u00f3n|Lectura bajo demanda|"
    r"Plantilla de referencia):\*\*\s*(N/A|n/a|-)\s*$"
)
RE_EXTENDED = re.compile(r"\*\*Documento extendido:\*\*")
RE_FV_BLOCK = re.compile(r"^##\s+[A-Z][A-Z0-9-]*-FV\b", re.M)
RE_BASE_MD = re.compile(r"knowledge/base\.md|`base\.md`")

# Ruta a un .md que quedo suelta en el texto. Se busca sobre la linea ya
# despojada de wikilinks y enlaces markdown (ver bare_paths), porque un
# lookbehind no basta: en `[[.agents/x.md]]` el caracter previo a la ruta es
# el punto, no el corchete.
RE_BARE_PATH = re.compile(r"((?:\.?[\w-]+/)+[\w.-]+\.md)")
RE_WIKILINK = re.compile(r"\[\[[^\]]*\]\]")
RE_MDLINK = re.compile(r"\[[^\]]*\]\([^)]*\)")


def bare_paths(line):
    """Rutas .md que no viven dentro de un wikilink ni de un enlace markdown."""
    cleaned = RE_MDLINK.sub(" ", RE_WIKILINK.sub(" ", line))
    return [m.group(1) for m in RE_BARE_PATH.finditer(cleaned)]

RE_DATE = re.compile(r"\b20\d{2}-\d{2}-\d{2}\b")
# Deliberadamente NO incluye "causa ra\u00edz" ni "enmienda" sueltas: ambas son
# vocabulario de dominio del proyecto \u2014 "Causa ra\u00edz" es un campo obligatorio de
# la ficha DEBT, y una enmienda de ADR es un tipo de entregable. Solo cuentan
# cuando vienen fechadas, que es lo que las convierte en bitacora.
RE_AMENDMENT = re.compile(
    r"\b(antes dec\u00eda|caso real:|lecci\u00f3n (de|STORY)|regla del usuario,?\s*20|"
    r"naci\u00f3 de|recuperado de|enmienda de[lst]?\s*20)",
    re.I,
)


def validate(rel_path, text):
    """Devuelve (duras, blandas) — dos listas de mensajes."""
    hard, soft = [], []
    lines = text.splitlines()
    code_free = strip_code_blocks(lines)
    is_template = rel_path in TEMPLATE_FILES

    # --- Slug ID ---------------------------------------------------------
    slug_match = RE_SLUG.search(text)
    if rel_path not in NO_SLUG_REQUIRED and not is_template:
        want = expected_slug(rel_path)
        if not slug_match:
            hard.append(f"falta el campo `Slug ID: \u0060{want}\u0060`")
        elif slug_match.group(1) != want:
            got = slug_match.group(1)
            why = ""
            if "_" in got:
                why = " (el slug usa guion medio, nunca guion bajo)"
            hard.append(f"Slug ID es `{got}` pero el archivo exige `{want}`{why}")

    # --- Campo Tamano ----------------------------------------------------
    if rel_path not in NO_SLUG_REQUIRED and not is_template:
        size_match = RE_SIZE.search(text)
        real = len(text)
        if not size_match:
            tokens = round(real / 4)
            hard.append(
                f"falta el campo `Tama\u00f1o: \u0060{real} caracteres "
                f"(~{tokens} tokens aprox., ratio 4:1)\u0060`"
            )
        else:
            declared = int(size_match.group(1))
            if abs(declared - real) > SIZE_TOLERANCE:
                tokens = round(real / 4)
                hard.append(
                    f"campo Tama\u00f1o declara {declared} pero el archivo mide "
                    f"{real} caracteres \u2014 escribe: `{real} caracteres "
                    f"(~{tokens} tokens aprox., ratio 4:1)`"
                )

    # --- IDs de 3 segmentos ---------------------------------------------
    if not is_template:
        for ln, line in code_free:
            for m in RE_ID3.finditer(line):
                hard.append(
                    f"L{ln}: ID de 3 segmentos `{m.group(0)}` \u2014 el tope es "
                    f"`<DOC>-N.M`; promueve el item a bloque propio"
                )

    # --- Campos de cierre en N/A ----------------------------------------
    for ln, line in code_free:
        if RE_NA.match(line.strip()):
            hard.append(
                f"L{ln}: campo de cierre en `N/A` \u2014 un campo sin contenido "
                f"real se omite entero"
            )

    # --- Campo retirado --------------------------------------------------
    if not is_template:
        for ln, line in code_free:
            if RE_EXTENDED.search(line):
                hard.append(
                    f"L{ln}: `Documento extendido` fue retirado del catalogo "
                    f"\u2014 usa `Lectura bajo demanda` con su disparador"
                )

    # --- Referencia al stub muerto ---------------------------------------
    if rel_path not in BASE_MD_ALLOWED and rel_path not in STUB_FILES:
        for ln, line in code_free:
            if RE_BASE_MD.search(line):
                hard.append(
                    f"L{ln}: referencia a `base.md`, que es un stub vacio "
                    f"\u2014 apunta a `governance.md` o al documento por tema"
                )

    # --- Avisos ----------------------------------------------------------
    if not RE_FV_BLOCK.search(text) and rel_path not in NO_SLUG_REQUIRED:
        want = expected_slug(rel_path)
        soft.append(f"sin bloque `## {want}-FV \u2014 Filtro de Verificaci\u00f3n`")

    if not is_template:
        for ln, line in code_free:
            for path in bare_paths(line):
                soft.append(
                    f"L{ln}: ruta `{path}` sin wikilink \u2014 escribe `[[{path}]]`"
                )
            if RE_DATE.search(line) or RE_AMENDMENT.search(line):
                soft.append(
                    f"L{ln}: bitacora en el cuerpo \u2014 la fecha, la enmienda o "
                    f"el caso historico van en git, no en la regla"
                )

    return hard, soft


# ---------------------------------------------------------------------------
# Salida
# ---------------------------------------------------------------------------


def report(rel_path, hard, soft, stream=sys.stderr):
    if not hard and not soft:
        return
    print(f"\n{rel_path}", file=stream)
    for msg in hard:
        print(f"  BLOQUEA  {msg}", file=stream)
    for msg in soft:
        print(f"  aviso    {msg}", file=stream)


def run_file(path, repo_root):
    rel = os.path.relpath(path, repo_root)
    if not in_scope(rel):
        return None
    try:
        with open(path, encoding="utf-8") as fh:
            text = fh.read()
    except (OSError, UnicodeDecodeError):
        return None
    return rel, validate(rel, text)


def main():
    repo_root = os.environ.get("CLAUDE_PROJECT_DIR") or os.getcwd()

    if "--audit" in sys.argv:
        idx = sys.argv.index("--audit")
        target = sys.argv[idx + 1] if len(sys.argv) > idx + 1 else ".agents"
        target = os.path.join(repo_root, target)
        total_hard = total_soft = files = 0

        # --audit acepta tanto un directorio a recorrer como un archivo suelto.
        if os.path.isfile(target):
            candidates = [target]
        else:
            candidates = [
                os.path.join(dirpath, name)
                for dirpath, _, filenames in os.walk(target)
                for name in sorted(filenames)
            ]

        for candidate in candidates:
            res = run_file(candidate, repo_root)
            if res is None:
                continue
            rel, (hard, soft) = res
            files += 1
            total_hard += len(hard)
            total_soft += len(soft)
            report(rel, hard, soft, stream=sys.stdout)
        print(
            f"\n{files} archivos \u2014 {total_hard} violaciones duras, "
            f"{total_soft} avisos",
            file=sys.stdout,
        )
        return 1 if total_hard else 0

    # Modo hook: el evento llega por stdin.
    try:
        event = json.load(sys.stdin)
    except (json.JSONDecodeError, ValueError):
        return 0
    path = (event.get("tool_input") or {}).get("file_path")
    if not path:
        return 0
    res = run_file(path, repo_root)
    if res is None:
        return 0
    rel, (hard, soft) = res
    report(rel, hard, soft)
    if hard:
        # PostToolUse corre despues de la escritura: el archivo en disco ya
        # tiene el cambio. El exit 2 no lo revierte — corta el turno para que
        # el agente no pueda seguir adelante sin corregir lo que rompio.
        print(
            "\nEl archivo quedo escrito con violaciones duras. No sigas con "
            "otra cosa: corrigelas ahora sobre este mismo archivo.",
            file=sys.stderr,
        )
        return 2
    return 0


if __name__ == "__main__":
    sys.exit(main())
