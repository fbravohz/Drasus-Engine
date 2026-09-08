# Catálogo de Formatos de Bloque

- Todo documento consumido por agentes combina formatos de bloque — hoy Lecturas, Secciones, Pipeline, Ejemplos — catalogados aquí para ser encontrables por ID y aplicables sin ambigüedad.
- SLUG-ID: `TEMPLATE-INSTRUCTION-FORMATS`
- Tamaño: `8483 caracteres (~2121 tokens aprox., ratio 4:1)`
- Lecturas:
  - [[.agents/templates/template-knowledge.md]] -> SIEMPRE -> Context Engineer

## TEMPLATE-INSTRUCTION-FORMATS-1: Catálogo

| Formato | Cuándo usarlo | Posición | ID |
|---|---|---|---|
| Lecturas | El documento declara qué otros documentos leer, y cuándo. | Bajo el título, en la cabecera. | Sin ID propio. |
| Pipeline | El skill ejecuta pasos en un orden fijo que importa. | `<DOC>-P`, tras la cabecera, antes de la primera Sección. | `<DOC>-P`, sin sub-numeración. |
| Secciones | Contenido de comportamiento — bullets planos sobre un tema. | `<DOC>-N`, en el cuerpo. | `<DOC>-N`. |
| Ejemplos | Mostrar 2+ bullets del documento actuando juntos sobre un caso real. | `<DOC>-EJ`, cierre del documento. | `<DOC>-EJ-K`. |

Formato nuevo: agrega fila aquí + su propia `## Formato: <Nombre>`, solo si un tipo de bloque se repite en 2+ documentos con forma propia que ninguno de los cuatro cubre bien.

## TEMPLATE-INSTRUCTION-FORMATS-2: IDs

- `<DOC>` es el slug en mayúsculas con guion medio, calcado del nombre de archivo — nunca guion bajo.
- Tope: `<DOC>-N` para una Sección de primer nivel. Sin sub-ID de bullet — ningún `<DOC>-N.M`: cada bullet dentro de una Sección no es citable por separado, solo la Sección completa.
- Excepción: `<DOC>-EJ-K`, un ID por caso dentro de Ejemplos — namespace propio, no compite con las Secciones.
- Append-only: un `<DOC>-N` publicado nunca se renumera ni se reutiliza. Sección nueva = siguiente número libre, Si importa el orden logico en la lectura. Retirar una Sección libera su número — pero tendra que buscarse dicho ID por toda la documentacion para actualizarlo.
- Nada de lenguaje de enmienda dentro de un bullet — si cambia, se edita directo, sin "(antes decía X)". El historial vive en git.

## TEMPLATE-INSTRUCTION-FORMATS-3: Espaciado

- Sin `---` entre bloques — solo salto de línea en blanco.
- Un bullet por línea, indentar el cuerpo en línea aparte.
- Etiqueta en negrita pegada al inicio del bullet, seguida de dos puntos, cuerpo en la misma línea.

## TEMPLATE-INSTRUCTION-FORMATS-4: Formato — Lecturas

- Vive en la cabecera del documento, no como Sección `<DOC>-N`.
- Una línea por documento: `[[ruta/al/doc.md]] -> <disparador> -> <alcance>`.
- `SIEMPRE` solo para el propio documento madre con supremacía declarada (ej. `governance.md`). Cualquier otra fila lleva un disparador concreto — palabra clave, ruta de archivo, o caso puntual.
- `SIEMPRE` con alcance `Todos` (los 2 universales) se carga solo: hook `SessionStart` ([[.agents/hooks/session-start-siempre.py]]) inyecta su contenido completo al inicio de cada sesión, sin depender de que el agente lo lea.
- `SIEMPRE` acotado a un skill concreto se fuerza vía `PreToolUse` sobre el tool `Skill` ([[.agents/hooks/gate-skill-siempre.py]]): bloquea la invocación hasta que `PostToolUse` sobre `Read` ([[.agents/hooks/track-siempre-reads.py]]) confirme que ya se leyó.
- Disparador por palabra clave se fuerza vía `UserPromptSubmit` ([[.agents/hooks/inject-context.py]]): indexa con SQLite FTS5, por Sección (`## <DOC>-N: `, no todo el archivo junto — un documento en formato legado cae indexado completo) `.agents/knowledge/*.md` y `.agents/skills/*/SKILL.md`, resuelve el prompt como query por palabras clave con ranking `bm25()`, y siempre inyecta un manifiesto de wikilinks — nunca contenido completo, el agente decide si los lee. El campo `Tags:` (cabecera o Sección) pondera repitiendo sus palabras dentro del texto indexado. No es semántico: un concepto parafraseado sin la palabra exacta no dispara nada ([[.agents/plans/haz-el-plan-de-agile-sketch.md]] Sección B3).

```
- Lecturas:
  - [[.agents/knowledge/governance.md]] -> SIEMPRE -> Todos
  - [[.agents/knowledge/writing-efficiency.md]] -> SIEMPRE -> Context Engineer
```

## TEMPLATE-INSTRUCTION-FORMATS-5: Formato — Sección

- `## <DOC>-N: <Título>` — dos puntos, sin paréntesis, sin subtítulo pegado. El `: ` inmediatamente tras el ID (sin guion largo de por medio) es lo que `inject-context.py` usa para partir el índice por Sección — un heading legado con `—` nunca dispara esa partición, cae indexado completo.
- Primer bullet opcional `- Tags: \`tag-uno\`, \`tag-dos\`` — mismo campo y mismo vocabulario controlado que el de la cabecera (`TEMPLATE-KNOWLEDGE-1`, H1). Los tags de la cabecera ya aplican a toda Sección del documento; este bullet solo agrega tags propios de esta Sección específica, para rankear mejor esa Sección puntual en `inject-context.py` (Sección H3 del plan de aligeramiento).
- Cuerpo: bullets planos, uno por línea. `- **<Nombre corto>:** <cuerpo>` — nombre en negrita si ayuda a escanear, se omite si el bullet ya es autoexplicativo.
- Sin `Aplica a`, sin `Precedencia`, sin campo de cierre formal. Si un bullet remite a otro archivo, wikilink directo en el cuerpo, o línea suelta al cierre de la Sección en negrita asi: `**Referencias:** <wikilinks>` tantos bullets de lectura como referencias sean necesarias.
- ❌/✅ solo con medición real o caso genuinamente no obvio — nunca por defecto.

Ejemplo real (`HARD-PROHIBITIONS-2`):

```
## HARD-PROHIBITIONS-2: Lenguaje Mortuorio

- **Saneamiento inmediato:** al detectar el término, se reemplaza en el momento — nunca se abre un `DEBT-XXX`.
- **Reemplazos fijados:** `Forense`/`Post-mortem`/`Simulación Forense` → `Repetición de Corrida`/`Reproducción`/`Revisión de Corrida`.
- **Aplicar prohibiciones:** todas las prohibiciones del [[.agents/knowledge/write-like-gemini.md#WRITE-LIKE-GEMINI-3]]

**Leer:** 
- [[.agents/memory/prohibicion-lenguaje-mortuorio.md]].
- [[.agents/knowledge/write-like-gemini.md]]
```

## TEMPLATE-INSTRUCTION-FORMATS-6: Formato — Pipeline

- `## <DOC>-P: Pipeline de Ejecución` — tras la cabecera, antes de la primera Sección. Solo si el skill ejecuta varios pasos en un orden que importa; si es una sola acción sin secuencia, se omite el bloque entero.
- Cuerpo: lista numerada, un paso por línea. No repitas el ID de la Sección que lo detalla — el documento se lee completo y las Secciones siguen el mismo orden, así que ese ID no distingue nada; sin sub-IDs de bullet. Cita un ID entre paréntesis solo si el paso remite a otro documento, o a una Sección fuera del orden de lectura esperado.
- Sin sub-numeración propia — sus pasos no son citables por otra regla.
- Si un paso puede desviar la secuencia (rechazo temprano que salta el resto), decláralo explícito ahí mismo.

```
## AUDITOR-P: Pipeline de Ejecución

1. Filtro de arranque: lee `governance.md` completo, declara rol.
2. Verificación de las tres anclas: si falta el plan, rechaza y no continúa.
3. Auditoría ciega: aplica el criterio de evidencia sin leer aún la narrativa del agente auditado.
```

## TEMPLATE-INSTRUCTION-FORMATS-7: Formato — Ejemplos

- `## <DOC>-EJ: Ejemplos`, física al cierre del documento — después de la última Sección, sin importar su número.
- IDs internos `<DOC>-EJ-1`, `<DOC>-EJ-2`... — namespace propio, append-only, título pegado al sub-ID.
- Por cada caso: `**Caso: <qué transforma>**`, luego `**Antes:**` y `**Después:**`, cada uno con el texto citado en blockquote (`>`).
- Medición real (tokens, palabras, tiempo) inline en el texto del caso si existe — nunca estimada, nunca en tabla aparte.

```
**Caso: <qué texto real se transformó>**

**Antes:**
> <texto sin aplicar el formato>

**Después:**
> <mismo texto, formato aplicado>
```

## TEMPLATE-INSTRUCTION-FORMATS-8: Formatos Retirados

- `Filtro de Verificación` por documento (namespace `<DOC>-FV`) — retirado del catálogo. Reemplazo previsto: juez `Stop` vía IA ([[.agents/plans/haz-el-plan-de-agile-sketch.md]], Sección G1), aún no implementado — hueco de auto-chequeo aceptado mientras tanto.
- `Aplica a`, `Precedencia`, `Lectura bajo demanda`, `Plantilla de referencia` como campos de cierre formales — retirados. Si el contenido es real, va en prosa dentro del bullet o en línea suelta; si no hay contenido real, no se escribe nada.
- Distinción Regla — Completo/Ligero — colapsada en un solo Formato: Sección (`TEMPLATE-INSTRUCTION-FORMATS-5`).
- Sub-ID de bullet (`<DOC>-N.M`) dentro de una Sección — retirado. Toda cita a un bullet puntual pasa a citar la Sección completa que lo contiene.
