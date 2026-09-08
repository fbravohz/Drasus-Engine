#!/usr/bin/env python3
"""Inyecta wikilinks de lecturas relevantes al prompt del usuario via FTS5+bm25.

Hook UserPromptSubmit -- se dispara en cada turno, sin matcher (no lo
soporta este evento). Indexa con SQLite FTS5 el contenido de
.agents/knowledge/*.md y .agents/skills/*/SKILL.md **por Seccion**, no por
archivo completo: cada documento en formato ligero (encabezados
`## <DOC>-N: Titulo`) se parte en un segmento por Seccion, mas un segmento
"cabecera" con todo lo anterior a la primera Seccion. Un documento en
formato legado (sin ese patron de encabezado) cae completo en un solo
segmento -- mismo comportamiento que antes de esta version.

Resuelve el prompt como query por palabras clave (no semantica) y, si hay
coincidencias, inyecta un manifiesto de wikilinks rankeados por bm25 --
por defecto nunca contenido completo, el agente decide si los lee. Cuando
el match es una Seccion concreta (no la cabecera), el wikilink cita su ID.

Switch de una sola propiedad -- FORCE_FULL_CONTENT (abajo). En falso (por
defecto): wikilinks siempre. En verdadero: pega el texto real (sin el
boost de tags) de cada Seccion que matcheo, no solo su link -- pensado
como troubleshooting si en la practica el wikilink no se lee lo bastante.
Con Secciones de por si chicas (200-2.000 caracteres tipico, no el
archivo completo de 9.000), el riesgo de forzar contenido es mucho menor
que cuando esto indexaba por archivo -- aun asi, FORCE_MAX_CHARS pone un
tope de seguridad combinado, para el caso de un documento legado grande
que aun no se partio por Seccion.

Pondera el campo `Tags:` (cabecera o dentro de una Seccion, ver
template-instruccion-formats.md TEMPLATE-INSTRUCTION-FORMATS-5) repitiendo
sus palabras TAG_REPEAT veces en el texto indexado de ese segmento -- boost
de frecuencia clasico dentro de una sola columna. Los tags de la cabecera
se heredan a cada Seccion del mismo documento (contexto del documento
completo); los propios de una Seccion se agregan encima. Se descarto a
proposito ponderar por argumento bm25(tabla, w0, w1) de una columna `tags`
separada: probado en vivo, ese enfoque acopla la normalizacion de longitud
de TODO el corpus (el promedio se recalcula con el peso), y el resultado
no era monotono -- subir el peso podia encoger la ventaja del documento
etiquetado en vez de crecerla. La repeticion en una sola columna se probo
monotona y predecible.

Ver .agents/plans/haz-el-plan-de-agile-sketch.md, Secciones B1 y H3.
"""

import json
import os
import re
import sqlite3
import sys

REPO_ROOT_FALLBACK = os.getcwd()
INDEX_REL_PATH = ".agents/hooks/.fts-index.db"
MAX_DOCS = 3
TAG_REPEAT = 4

# Switch de troubleshooting: en falso, wikilinks siempre (comportamiento
# por defecto). En verdadero, pega el texto real de cada Seccion que
# matcheo -- util si en la practica el agente no abre los wikilinks lo
# bastante. FORCE_MAX_CHARS es el tope de seguridad combinado incluso con
# el switch activo -- un documento legado (sin split por Seccion) puede
# seguir siendo grande, esto evita que un solo match domine el turno.
FORCE_FULL_CONTENT = False
FORCE_MAX_CHARS = 12000

# Conjunto compacto -- basta para no dejar pasar ruido de alta frecuencia,
# no pretende ser una lista exhaustiva de stopwords del espanol.
STOPWORDS = {
    "de", "la", "el", "en", "que", "y", "a", "para", "con", "por", "es",
    "un", "una", "los", "las", "se", "su", "lo", "como", "mas", "pero",
    "sus", "le", "ya", "o", "este", "esta", "entre", "cuando", "muy",
    "sin", "sobre", "tambien", "me", "hasta", "hay", "donde", "quien",
    "desde", "todo", "nos", "todos", "uno", "les", "ni", "contra",
    "otros", "ese", "eso", "ante", "ellos", "esto", "mi", "antes",
    "algunos", "que", "unos", "yo", "otro", "otras", "otra", "el",
    "tanto", "esa", "estos", "mucho", "quienes", "nada", "muchos",
    "cual", "poco", "ella", "estar", "estas", "algunas", "algo",
    "nosotros", "tu", "te", "ti", "tus", "del", "al", "sea", "son",
    "the", "and", "for", "with", "que", "esta", "estan", "sido",
}

RE_TOKEN = re.compile(r"[a-zA-Záéíóúñü]+", re.IGNORECASE)
RE_TAGS_LINE = re.compile(r"^\s*-?\s*Tags:\s*(.+)$", re.M)
RE_TAG_VALUE = re.compile(r"`([^`]+)`")
RE_SECTION_HEADING = re.compile(
    r"^## ([A-Z][A-Z0-9]*(?:-[A-Za-z0-9]+)+): .+$", re.M
)


def find_source_files(repo_root):
    out = []
    knowledge_dir = os.path.join(repo_root, ".agents", "knowledge")
    if os.path.isdir(knowledge_dir):
        for name in sorted(os.listdir(knowledge_dir)):
            if name.endswith(".md"):
                out.append(os.path.join(knowledge_dir, name))
    skills_dir = os.path.join(repo_root, ".agents", "skills")
    if os.path.isdir(skills_dir):
        for skill in sorted(os.listdir(skills_dir)):
            candidate = os.path.join(skills_dir, skill, "SKILL.md")
            if os.path.isfile(candidate):
                out.append(candidate)
    return out


def index_is_stale(db_path, source_files):
    if not os.path.isfile(db_path):
        return True
    db_mtime = os.path.getmtime(db_path)
    return any(os.path.getmtime(f) > db_mtime for f in source_files)


def extract_tags(text):
    m = RE_TAGS_LINE.search(text)
    if not m:
        return []
    return RE_TAG_VALUE.findall(m.group(1))


def split_sections(content):
    """(section_id, texto) por cada bloque -- section_id vacio = cabecera
    (todo antes de la primera Seccion). Documento sin encabezados
    `## <DOC>-N: ` (formato legado) devuelve un unico segmento vacio con
    el contenido completo -- mismo comportamiento que indexar por archivo."""
    matches = list(RE_SECTION_HEADING.finditer(content))
    if not matches:
        return [("", content)]
    segments = [("", content[: matches[0].start()])]
    for i, m in enumerate(matches):
        start = m.start()
        end = matches[i + 1].start() if i + 1 < len(matches) else len(content)
        segments.append((m.group(1), content[start:end]))
    return segments


def build_indexed_text(header_tags, segment_text):
    tags = list(header_tags)
    for t in extract_tags(segment_text):
        if t not in tags:
            tags.append(t)
    if not tags:
        return segment_text
    boost = (" ".join(tags) + " ") * TAG_REPEAT
    return boost + segment_text


def rebuild_index(db_path, source_files, repo_root):
    os.makedirs(os.path.dirname(db_path), exist_ok=True)
    if os.path.isfile(db_path):
        os.remove(db_path)
    con = sqlite3.connect(db_path)
    con.execute(
        'CREATE VIRTUAL TABLE docs USING fts5('
        'path UNINDEXED, section UNINDEXED, raw UNINDEXED, content, '
        'tokenize="unicode61 remove_diacritics 2")'
    )
    for path in source_files:
        try:
            with open(path, encoding="utf-8") as fh:
                file_text = fh.read()
        except OSError:
            continue
        rel = os.path.relpath(path, repo_root)
        segments = split_sections(file_text)
        header_tags = extract_tags(segments[0][1])
        for section_id, text in segments:
            con.execute(
                "INSERT INTO docs (path, section, raw, content) VALUES (?, ?, ?, ?)",
                (rel, section_id, text, build_indexed_text(header_tags, text)),
            )
    con.commit()
    con.close()


def tokens_for_query(prompt):
    words = RE_TOKEN.findall(prompt.lower())
    seen = []
    for w in words:
        if len(w) <= 2 or w in STOPWORDS:
            continue
        if w not in seen:
            seen.append(w)
    return seen


def build_fts_query(tokens):
    return " OR ".join(f'"{t}"' for t in tokens)


def emit_context(text):
    print(json.dumps({
        "hookSpecificOutput": {
            "hookEventName": "UserPromptSubmit",
            "additionalContext": text,
        }
    }))


def main():
    repo_root = os.environ.get("CLAUDE_PROJECT_DIR") or REPO_ROOT_FALLBACK
    try:
        event = json.load(sys.stdin)
    except (json.JSONDecodeError, ValueError):
        return 0

    prompt = event.get("prompt") or ""
    if not prompt.strip():
        return 0

    tokens = tokens_for_query(prompt)
    if not tokens:
        return 0

    source_files = find_source_files(repo_root)
    if not source_files:
        return 0

    db_path = os.path.join(repo_root, INDEX_REL_PATH)
    if index_is_stale(db_path, source_files):
        rebuild_index(db_path, source_files, repo_root)

    con = sqlite3.connect(db_path)
    query = build_fts_query(tokens)
    try:
        rows = con.execute(
            "SELECT path, section, raw, bm25(docs) AS score FROM docs "
            "WHERE docs MATCH ? ORDER BY score LIMIT ?",
            (query, MAX_DOCS),
        ).fetchall()
    except sqlite3.OperationalError:
        con.close()
        return 0
    con.close()

    if not rows:
        return 0

    def label(path, section):
        return f"[[{path}]] — sección {section}" if section else f"[[{path}]]"

    if FORCE_FULL_CONTENT:
        parts = [
            "CONTEXTO RELEVANTE detectado por inject-context.py (FTS5 bm25 "
            "sobre tu prompt, indexado por Seccion) -- FORCE_FULL_CONTENT "
            "activo, contenido real de cada Seccion que matcheo:"
        ]
        budget = FORCE_MAX_CHARS
        for path, section, raw, _ in rows:
            if len(raw) <= budget:
                parts.append(f"\n--- {label(path, section)} ---\n{raw}")
                budget -= len(raw)
            else:
                parts.append(f"- {label(path, section)} (excede el resto del presupuesto, revisa con Read)")
        emit_context("\n".join(parts))
    else:
        lines = [
            "CONTEXTO RELEVANTE detectado por inject-context.py (FTS5 bm25 "
            "sobre tu prompt, indexado por Seccion) -- manifiesto de "
            "wikilinks, tu decides si aplican a tu tarea y si los lees con "
            "Read:"
        ]
        for path, section, _, _ in rows:
            lines.append(f"- {label(path, section)}")
        emit_context("\n".join(lines))

    return 0


if __name__ == "__main__":
    sys.exit(main())
