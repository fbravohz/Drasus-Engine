# Plantilla de Documento — Formato Estándar para `.agents/knowledge/`

Cómo estructurar un documento completo de `.agents/knowledge/` (no una regla suelta — para eso, `.agents/templates/RULE.md`). Un documento de knowledge es un contenedor de reglas: define el alcance, aloja N reglas con el formato de `RULE.md`, y cierra con una sección de Ejemplos compuestos.

**Regla de oro:** todo documento de knowledge usa `RULE.md` para cada regla individual que contiene. Esta plantilla no repite esos campos — dice cómo se ensamblan varias reglas en un documento coherente.

---

## Estructura del Documento

```
# <Título del Documento>

<Descripción rápida: 1-2 líneas, qué agrupa este documento y para quién — sin backstory, WRITING_EFFICIENCY-5.1>

Slug ID: `<DOC>`.

---

## <DOC>-1 — <Primera regla, formato RULE.md completo>

...

## <DOC>-2 — <Segunda regla, formato RULE.md completo>

...

## <DOC>-N — Filtro de Verificación
<Debe existir una seccion de chequeo interno siempre, redactalo en base a las reglas previas — WRITING_EFFICIENCY-6>

---

## <DOC>-N+1 — Ejemplos

### <DOC>-N+1.K — <Título del bloque de casos>
<Ver "Sección de Ejemplos" abajo y RULE.md, "Excepción: Sección de Ejemplos, No Regla">
```

---

## Sección de Ejemplos: Al Final del Documento, No Por Regla

**Decisión:** los ejemplos compuestos van en una sección única al cierre del documento — no repetidos dentro de cada regla.

* **Por qué no por regla:** `RULE.md` ya exige un par ❌/✅ atómico por cada definición específica (uno por ítem, dentro de la regla misma). Ese ejemplo prueba la regla aislada. Un ejemplo *compuesto* — texto real donde 2+ reglas del documento actúan juntas — es información nueva, no una repetición; meterlo dentro de cada regla la infla y duplica contenido entre reglas vecinas.
* **Por qué al final:** las reglas de un documento rara vez se aplican una por una — se aplican juntas sobre el mismo párrafo. Un ejemplo o varios al cierre puede mostrar eso (texto antes/después citando qué IDs de regla intervinieron) sin que ninguna regla individual cargue con el peso de la demostración completa.

**Formato de cada ejemplo (idéntico a `WRITING_EFFICIENCY-7`, líneas 149-173):**

`````
### <DOC>-N.K — <Título del bloque de casos>

<Intro: qué son los casos compuestos de esta sección y qué NO reemplazan
(los ❌/✅ atómicos de cada regla) — 1-2 líneas>

**Caso: <qué texto real se transformó> — demuestra `<DOC>-N.N`, `<DOC>-N.N` [...].**

* **Sin estas reglas (<X tokens>):** "<texto real o representativo, sin aplicar las reglas>"
* **Aplicando `<DOC>-N.N`, `<DOC>-N.N` — <Y tokens>:**
  ```
  <mismo contenido, reglas aplicadas>
  ```
`````

Si hay medición real disponible (tokens, palabras, tiempo), agrégala como tabla debajo del caso — no la inventes; si no la mediste, omite la fila en vez de estimar.

---

## Ejemplo Aplicado (Retrofit Real)

Así se ve esta plantilla aplicada — `.agents/knowledge/WRITING-EFFICIENCY.MD` es el primer documento retrofitteado con este formato:

* 6 reglas (`WRITING_EFFICIENCY-1` a `-6`), cada una con el formato completo de `RULE.md`.
* Una sección `## WRITING_EFFICIENCY-7 — Ejemplos` al final, con sub-ID `### WRITING_EFFICIENCY-7.1 — Casos Compuestos` y un caso compuesto citando `WRITING_EFFICIENCY-1.1`, `-1.2` y `-2.1` a la vez, más la tabla de medición real tomada de `.agents/documents/tmp/gemini_vs_claude.md`.

---

## Checklist Antes de Guardar un Documento de Knowledge Nuevo

- [ ] ¿El documento tiene un slug `<DOC>` único, declarado cerca del título?
- [ ] ¿Cada regla dentro usa el formato completo de `RULE.md` (ID, Aplica a, definiciones con sub-ID, Precedencia, Filtro de verificación)?
- [ ] ¿Los IDs son planos (`<DOC>-N`) o anidados (`<DOC>-N.N`) según si existe una sección H1 numerada real — ver `RULE.md`, "Niveles de ID"?
- [ ] ¿La sección `## Ejemplos` está al final, no repetida dentro de cada regla, y sin `Aplica a`/`Precedencia`/`Filtro de verificación` (RULE.md, "Excepción: Sección de Ejemplos, No Regla")?
- [ ] ¿Cada sub-ID `### <DOC>-N.K` de Ejemplos lleva su título pegado, con el párrafo de intro antes del primer Caso?
- [ ] ¿Cada ejemplo compuesto cita los IDs de regla que demuestra?
- [ ] ¿Toda medición mostrada es real (con fuente), nunca estimada sin decirlo?
