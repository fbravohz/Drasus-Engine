# Plan — Corrección de pérdidas en la partición de `tech-lead/SKILL.md`

## Contexto

El usuario pidió, vía `/adversary`, auditar en frío el histórico de `.agents/skills/tech-lead/SKILL.md` porque sospecha que la reciente separación del archivo (que lo partió en `tech-lead/SKILL.md` + dos knowledge docs nuevos) perdió aprendizajes acumulados por el equipo a lo largo de sesiones de trabajo, en vez de trasladarlos íntegros a su nuevo destino. Hay un Auditor corriendo el mismo análisis en paralelo; este documento es la re-derivación ciega e independiente del Adversario (`ADVERSARY-4.1`) — se comparará contra el informe del Auditor cuando llegue, pero el hallazgo y la corrección no dependen de esa comparación.

**Hallazgo clave de encuadre:** la partición **no está comiteada** — vive en el árbol de trabajo actual (`git status` marca `tech-lead/SKILL.md` como modificado y los dos knowledge docs nuevos como `??`, sin tracking). No hay "commits de los últimos 10" que la contengan: es trabajo en curso de esta misma sesión, sobre `HEAD` (`6d1055e`). Esto es buena noticia — nada se enterró bajo commits posteriores; se puede corregir antes de comitear.

**Verificación realizada:** diff completo línea por línea entre `git show HEAD:.agents/skills/tech-lead/SKILL.md` (418 líneas, 87 841 caracteres, la versión previa a la partición) y el trío actual — `tech-lead/SKILL.md` (240 líneas) + `.agents/knowledge/execution-protocol.md` (nuevo) + `.agents/knowledge/order-compliance-gate.md` (nuevo) — más los cinco knowledge docs ya existentes a los que ambos apuntan por wikilink (`mutation-gate.md`, `ui-verification-model.md`, `coherence-sweep.md`, `test-discrimination.md`, `execution-vocabulary.md`, `brief.md`).

**Veredicto:** la partición es en su gran mayoría fiel — el 90%+ de las reglas migró íntegro, incluidos varios ejemplos numéricos concretos que sí sobrevivieron bien (`pico >= 2` en `test-discrimination.md`, el patrón de harness SVF genérico en `ui-verification-model.md`, el choque `STORY-004`/`TASK-004` en `execution-vocabulary.md`). Pero **sí hay pérdidas reales**, y siguen el mismo patrón que ya detectó la auditoría forense previa (`.agents/plans/auditoria-forense-commits-972bab5-3007f92.md`, hallazgo N-5): se aplicó correctamente la purga de "lenguaje de bitácora" (fecha, "regla del usuario", nombre de sesión) que exige `.claude/rules/agents-docs-format.md`, pero en el proceso se recortó también **dato técnico y caso borde** que `CONTEXT-ENGINEER-3.2` prohíbe cortar — y cuya precedencia sobre la economía de redacción esa misma regla declara explícita ("`CONTEXT-ENGINEER-3.2` sigue mandando... nunca [se corta] contenido normativo"). Es la segunda vez que ocurre el mismo patrón; la causa de fondo no se corrigió tras la primera.

---

## Hallazgos verificados (evidencia citada)

### 🔴 P-1 — Regla operativa completa, ausente (no es anécdota, es procedimiento)

`tech-lead/SKILL.md` (HEAD) línea 314 traía una instrucción de contingencia: *"Si la Orden usa una numeración de secciones distinta a la canónica (caso legado, ej. STORY-044), añade de todos modos una sección final rotulada `## Cierre ejecutivo`"*. Esta regla **no aparece** en `brief.md` (el documento al que `EXECUTION-PROTOCOL-23.1` remite para el Cierre Ejecutivo) ni en `execution-protocol.md`. No es una anécdota purgable — es la única instrucción escrita para el caso real de una Orden con formato legado (`STORY-044` existe de verdad en `docs/execution/`). Sin ella, un Tech-Lead futuro que audite una Orden vieja no sabe qué hacer.

### 🔴 P-2 — Permiso operativo perdido

`tech-lead/SKILL.md` (HEAD) línea 297: *"Cazar survivientes por tu cuenta (añadir tests de QA-cierre) es parte legítima de tu gate; si el ingeniero se estancó, completar el patrón mecánico es aceptable."* Ni `EXECUTION-PROTOCOL-9` ni `mutation-gate.md` conservan este permiso. Cambia lo que el Tech-Lead puede hacer sin escalar — no es un matiz decorativo.

### 🟠 P-3 — Casos borde cuantificados, perdidos sin sustituto (mismo patrón que N-5)

| Regla | Caso borde perdido | Dónde debería vivir |
|---|---|---|
| Gate de mutación corrido temprano (`EXECUTION-PROTOCOL-9`) | STORY-051: los huecos aparecieron en el gate final y costaron 3 rondas (QA detecta 14 → ingeniero cierra → TL corre completo y encuentra 32 más → ingeniero cierra → QA revalida) | `mutation-gate.md` (`MUTATION-GATE-1` o un ejemplo nuevo en `MUTATION-GATE-EJ`) |
| Barrido de unidades (`EXECUTION-PROTOCOL-10.1`) | STORY-068: el spread de MetaTrader viene en puntos, no en pips — factor de diez sin sellar; "lo cazó el propietario, no yo" | `EXECUTION-PROTOCOL-10.1` mismo (es la regla que ilustra) |
| Lista de alcance documental (`EXECUTION-PROTOCOL-11`) | TASK-072: la lista del Architect omitió un parámetro porque había dos núcleos, uno por crate, y la resolución solo mencionaba uno | `EXECUTION-PROTOCOL-11` mismo |
| Refactor lateral (`EXECUTION-PROTOCOL-30`) | STORY-026: mover `frosted` sin su sub-helper `glassEnhanced` relocalizó la dependencia invertida en vez de eliminarla | `EXECUTION-PROTOCOL-30` mismo |
| Sincronía documental por etapa (`EXECUTION-PROTOCOL-12`) | El banner "🧊 Congelada — el puente está sin despachar" mintiendo tras el cierre real | `EXECUTION-PROTOCOL-12` mismo |

Estos cinco son recuperables — el caso real vive en `docs/execution/STORY-051-*.md`, `STORY-068-*.md`, `TASK-072-*.md`, `STORY-026-*.md` y `.agents/state/tech-lead/PROGRESS.md` — pero la regla que debía enseñarlos ya no los cita.

### 🟡 P-4 — Narrativa de justificación debilitada (severidad menor)

`coherence-sweep.md` conserva el procedimiento completo de la Etapa 0.4 con fidelidad casi perfecta (los 5 pasos, las 4 categorías de preguntas, los 3 cubos, el entregable) — pero perdió la historia de origen que motivó crear el gate: STORY-051, el cálculo de precio medio ponderado que "nunca debió existir en esa fase", y la cita del propietario ("un teléfono descompuesto que empeora en cada paso"). El procedimiento en sí está intacto; lo que falta es el "por qué vale la pena el costo de este gate".

### ✅ Verificado sin pérdida

`mutation-gate.md`, `ui-verification-model.md`, `test-discrimination.md`, `execution-vocabulary.md` preservan la inmensa mayoría de sus reglas de origen con fidelidad alta, incluidos varios ejemplos numéricos concretos (STORY-024 `pico >= 2`, patrón de harness SVF genérico + tab/input/botón/derecha, choque `STORY-004`/`TASK-004`, `~18 minutos de CPU`). Los dos archivos de memoria referenciados (`criterio-verificable-por-grep-no-es-criterio.md`, `despacho-no-precocinar-solucion.md`) existen y no son wikilinks rotos.

---

## Plan de corrección

Alcance acotado a los 4 hallazgos reales — no se toca nada más del árbol de trabajo actual.

1. **P-1 — `brief.md`:** añadir la regla de la Orden con numeración legada (`## Cierre ejecutivo` igual se escribe) como un ítem nuevo bajo `BRIEF-3` (Salida), citando `STORY-044` como el caso real existente en el repo.
2. **P-2 — `mutation-gate.md`:** añadir a `MUTATION-GATE-5.2` (o un ítem nuevo `MUTATION-GATE-6.4`) el permiso de que el Tech-Lead complete el patrón mecánico si el ingeniero se estancó.
3. **P-3 — cinco reglas:** insertar el caso borde correspondiente (con sus números concretos) directamente en la regla que perdió su ejemplo — `EXECUTION-PROTOCOL-9`/`mutation-gate.md`, `-10.1`, `-11`, `-12`, `-30` — cortando solo la fecha/nombre de sesión, conservando el dato técnico, tal como ya hicieron bien `test-discrimination.md` y `execution-vocabulary.md`.
4. **P-4 — `coherence-sweep.md`:** añadir a `COHERENCE-SWEEP-1` (o una nota bajo el título) una versión sin fecha del origen — el riesgo real que motiva el gate (construir algo que ya estaba decidido, con el costo de deshacerlo) — sin necesariamente restaurar la cita textual si se juzga anecdótica.
5. **Verificación de cierre:** releer los 4 archivos tocados completos tras la edición, confirmar que ningún ID se duplicó, y correr `wc -m` para actualizar el campo `Tamaño` de cada uno (regla de `agents-docs-format.md`).
6. **Causa raíz (opcional, recomendado):** dejar constancia en `.agents/knowledge/debt-management.md` o como nota en `context-engineer/SKILL.md` de que este es el segundo caso del mismo patrón (N-5 → esto), para que la próxima canonización no lo repita una tercera vez.

**Dueño de la corrección:** `context-engineer` (es su dominio — edita `.agents/skills/` y `.agents/knowledge/`, y `CONTEXT-ENGINEER-3.2` es la regla que gobierna exactamente este tipo de restauración).

**No incluido en este plan:** comparar contra el informe del Auditor que corre en paralelo (no ha llegado) ni escribir la entrada en `AUDITOR-MISSES.md` — eso es un paso posterior del pipeline `ADVERSARY-P`, solo aplicable si el informe del Auditor pasa por alto alguno de estos 4 hallazgos.

## Verificación

- `grep -rn "STORY-044" .agents/knowledge/brief.md` → 1 ocurrencia tras la corrección.
- `grep -rn "cazar\|survivientes por tu cuenta" .agents/knowledge/mutation-gate.md` → 1 ocurrencia.
- Los 5 casos de P-3 citables por `grep` en su archivo destino (número/nombre de Story concreto presente).
- `wc -m` de los 4 archivos coincide con el campo `Tamaño` declarado en su cabecera.
- Ningún ID `<DOC>-N.M` se repite tras las inserciones (revisión visual, son documentos pequeños).
