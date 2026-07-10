# Plan: Reforma de Gestión de Contexto y Conocimiento (Tech-Lead)

## Contexto

El proyecto usa desarrollo dirigido por especificación con IA (spec-driven development): la calidad de los agentes depende 100% de que la base de conocimiento (`docs/`, `.claude/`) sea impecable y barata de leer. El usuario reporta varios síntomas tras semanas de trabajo:

1. **Gasto de tokens creciente y pérdida de coherencia en sesiones largas ("lost in the middle"):** el Tech-Lead carga instrucciones extensas al inicio y debe "recordarlas a rajatabla" 50+ turnos después, con el contexto saturado de resultados de subagentes y código.
2. **Documentos operativos creciendo sin la disciplina de partición que YA existe para ADRs** (`docs/ADR.md` índice + `docs/adr/ADR-XXXX.md` individual, 145 archivos, patrón probado): `PROGRESS.md` (307 líneas) y `DEBT.md` (115 líneas) narran todo en un solo archivo cada uno. `docs/execution/` (36 Órdenes) ordena mal porque el prefijo es el TIPO, no el número.
3. **Sesiones enteras perdidas en refactor** porque los agentes no reutilizaron código existente ni pensaron con criterio senior antes de construir — el skill `ponytail` (disciplina anti-sobreingeniería) existe en el repo pero no está incorporado a los skills de ingeniería.
4. **Solapamiento y ruido entre documentos de gobernanza:** no está claro qué vive en `CLAUDE.md` vs `base/SKILL.md` vs los índices/gates de cada skill; hay al menos una duplicación verbatim confirmada (protocolo de lectura progresiva). El límite entre memoria (`.agents/memory/`) y la bitácora operativa (`PROGRESS.md`) se explica hoy en tres lugares con matices distintos.
5. Un bloque "CAVEMAN" en `tech-lead/SKILL.md` que el usuario considera que no aportó valor.

**Corrección de una creencia del usuario (verificado, no requiere acción):** `.claude/memory/` NO está en `.gitignore` — el archivo tiene la línea explícita `# .claude/memory/ SÍ se versiona`, y los 18 archivos (17 memorias + índice) ya están 100% trackeados y sin cambios pendientes. Esto ya se corrigió en el commit `61701a5 chore(memory): versionar .claude/memory/`. No es parte de este plan.

**Decisión de diseño central:** en vez de delegar la autoridad del Tech-Lead a otro skill/agente (trasladaría el mismo problema a otra ventana de contexto), la causa real del "olvido en el medio" es que todo el contenido se carga de una vez al inicio y debe sobrevivir intacto 50 turnos después. La corrección aplica el mismo patrón que ya funciona para ADR/SAD: **núcleo pequeño siempre activo + archivos de detalle releídos frescos justo en el momento en que ese gate/paso se ejecuta.**

## Proceso en dos fases (restricción del usuario — gobierna TODO este plan)

- **Fase A — esta pasada, reorganización estructural pura.** Cortar y pegar contenido LITERAL a los archivos nuevos: cero reescritura, cero resumen, cero pérdida de información. Única excepción: deduplicar bloques verbatim-idénticos ya confirmados (ver punto 8) dejando un solo texto canónico y un puntero desde el otro lado. Cualquier otro contenido que "huela" a relleno, ejemplo prescindible o texto compactable se ANOTA como candidato (lista al final del plan) pero NO se toca ahora.
- **Fase B — futura, pase separado, fuera de este plan.** Una vez los flujos nuevos estén en uso real, se revisa cada archivo buscando relleno recortable. La dispara el usuario explícitamente, no ocurre automáticamente al cerrar la Fase A.

## Alcance de este plan (confirmado con el usuario)

1. Partir `tech-lead/SKILL.md` en núcleo + gates on-demand.
2. Partir `ui-designer/SKILL.md` en núcleo + referencias on-demand.
3. Extender el mismo patrón a `qa-engineer`, `flutter-engineer`, `bridge-engineer` y `architect` (los 4 con bloques extraíbles reales, según auditoría — ver punto 7). `rust-engineer`, `quant-engineer`, `refactoring-engineer` se dejan intactos: no tienen bloque aislable de tamaño relevante y partirlos sería abstracción sin necesidad real.
4. Codificar el patrón "núcleo + on-demand" como convención general en `base/SKILL.md`.
5. Partir `docs/DEBT.md` en índice + `docs/debt/DEBT-XXX.md` (mismo patrón que ADR).
6. Partir `.agents/state/tech-lead/PROGRESS.md` en índice + `.agents/state/tech-lead/progress/<entrada>.md`.
7. Renombrar `docs/execution/*.md` de `TIPO-NNN-slug.md` a `NNN-TIPO-slug.md`, y `docs/lessons/**/STORY-NNN-slug.md` al mismo esquema.
8. Retirar el bloque "CAVEMAN" de `tech-lead/SKILL.md` (confirmado: es el único de los 8 skills operativos que lo tiene).
9. Incorporar la disciplina de reutilización/simplicidad de `ponytail` como sección permanente de `base/SKILL.md` (piso mínimo siempre activo — `ponytail` sigue existiendo aparte como modo invocable de intensidad superior).
10. Deduplicar el protocolo de "lectura progresiva por offset", hoy repetido casi textualmente en `CLAUDE.md` §3 y `base/SKILL.md`.
11. Centralizar en un único lugar (`base/SKILL.md`) la frontera memoria↔PROGRESS, hoy explicada con matices distintos en tres archivos.
12. Retirar el Bloque B (despacho opencode) de `tech-lead/SKILL.md` — condicionado a que el usuario ya haya borrado `.opencode/`.
13. Corregir en `tech-lead/SKILL.md` la referencia a secciones de `ROADMAP.md` que no existen con ese título literal ("Registro de Estado" / "Descubrimientos y decisiones" → nombres reales: "Estado de las entregas de EPIC-N").

**Decisión ya resuelta (no vuelve a discutirse en este plan):** la memoria sigue GLOBAL, no se parte por skill. Razón: de las 17 memorias actuales, ninguna es exclusiva de un rol — son hechos transversales (preferencias del usuario, decisiones de producto, feedback de proceso) que cualquier skill necesita leer. Partirla en `memory/<skill>/` obligaría a decidir arbitrariamente de quién es cada hecho compartido y un skill dejaría de ver lo que quedó en la carpeta de otro. Que cualquier skill pueda leer y escribir memoria es correcto, no un defecto: permite que un hecho aprendido en una sesión de Rust esté disponible en la siguiente sesión de QA sin re-derivarlo.

---

## 1. `tech-lead/SKILL.md`: núcleo + gates on-demand

**Núcleo (`~150-170 líneas`):** Setup (sin CAVEMAN — ver punto 6), Identidad, Mecanismo de Despacho (solo Bloque A Claude Code — ver punto 9), Vocabulario de identificadores, diagrama de flujo de control, tabla-resumen de Etapas 0-7 (trigger + qué hace + qué gate leer), y "Órdenes de Trabajo" recortada a su flujo de 4 pasos.

**Gates nuevos en `.claude/skills/tech-lead/gates/` (cortados y pegados literalmente, releídos justo antes de ejecutar ese paso — no al inicio de la sesión):**

| Archivo | Contenido que se mueve (corte literal) |
|---|---|
| `gate-coherencia-pre-despacho.md` | Checklist completo "Gate de Coherencia Pre-Despacho" (Etapa 0, paso 0): stack limpio, barrido ADR completo, ADR-0020 (4 pasos), Puertos de Integración, Contrato UI+SVF, FCIS, TTRs completos, referencias huérfanas, impacto SAD, calidad de spec, modelo de tabla única ADR-0003, checks M1-M12/R1-R7/F1-F3 de ADR-0141. Es ~55% del archivo actual. |
| `gate-dod-cierre-svf.md` | Definición de Terminado (§4): SVF obligatoria, harness genérico, galería con mocks, Tres Manifestaciones de UI + Canal #2. |
| `gate-escalamiento-architect.md` | §3 completo. |
| `gate-etapa7-retroalimentacion.md` | Etapa 7 completa. |
| `gate-secuenciacion-roadmap.md` | §5 completo — con la corrección del punto 13 (nombres reales de las tablas de `ROADMAP.md`). |

**Regla de uso (en el núcleo):** "Antes de ejecutar el paso correspondiente, lee el archivo de gate con Read — aunque ya lo hayas leído antes en esta sesión. Es relectura intencional contra degradación de contexto largo, no repetición evitable."

## 2. `ui-designer/SKILL.md`: núcleo + referencias on-demand

**Núcleo (~280 líneas):** Setup, Identidad, Posición en el flujo, Pipeline Pasos 1-6 completo (incluida la plantilla exacta del Paso 4 — es el entregable, se queda), Restricciones Absolutas, Registro de Decisiones Vinculantes, Criterio de Aceptación.

**Referencias nuevas en `.claude/skills/ui-designer/reference/` (corte literal, releídas justo antes del Paso 4):**

| Archivo | Contenido que se mueve |
|---|---|
| `vocabulario-tokens.md` | Sección "🎨 REFERENCIA RÁPIDA — VOCABULARIO CANÓNICO" completa. |
| `componentes-trading.md` | Sección "🧩 COMPONENTES MÁS USADOS EN FEATURES DE TRADING" completa. |

Justificación: el Paso 2 de Setup ya obliga a leer `DESIGN.md` completo al inicio — estas secciones eran un cheat-sheet condensado del mismo contenido, cargado siempre aunque solo hace falta en el Paso 4.

## 3. Extender núcleo + gate/reference a 4 skills más

Auditoría confirmó bloques extraíbles reales (>20-38 líneas, usados en un solo paso puntual) en:

| Skill | Bloque a extraer | Archivo nuevo |
|---|---|---|
| `qa-engineer` | §1c "Revisión de Lógica de Código" (~24 líneas: checklist de 5 puntos + señales de alerta por lenguaje) | `.claude/skills/qa-engineer/gates/gate-revision-logica.md` |
| `flutter-engineer` | §2c "Biblioteca de Componentes — Contrato de Tokens" + §2d "SVF" (~40 líneas combinadas) | `.claude/skills/flutter-engineer/reference/componentes-y-svf.md` |
| `bridge-engineer` | §2c "Post-Codegen Obligatorio" (~38 líneas: bloques de código + tabla de plataformas + lección de recompilación) | `.claude/skills/bridge-engineer/reference/post-codegen.md` |
| `architect` | "Protocolo de Mantenimiento de ADRs" (~34 líneas: checklist de cierre de 6 pasos) | `.claude/skills/architect/gates/gate-mantenimiento-adrs.md` |

`rust-engineer`, `quant-engineer`, `refactoring-engineer` quedan intactos — sin bloque aislable de tamaño relevante confirmado por la auditoría.

## 4. Convención nueva en `base/SKILL.md` (patrón núcleo + on-demand)

Sección corta tras "Protocolo de Lectura Progresiva":

> **Núcleo + On-Demand (skills grandes):** cuando un `SKILL.md` supere ~250 líneas o contenga un checklist/referencia extenso usado en UN paso puntual del pipeline del rol, ese bloque se extrae a `<skill>/gates/` (checklist de un paso) o `<skill>/reference/` (tabla de consulta). El `SKILL.md` queda como núcleo permanente con instrucción explícita de CUÁNDO releer cada archivo separado. Es más fiable releer un archivo pequeño justo antes de usarlo que confiar en recordar un archivo grande leído al inicio de una sesión larga.

## 5. `docs/DEBT.md` → índice + `docs/debt/DEBT-XXX.md`

Igual que el plan original: `docs/DEBT.md` como índice (tabla `DEBT-XXX | Severidad | Una línea | Estado | Detalle`), 11 archivos nuevos en `docs/debt/` con la estructura completa actual copiada literal. Actualizar en `base/SKILL.md` §"Registrar SIEMPRE lo que difieres" la descripción del formato.

## 6. `PROGRESS.md` → índice + `progress/<entrada>.md`

Igual que el plan original: `PROGRESS.md` conserva "Estado actual" + "Reglas activas" + tabla índice de la bitácora; 17 archivos nuevos en `.agents/state/tech-lead/progress/YYYY-MM-DD-<slug>.md` con el contenido narrativo íntegro copiado literal. Las entradas NUEVAS futuras deben ser punteros de una línea a la Orden de Trabajo, reforzando la regla que el propio archivo ya se autoimpone (línea 4) y que se venía violando.

## 7. Renombrado de `docs/execution/` y `docs/lessons/`

Igual que el plan original: `NNN-TIPO-slug.md` (el identificador `STORY-024`/`BUG-013` no cambia, solo el orden de segmentos en el nombre de archivo). Procedimiento: `git mv` de los 36 archivos de `docs/execution/` + 17 de `docs/lessons/`, seguido de grep de verificación (`execution/STORY-`, `execution/BUG-`, `execution/TASK-`, `lessons/.*/STORY-[0-9]`) hasta devolver 0 resultados con el patrón viejo. `_TEMPLATE.md` no se toca (no es ticket numerado).

## 8. Retirar CAVEMAN de `tech-lead/SKILL.md`

Se elimina el bloque `### CAVEMAN` completo de la sección "SETUP: Siempre Activo" (incluye la instrucción "NO MUESTRES TU PENSAMIENTO..." y la de git). La regla de "no mostrar pensamiento" ya es comportamiento nativo de la herramienta (razonamiento interno no se expone); la de autorización git explícita para cada operación se conserva pero se reubica como viñeta normal dentro de "Identidad" o "Setup", no como bloque con nombre propio.

## 9. Disciplina Ponytail en `base/SKILL.md`

Nueva sección corta (~15-20 líneas), aplicable a los 7 skills de ingeniería, condensando la escalera de `ponytail` sin copiarlo palabra por palabra (está tuneado para un persona genérico, no para este repo):

> **Disciplina de Reutilización y Simplicidad (piso permanente, inspirado en `ponytail`):** antes de escribir código nuevo, en orden: (1) ¿hace falta que esto exista? (YAGNI — si es especulativo, no se construye); (2) ¿ya existe en `shared/`, en el crate de la feature, o en código ya escrito por otro Ingeniero? Búscalo con `grep`/Explore antes de escribir; (3) ¿la librería estándar de Rust/Dart lo resuelve?; (4) ¿una dependencia ya instalada lo resuelve?; (5) solo entonces, el código mínimo necesario. Sin abstracciones no solicitadas (interfaz con una sola implementación, config para un valor que nunca cambia, scaffolding "para después"). El modo `/ponytail` sigue disponible para intensidad superior (YAGNI extremo); esta sección es el mínimo siempre activo, no un reemplazo.

## 10. Deduplicar protocolo de lectura progresiva (`CLAUDE.md` ↔ `base/SKILL.md`)

`base/SKILL.md` §"Protocolo de Lectura Progresiva" conserva el mecanismo completo (offset=0, encadenamiento, ejemplo). `CLAUDE.md` §3 puntos 3 y 6 (que hoy restatean el mismo mecanismo con distintas palabras) se recortan a un puntero: "mecanismo exacto de encadenamiento de offsets → ver `base/SKILL.md` §Protocolo de Lectura Progresiva". Es la única deduplicación de contenido que se ejecuta en esta Fase A (verbatim-idéntica confirmada); cualquier otro solapamiento detectado (ej. delegación de barridos a subagentes, mencionada en CLAUDE.md §3.4 y en tech-lead §"Análisis de Eficiencia de Tokens" con matices distintos — no es duplicación exacta, son reglas relacionadas pero no iguales) queda anotado como candidato de Fase B, no se toca ahora.

## 11. Frontera memoria ↔ PROGRESS — un solo lugar canónico

Añadir en `base/SKILL.md` (junto a la sección de memoria/persistencia ya existente) el párrafo canónico único: "`.agents/memory/` = hechos durables, curados, entre sesiones, leíbles/escribibles por cualquier skill (user/feedback/project/reference). `.agents/state/tech-lead/PROGRESS.md` = bitácora operativa del Tech-Lead exclusivamente (qué se despachó, a qué agente, siguiente paso) — el detalle de cada trabajo vive en su Orden de Trabajo, no en PROGRESS ni en memoria." `CLAUDE.md` §4 y `tech-lead/SKILL.md` §"Memoria de Progreso" se recortan para apuntar a este párrafo en vez de restatearlo con matices propios.

---

## Candidatos de Fase B (compactación futura — NO se tocan en este plan)

- Bloque "MODOS DE ACOMPAÑAMIENTO" repetido (adaptado por rol) en los 7 `SKILL.md` de ingeniería.
- Posible solapamiento parcial entre CLAUDE.md §3.4 (delegar barridos a Explore) y tech-lead §"Análisis de Eficiencia de Tokens" (política de modelos por subagente) — relacionados, no idénticos.
- Ejemplos o prosa explicativa en cualquier archivo que un agente no necesite para actuar correctamente (a revisar archivo por archivo una vez los flujos nuevos estén validados en uso real).

---

## Verificación end-to-end

1. **Conteo de líneas post-split:** `wc -l` de cada núcleo (tech-lead ~170, ui-designer ~280, qa-engineer/flutter-engineer/bridge-engineer/architect reducidos por su bloque extraído) — todos bajan frente al original.
2. **Nada se perdió:** líneas de núcleo + suma de gates/reference nuevos ≈ líneas originales de cada skill (más encabezados nuevos). Diff manual de que el contenido movido es idéntico carácter por carácter (no reescrito).
3. **Paridad de patrón:** `ls docs/debt/ | wc -l` = 11, `ls .agents/state/tech-lead/progress/ | wc -l` = 17.
4. **Cero referencias rotas:** greps de verificación del punto 7 devuelven 0 con el patrón viejo tras el rename.
5. **Orden real corregido:** `ls docs/execution/` alfabético coincide con el orden cronológico de creación (`git log --diff-filter=A --name-only -- docs/execution/`).
6. **CAVEMAN retirado:** `grep -n "CAVEMAN" .claude/skills/tech-lead/SKILL.md` devuelve 0 resultados.
7. **Ponytail incorporado:** `grep -n "Disciplina de Reutilización" .claude/skills/base/SKILL.md` confirma la sección nueva.
8. **Dedup de lectura progresiva:** `grep -c "offset=0" CLAUDE.md` baja a 0 (o queda solo la referencia), mientras `base/SKILL.md` conserva el mecanismo completo.
9. **Sesión de humo:** invocar `/tech-lead` en una sesión nueva, confirmar que arranca solo con el núcleo (no los 5 gates de golpe) y que al llegar a cada Etapa declara explícitamente qué gate está releyendo.
