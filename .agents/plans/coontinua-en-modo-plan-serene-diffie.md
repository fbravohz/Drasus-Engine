# Plan: aligerar `template-knowledge.md` y `template-instruction-formats.md` al estilo `write-like-gemini.md`

## Contexto

El usuario pidió tomar `.agents/knowledge/write-like-gemini.md` como molde de estilo (títulos, subtítulos, secciones, formatos, IDs, separación y espacios) y reescribir con él los dos documentos que definen cómo se escribe cualquier knowledge doc del proyecto: `.agents/templates/template-knowledge.md` (cómo se ensambla un documento completo) y `.agents/templates/template-instruction-formats.md` (el catálogo de formatos de bloque del que depende). Esto es la pieza que faltaba antes de que el propietario + Gemini puedan migrar el resto del corpus (`.agents/plans/haz-el-plan-de-agile-sketch.md`, Sección A) — sin una plantilla escrita en el formato objetivo, no hay especificación a seguir.

Regla de decisión que fijó el usuario explícitamente: **todo elemento estructural que no exista en `write-like-gemini.md` se elimina de ambos templates.** Se validó con 4 preguntas puntuales donde la eliminación tenía una consecuencia técnica real (ver "Decisiones" abajo). El resto de las diferencias se retira directo, sin más confirmación.

## Decisiones tomadas (las 4 con consecuencia técnica)

1. **Sub-IDs citables por bullet (`<DOC>-N.M`): se eliminan.** Solo la sección `<DOC>-N` completa queda citable — igual que `write-like-gemini.md`, donde no existe ningún `WRITE-LIKE-GEMINI-1.1`. Consecuencia aceptada por el usuario: rompe las ~600 citas activas del corpus que hoy apuntan a un `<DOC>-N.M` puntual — **el remapeo de esas citas queda fuera del alcance de esta tarea** (es trabajo de migración de contenido, ya asignado a propietario+Gemini en el plan grande). Se deja como nota de riesgo, no se ejecuta aquí.
2. **Tabla de Lecturas Obligatorias: se conserva, pero sin tabla Markdown.** Pasa a lista plana, una lectura por línea, formato `Documento -> Disparador -> Alcance`.
3. **Headings de sección: adoptan `:` en vez de `—`** (ej. `## HARD-PROHIBITIONS-2: Lenguaje Mortuorio`), igual que `WRITE-LIKE-GEMINI-1: Comportamiento`. Esto contradice la regla vigente del hook/Rule (`.claude/rules/agents-docs-format.md`, "sin dos puntos") — **actualizar ese hook queda fuera de esta tarea**, es una dependencia a resolver aparte (coincide con el "Paso 0" de la Sección E del plan grande).
4. **Filtro de Verificación por documento: se retira ya**, sin esperar a que el juez `Stop` (G1 del plan grande) esté implementado. Hueco temporal de auto-chequeo aceptado explícitamente por el usuario.

## Inventario completo de elementos retirados (sin objeción, "todo lo que no esté se elimina")

- `**Aplica a:** <texto>` como campo obligatorio por regla.
- `**Precedencia:**` como campo de cierre.
- Sección `<DOC>-FV` completa (preguntas numeradas + línea "declara...APROBADO").
- Definición con etiqueta en su línea + línea en blanco + cuerpo indentado 2 espacios en la siguiente → colapsa a una sola línea por bullet.
- ❌/✅ como práctica default → solo si hay medición real o caso genuinamente no obvio (igual que `write-like-gemini.md`, que no usa el símbolo ni una sola vez).
- `**Lectura bajo demanda:** [[...]] — <disparador>` como campo de cierre formal → se menciona en prosa suelta si hace falta, sin la etiqueta.
- `**Plantilla de referencia:**` + bloque de código como campo formal → se retira; si hace falta un bloque copiable, va directo en el cuerpo.
- Checklist final ("Checklist Antes de Guardar...") en ambos documentos → se retira.
- `Slug ID: \`<DOC>\`.` y `Tamaño: \`<N> caracteres...\`.` como párrafos propios con líneas en blanco alrededor → ambos pasan a bullet de cabecera (`SLUG-ID: \`<DOC>\``, `Tamaño: \`<N> caracteres (~<M> tokens aprox., ratio 4:1)\``). **Corrección post-ejecución:** el plan original decía que el campo Tamaño desaparecía del todo (Sección A del plan grande) — el usuario pidió conservarlo declarado, ver "Cambios posteriores a la salida de modo plan" abajo.
- `---` entre cada bloque → se retiran, solo saltos de línea en blanco (igual que `write-like-gemini.md`).
- Distinción "Regla — Completo" vs "Regla — Ligero" → colapsa a una sola forma (sin Aplica a, sin Precedencia, sin campos de cierre, la diferencia entre ambas variantes ya no existe).
- Sección "IDs para Listas Dentro de un Bloque" (`template-instruction-formats.md`) → se elimina entera: ya no hay sub-listas con ID individual dentro de una sección.

**Se conserva:** wikilinks `[[ruta/archivo.md]]` para toda referencia a otro documento; el slug `<DOC>` en mayúsculas con guion medio calcado del nombre de archivo; IDs append-only (nunca se renumeran ni reutilizan), ahora a nivel de sección `<DOC>-N`; namespace propio `<DOC>-EJ-K` para Ejemplos, sin competir con la numeración de Reglas; Pipeline de Ejecución como bloque opcional raro (no estaba en `write-like-gemini.md` porque no aplicaba a ese documento, se mantiene su función pero con el heading `:` y sin aparato).

## Estructura objetivo — Documento de Knowledge

```
# <Título>

- <Objetivo/propósito: 1-2 líneas, qué agrupa y para quién>
- SLUG-ID: `<DOC>`
- Lecturas:
  - [[ruta/doc1.md]] -> SIEMPRE -> Todos
  - [[ruta/doc2.md]] -> `keyword1`, `keyword2` -> <alcance>

## <DOC>-1: <Título de la sección>

- **<Nombre corto>:** cuerpo en una línea, directo.
- **<Nombre corto>:** cuerpo en una línea, directo.
- <bullet sin nombre en negrita si ya es autoexplicativo>

## <DOC>-2: <Título de la siguiente sección>

- ...

## <DOC>-EJ: Ejemplos

**Caso: <qué transforma>**

**Antes:**
> <texto o fragmento citado>

**Después:**
> <texto o fragmento citado, con medición real inline si existe — nunca estimada>
```

Ejemplo aplicado ya validado con el usuario (fragmento real, `HARD-PROHIBITIONS-2`):

```
## HARD-PROHIBITIONS-2: Lenguaje Mortuorio

- **Prohibición:** ningún término del campo semántico de la muerte (muerte, difunto, cadáver, tumba, entierro, cementerio, autopsia, forense, post-mortem, necro-, luto) en features, módulos, ADRs o cualquier documento — aplica al campo completo, no a una lista corta.
- **Saneamiento inmediato:** al detectar el término, se reemplaza en el momento — nunca se abre un `DEBT-XXX`.
- **Reemplazos fijados:** `Forense`/`Post-mortem`/`Simulación Forense` → `Repetición de Corrida`/`Reproducción`/`Revisión de Corrida`.

Caso de origen: [[.agents/memory/prohibicion-lenguaje-mortuorio.md]].
```

## Método de ejecución

Al salir de modo plan, antes de escribir cualquiera de los dos archivos se invoca la skill `/high-density-writing` (máxima densidad, apunta a recortar ~80% del texto generado) para redactar el contenido final — se aplica junto con el molde estructural de `write-like-gemini.md` ya fijado arriba, no en su lugar: la skill gobierna la densidad de la prosa suelta (descripciones, cuerpos de bullet), el molde de esta sección gobierna la estructura (qué campos existen, IDs, espaciado). Ninguna de las dos cede contenido técnico o caso borde para lograr la densidad — mismo límite que ya rige todo el proyecto (`CONTEXT-ENGINEER-3.2`, `WRITING-EFFICIENCY-1.2`).

## Archivos a modificar

1. **`.agents/templates/template-knowledge.md`** — reescritura completa (autorizada por el usuario en este turno, cambia el formato entero del documento):
   - Cabecera nueva (bullets, sin Slug ID/Tamaño como párrafos formales, sin campo Tamaño).
   - Bloque de Lecturas como lista plana con flechas, no tabla.
   - Elimina el bloque `<DOC>-FV` de la estructura obligatoria.
   - Reglas como secciones `<DOC>-N: <Título>` con bullets planos, sin sub-ID.
   - Sección de Ejemplos en formato blockquote.
   - Retira "Campo Tamaño: Cómo Calcularlo", el checklist final, y actualiza "Ejemplo Aplicado" para citar el nuevo formato en vez de `writing-efficiency.md` (que sigue en formato viejo).

2. **`.agents/templates/template-instruction-formats.md`** — reescritura completa:
   - "Formato: Lecturas Obligatorias" → cambia de tabla a lista plana `Documento -> Disparador -> Alcance`.
   - "Formato: Regla — Completo/Ligero" → colapsa a un solo "Formato: Regla" (sección + bullets planos, sin sub-ID, sin Aplica a/Precedencia/campos de cierre).
   - "Formato: Filtro de Verificación" → se retira del catálogo (nota: reemplazado por el juez `Stop`, G1, pendiente de implementar).
   - "Formato: Pipeline de Ejecución" → se conserva, heading con `:`, sin más cambios de fondo.
   - "Formato: Ejemplos" → cambia de bloques de código a blockquote.
   - "Niveles de ID" → se reescribe: tope ahora es `<DOC>-N` (1 segmento) para Reglas; `.M`/`-K` queda solo para los namespaces especiales que ya lo usaban así (`<DOC>-EJ-K`).
   - "IDs para Listas Dentro de un Bloque" → se elimina entera.
   - "Espaciado y Saltos de Línea" → se reescribe: sin `---`, sin indentación de 2 espacios en línea separada, un bullet por línea.
   - "Regla de Oro" → se elimina o se reduce (los campos que "podían omitirse" ya no existen, no aplica la convención de omisión condicional).
   - Checklist final → se elimina.

## Cambios posteriores a la salida de modo plan

Lo que pasó después de `ExitPlanMode`, no descrito en el plan original — registrado aquí para que el archivo quede fiel a lo ejecutado:

- **El usuario restauró el campo Tamaño a mano** en ambos archivos (contradice la línea del plan original que decía "desaparece del todo, nunca se declara a mano", heredada de `haz-el-plan-de-agile-sketch.md` Sección A). Queda declarado en la cabecera de ambos, formato `Tamaño: \`<N> caracteres (~<M> tokens aprox., ratio 4:1)\`` — mismo formato que ya usaba el resto del corpus, para que el regex del hook lo reconozca.
- **El usuario editó `template-instruction-formats.md` directamente en disco**, fuera de esta conversación: reordenó la tabla de `TEMPLATE-INSTRUCTION-FORMATS-1` (Pipeline antes de Secciones), cambió la política de IDs retirados en `-2` (de "no libera su número, queda vacante para siempre" a "libera el número, pero hay que buscar y actualizar la referencia en toda la documentación"), cambió la indentación en `-3` (de "sin indentar el cuerpo en línea aparte" a "indentar"), agregó el patrón `**Referencias:**`/`**Leer:**` como forma de cierre de Sección en `-5`, y reescribió el ejemplo de `HARD-PROHIBITIONS-2` (quitó el bullet "Prohibición", agregó uno que remite a `write-like-gemini.md`). No se revirtió nada de esto.
- **Se modificó `.agents/hooks/validate-agent-docs.py`** — fuera del alcance original ("Riesgos" abajo lo daba como no tocado en esta tarea, pero sí se tocó parcialmente):
  - Nuevo conjunto `TEMPLATE_FILES_SIZE_CHECKED = {template-instruction-formats.md, template-knowledge.md}`: estos dos archivos ya no están exentos de la validación del campo Tamaño (el resto de exenciones de `TEMPLATE_FILES` sigue igual; `template-session-report.md`, el tercer archivo de esa lista, no se tocó y sigue exento de Tamaño también).
  - `RE_SIZE` ajustado para reconocer el campo como bullet de cabecera (`- Tamaño: ...`), no solo como párrafo suelto — necesario para el formato nuevo.
  - Probado en vivo: el hook bloqueó dos veces (`exit 2`, mensaje con el número exacto a escribir) mientras se ajustaba el campo Tamaño de ambos archivos tras las ediciones — confirma que el mecanismo "bloqueo + fix sugerido" ya funciona sobre estos dos archivos.
- **Conteo real de citas `<DOC>-N.M` en `.agents/` y `.claude/`: 1123** (`grep -rnoE` sobre `*.md`) — no las ~600 que estimaba el plan grande. Sigue sin remapearse, fuera de alcance de esta tarea.

## Riesgos y dependencias fuera de alcance (se documentan, no se ejecutan en esta tarea)

- **Remapeo de 1123 citas `<DOC>-N.M`** en todo `.agents/` y `.claude/` que hoy apuntan a un sub-ID de definición que dejará de existir como tal — pasarán a citar la sección `<DOC>-N` contenedora. Tarea de migración de contenido, no de mecanismo.
- **`.claude/rules/agents-docs-format.md`** sigue exigiendo el formato viejo (heading sin dos puntos, tope de 2 segmentos con `.M` citable, Slug ID con etiqueta "Slug ID:"). `validate-agent-docs.py` ya se ajustó parcialmente (ver arriba) para el campo Tamaño de estos dos archivos, pero el resto del script (Slug ID, IDs de 3 segmentos, bloque FV esperado) sigue validando el formato viejo para el resto del corpus. Sin actualizar la Rule y el resto del script, escribir un documento nuevo cualquiera con este formato haría fallar el hook `PostToolUse`. Coincide con el "Paso 0" ya descrito en `.agents/plans/haz-el-plan-de-agile-sketch.md`.
- Los dos documentos ya usados como ejemplo dentro de las plantillas actuales (`writing-efficiency.md` en `template-knowledge.md`, `AUDITOR-P` en `template-instruction-formats.md`) siguen en formato viejo — se actualiza la referencia textual al patrón nuevo, sin tocar esos archivos de dominio.

## Verificación

1. Releer ambos archivos reescritos completos y confirmar que no queda ningún elemento de la lista "retirados" (grep manual de `Aplica a:`, `Precedencia:`, `Filtro de verificación:`, `---`, tabla `|---|---|`) — hecho.
2. Confirmar que `template-knowledge.md` y `template-instruction-formats.md` quedan mutuamente consistentes — ningún campo que uno exige y el otro ya no define.
3. Aplicar el nuevo formato a un segundo fragmento real del corpus (no solo `HARD-PROHIBITIONS-2`, ya validado) para confirmar que el molde generaliza — por ejemplo una Regla con más de 3 definiciones.
4. Reportar al usuario la lista de citas `<DOC>-N.M` detectadas por `grep -rn` que quedarán huérfanas por el cambio de nivel de ID, sin remapearlas todavía — hecho, 1123 citas (ver "Cambios posteriores a la salida de modo plan").
5. `python3 .agents/hooks/validate-agent-docs.py --audit .agents/templates/template-knowledge.md` y lo mismo para `template-instruction-formats.md` — hecho, ambos en 0 violaciones duras, 1 aviso soft cada uno (bloque FV esperado por el hook, retirado a propósito).
