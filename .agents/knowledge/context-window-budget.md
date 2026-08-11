# Presupuesto de Contexto por Modelo

Rangos de referencia para dimensionar cuánto contexto cargar al arranque de un skill y cuándo compactar una conversación operativa, por modelo.

Slug ID: `CONTEXT-WINDOW-BUDGET`.

Tamaño: `2970 caracteres (~742 tokens aprox., ratio 4:1)`.

---

## CONTEXT-WINDOW-BUDGET-FV — Filtro de Verificación

**Aplica a:** Context Engineer, Architect y Tech-Lead, al dimensionar el arranque de un skill o decidir si compactar.

* **CONTEXT-WINDOW-BUDGET-FV-1:** ¿dimensioné contra el contexto confiable del modelo, no contra su ventana nominal? (`CONTEXT-WINDOW-BUDGET-1`)
* **CONTEXT-WINDOW-BUDGET-FV-2:** ¿declaré que los rangos son heurística derivada, nunca un umbral oficial del proveedor? (`CONTEXT-WINDOW-BUDGET-1.1`)
* **CONTEXT-WINDOW-BUDGET-FV-3:** si el modelo en juego no está en la tabla, ¿revalidé su ventana en vez de extrapolar? (`CONTEXT-WINDOW-BUDGET-1.2`)

De ser satisfactoria cada validación de la lista, declara `CONTEXT-WINDOW-BUDGET - FILTRO DE VERIFICACION APROBADO`.

---

## CONTEXT-WINDOW-BUDGET-1 — Ventana Nominal vs. Contexto Confiable

La ventana de contexto publicada no es el contexto confiable: el rendimiento en tareas de razonamiento multi-hop, agregación o código se degrada bastante antes del límite técnico. Una conversación operativa larga (resultados de herramientas, ida y vuelta) degrada más rápido que un documento denso y curado del mismo tamaño, porque mete más ruido por token útil.

**Aplica a:** todos los skills/agentes, al dimensionar Lecturas Obligatorias, el arranque de un skill, o al decidir cuándo compactar una conversación.

* **CONTEXT-WINDOW-BUDGET-1.1 — Rangos por Modelo:**

  | Modelo | Ventana nominal | Contexto confiable (tareas complejas) | Arranque óptimo (system+skill+knowledge fijo) | Techo antes de compactar |
  |---|---|---|---|---|
  | Sonnet 5 / Opus 4.8 (1M) | 1,000,000 | ~450K–650K | 5K–30K | ~450K–550K acumulados |
  | Linaje 200K (Sonnet 4.5 y anteriores, Haiku 4.5) | 200,000 | ~100K–130K | 3K–15K | ~100K–130K acumulados |

  "Contexto confiable" y "techo antes de compactar" son heurística derivada de que el contexto efectivo ronda 50-65% del nominal en tareas simples de recuperación, y menos aún en razonamiento multi-hop — no un umbral oficial publicado por el proveedor del modelo.
  * ✅ *Correcto:* el skill [[.agents/skills/context-engineer/SKILL.md]] cae dentro del arranque óptimo; el diseño de [[CLAUDE.md]] §3 (mapa no volcado, lectura bajo demanda, barridos a subagentes) ya aplica este mismo principio a nivel de proyecto.

* **CONTEXT-WINDOW-BUDGET-1.2 — Vigencia de la Tabla:**

  las ventanas nominales cambian con cada release de modelo. Revalida antes de aplicar la tabla a un modelo que no aparezca en ella — nunca extrapoles por familia ni por nombre parecido. Fuentes de la medición: Anthropic Platform Docs y el estudio "Context Rot" de Chroma.

**Filtro de verificación:** `CONTEXT-WINDOW-BUDGET-FV-1`, `CONTEXT-WINDOW-BUDGET-FV-3`.
