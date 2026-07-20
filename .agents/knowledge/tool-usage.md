# Uso de Herramientas — Casos Especiales de Lectura

Dos limitaciones mecánicas de las herramientas de lectura que solo importan en dos situaciones puntuales: buscar dentro de una carpeta que empieza con punto, o leer un archivo que no cabe en una sola llamada.

Slug ID: `TOOL-USAGE`.

Tamaño: `2593 caracteres (~648 tokens aprox., ratio 4:1)`.

---

## TOOL-USAGE-1 — Dot-Directories (Limitación Crítica de `glob`)

**Aplica a:** todos los agentes, al buscar un archivo dentro de una carpeta que empieza con `.`.

* **TOOL-USAGE-1.1 — Nunca `glob` en Dot-Directories:**

  la herramienta `glob` **NO encuentra** archivos dentro de directorios que empiezan con `.` (ej. `.claude/`, `.agents/`, `.git/`, `.github/`), devolviendo "No files found". En su lugar: `Read` directo con ruta conocida, `Read`/`ls` sobre el directorio para listar contenido, `find` para buscar por nombre, `grep` para buscar por contenido (sí funciona en dot-directories).
  * ❌ *Incorrecto:* usar `glob` para buscar un archivo dentro de `.agents/skills/`.
  * ✅ *Correcto:* `find .agents -name "patrón"` o `grep` sobre la ruta conocida.

**Filtro de verificación:** `TOOL-USAGE-3.1`.

---

## TOOL-USAGE-2 — Protocolo de Lectura Progresiva (Archivos Extensos)

**Aplica a:** todos los agentes, al leer un archivo que excede el límite de una sola llamada (>2000 líneas o >50KB).

* **TOOL-USAGE-2.1 — Encadenar por Offset, Nunca Repetir:**

  primera lectura con `offset=0` sin límite explícito — identifica en el mensaje de truncamiento dónde se detuvo (ej. *"Use offset=516 to continue"*). Lecturas siguientes encadenan el offset exacto indicado — nunca desde 0 otra vez, nunca saltos arbitrarios. Se permite disparar lecturas con distintos offsets en paralelo si los rangos son conocidos de antemano. Prohibido: leer el mismo rango dos veces, usar offsets fuera de secuencia, o saltar bloques sin cubrirlos.
  * ❌ *Incorrecto:* volver a leer desde `offset=0` en la segunda llamada, o saltar un rango sin cubrirlo.
  * ✅ *Correcto:* `Read offset=0` (líneas 1-515) → `Read offset=516` (líneas 516-1050) → `Read offset=1051` (líneas 1051-1546), encadenado sin huecos.

**Filtro de verificación:** `TOOL-USAGE-3.2`.

---

## TOOL-USAGE-3 — Filtro de Verificación

**Aplica a:** todos los agentes, al terminar de usar `glob`/`Read` sobre dot-directories o archivos extensos.

* **TOOL-USAGE-3.1:** si busqué dentro de una carpeta con punto, ¿evité `glob` y usé `find`/`grep`/`Read` directo (`TOOL-USAGE-1`)?
* **TOOL-USAGE-3.2:** si leí un archivo extenso, ¿encadené los offsets sin repetir ni saltar rangos (`TOOL-USAGE-2`)?
