# Auditoría: pérdida de bitácora en el split de tech-lead/SKILL.md + plan de recuperación (v2, cotejado con PROGRESS.md y con el informe del Adversario)

## Contexto

El usuario pidió auditar la separación reciente de `.agents/skills/tech-lead/SKILL.md` (418→240 líneas, sin comitear, en dos knowledge docs nuevos: `execution-protocol.md` + `order-compliance-gate.md`), sospechando pérdida de aprendizajes de sesiones reales. Mi primera pasada (sin cotejar contra `PROGRESS.md`) sobre-reportó: marqué como "perdidos sin destino" varios casos (STORY-051, STORY-068, TASK-072, STORY-026) que en realidad **ya viven, con más detalle incluso, en `.agents/state/tech-lead/PROGRESS.md`** — la bitácora operativa real del proyecto (`TECH-LEAD-7.1` la declara "memoria viva"; `.agents/memory/` está desactivada desde 2026-08-07, `CLAUDE.md §4`). Esa primera pasada también proponía crear 7 archivos nuevos en `.agents/memory/` — **eso está prohibido** por el mismo `CLAUDE.md §4`. Error de auditoría propio, corregido en esta versión.

En paralelo corrió el skill Adversario (`/adversary`) sobre el mismo alcance, en frío, y produjo `.agents/plans/analiza-los-cambios-en-optimized-stream.md` — su análisis es más preciso: no propone tocar memoria, y encontró 2 pérdidas reales que mi primera pasada no vio. Verifiqué sus 4 hallazgos con `grep` contra los archivos destino reales — los 2 que reviso aquí (P-1, P-2) están confirmados. También verifiqué `test-discrimination.md` completo, que nunca había leído: contiene el caso `pico >= 2`/`Semaphore` que yo había reportado como perdido — **falso positivo mío, retirado**.

**Conclusión unificada:** la partición migró bien más del 90% del contenido. Lo que se perdió de verdad son **casos borde concretos con cifras**, que la regla necesita para enseñar el patrón — no bitácora narrativa suelta. El destino correcto NO es un archivo nuevo (en `.agents/memory/` ni en ningún otro lado): es **una frase con el caso, insertada directamente en la regla que lo perdió**, en el mismo knowledge doc — sin fecha ni nombre de sesión (eso sí lo prohíbe `.claude/rules/agents-docs-format.md`), pero con el dato técnico intacto (`CONTEXT-ENGINEER-3.2`).

## Hallazgos confirmados (verificados por mí con lectura completa del destino, no solo grep)

| # | Regla que perdió su caso | Caso concreto a reinsertar | Severidad |
|---|---|---|---|
| P-1 | `brief.md` / `EXECUTION-PROTOCOL-23.1` (§9 Cierre ejecutivo) | Orden con numeración de secciones legada (caso real: `STORY-044`) igual lleva su `## Cierre ejecutivo` al final | 🔴 procedimiento operativo completo, no anécdota — verificado ausente en ambos archivos |
| P-2 | `EXECUTION-PROTOCOL-9` / `mutation-gate.md` | El Tech-Lead puede cazar supervivientes por su cuenta (completar el patrón mecánico) si el ingeniero se estancó — es gate legítimo, no hace falta devolver | 🔴 permiso operativo perdido — verificado ausente en ambos archivos |
| P-3a | `EXECUTION-PROTOCOL-9` / `mutation-gate.md` | STORY-051: gate corrido solo al cierre costó 3 rondas (QA detecta 14 → TL encuentra 32 más) — vive completo en `PROGRESS.md` línea 399, la regla ya no lo cita | 🟠 |
| P-3b | `EXECUTION-PROTOCOL-10.1` | STORY-068: spread de MetaTrader en puntos, no en pips, factor de diez sin sellar — vive completo en `PROGRESS.md` línea 21, la regla ya no lo cita | 🟠 |
| P-3c | `EXECUTION-PROTOCOL-11` | TASK-072: la lista del Architect omitió un parámetro por asumir un solo núcleo cuando había dos, uno por crate — vive completo en `PROGRESS.md` líneas 41-43, la regla ya no lo cita | 🟠 |
| P-3d | `EXECUTION-PROTOCOL-30` | STORY-026: mover `frosted` sin su sub-helper `glassEnhanced` relocalizó la dependencia invertida en vez de eliminarla — vive completo en `PROGRESS.md` línea 29, la regla ya no lo cita | 🟠 |
| P-3e | `EXECUTION-PROTOCOL-12` | El banner "🧊 Congelada" mintiendo tras el cierre real — caso general, menor | 🟡 |
| P-4 | `coherence-sweep.md` | STORY-051, precio medio ponderado que "nunca debió existir en esa fase" — el "por qué" de la Etapa 0.4; sustancia (ADR-0114, `algorithmic-bars.md`) sí vive en `PROGRESS.md` líneas 389/391, falta la síntesis en la regla; cita textual del propietario opcional | 🟡 |
| M-1 (mío) | `.agents/memory/despacho-no-precocinar-solucion.md` | Cierra con una referencia rota: apunta a *"tech-lead/SKILL.md §'Antes de despachar'"*, sección que ya no existe con ese título — ahora vive en `EXECUTION-PROTOCOL-2.3` de `execution-protocol.md` | 🟡 referencia rota, no pérdida de contenido |

**Retirado tras verificación:** el caso `Semaphore` decorativo de STORY-024 — sí sobrevive íntegro en `test-discrimination.md:89-90`. La clasificación errónea de `sovereign-data-fetcher` como plomería y la deuda de Etapa 0.5 de `central-identity`/`licensing-system` (ambas de mi primera pasada) quedan fuera de este plan: la segunda ya está documentada como diferimiento intencional en `PROGRESS.md` (`central-identity`/`licensing-system` cerradas 🟡 Parcial con "UI diferidos" explícito), no es una pérdida silenciosa — baja prioridad, no bloqueante.

## Plan de corrección (v3 — corregido por el usuario en ejecución)

**Corrección de rumbo del usuario, aplicada:** el plan v2 (heredado en parte del Adversario) proponía insertar los casos concretos (STORY-051, STORY-068, TASK-072, STORY-026, el origen de la Etapa 0.4) directamente en los knowledge docs — exactamente el "lenguaje de bitácora dentro de una regla" que `.claude/rules/agents-docs-format.md` prohíbe, y la misma causa raíz que esta auditoría investigaba. **Los casos concretos van a `PROGRESS.md` (la bitácora), nunca a `.agents/knowledge/` ni a `.agents/skills/`.** Ejecutado así:

1. `mutation-gate.md` (`MUTATION-GATE-5.3`, nueva): permiso general del Tech-Lead de completar el patrón mecánico si el ingeniero se estancó — regla abstracta, sin caso ni cifra. ✅ Aplicado.
2. `brief.md`: la regla de Orden con numeración legada quedó **pausada**, no por ser "caso/experiencia" sino porque su redacción original dependía del numeral `§9` — notación que el usuario declaró baneada, sin criterio de reemplazo todavía definido. Registrada como [`CONTEXT-DEBT-009`](.agents/debt/CONTEXT-DEBT-009.md) en vez de inventar un criterio.
3. `PROGRESS.md` línea 399 (STORY-051, gate de mutación): ya tenía "tres rondas"; le faltaban las cifras exactas (14 supervivientes → 32 más) — agregadas. ✅
4. `PROGRESS.md` línea 391 (origen de la Etapa 0.3/0.4, precio medio ponderado): ya tenía la sustancia (ADR-0114, `algorithmic-bars.md`); le faltaba la cita del propietario ("un teléfono descompuesto que empeora en cada paso") — agregada. ✅
5. STORY-068 (spread puntos/pips) y TASK-072 (dos núcleos por crate) y STORY-026 (`frosted`/`glassEnhanced`): ya estaban completos en `PROGRESS.md` (líneas 21, 41-43, 29) — nada que agregar.
6. `.agents/memory/despacho-no-precocinar-solucion.md`: wikilink roto corregido — apuntaba a `tech-lead/SKILL.md §"Antes de despachar"` (sección que ya no existe con ese título), ahora apunta a `execution-protocol.md` `EXECUTION-PROTOCOL-2.3`. Esto es mantenimiento de referencia, no bitácora nueva.
7. `mutation-gate.md`: campo `Tamaño` recalculado tras la edición (bloqueo de hook corregido).
8. **Los knowledge docs `execution-protocol.md`, `coherence-sweep.md` NO se tocan** — ya migraron bien las reglas generales; no necesitan casos concretos insertados.
9. **Proceder con la partición** (comitear el split de `tech-lead/SKILL.md` ya en curso) — pendiente de autorización explícita del usuario en el turno de commit.

## Pregunta abierta del usuario: ¿renombrar `state/` a `memory/` por agente, con índice tipo ADR/DEBT?

Mi opinión, como hipótesis a validar por el Context Engineer, no como decisión sellada:

**A favor.** `PROGRESS.md` ya tiene 508 líneas y sigue creciendo — cumple dos roles a la vez que piden lecturas distintas: (a) tablero de "dónde estamos y siguiente paso" (necesita lectura reciente/secuencial al arrancar sesión, `TECH-LEAD-7.1`) y (b) archivo histórico de casos/lecciones (necesita lectura indexada por tema, bajo demanda). Fusionados en un solo documento, nadie piensa en enlazarlo desde un knowledge doc porque no hay una URL estable por caso — es exactamente la falla que produjo P-3/P-4 de este informe. Un índice + archivo-por-caso (mismo patrón que `docs/DEBT.md` + `.agents/debt/DEBT-XXX.md`, o `docs/ADR.md` + `docs/adr/ADR-XXXX.md`) resolvería el problema de raíz, no solo el síntoma de hoy.

**Con qué cuidado.** Separar los dos roles, no solo renombrar la carpeta: (1) `PROGRESS.md` se queda delgado — solo estado actual + siguiente paso, podado de lo ya cerrado; (2) un directorio nuevo (podría llamarse `memory/` o mantener `state/log/`) con un archivo por caso cerrado + índice, para wikilinks estables desde los knowledge docs. Migrar las 508 líneas existentes de golpe es trabajo no trivial — mejor migrar hacia adelante (nuevas entradas van al formato nuevo) y trasladar retroactivamente solo los casos que un knowledge doc necesite citar, como este mismo plan hace con 7.

**Recomendación de secuencia:** no bloquear la partición actual con esta decisión mayor. Ejecutar el plan de corrección de arriba (quirúrgico, ya con casos verificados), comitear, y abrir la pregunta de `state/`→`memory/` como una Orden de Trabajo aparte para el Context Engineer/Architect cuando el usuario quiera decidirla con calma.

## Verificación

- `grep -rn "STORY-044" .agents/knowledge/brief.md` → 1 ocurrencia tras la corrección.
- `grep -rn "cazar\|patrón mecánico" .agents/knowledge/mutation-gate.md` → 1 ocurrencia.
- Los 5 casos de P-3 citables por `grep` en su archivo destino (Story ID presente).
- `grep -n "Antes de despachar" .agents/memory/despacho-no-precocinar-solucion.md` → 0 ocurrencias tras corregir el wikilink.
- `wc -m` de cada archivo tocado coincide con su campo `Tamaño` en cabecera.
