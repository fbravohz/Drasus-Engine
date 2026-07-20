# Presupuesto de Contexto por Modelo

Rangos de referencia para dimensionar cuánto contexto cargar al arranque de un skill y cuándo compactar una conversación operativa, por modelo.

Slug ID: `CONTEXT_BUDGET`.

Tamaño: `2122 caracteres (~531 tokens aprox., ratio 4:1)`.

---

## CONTEXT_BUDGET-1 — Ventana Nominal vs. Contexto Confiable

La ventana de contexto publicada no es el contexto confiable: el rendimiento en tareas de razonamiento multi-hop, agregación o código se degrada bastante antes del límite técnico. Una conversación operativa larga (resultados de herramientas, ida y vuelta) degrada más rápido que un documento denso y curado del mismo tamaño, porque mete más ruido por token útil.

**Aplica a:** todos los skills/agentes, al dimensionar Lecturas Obligatorias, el arranque de un skill, o al decidir cuándo compactar una conversación.

* **CONTEXT_BUDGET-1.1 — Rangos por Modelo:**

  | Modelo | Ventana nominal | Contexto confiable (tareas complejas) | Arranque óptimo (system+skill+knowledge fijo) | Techo antes de compactar |
  |---|---|---|---|---|
  | Sonnet 5 / Opus 4.8 (1M, GA desde 2026-03) | 1,000,000 | ~450K–650K | 5K–30K | ~450K–550K acumulados |
  | Linaje 200K (Sonnet 4.5 y anteriores, Haiku 4.5 hoy) | 200,000 | ~100K–130K | 3K–15K | ~100K–130K acumulados |

  "Contexto confiable" y "techo antes de compactar" son heurística derivada de que el contexto efectivo ronda 50-65% del nominal en tareas simples de recuperación, y menos aún en razonamiento multi-hop — no un umbral oficial publicado por el proveedor del modelo.
  * ✅ *Correcto:* el skill `.agents/skills/context-engineer/SKILL.md` (~2,156 tokens estimados) cae dentro del arranque óptimo; el diseño de `CLAUDE.md` §3 (mapa no volcado, lectura bajo demanda, barridos a subagentes) ya aplica este mismo principio a nivel de proyecto.

**Vigencia:** ventanas nominales verificadas por búsqueda web el 2026-07-19 — cambian con cada release de modelo, revalida antes de aplicarlo a un modelo no listado aquí.

**Documento extendido:** conversación fuente (2026-07-19) citó Anthropic Platform Docs y el estudio "Context Rot" de Chroma.
