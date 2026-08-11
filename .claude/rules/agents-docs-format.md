---
paths:
  - ".agents/skills/**/*.md"
  - ".agents/knowledge/**/*.md"
  - ".agents/templates/**/*.md"
---

# Formato de los Documentos Normativos de `.agents/`

Estás editando un documento que otro agente va a leer y ejecutar. Cumple el formato canónico o un hook `PostToolUse` bloquea la escritura.

## Cabecera obligatoria

```
# <Título>

<1-2 líneas: qué agrupa y para quién. Sin historia de origen.>

Slug ID: `<DOC>`.

Tamaño: `<N> caracteres (~<M> tokens aprox., ratio 4:1)`.
```

`<DOC>` es el nombre del archivo en mayúsculas, con **guion medio** (`memory-policy.md` → `MEMORY-POLICY`). Un `SKILL.md` hereda el slug de su carpeta. Recalcula `Tamaño` con `wc -m` en la misma edición que cambia el contenido — el hook lo compara contra el archivo real.

## Orden de los bloques

`<DOC>-0` Lecturas Obligatorias (tabla de 4 columnas) → `<DOC>-FV` Filtro de Verificación → reglas `<DOC>-1`, `<DOC>-2`… → `<DOC>-EJ` Ejemplos al cierre.

El Filtro de Verificación va **segundo**, nunca al final, y usa namespace propio `<DOC>-FV-N` — no compite por número con las reglas.

## Reglas duras

- **Tope de 2 segmentos por ID:** `<DOC>-N` o `<DOC>-N.M`. `<DOC>-N.M.K` está prohibido — si un ítem necesita su propia lista citable, promuévelo a bloque de primer nivel.
- **IDs append-only:** un ID publicado nunca se renumera ni se reutiliza. Regla nueva = siguiente número libre. Renumerar exige reescritura completa autorizada por el usuario en el turno actual.
- **Sin lenguaje de bitácora:** ninguna fecha, "enmienda", "antes decía X", nombre de sesión ni caso histórico dentro de una regla. Esa historia vive en git. Preserva la regla; corta la anécdota.
- **Campo de cierre ausente = no aplica.** Nunca escribas `N/A` en `Precedencia`, `Filtro de verificación`, `Lectura bajo demanda` ni `Plantilla de referencia`.
- **`Documento extendido` fue retirado.** Usa `Lectura bajo demanda` con su disparador explícito.
- **Wikilinks:** toda referencia a otro documento es `[[ruta/al/archivo.md]]`, nunca una ruta suelta en texto plano.
- **`base.md` no existe como documento.** Apunta a `governance.md` o al documento por tema.

## Formato de una regla

```
## <DOC>-N — <Título sin paréntesis ni dos puntos>

<1-2 líneas de contexto: qué problema resuelve.>

**Aplica a:** <rol/modo, o "Todos los roles/skills">

* **<DOC>-N.1 — <Nombre>:**

  <cuerpo indentado 2 espacios, tras una línea en blanco>
  * ❌ *Incorrecto:* <solo si el error es real y no obvio>
  * ✅ *Correcto:* <...>
```

Un `---` entre bloques. Una línea en blanco entre ítems. Variante ligera (una línea, sin campos de cierre) cuando la regla es una sola idea sin conflicto ni plantilla.

## Estilo

Modo Gemini Relajado + Listas: el hecho primero, sin narrar el proceso; 2+ ítems paralelos van en viñetas, no en prosa corrida. Nunca cortes dato técnico, caso borde ni matiz para acortar — la economía corta conectores, no contenido.

**Detalle completo:** [[.agents/templates/template-instruction-formats.md]] (catálogo de formatos) y [[.agents/templates/template-knowledge.md]] (cómo se ensambla un documento).
