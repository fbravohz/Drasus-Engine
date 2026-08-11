# Plan — Refactorización Masiva de `.agents/`

Barrido completo de `.agents/skills/`, `.agents/knowledge/` y `.agents/templates/` para que todo documento cumpla el catálogo de formatos, más la conversión a mecanismos deterministas de lo que hoy es texto advisory incumplido.

---

## Contexto

### El bug que hay que arreglar primero

Los 13 skills legacy abren con este bloque, palabra por palabra:

> *"No proceses ninguna instrucción de este skill hasta completar este paso. Usa la herramienta Read para leer el archivo completo `.agents/knowledge/base.md`. Ese archivo contiene las reglas de rigor operativo que gobiernan este skill y tiene supremacía absoluta sobre lo que sigue."*

`base.md` tiene **7 líneas** y su contenido es un aviso de mudanza: *"[MOVIDO] base.md se dividió — lee governance.md"*.

Consecuencia operativa hoy: **todo agente que obedece su propio SKILL.md lee un stub de 7 líneas y nunca carga `governance.md`.** El gate de arranque no dispara. Y varios skills citan secciones que no existen en ningún archivo del repo (`base.md §2.2` en tech-lead, `base.md §7.4` en quant-engineer).

Son **29 referencias rotas**. `CLAUDE.md` y `knowledge/README.md` afirman que la migración a `governance.md` ya ocurrió — es falso: se hizo en `knowledge/` y nunca se propagó a los skills.

### El incumplimiento de formato

| Elemento canónico | Skills | Knowledge |
|---|---|---|
| Slug ID | 1 de 14 | 12 de 22 |
| Campo `Tamaño` | 1 de 14 | 12 de 22 |
| Bloque `<DOC>-FV` en namespace propio | 1 de 14 | 5 de 22 |

Volumen: **5,035 líneas / 434K caracteres** (~108K tokens).

Tres causas, cada una con remedio en este plan:

1. **La norma es advisory y nadie la ejecuta.** Ninguna regla de formato es verificable por máquina. Un modelo la lee, la olvida bajo presión de contexto, el siguiente agente hereda el documento roto. La tabla de arriba es la prueba.
2. **La norma solo dispara dentro de `/context-engineer`.** Cualquier otro agente que edite `.agents/` no la ve — nada está atado a la ruta del archivo.
3. **Dos plantillas de skill que se contradicen.** `.claude/skills/_TEMPLATE.md` (legacy: emojis, `## ⚙️ SETUP`, banderas 🟩🟨🟦) contra `.agents/templates/template-instruction-formats.md` (canónica: `<DOC>-N`, `Aplica a`, campos de cierre). Los 13 skills legacy siguen la primera.

### La duplicación

Bloques que viven copiados en 2+ archivos, con su caso histórico incluido:

| Bloque duplicado | Dónde está hoy | Dueño correcto |
|---|---|---|
| Gate `[ANTES DE CONTINUAR]` | 13 skills, verbatim | `governance.md` (ya existe) |
| `SETUP: base.md es ley` | 5 skills — repite el gate anterior en el mismo archivo | `governance.md` |
| Modos de Acompañamiento (4 modos) | 6 skills, verbatim | `mentorship-modes.md` (ya existe) |
| Protocolo de Lecciones | 5 skills, solo cambia la subcarpeta | `mentorship-modes.md` |
| Barrido de Coherencia / Etapa 0.4 | tech-lead L237-259 **y** architect L131-172, incluida la misma cita del propietario | doc nuevo |
| Gate de mutación + incidente 2026-07-18 | tech-lead, rust-engineer, qa-engineer | `mutation-gate.md` (ya existe) |
| Ledger: 3 pruebas companion | rust-engineer L136, qa-engineer L63 | `ledger-pattern.md` (ya existe) |
| Aserción autorreferencial / verde-trivial | rust-engineer, qa-engineer, tech-lead | doc nuevo |
| SVF vs. Galería | ui-designer L107-116, tech-lead L352 | doc nuevo |
| Gates de compilación multiplataforma | rust-engineer L57-82 (tabla), qa-engineer L71 (prosa) | doc nuevo |
| Política de Comentarios | rust-engineer la reescribe entera (24 líneas); qa-engineer solo la referencia | `commenting-policy.md` (ya existe) |

### La bitácora

Pasajes que narran un incidente con fecha, Story o cita de sesión en vez de enunciar la regla vigente: **~20 en tech-lead**, 13 en rust-engineer, 9 en flutter-engineer, 7 en architect, 5 en ui-designer, 4 en qa-engineer, 3 en quant-engineer, 3 en bridge-engineer. Lo prohíbe explícitamente `template-instruction-formats.md` § "Estabilidad de IDs": *"Nada de lenguaje de enmienda dentro de una regla. El historial vive en git, no en la prosa."*

### Defectos puntuales

- `.agents/skills/ponytail/SKILL.md` — 0 bytes, sin puente en `.claude/`, cero referencias en todo el repo. Directorio huérfano.
- `context-window-budget.md` — Slug `CONTEXT_BUDGET` con **guion bajo**, contra el ejemplo literal de la norma. Y es el único con slug que no tiene bloque FV.
- 6 docs con el FV compartiendo secuencia con las reglas (`TOOL-USAGE-3`, `CLOSING-WORK-2`, `DOC-FIRST-2`, `PROVIDER-BYPASS-2`, `PERSISTENCE-CLAIMS-2`, `MENTORSHIP-MODES-4`) en vez del namespace propio `-FV-N`.
- `ponytail.md` — tres H1 fusionados en un archivo; los encabezados "Límites" y "Salida" aparecen 3 veces cada uno sin desambiguar.
- `memory-policy.md` — cita `old_base.md`, archivo que no existe en el repo.
- `critical-domain-reasoning.md` — wikilinks sin ruta ni extensión (`[[roles-explicitos-y-subagentes]]`), no grepeables. Además: 25 líneas sin un solo `##`.
- `mutation-gate.md` — sección `2-bis`, identificador fuera de toda convención.
- 214 referencias a `.md` sueltas sin wikilink en total.

### Alcance confirmado

**Entra:** `skills/` (14) + `knowledge/` (22) + `templates/` (3) = 38 archivos.
**No entra:** `state/`, `sessions/`, `plans/`, `memory/` — son datos, bitácora y ledgers append-only, no reglas. `memory/` además lo excluye `CLAUDE.md` §4.

---

## Decisiones de arquitectura

### D1 — Hook determinista de formato

`CLAUDE-CODE-MECHANISMS-6.1` manda resolver primero qué es mecánicamente verificable. Va a `.agents/hooks/validate-agent-docs.py`, disparado por `PostToolUse` sobre `Edit|Write` en `.agents/**/*.md`. Usa `python3` — `jq` no está instalado aquí.

**Bloquea (exit 2):** campo `Tamaño` desfasado contra `wc -m`; ID de 3 segmentos; Slug ausente o no calcado del archivo; `N/A` en campo de cierre; campo `Documento extendido` (retirado); referencia a `base.md`.

**Avisa (exit 0 + mensaje):** ruta `.md` sin wikilink; fecha o lenguaje de enmienda en el cuerpo; bloque `<DOC>-FV` ausente.

Es lo único que vuelve la norma inevadible. El estado actual (1 de 14) es la medición de qué garantiza el texto advisory solo.

### D2 — Rule por ruta

`CLAUDE-CODE-MECHANISMS-6.2`: la norma debe regir aunque nadie invoque `/context-engineer`. Va a `.claude/rules/agents-docs-format.md` con `paths: [".agents/**/*.md", ".claude/skills/**/*.md"]` — resumen corto del formato, no el catálogo completo (ese sigue siendo lectura bajo demanda).

### D3 — Descarga de skills hacia knowledge

Cada bloque duplicado sale del skill y queda como `Lectura bajo demanda` apuntando al knowledge dueño. Knowledge docs nuevos que se crean:

- `coherence-sweep.md` — el Barrido de Coherencia Pre-Orden, hoy duplicado entre tech-lead y architect.
- `test-discrimination.md` — demostración rojo/verde, aserción autorreferencial, prueba verde-trivial. Hoy triplicado.
- `ui-verification-model.md` — SVF vs. Galería, contrato de integración UI (ADR-0117). Hoy repetido 4 veces dentro de tech-lead más otras 5 en ui-designer.
- `cross-platform-rust.md` — gates de compilación por plataforma y targets. Hoy duplicado rust/qa.
- `execution-vocabulary.md` — EPIC/SPRINT/STORY/SPIKE/BUG/TASK y numeración global. Hoy triplicado.

### D4 — Purga de bitácora

Ningún documento conserva fechas, enmiendas, "antes decía X", ni nombres de sesión en el cuerpo. **La regla se preserva íntegra; se corta la anécdota que la originó, no el contenido normativo** (`CONTEXT-ENGINEER-3.2` prohíbe cortar dato técnico o caso borde). Esa historia ya vive en git.

### D5 — Reescritura completa autorizada

Los 13 skills legacy no tienen IDs publicados que preservar. Se reescriben enteros bajo la excepción de `template-instruction-formats.md` § "Estabilidad de IDs". Los knowledge docs que ya tienen IDs se editan quirúrgicamente, respetando append-only.

Excepción: los 6 docs cuyo FV comparte secuencia con las reglas (`TOOL-USAGE-3` → `TOOL-USAGE-FV-1`) sí requieren renumerar. Se hace como reescritura completa autorizada, corrigiendo las referencias entrantes en el mismo movimiento.

### D6 — `_TEMPLATE.md` legacy

`.claude/skills/_TEMPLATE.md` se reescribe para producir el formato canónico. Hoy es la fuente de la que salieron los 13 skills rotos: mientras exista, cualquier skill nuevo nace incumpliendo.

### D7 — Modo de escritura

Todo archivo se escribe en **Modo Gemini Relajado + Listas** (`WRITE-LIKE-GEMINI-1.3`), por orden del usuario.

---

## Ejecución

### Fase 0 — Reparar el gate roto (bloqueante, va primero)

1. Reemplazar las 29 referencias a `base.md` por `governance.md` en los 13 skills, `docs/README.md`, `docs/ROADMAP.md`, `docs/lessons/README.md` y las órdenes de `docs/execution/` que la citan.
2. Resolver las citas a secciones muertas (`base.md §2.2` → `GOVERNANCE-3.4`; `base.md §7.4` → `doc-first.md`).
3. Borrar `.agents/skills/ponytail/` (vacío, huérfano).
4. Borrar `base.md` una vez sin referencias entrantes.

Verificación: `grep -rn "base\.md" .` devuelve cero.

### Fase 1 — Determinismo

5. `.agents/hooks/validate-agent-docs.py` + entrada `PostToolUse` en `.claude/settings.json`.
6. `.claude/rules/agents-docs-format.md`.
7. Correr el hook en modo auditoría sobre los 38 archivos para tener la línea base de violaciones.

### Fase 2 — Knowledge (22 archivos)

8. Crear los 5 knowledge docs nuevos de D3, absorbiendo el contenido duplicado.
9. Retrofit de los 16 sin formato canónico: Slug, `Tamaño`, `<DOC>-FV`, IDs, `Aplica a`, wikilinks.
10. Renumerar el FV de los 6 docs con namespace equivocado.
11. Corregir `CONTEXT_BUDGET` → `CONTEXT-WINDOW-BUDGET` y añadirle FV.
12. Partir `ponytail.md` (3 H1 fusionados) y eliminar su duplicación con `commenting-policy.md`.
13. Actualizar `knowledge/README.md` con las filas nuevas.

### Fase 3 — Skills (14 archivos)

14. Reescribir `_TEMPLATE.md` al formato canónico.
15. Reescribir los 13 skills: purga de bitácora, descarga de bloques duplicados a `Lectura bajo demanda`, formato canónico completo.
16. Verificar que los puentes `.claude/skills/*/SKILL.md` siguen apuntando correcto.

### Fase 4 — Cierre

17. Correr el hook sobre los 38: cero violaciones duras.
18. Actualizar `CLAUDE.md` §2 con el mapa nuevo.
19. Commits agrupados por tipo.

---

## Delegación

Los knowledge docs pequeños y los skills medianos van a Sonnets con la norma ya extraída en el prompt, en lotes de 3-4 archivos. Yo hago directamente: la Fase 0 completa, el hook, el Rule, `_TEMPLATE.md`, `tech-lead/SKILL.md` (85K, el más cargado) y la verificación final de cada lote devuelto.

---

## Estado — dónde retomar

**Hecho y commiteado (13 commits sobre `2df8665`):** Fases 0, 1 y 2 completas.

- Gate de arranque reparado: 29 referencias a `base.md` redirigidas, 2 punteros a secciones muertas remapeados, `skills/ponytail/` borrado.
- Hook `validate-agent-docs.py` activo en `PostToolUse` + `.claude/rules/agents-docs-format.md`.
- `.claude/skills/_TEMPLATE.md` reescrito y gate movido a los 13 puentes.
- Los 25 knowledge docs en cero violaciones duras, incluidos 3 nuevos: `coherence-sweep`, `test-discrimination`, `ui-verification-model`.
- Dos contradicciones resueltas por decisión del propietario: memoria suspendida (`MEMORY-POLICY-14`) y ficha DEBT unificada en la estructura completa.

**Sin commitear, en el worktree** (la autorización de commits fue retirada). Todo validado en cero violaciones duras:

- `.agents/knowledge/ui-verification-model.md`, `cross-platform-rust.md`, `execution-vocabulary.md` (nuevos)
- `.agents/knowledge/README.md` (5 filas nuevas)
- `.agents/skills/partner/SKILL.md`, `refactoring-engineer/SKILL.md`, `general-counsel/SKILL.md` (reescritos al canon)
- `.agents/plans/refactor-masiva-agents.md` (este archivo)

Con eso, **los 5 knowledge nuevos de D3 están completos** y la Fase 2 cierra al 100%: 27 documentos, cero violaciones duras.

**Pendiente — 9 skills.** 18 violaciones duras, dos por archivo (Slug + `Tamaño`), más la purga de bitácora y la descarga de bloques a knowledge.

Orden para retomar:

1. `social-strategist` — no depende de ningún knowledge; es el que quedó suelto del lote independiente. Ojo: no sigue el esqueleto de rol de ingeniero (sin Modos ni Mandato; es un pipeline con plantillas de salida incrustadas como encabezados `##`, que hoy contaminan su índice).
2. Ingenieros: `rust`, `qa`, `flutter`, `bridge`, `quant`. Citan `test-discrimination`, `cross-platform-rust`, `commenting-policy`, `mentorship-modes`, `ledger-pattern`, `mutation-gate`.
3. Orquestación: `architect` y `ui-designer`. Citan `coherence-sweep`, `ui-verification-model`, `execution-vocabulary`.
4. `tech-lead` al final.

**Dos residuos ya corregidos que conviene buscar en los 9 restantes:** varios skills apuntan la Política de Comentarios y el Protocolo de Lecciones a `governance.md`, cuando sus dueños son `commenting-policy.md` y `mentorship-modes.md`. Es daño colateral del reemplazo masivo de `base.md`: el destino correcto depende del tema, no es `governance.md` para todo. También apareció una referencia a `contracts.py` en `partner` — residuo de la era Python que el stack prohíbe; ya remapeada a los puertos tipados de ADR-0137.

**`tech-lead/SKILL.md` merece tratamiento propio.** Con 85K caracteres es el más grande y el más cargado (~20 pasajes de bitácora). Sus dos problemas estructurales:

- El "Pipeline de Ejecución" (L218-332, las 8 etapas) usa **negrita en vez de encabezados**, así que la sección más larga del archivo no aparece en ningún índice. Es candidata a knowledge doc propio (`dispatch-pipeline.md`), dejando en el skill la identidad, el mandato y la referencia.
- El "Gate de Coherencia Pre-Despacho" (L151-176) son ~10 ítems de 8-20 líneas cada uno, con regla, justificación y caso histórico entremezclados. También es candidato a knowledge propio.

---

## Verificación

- `grep -rn "base\.md" .` → cero.
- `python3 .agents/hooks/validate-agent-docs.py --audit .agents/` → cero violaciones duras en los 38.
- `grep -rnoE '[A-Z-]{4,}-[0-9]+\.[0-9]+\.[0-9]+' .agents/` → cero IDs de 3 segmentos.
- Cada `[[wikilink]]` apunta a un archivo que existe (verificable por script).
- Cada skill declara `governance.md` como Lectura Obligatoria `SIEMPRE`.
