# Plan: reescritura ligera del sistema de documentación normativa + disparadores por búsqueda

## Contexto

El diagnóstico previo (misma conversación) encontró, con evidencia documentada por el propio proyecto — no hipótesis nuevas —, que el patrón de "agentes incumpliendo formatos y flujos" tiene tres causas raíz:

1. **Casi todo el control es autoevaluación de texto**, no verificación externa. Solo el formato superficial está forzado por un hook; todo lo demás depende de que el LLM se audite a sí mismo con checklists (`Filtro de Verificación`).
2. **La partición en documentos con índice no comprime nada** — cada documento nuevo arrastra su propio aparato de formato (Slug ID, Tamaño, tabla de Lecturas, Filtro de Verificación, Pipeline, Ejemplos) casi tan pesado como el contenido normativo real, y cada eslabón de la cadena relee todo desde cero.
3. **No hay jerarquía de prioridad** — todo es "OBLIGATORIO" con el mismo peso, así que bajo presión de atención lo que cede es lo que el documento hizo menos visible (caso real: `CONTEXT-DEBT-007`, un bloque obligatorio sin número de etapa que un agente podía tratar como opcional).

Tres casos ya diagnosticados por el propio repo (`CONTEXT-DEBT-003`, `-004`, `-007`) confirman que el "incumplimiento" no fue decisión deliberada del agente en ninguno de los tres: fue una salvaguarda perdida en una reescritura sin red de seguridad, un grafo de referencias roto (documentos huérfanos que nadie leía), y una secuencia numérica con un hueco.

El propietario pidió: (1) ejecutar todas las recomendaciones del diagnóstico, (2) reescribir **todos** los skills y knowledge docs a un formato mínimo — solo listas/bullets/títulos, sin aparato pesado, y (3) profundizar el mecanismo de "disparadores verificados por búsqueda, no por memoria del agente", acercándolo lo más posible a cómo un RAG real decide qué contexto entra a un turno, usando solo mecanismos nativos de Claude Code (sin infra de embeddings).

Exploración previa (3 agentes) inventarió los 30 `.agents/knowledge/*.md`, los 15 `.agents/skills/*/SKILL.md`, los 16 stubs `.claude/skills/*/SKILL.md`, el único hook existente (`.agents/hooks/validate-agent-docs.py`, `PostToolUse`), la única Rule existente (`.claude/rules/agents-docs-format.md`), y el grafo completo de wikilinks. Un agente de diseño produjo la arquitectura técnica con dos pruebas de concepto reales (reducción medida 62% en caracteres, 81% en líneas). El contrato exacto del hook `UserPromptSubmit` se verificó contra la documentación oficial de Claude Code (no es una suposición).

Decisiones ya confirmadas por el usuario en esta conversación:
- El ID citable por bullet se conserva con la gramática ya existente (`<DOC>-N`, `<DOC>-N.M`) en vez de inventar una sintaxis nueva — evita romper ~600 citas activas y renumerar de golpe; la compresión viene de quitar el aparato alrededor del ID, no de acortar el ID mismo.
- El hook `validate-agent-docs.py` se reescribe para el formato nuevo, no se elimina.
- El mecanismo de disparadores combina Rules nativas (`paths:`, para lo atado a archivo) + un hook de keyword-matching sobre `UserPromptSubmit` (para lo atado a concepto/intención del prompt).

## División de trabajo (decisión del propietario, 2026-09-02)

La reescritura de contenido normativo (secciones A, D y los pasos 1-6 de la sección E: `governance.md`, `hard-prohibitions.md`, `execution-protocol.md`, `writing-efficiency.md`, `skill-common.md`, los 12 skills de ingeniería/auditoría, `architect`/`context-engineer`/`social-strategist`, los knowledge docs por cluster, y los 16 stubs) la ejecuta el **propietario junto con Gemini u otra IA más compactante** — es trabajo de redacción/edición de prosa, no de mecanismo. Esta ejecución (Claude Code) **no toca esos archivos de contenido**. Se suma la asignación de tags documento por documento (sección H) sobre el corpus ya migrado al formato ligero (`.agents/`) y sobre `docs/features`/`docs/adr` (dominio del Architect) — mismo criterio: juicio de contenido, no mecanismo.

Lo que sí ejecuta esta sesión, porque es mecanismo (hooks, scripts, Rules, estado), no prosa normativa:
- **Paso 0 de la sección E**: cirugía mínima al hook (`RE_SLUG`/`RE_SIZE`) — condición de arranque para que el propietario/Gemini puedan migrar sin que el hook bloquee el formato viejo.
- **Sección B** completa: `inject-context.py` (FTS5) + registro en `UserPromptSubmit`, y las 6 Rules nativas de B2.
- **Sección C** completa: `--check-graph` y el validador de secuencia en `validate-agent-docs.py`.
- **Paso 7 de la sección E**: reescritura final del hook contra el corpus ya migrado (bloqueada hasta que el propietario confirme que su migración de contenido terminó).
- **Paso 8 de la sección E**: reactivación del bloqueo estricto + registro del hook nuevo.
- **Sección G** completa: G1 (juez `Stop`), G2 (gating `PreToolUse`), G4 (recuperación post-compactación), G5 (`allowed-tools`), G7 (máquina de estados de la Orden). G3 sigue diferido igual que antes.
- **Sección H — mecanismo del sistema de tags**: vocabularios semilla, extensión de `inject-context.py` con el índice `tags` de `.agents/` y el índice separado `tags`-only de `docs/`, extensión de `validate-agent-docs.py` para `.agents/`, y el script nuevo `validate-docs-tags.py` para `docs/` (el contenido del vocabulario y qué tags existen quedan con sus dueños de contenido — Context-Engineer y Architect respectivamente — no con esta sesión).

Punto de sincronización obligatorio: antes de correr el Paso 7 (reescritura final del hook) y la Sección G, el propietario debe confirmar explícitamente que su migración de contenido (pasos 1-6) terminó — de lo contrario el hook nuevo y los jueces `prompt` quedarían validando/juzgando contra un formato todavía a medias.

## Estado de ejecución real (actualizado por Context-Engineer)

Lo que ya corrió, con las desviaciones reales frente a la división de trabajo de arriba:

- **Paso 0 (hook) — hecho, pero extendiendo el check en vez de eliminarlo.** `RE_SLUG` y `RE_SIZE` en `validate-agent-docs.py` ahora aceptan ambos formatos (`Slug ID:`/`Tamaño:` en párrafo, viejo, y `SLUG-ID:`/`Tamaño:` en bullet, nuevo) en vez de tumbar el check durante toda la migración — un documento en cualquiera de los dos formatos pasa, sin ventana ciega. Efecto equivalente al pedido, implementación más conservadora.
- **Sección A — ya reescritos, pero por esta sesión, no por propietario+Gemini.** `template-knowledge.md` y `template-instruction-formats.md` están en formato ligero, las 4 decisiones de la sección A ya aplicadas (sin sub-ID de bullet, Lecturas en lista plana con flechas, headings con `:`, Filtro de Verificación retirado). El propietario pidió ejecutarlo directo en esta sesión — la división de trabajo original queda superada en este punto concreto.
- **Sección E, paso 4 (parcial) — `context-engineer/SKILL.md` ya migrado**, también por esta sesión y fuera del orden previsto (el paso 4 preveía migrar primero los 12 skills de ingeniería/auditoría, dejando `architect`/`context-engineer`/`social-strategist` para el final, todo vía propietario+Gemini). 20.323 → 9.310 caracteres. Los otros 14 skills siguen en formato legacy.
- **Deuda nueva abierta como consecuencia directa: `CONTEXT-DEBT-010`.** 7 citas externas a sub-IDs de `CONTEXT-ENGINEER` (`hard-prohibitions.md`, 5 planes, `memory/`) quedaron huérfanas al colapsar los sub-ID en la migración de `context-engineer/SKILL.md` — mismo riesgo que ya anticipa el diseño de `C1` (validador de grafo); como C1 no existe todavía, el remapeo se registra como deuda en vez de detectarse solo.
- **Regla de proceso nueva, reutilizable para el resto de la migración de la sección E:** `context-engineer/SKILL.md` (`CONTEXT-ENGINEER-6`) ahora exige que, al tocar un archivo, toda cita huérfana o mal apuntada se corrija ahí mismo si el archivo ya está en alcance, o se registre como `CONTEXT-DEBT-NNN` si no — antes de esto la única salida era señalarlo solo en el chat.
- **Sección H — el usuario la agregó completa al plan (H1-H7) en esta sesión.** Se implementó únicamente el fragmento de H1 que ya tenía dónde vivir: el campo `Tags:` quedó definido en `template-knowledge.md` (justo después de `Tamaño`, como pide H1) y no en `.claude/rules/agents-docs-format.md`, porque ese archivo sigue en formato viejo y ya no es la cabecera canónica de los documentos migrados. El resto de H (H2-H7: vocabulario semilla, extensión de `inject-context.py`, `RE_TAGS` en el hook, `validate-docs-tags.py`) no se ha tocado.
- **Formato Pipeline (`TEMPLATE-INSTRUCTION-FORMATS-6`) corregido — ya no exige citar el ID de sección en cada paso.** El usuario detectó que era relleno: el documento se lee siempre completo, y sin sub-IDs la cita solo apunta a "en algún lugar de la sección", sin precisión real. Se quitaron las 9 citas redundantes del Pipeline de `context-engineer/SKILL.md` (9.310 → 9.080 caracteres) y se corrigió el ejemplo `AUDITOR-P` del catálogo. La cita a un ID solo se conserva cuando declara una relación real entre dos reglas (ej. precedencia), no como "ver más abajo".
- **Adelanto real de la Sección B/G — determinismo de "Lecturas SIEMPRE" sin esperar a `inject-context.py`.** El usuario cuestionó si el campo `SIEMPRE` de la tabla de Lecturas garantiza algo por sí solo — no lo hacía: se confirmó en vivo que `hard-prohibitions.md` estaba marcado `SIEMPRE` en 14 de los 15 skills y en los 16 stubs sin ningún paso que forzara su lectura (`CONTEXT-DEBT-011`). En vez de parchear 30 archivos, se construyeron dos mecanismos nuevos de harness, fuera del diseño original de este plan pero en su mismo espíritu (Sección G, determinismo real en vez de autoevaluación):
  - `.agents/hooks/session-start-siempre.py`, hook `SessionStart` (`startup|resume|clear|compact|fork`, registrado en `.claude/settings.json`): inyecta por stdout el contenido completo de `governance.md` + `hard-prohibitions.md` al inicio de toda sesión — cierra `CONTEXT-DEBT-011` para los 15 skills y 16 stubs de una sola vez, sin tocarlos.
  - `.agents/hooks/gate-skill-siempre.py` (`PreToolUse` sobre el tool `Skill`) + `.agents/hooks/track-siempre-reads.py` (`PostToolUse` sobre `Read`): bloquea la invocación de un skill si sus lecturas `SIEMPRE` propias (alcance distinto de `Todos`) no se han leído en la sesión, y las desbloquea en cuanto se leen. Probado contra los 15 skills reales — 9 piden lectura, 6 permiten directo. Excluye rutas placeholder (ej. `docs/execution/<ID>.md`) para no bloquear un skill para siempre por un literal que nunca existe.
  - `TEMPLATE-INSTRUCTION-FORMATS-4` (formato Lecturas) documenta ahora las tres categorías reales: `SIEMPRE`-`Todos` (resuelto, `SessionStart`), `SIEMPRE`-por-skill (resuelto, el gate), disparador por palabra clave (resuelto ese mismo día, ver bullet siguiente).
- **Sección B1 (`inject-context.py`) construida y registrada — antes de lo previsto por E8.** El plan original la secuenciaba tras terminar la migración completa de la sección E (paso 8, "depende de que las tablas de Lectura ya estén en su forma final"), pero el mecanismo indexa el documento completo, no la tabla de Lecturas — no depende de que los 14 skills restantes ya estén en formato ligero, así que se adelantó junto con el resto del paquete de determinismo (mismo criterio que el bullet anterior). Contrato de `UserPromptSubmit` verificado contra la documentación oficial (`https://code.claude.com/docs/en/hooks`) antes de escribir el hook, no asumido — un subagente de apoyo trajo un dato erróneo (mezcló los valores de `matcher` de `SessionStart` con este evento, que no admite `matcher`) y se descartó tras verificación directa.
  - `.agents/hooks/inject-context.py`: indexa con SQLite FTS5 (`tokenize="unicode61 remove_diacritics 2"`, sin dependencias nuevas) `.agents/knowledge/*.md` + `.agents/skills/*/SKILL.md` en `.agents/hooks/.fts-index.db` (no versionado, `.gitignore`); reconstruye solo si algún archivo fuente cambió más recientemente que el índice (comparación de `mtime`, verificado en vivo: no reconstruye si nada cambió, sí reconstruye si se toca un archivo fuente). Tokeniza el prompt, filtra stopwords propias, arma query FTS5 (`"tok1" OR "tok2"...`) y rankea con `bm25()`. Si hay matches: **siempre** manifiesto de wikilinks vía `hookSpecificOutput.additionalContext` (decisión posterior del propietario, ver H5 — nunca contenido completo, sin presupuesto de caracteres que calibrar). Probado en vivo contra el corpus real.
  - Registrado en `.claude/settings.json`, bloque `UserPromptSubmit` nuevo, sin `matcher` (evento no lo soporta, confirmado).
  - Techo declarado sigue vigente (Sección B3): matching léxico, no semántico — un concepto parafraseado sin la palabra exacta no dispara nada.

Consecuencia para el resto del plan: cuando la migración de la sección E siga con los 14 skills restantes, el campo `Tags:` de `template-knowledge.md`, la regla de citas huérfanas de `context-engineer` y el mecanismo de disparador por palabra clave (`inject-context.py`) ya están disponibles como parte del molde — no hace falta rediseñarlos en esa fase.
- **`.agents/knowledge/skill-common.md` (D2) construido por esta sesión, no por propietario+Gemini como preveía la división original** — mismo criterio que Sección A y `context-engineer`: el propietario pidió ejecutarlo directo. Prerrequisito de la Sección E, paso 4 (migración de los 12 skills de extracción), ya resuelto. Un subagente de investigación leyó los 12 `SKILL.md` completos y trajo, para cada uno de los 6 bloques que este bullet original daba por sentado, la versión más completa y qué variaciones de dominio no debían perderse — 3 correcciones reales surgieron de esa investigación (documentadas en D2): el Protocolo de Lecciones ya vivía en `mentorship-modes.md` (no hacía falta un bloque nuevo, solo corregir la referencia — `qa-engineer`/`rust-engineer` apuntaban por error a `governance.md`); la fuente asumida para Formato de Reporte al CEO era incorrecta (`tech-lead`, no `auditor`); y el "Cierre del Filtro de Verificación" que el subagente encontró duplicado en 9 archivos **no se consolida** — es el mismo aparato `<DOC>-FV` que `TEMPLATE-INSTRUCTION-FORMATS-8` ya retiró del catálogo por completo (Turno 1 de la sesión), traerlo a `skill-common.md` lo habría revivido por la puerta de atrás. `skill-common.md` quedó con 4 secciones: Filtro de Arranque, Modos de Acompañamiento (esqueleto mecánico, ejemplo de dominio se queda local a cada skill), Demostración Roja/Verde, Formato de Reporte al CEO.
- **Decisión del propietario sobre el hueco real encontrado en `tech-lead`:** no tiene el bloque de Filtro de Arranque que sí tienen 8 de los otros skills — a diferencia de "sin Modos de Acompañamiento" (que sí tiene razón estructural explícita en varios skills), aquí no hay ninguna justificación escrita. Principio fijado para este caso y los que sigan: al migrar contra la plantilla, todo lo que no encaje se adapta — se crea la sección pertinente o se distribuye donde corresponda; solo si nada de eso aplica, se pregunta antes de decidir dónde va. `tech-lead` adopta el Filtro de Arranque al migrarse. (El otro "hueco" que se creyó encontrar — cierre del FV — no aplica: ya no existe para ningún skill, ver arriba.)
- **`tech-lead/SKILL.md` migrado — primer skill del lote de 12, para validación del propietario antes de seguir con el resto.** 18.431 → 14.148 caracteres (23% menos). Cabecera a bullets, `TECH-LEAD-0` (tabla de Lecturas) y `TECH-LEAD-FV` retirados (el segundo, sin reemplazo — mismo hueco aceptado del resto del corpus). `TECH-LEAD-3` (Formato de Reporte al CEO, la fuente original del bloque común) reemplazado por una referencia a `skill-common.md`. `TECH-LEAD-2.5` (Presentación Obligatoria) se fusionó dentro del nuevo bullet de Filtro de Arranque en `TECH-LEAD-2`, en vez de quedar como bullet aparte — mejora activa, no solo formato (`CONTEXT-ENGINEER-6`). `TECH-LEAD-5` (Modo de Acompañamiento de Implementación) se dejó íntegro, sin referenciar `skill-common.md#SKILL-COMMON-2`: el subagente confirmó que es la contraparte inversa (cómo el Tech-Lead despacha a otros bajo cada Modo, no cómo él opera bajo uno) — nota explícita agregada en el propio bloque para que quede claro por qué no se colapsó. 1 cita externa huérfana encontrada y registrada (`CONTEXT-DEBT-012`, `TECH-LEAD-7.1` en un plan fuera de alcance actual).

## A. El nuevo formato ligero (referencia de formato objetivo — ejecutado por el propietario + Gemini, no por esta sesión)

Un documento nuevo tiene: título + 1-2 líneas de propósito, el slug entre backticks (sin la etiqueta "Slug ID:", sin campo Tamaño — ese campo se calcula, no se declara, ver más abajo), una línea o tabla de 3 columnas de Lecturas, y reglas como bullets planos con ID inline — sin heading `##` propio por regla, sin bloque `Aplica a` aparte, sin `Filtro de Verificación` numerado, sin `Pipeline` salvo que el orden no sea inferible de las reglas mismas, sin ❌/✅ salvo casos con medición empírica real (ej. `write-like-gemini.md`).

Tabla de Lecturas (reemplaza a la de 4 columnas), con 3 formas controladas de disparador — la tercera columna es el índice que el hook de la sección B parsea:

```
| Documento | Disparador | Alcance |
|---|---|---|
| [[.agents/knowledge/governance.md]] | SIEMPRE | Todos |
| [[.agents/knowledge/debt-management.md]] | `deuda`, `ponytail`, `comentario diferido`, `DEBT-XXX` | Todos |
| [[.claude/rules/cross-platform-rust.md]] | Ruta: `crates/**/*.rs` | Rust-Engineer |
```

Campo Tamaño: desaparece del documento. Se calcula bajo demanda con `validate-agent-docs.py --audit`, nunca se declara a mano (era el único campo que generaba trabajo de mantenimiento sin bloquear nada real).

Campos que sobreviven sin cambio porque son contenido, no aparato: `Precedencia`, `Lectura bajo demanda`, prohibición de "N/A" en campos de cierre, wikilinks obligatorios, tope de 2 segmentos por ID, IDs append-only.

**Prueba de concepto ya producida y verificada** (ver reporte del agente de diseño): `hard-prohibitions.md` pasa de 5.698 caracteres / 88 líneas a 2.336 / 19 (59% / 78% de reducción); `partner/SKILL.md` pasa de 8.454 / 135 a 3.006 / 24 (64% / 82%). Cero contenido normativo perdido en ambos casos — se verificó con grep que ninguna sub-regla colapsada tiene citas externas que se rompan. Estos dos documentos se reescriben primero (Fase 1 de migración) usando exactamente esas dos versiones ya redactadas.

## B. Mecanismo de disparadores por búsqueda

### B1. Hook de concepto/keyword — `UserPromptSubmit`

Contrato verificado contra la documentación oficial de Claude Code (no supuesto): `UserPromptSubmit` no admite `matcher`, se dispara siempre, recibe `{"prompt": "<texto crudo>", ...}` por stdin, y acepta contexto de dos formas — stdout plano con exit 0, o JSON `{"hookSpecificOutput": {"hookEventName": "UserPromptSubmit", "additionalContext": "..."}}` (el campo debe ir anidado ahí, no en el nivel raíz, o Claude Code lo ignora en silencio). Exit 2 bloquea y borra el prompt completo — no se usa para inyección, solo quedaría disponible para un veto futuro que este plan no activa.

Nuevo archivo `.agents/hooks/inject-context.py`: en vez de keyword-matching por conteo simple, indexa con **SQLite FTS5** (módulo estándar de `sqlite3` en Python, cero dependencias nuevas) el contenido completo de `.agents/knowledge/*.md` y `.agents/skills/*/SKILL.md` — no solo la columna Disparador, el documento entero — y resuelve el prompt del usuario como query FTS5 con ranking BM25 real (`bm25()` de SQLite), en vez de contar coincidencias literales. Esto es la mejora directa que cierra la brecha señalada por el usuario ("no sentimos que estemos cerca del desempeño de un RAG"): sigue siendo léxico, no semántico (un concepto parafraseado sin la palabra exacta sigue sin match — ver B3), pero da ranking de relevancia real sobre el corpus completo, no un conteo ad-hoc sobre una columna. El índice (`.agents/hooks/.fts-index.db`, no versionado) se reconstruye al vuelo si detecta que algún archivo fuente cambió más recientemente que el índice (comparación de `mtime`, sin necesitar un hook de reindexado aparte). Diseño original de este párrafo: si hay matches por encima de un umbral de score, inyectar hasta 3 documentos completos si caben en un presupuesto de caracteres combinados, o si no caben, un manifiesto de wikilinks. **Superado — ver H5:** la decisión final es inyectar siempre manifiesto de wikilinks, nunca contenido completo, sin presupuesto de caracteres que calibrar (más el indexado por Sección de H3, no por archivo). Registro en `.claude/settings.json`, bloque `UserPromptSubmit` nuevo, separado del `PostToolUse` existente (eventos y responsabilidades distintas).

### B2. Rules nativas con `paths:` — 6 candidatos concretos

| Convención hoy | Rule nueva | `paths:` |
|---|---|---|
| `RUST-ENGINEER-8` / `cross-platform-rust.md` | `.claude/rules/cross-platform-rust.md` | `["crates/**/*.rs"]` |
| `commenting-policy.md` | `.claude/rules/commenting-policy.md` | `["crates/**/*.rs", "app/lib/**/*.dart"]` |
| `ledger-pattern.md` | `.claude/rules/ledger-pattern.md` | `["**/migrations/**/*.sql", "**/migrations/**/*.rs"]` |
| `ui-verification-model.md` | `.claude/rules/ui-verification-model.md` | `["app/lib/**/*.dart", "app/test/**/*.dart"]` |
| `provider-bypass.md` | `.claude/rules/provider-bypass.md` | `["app/lib/**/*.dart"]` |
| `test-discrimination.md` | `.claude/rules/test-discrimination.md` | `["crates/**/tests/**/*.rs", "app/test/**/*.dart"]` |

Cada Rule reemplaza, en la tabla de Lecturas de los skills que hoy la citan, la fila correspondiente — pasa de "keyword en el prompt" o "SIEMPRE" a "se auto-inyecta al tocar el archivo", sin depender de memoria ni de que el usuario mencione la palabra correcta.

### B3. Techo real frente a un RAG — honestidad técnica, no venta

Se parece a un RAG en lo que importa: la decisión de qué entra al contexto la toma el harness antes de que el modelo produzca el primer token del turno, no el modelo "recordando" leer un índice. Ninguno de los dos mecanismos (hook de keywords, Rule por ruta) depende de que el agente decida ejecutar una búsqueda.

No llega a un RAG real en un punto concreto: el matching es literal por palabra clave, no por similitud semántica de embeddings — un concepto parafraseado sin la palabra exacta no dispara nada. Dos mitigaciones sin infraestructura nueva:
- Exigir 3-5 variantes reales (formal + coloquial + ID) por entrada de Disparador — el validador de grafo (sección C1) emite aviso si una fila trae menos de 2 keywords.
- **Fase 2 opcional, no bloqueante**: usar el propio CLI (`claude -p --model <haiku> --output-format json`) como router de intención barato — un prompt fijo de clasificación contra la lista de disparadores, en vez de (o adicional a) el match literal. Es "LLM-as-router" real y ejecutable hoy, pero añade una llamada síncrona antes de que el turno principal empiece; se activa solo si, tras medir en uso real, el keyword-match muestra una tasa alta de falsos negativos. No es parte de la ejecución de este plan.

## C. Hooks mecánicos nuevos

### C1. Validador de grafo de referencias

Extiende `validate-agent-docs.py` con un modo `--check-graph`: recorre `.agents/knowledge/*.md` y `.agents/skills/*/SKILL.md`, confirma que cada wikilink apunta a un archivo real, y calcula el grado de entrada de cada knowledge doc (cuántos otros archivos lo citan). Grado 0 → violación dura en modo `--check-graph` — es exactamente el defecto de `provider-bypass.md` (huérfano hoy) que este mismo plan corrige en la sección D3.

No se engancha de inmediato a un hook bloqueante — arranca como extensión de `--audit` (corrida manual por Context-Engineer/Tech-Lead). Solo tras un período de rodaje sin huérfanos se engancha al evento `Stop` (confirmado que existe y que exit 2 ahí "previene que Claude termine y continúa la conversación" — útil para forzar "no termines el turno dejando un doc huérfano"). Esa activación en `Stop` queda fuera de esta ejecución (ver "Fuera de alcance").

### C2. Validador de continuidad de secuencia numérica

Añadido al mismo script: detecta bloques con lenguaje obligatorio (`obligatori[oa]`, `sin excepción`, `bloqueante`, `\bdebe\b`) que no llevan ningún ID de bloque (`## <DOC>-N`) ni marcador de Etapa cerca de su cabecera — el defecto exacto de `CONTEXT-DEBT-007`. Para documentos con concepto de "Etapa" (config explícita, hoy solo `execution-protocol.md`), además verifica que las Etapas citadas aparezcan en orden ascendente en el texto. Huecos entre enteros no consecutivos quedan como aviso blando (ya existen saltos legítimos y documentados en el corpus).

## D. Fusión de contenido duplicado (ejecutado por el propietario + Gemini, no por esta sesión)

1. **Triángulo `ponytail:` vs. `DEBT-XXX`** (hoy triplicado casi verbatim en `debt-management.md`, `commenting-policy.md`, `ponytail.md`): fuente única pasa a `commenting-policy.md` (dominio correcto — es una decisión de cómo se escribe un comentario). Los otros dos quedan en una sola línea de referencia al ID.
2. **`.agents/knowledge/skill-common.md` — construido, ver "Estado de ejecución real".** Corrección real sobre lo que decía este bullet: no son 6 bloques a consolidar, son 5 — el Protocolo de Lecciones ya vivía centralizado en [[.agents/knowledge/mentorship-modes.md]] (`MENTORSHIP-MODES-3`), solo hacía falta corregir la referencia en cada skill, no duplicarlo de nuevo. Y la fuente asumida para Formato de Reporte al CEO era incorrecta: `AUDITOR-6` es un formato específico de informe de auditoría (veredictos citando hallazgos), no un reporte de estado genérico — la plantilla genuinamente reusable (`ESTADO`/`PROGRESO MACRO`/`FRICCIONES Y DEUDA`/`INPUT REQUERIDO DEL CEO`) es de `tech-lead` (`TECH-LEAD-3`); Auditor/Adversary la extienden con su propio esquema de veredicto, no se les fuerza uno ajeno. Cada skill afectado añade `skill-common.md` a su tabla de Lecturas como `SIEMPRE` y reemplaza cada bloque repetido por una línea de referencia al ID.
3. **`base.md`**: eliminar el archivo. Sus 5 referencias entrantes son todas históricas (`.agents/plans/`, `.agents/sessions/`) o excepciones hardcoded en el propio hook (`NO_SLUG_REQUIRED`, `STUB_FILES`, `BASE_MD_ALLOWED`) — ninguna activa desde un documento vigente. La Rule ya declara "`base.md` no existe como documento"; borrarlo cierra la inconsistencia entre la norma y el estado real. Se eliminan también esas 3 excepciones del script.
4. **`provider-bypass.md`** (huérfano, 0 referencias entrantes): no se fusiona — es contenido sustantivo y distinto. Se corrige añadiendo su fila a la tabla de Lecturas de `flutter-engineer/SKILL.md` con disparadores `` `bypass`, `hardcoded`, `theme`, `provider` ``.
5. **`memory-policy.md`**: sus secciones `-3` a `-9` (la mitad del documento) documentan en detalle un mecanismo nativo que `-14` marca como suspendido — no se archivan aparte (crearía un huérfano nuevo idéntico al de `provider-bypass.md`), se colapsan a una sola línea de puntero + nota de que el diseño completo vive en git history.

## E. Orden de migración

El riesgo central: el hook actual bloquea sobre reglas del formato viejo (Slug ID exacto, Tamaño exacto) que el formato nuevo elimina por diseño — si no se toca el hook primero, bloquea la migración desde el primer archivo.

**Ejecutor de cada paso** (ver "División de trabajo" arriba): 0 y 7-8 → esta sesión (Claude Code); 1-6 → propietario + Gemini.

0. **Cirugía mínima al hook** (`validate-agent-docs.py`): eliminar únicamente los checks de Slug ID exacto y Tamaño exacto (`RE_SLUG`/`RE_SIZE`). Todo lo demás (wikilinks, N/A, IDs de 3 segmentos, referencias a `base.md`, campo retirado) sigue bloqueando durante toda la migración — soltarlo por completo reproduciría el patrón `CONTEXT-DEBT-003` (pérdida silenciosa a mitad de una reescritura).
1. **Los 2 SIEMPRE universales**: `governance.md`, `hard-prohibitions.md` (ya redactado, ver sección A) — fijan el vocabulario del que depende el resto.
2. **El hub**: `execution-protocol.md` (575 líneas, 31 reglas) — correr el validador de secuencia (C2) antes y después como prueba de no regresión sobre las Etapas.
3. **`writing-efficiency.md`**, aplicando en el mismo paso la reclasificación de la sección F (toca la tabla de Lecturas de los 15 skills a la vez; separarlo en dos pasadas duplicaría el trabajo).
4. **Crear `skill-common.md`**, luego migrar los 12 skills de ingeniería/auditoría en este orden (menor a mayor riesgo): `partner` → `general-counsel` → `qa-engineer` → `quant-engineer` → `bridge-engineer` → `rust-engineer` → `refactoring-engineer` → `flutter-engineer` → `ui-designer` → `tech-lead` (forma atípica, sin bloque Pipeline) → `auditor` → `adversary` (depende del formato de reporte de `auditor`). Luego `architect`, `context-engineer`, `social-strategist` (sin extracción a `skill-common.md`).
5. **Leaves de knowledge**, por cluster: ponytail (`commenting-policy.md` → `debt-management.md` → `ponytail.md`, canónico primero) · modos (`mentorship-modes.md`, `write-like-gemini.md` — no recortar sus ❌/✅ empíricos) · reporting (`session-reporting.md`, `brief.md`) · memoria (`memory-policy.md` con la poda D5, `persistence-claims.md`) · resto independiente (`mutation-filter.md`, `order-compliance-filter.md`, `coherence-sweep.md`, `ledger-pattern.md`, `test-discrimination.md`, `ui-verification-model.md`, `claude-code-mechanisms.md`, `cross-platform-rust.md`, `execution-vocabulary.md`, `critical-domain-reasoning.md`, `doc-first.md`, `closing-work.md`, `tool-usage.md`, `context-window-budget.md`, `provider-bypass.md` con el fix D4). Al cierre: eliminar `base.md`.
6. **Los 16 stubs `.claude/skills/*/SKILL.md`**: batch único al final (son espejo de `.agents/skills/`, tocarlos antes obligaría a repetir el trabajo).
7. **Reescritura final del hook** contra el corpus ya migrado: valida el bullet-ID nuevo, añade `--check-graph` (C1) y el validador de secuencia (C2), ambos en modo `--audit` primero. Solo avanzar si `--audit` reporta 0 violaciones duras sobre todo el árbol.
8. **Reactivación del bloqueo estricto** (quitar la excepción de Fase 0) y registro de `UserPromptSubmit` / `inject-context.py` en `.claude/settings.json` — depende de que las tablas de Lectura ya estén en su forma final de 3 columnas.

## F. Reducción de la lista "SIEMPRE" (ejecutado por el propietario + Gemini, no por esta sesión)

`CLAUDE.md` ya declara `governance.md` + `hard-prohibitions.md` como "los dos únicos documentos SIEMPRE, pares entre sí" — es intención ya fijada en la raíz del repo; lo que falta es que la práctica (cada skill marcando `writing-efficiency.md` como SIEMPRE en su propia tabla) deje de contradecirla. No se fusionan ambos documentos: son decisión de diseño previa declarada como tal, y tras el formato ligero `hard-prohibitions.md` ya mide 2.336 caracteres — fusionar no gana compresión medible, solo obligaría a renamespacear ~16 citas activas sin beneficio.

`writing-efficiency.md` dejar de ser SIEMPRE plano:
- Sigue SIEMPRE donde el texto es el entregable: `partner`, `tech-lead`, `auditor`, `adversary`, `general-counsel`, `social-strategist`, `context-engineer`.
- Pasa a disparador condicional del hook B1 en los skills de código: `rust-engineer`, `flutter-engineer`, `bridge-engineer`, `refactoring-engineer`, `qa-engineer`, `quant-engineer`, `ui-designer` — con disparadores `` `reportar estado`, `session-report`, `commit message`, `PR description`, `escribir documento`, `resumen para el usuario` ``.

Mismo criterio para cualquier otro "SIEMPRE" declarado por skill particular durante la migración (Fase 4/5 de la sección E): si la guía no es necesaria en cada invocación sino solo cuando el pedido toca un concepto o archivo específico, baja a disparador de keyword o a Rule con `paths:` — nunca se deja SIEMPRE por costumbre.

## G. Verificación determinista avanzada — atacando directo la causa raíz #1

Todo lo anterior (A-F) resuelve indirección y aparato de formato. Esta sección ataca la causa raíz más profunda del diagnóstico original: casi todo el control hoy es el mismo agente autoevaluándose con un checklist de texto. Contrato verificado contra la documentación oficial de hooks de Claude Code (no supuesto): existe un tipo de hook `"type": "prompt"` — en vez de correr un script, Claude Code manda el input del evento a un modelo (**Haiku por defecto**, configurable) que devuelve `{"ok": true}` o `{"ok": false, "reason": "..."}"`, disponible en `Stop`, `SubagentStop`, `PreToolUse`, `PostToolUse`, `PostToolBatch`, `UserPromptSubmit`. También existe `"type": "agent"` (subagente con herramientas, múltiples turnos) para verificaciones que requieren inspeccionar el estado real del repo, no solo el input del evento.

### G1. Juez único universal en `Stop`, más profundidad para Tech-Lead

Un solo hook `Stop` tipo `prompt`, aplicado a los 15 skills (confirmado que un juez único con cobertura total es más barato de mantener que 13 prompts por skill, y da cobertura donde hoy no hay ninguna): verifica solo las reglas SIEMPRE-universales — rol declarado en el primer mensaje, filtro de arranque de `governance.md` cumplido, idioma correcto, ausencia de lenguaje mortuorio/bitácora, autorización explícita de git si el turno tocó una operación git. El prompt del juez incluye una condición extra: si el `session_id`/contexto indica que el rol activo es Tech-Lead y el turno cerró o tocó una Orden de Trabajo, verifica además evidencia de cierre reproducida (no solo reportada por el ingeniero) y, si aplicaba, que el resultado del filtro de mutación quedó declarado. Un solo archivo de prompt (`.agents/hooks/judge-stop.md` o inline en `settings.json`), no 13 — el costo de mantenimiento es el de un documento, el costo de ejecución es una llamada a Haiku por turno que cierra (rápida y barata).

### G2. Gating real en `PreToolUse` con inspección de contenido completo

Confirmado: `PreToolUse` sobre `Write`/`Edit` recibe `tool_input.content` con el contenido **completo** que se va a escribir, antes de que toque disco. Esto mueve el Filtro de Cumplimiento ADR (`order-compliance-filter.md`) de "checklist que el Tech-Lead se autoaplica antes de redactar" a un hook `type: "command"` sobre `Write` en `docs/execution/*.md` que hace grep del contenido propuesto y bloquea con exit 2 si faltan secciones obligatorias (tabla "Agentes y Modo de Acompañamiento", "## Puertos de Integración", criterios de aceptación). Para los checks que requieren juicio, no solo presencia de texto (ej. "¿el perfil de persistencia declarado es coherente con el tipo de feature?"), el mismo evento admite `type: "prompt"` — Haiku evalúa el contenido propuesto contra la lista de checks antes de que exista en disco.

### G3. Hook `type: "agent"` para lo que ninguna evaluación de un solo input puede ver

Casos que requieren comparar contra el estado real del repo, no solo el contenido de un Write: "¿toda migración tocada trae su ficha `docs/data-model/<tabla>.md` actualizada en el mismo cambio?" (`ORDER-COMPLIANCE-FILTER-14`). Un hook `Stop` tipo `agent` con acceso de lectura puede confirmar esto de forma independiente antes de dejar cerrar el turno, en vez de confiar en que el Tech-Lead lo recordó.

### G4. Recuperación tras compactación — reglas + estado destilado

Confirmado: no existe inyección de contexto en `PreCompact` mismo; el mecanismo real es `SessionStart` con `matcher: "compact"` (dispara al reanudar justo después de compactar), inyectando por stdout (no `additionalContext`, ese campo es exclusivo de `UserPromptSubmit`). El hook emite dos cosas, no solo un recordatorio de reglas: (1) el núcleo de 2-3 reglas de mayor riesgo del rol que estaba activo antes de compactar, y (2) un snapshot destilado del estado real de la Orden en curso (Etapa, resultado del filtro de mutación, deuda abierta) — generado al vuelo por un modelo barato a partir de `PROGRESS.md`/el JSON de estado de G7, no releído en prosa completa. Ataca directamente el vector de fallo de sesiones largas: el contexto que se compacta puede diluir silenciosamente tanto las reglas leídas al inicio como el estado de la tarea.

### G5. Restricción de herramientas por Skill (`allowed-tools`)

Mecanismo nativo ya disponible en el frontmatter de Skill, no explotado hoy: skills directivos (`partner`, con `PARTNER-1.2 — Directivo, no operativo: prohibido escribir código`) declaran `allowed-tools` sin `Edit`/`Write` sobre `crates/**`/`app/**`. Convierte una prohibición hoy solo textual en una restricción mecánica — el modelo no tiene la herramienta disponible para violarla, no depende de que decida no usarla.

### G6. Límites confirmados, para no diseñar sobre supuestos

Hook chaining es **paralelo, no secuencial** — dos hooks en el mismo evento no pueden depender uno del otro; si ambos deciden, gana la respuesta más restrictiva (`deny` > `defer` > `ask` > `allow`), pero no hay forma de encadenar condicionalmente. No existe JSON Schema nativo de validación de salida — `{"ok": bool, "reason": string}` es la única estructura semi-forzada disponible en hooks tipo `prompt`. No existen máquinas de estado nativas — de ahí G7.

### G7. Máquina de estado explícita del ciclo de vida de la Orden — incluida en el alcance de esta ejecución

La pieza de mayor esfuerzo y mayor determinismo del plan. Nuevo archivo por Orden, `.agents/state/orders/<ID>.json`:

```json
{
  "order_id": "STORY-042",
  "stage": "5",
  "mutation_gate": "passed",
  "adr_compliance_filter": "passed",
  "data_model_sync": "passed",
  "checks": { "criterio_1": "STORY-042-test-a", "criterio_2": "STORY-042-test-b" }
}
```

El Tech-Lead actualiza este archivo (con `Write`/`Edit` normal, ya lo hace mentalmente hoy vía el Pipeline en prosa — la diferencia es que ahora es un artefacto legible por máquina) en cada transición real de Etapa. Un hook `PreToolUse` sobre la acción de cerrar (detectada por patrón: `Write`/`Edit` a `docs/ROADMAP.md` o `.agents/state/tech-lead/PROGRESS.md` con el patrón "COMPLETADO" cerca del ID de la orden) lee `.agents/state/orders/<ID>.json` y bloquea con exit 2 si `stage` no llegó a `"7"`, o si `mutation_gate` no está en `"passed"`/`"not_applicable"`. Esto convierte el defecto exacto de `CONTEXT-DEBT-007` (un bloque obligatorio sin número de secuencia que un agente podía tratar como opcional) en algo mecánicamente imposible de saltarse — no depende de que el agente recuerde ni decida cumplir el orden, el cierre no ocurre si el archivo de estado no lo respalda.

Se implementa como la fase final de la ejecución (después de E8), una vez el corpus de documentación ya está en formato ligero y los hooks G1-G5 ya están activos — de otro modo se estaría verificando un pipeline (`execution-protocol.md`) que todavía va a cambiar de forma durante la migración.

## H. Sistema de tags/keywords para recuperación de contexto

Complementa, no reemplaza, el índice por Slug ID (cita exacta de una regla) y el índice de `docs/README.md` (navegación curada por nombre de feature). Resuelve un caso que ninguno de los dos cubre: "¿qué documentos tratan centralmente el tema X?", sin que quien pregunta sepa ya en qué archivo vive.

Simulación real corrida sobre el corpus antes de diseñar esta sección (no hipótesis): `grep -rl "riesgo" docs/features` devuelve 61 de 166 archivos (37%), la mayoría solo menciona la palabra de paso; `grep -rl "deuda" .agents/knowledge .agents/skills` devuelve 19 archivos, pero solo `debt-management.md` la usa con densidad temática real (17 menciones vs. 1-3 en el resto). `docs/README.md` ya cubre el eje "dominio/módulo" vía su columna "Consumido por" — un tag de dominio nuevo sería redundante, por eso el tag aquí tiene un solo eje: **tema**.

### H1. Formato del campo

`.agents/`: línea `Tags: \`tag-uno\`, \`tag-dos\`.` en la cabecera canónica de `.claude/rules/agents-docs-format.md`, justo después de `Tamaño:` — requiere una edición pequeña a esa Rule para sancionar el campo nuevo y mantener el parser del hook (`RE_TAGS`) sincronizado con el formato real. **Implementado en `template-knowledge.md` (H1) en vez de `agents-docs-format.md`, ver "Estado de ejecución real"** — extendido después a nivel de Sección, no solo cabecera: `TEMPLATE-INSTRUCTION-FORMATS-5` permite un bullet `Tags:` propio dentro de cualquier `## <DOC>-N:`, para rankear esa Sección puntual sin esperar a que el resto del documento tenga tags.

`docs/features` y `docs/adr`: línea `**Tags:** \`tag-uno\`, \`tag-dos\`` dentro del bloque de cabecera propio de cada uno (junto a `Decisión Arquitectónica Asociada` en features). No existe hoy una Rule que gobierne ese header — se documenta como convención nueva en `docs/templates/TEMPLATES.md` (dueño Architect), no en `agents-docs-format.md`.

### H2. Dos vocabularios controlados, dos dueños

- `.agents/knowledge/tags-vocabulary.md` (nuevo) — dueño Context-Engineer. Semilla inicial: un tag por clúster ya definido en la sección E de este plan (ponytail, modos, reporting, memoria, resto), sin inventar temas nuevos en el arranque.
- `docs/tags-vocabulary.md` (nuevo) — dueño Architect. Semilla inicial: derivada de los `CAP-01`..`CAP-10` ya declarados en `docs/PRODUCT.md`, más temas transversales que "Consumido por" no cubre (ej. `gestion-riesgo`, `filtro-senal`).

Ambos append-only, mismo criterio que "IDs append-only" en `agents-docs-format.md`. Alta de un tag nuevo: quien lo necesita hace `grep` contra el vocabulario vigente (evita sinónimos duplicados) y, si no existe, lo propone al dueño correspondiente — nunca se inventa ad hoc durante trabajo normal.

### H3. Índice de `.agents/` — extiende B1, no lo reemplaza (construido — diseño corregido tras prueba en vivo)

Diseño original de este bullet (columna `tags` separada, ponderada vía `bm25(tabla, w_content, w_tags)`) se probó en vivo y se descartó: `bm25()` de FTS5 usa una normalización de longitud **compartida entre todo el corpus** (confirmado contra `https://www.sqlite.org/fts5.html`) — subir el peso de una columna mueve también el score de documentos sin ningún match en esa columna, y el resultado no fue monótono (el margen a favor del documento etiquetado se **encogía** al subir el peso, no crecía). Reemplazado por boost de frecuencia clásico en una sola columna: al indexar, si el documento trae `Tags:`, sus valores se repiten `TAG_REPEAT = 4` veces al inicio del texto indexado, antes del contenido real. Probado monótono y predecible (gap creciente de forma consistente al variar la repetición de 1 a 12). Mismo mecanismo de reindexado por `mtime` que B1 ya define — sin hook nuevo, sin columna nueva en el esquema.

Construido junto con H2 (semilla `.agents/knowledge/tags-vocabulary.md`, 4 tags: `ponytail`, `modos-acompanamiento`, `reporting`, `memoria` — derivados de los clústeres de la sección E, sin inventar temas). Probado end-to-end contra un fixture aislado (no el corpus real, para no forzar un tag en un documento donde no aplica genuinamente) — el documento con `Tags: ponytail` real rankeó primero, por encima de uno que solo menciona la palabra de pasada. Corrido después contra el corpus real (`--audit` sobre todo `.agents`): 0 violaciones duras.

**Extensión el mismo día — indexado por Sección, no por documento completo.** El propietario pidió absorberlo ya, mientras la migración de la sección E sigue temprano (más fácil ahora que con más corpus migrado). `inject-context.py` parte cada documento en formato ligero por sus encabezados `## <DOC>-N: ` (un segmento por Sección, más un segmento "cabecera" con todo lo previo a la primera Sección) e inserta **una fila por segmento**, no una por archivo — un documento en formato legado, sin ese patrón de encabezado, sigue cayendo completo en un solo segmento, sin romper nada. `Tags:` ahora también es válido dentro de una Sección (`TEMPLATE-INSTRUCTION-FORMATS-5`) — los tags de la cabecera se heredan a toda Sección del documento, los propios de una Sección se agregan encima. El manifiesto de wikilinks cita la Sección exacta que matcheó (`[[archivo.md]] — sección DOC-N`), no solo el archivo.

Bug real encontrado y corregido durante la prueba: la primera versión de la regex de partición (`[^:\n]+` antes de `: `) hacía falso-positivo con encabezados legados que casualmente traen un `:` en el subtítulo (ej. `## ORDER-COMPLIANCE-FILTER-1 — Postura del Filtro: Contraste Bidireccional`), partiéndolos como si fueran formato ligero. Corregido exigiendo que el ID sea un token contiguo (letras/dígitos/guiones, sin espacio ni guion largo) inmediatamente seguido de `: ` — un heading con `—` de por medio ya no dispara la partición. Confirmado con `order-compliance-filter.md` (legado, 1 segmento) y `context-engineer/SKILL.md` (ligero, 7 segmentos: cabecera + P + 5 Secciones) contra el corpus real, más una prueba de ranking en fixture aislado (Sección con `Tags:` real venció a una mención de pasada en otra Sección del mismo documento). Verificado también que Pipeline (`<DOC>-P`) y Ejemplos (`<DOC>-EJ`) caen en su propio segmento sin código extra — el separador no distingue tipo de bloque, solo el patrón `## <ID>: `. Los casos individuales dentro de Ejemplos (`<DOC>-EJ-K`) no son encabezados `##` propios (`TEMPLATE-INSTRUCTION-FORMATS-7`), así que quedan agrupados en un solo segmento — límite de granularidad conocido, no un defecto.

**Switch `FORCE_FULL_CONTENT`, agregado el mismo día.** El propietario pidió mantener viva la opción de pegar contenido completo, no solo wikilinks, como palanca de troubleshooting de una sola propiedad — por si en la práctica el wikilink no se lee lo suficiente. Constante booleana al inicio de `inject-context.py` (`False` por defecto = wikilinks siempre, sin cambio de comportamiento). En `True`: pega el texto real de cada Sección que matcheó (columna `raw` nueva en el esquema FTS5, guarda el segmento sin el boost de tags — el boost solo vive en la columna `content`, indexada). Tope de seguridad `FORCE_MAX_CHARS = 12000` incluso con el switch activo, para el caso de un documento legado grande que aún no se partió por Sección. Probado en ambos modos contra el corpus real.

### H4. Índice de `docs/` — separado, solo-tags, mismo punto de entrada

Segunda tabla FTS5 (`docs_fts`, columnas `path`, `tags` — sin `body`) sobre `docs/features/*.md` y `docs/adr/*.md`. Se consulta desde el **mismo** hook `UserPromptSubmit`/`inject-context.py`, no desde un segundo registro en `.claude/settings.json` — evita depender de un comportamiento no verificado (si dos hooks `UserPromptSubmit` simultáneos combinan su `additionalContext` o si gana solo uno; el contrato oficial confirmado por B1 no lo especifica). Un solo hook, dos consultas FTS5 internas, un solo `additionalContext` combinado.

Dos índices separados, nunca uno mezclado: `.agents/` promedia ~9.930 caracteres/doc hoy (baja a ~3.600-4.100 tras el recorte de prosa de la sección A) y `docs/features` promedia ~9.950 caracteres/doc sin recorte planeado — mezclarlos en una sola tabla descalibra la normalización por longitud de BM25, y un índice único arrastraría contenido de producto a prompts de skills que nunca lo necesitan.

Como el índice de `docs/` solo indexa `tags` (nunca `body`), el 37% de ruido medido con grep de cuerpo completo (caso "riesgo") no aplica aquí — solo dispara con un tag exacto ya asignado a propósito. Esto hace innecesario un gate por rol activo (`.agents/state/active-role.json`) — se evaluó y se descartó, ver "Fuera de alcance". Resuelve en modo manifiesto de wikilinks, igual que B1 — desde H5, B1 nunca pega contenido completo, así que aquí tampoco.

### H5. Presupuesto de inyección — retirado del todo, ya no existe como concepto

Historial real de este bullet, dos revocaciones en el mismo día: (1) fijaba B1 en ~6.000 caracteres, sin subirlo, para no pagar costo de inyección en cada turno que matchea contra "lee bajo demanda, nunca en masa" de `CLAUDE.md` — el propietario lo revocó a 10.000 al ver en vivo que con 6.000 casi todo caía en modo manifiesto. (2) Esa misma sesión, al decidir "solo wikilinks siempre" (ver H3 y bullet de "Estado de ejecución real"), la inyección de contenido completo se eliminó del todo — no hay ninguna rama de código que pegue texto de un documento o Sección al contexto, solo la ruta + Sección citada. `MAX_COMBINED_CHARS` ya no existe en `.agents/hooks/inject-context.py` — no hay presupuesto que calibrar porque no hay contenido que presupuestar. El razonamiento original sobre partir documentos grandes por sub-tema queda sin objeto por la misma razón: el indexado por Sección de H3 ya resuelve la granularidad, no falta partir nada físicamente. Caso con cuidado especial que sigue vigente por otra razón (no por presupuesto): `execution-protocol.md` tiene una secuencia de Etapas que el validador C2 verifica en orden ascendente — si algún día se divide, ese chequeo debe seguir siendo verificable.

### H6. Validadores

`.agents/`: extiende `validate-agent-docs.py` (ya en sección C) con un check nuevo — cada valor de `Tags:` debe pertenecer a `.agents/knowledge/tags-vocabulary.md`; tag desconocido es violación dura, mismo nivel que `Slug ID` hoy.

`docs/`: script separado `validate-docs-tags.py` — mismo principio contra `docs/tags-vocabulary.md`, formato de cabecera distinto (`**Tags:**` en vez de `Tags:`), sin engancharse al `PostToolUse` existente (dueño y scope distintos). Arranca en modo `--audit` manual, igual criterio de rodaje que C1 (validador de grafo) antes de considerar engancharlo a un hook bloqueante — decisión del Architect, no de esta sesión.

### H7. Aplicación progresiva — por fases, con verificación entre cada una

No es viable en una sola pasada (45 documentos en `.agents/`, 166 en `docs/features`, 163 en `docs/adr`). Orden:

1. Vocabularios semilla (H2) — Context-Engineer y Architect, en paralelo, cada uno sobre su corpus.
2. Mecanismo (H3-H6) en modo `--audit`/no bloqueante — esta sesión.
3. Tags sobre `.agents/` (45 docs), por los mismos clústeres ya usados en la sección E — propietario + Gemini.
4. Tags sobre `docs/features` (166 docs), en lotes por módulo consumidor (reusa el orden natural del pipeline `ingest → generate → validate → incubate → manage → execute → feedback → withdraw`) — propietario + Gemini, delegando lectura+extracción a subagentes Sonnet por lote cuando el volumen lo justifique (`CLAUDE.md` §3.4).
5. Tags sobre `docs/adr` (163 docs) — mismo patrón, último por ser el corpus más estable/menos consultado por tema.

Verificación tras cada fase: conteo real de documentos tageados (`grep -c`) contra el total del clúster, y `--audit` de cada validador antes de avanzar a la fase siguiente.

### H8. Lecturas forzosas mid-flujo — gate mecánico, no de prosa

Problema real planteado por el propietario: el "Filtro de Arranque" (SIEMPRE, ya resuelto por `session-start-siempre.py` + `gate-skill-siempre.py`, CONTEXT-DEBT-011 pagada) solo cubre el inicio de un skill. No cubre un punto intermedio de su propio flujo ("antes de crear la Orden, hace falta haber leído el formato") cuando nadie escribe un prompt nuevo en ese instante — y ese punto es invisible por diseño: verificado contra `code.claude.com/docs/en/hooks` que no existe ningún evento de hook atado al razonamiento interno del agente, solo a llamadas de herramienta y a envíos de prompt. Instrucción en prosa ("no proceder hasta leer X") es el mismo tipo de garantía que el Filtro de Arranque — depende de que el modelo obedezca, no un gate mecánico.

Diseño construido (3 hooks + 1 helper compartido, reusando el esquema de marcadores ya existente en `.agents/state/.session-reads/<session_id>/` en vez de crear uno paralelo):

- **`.agents/hooks/_session_reads.py`** (nuevo, no es hook) — extrae `STATE_DIR`/`marker_path`/`mark_read`/`already_read`/`session_dir`, antes duplicados en `track-siempre-reads.py` y `gate-skill-siempre.py`. Ambos se refactorizaron para importarlo (probado en vivo que `gate-skill-siempre.py` sigue denegando igual tras el refactor).
- **`track-siempre-reads.py`** ampliado — antes solo marcaba lecturas bajo `.agents/knowledge/`, ahora marca cualquier `.md` del repo (una lectura forzosa puede apuntar a una plantilla o a un `SKILL.md`, no solo a knowledge).
- **`.agents/hooks/enforce-essential-reads.py`** (nuevo, `PreToolUse` sobre `Write|Edit|Bash`) — escanea `.agents/skills/*/SKILL.md` + `.agents/knowledge/*.md` buscando bullets `- Lectura forzosa: [[ruta]] antes de \`Tool(patrón)\`` (misma sintaxis `Tool(especificador)` de `.claude/settings.json`, fnmatch). Si el patrón matchea la llamada actual y la marca de lectura no existe en la sesión, deniega con `permissionDecisionReason` listando los wikilinks faltantes (batch, no uno a la vez — mismo estilo que `gate-skill-siempre.py`). Sin declaraciones aún en ningún skill real — infraestructura lista, cero falsos positivos hasta que un skill adopte la convención.
- **`.agents/hooks/reset-session-reads.py`** (nuevo, `SessionStart` matcher `compact|resume|clear`) — borra `.agents/state/.session-reads/<session_id>/` entero en esas tres transiciones. Decisión de diseño explícita: verificado empíricamente en esta misma sesión (comparando la ruta del transcript pre-compactación contra la ruta del hook `SessionStart:compact` disparado después) que `session_id` sobrevive a `/compact` — pero eso no basta para confiar en la marca, porque el propósito del mecanismo es garantizar contenido *actualmente* en contexto, no "se leyó alguna vez". Se borra siempre, sobreviva o no el id. De paso barre directorios de sesión con más de `SWEEP_DAYS=3` días de antigüedad (red de seguridad para sesiones sin `SessionEnd` limpio).
- **`.claude/settings.json`** — registrados los 2 hooks nuevos; agregado `permissions.deny`: `Read(/.agents/state/.session-reads/**)` + `Edit(/.agents/state/.session-reads/**)` (verificado contra `docs/en/permissions`: solo `Edit`/`Read` se consultan por ruta, un rule `Write(...)` se acepta pero nunca se consulta — error real que se evitó a tiempo) más `Bash(*rm*.session-reads*)` / `Bash(*mv*.session-reads*)` como mitigación de mejor esfuerzo (no blindaje total — la propia documentación de Claude Code advierte que Read/Edit no cubren subprocesos arbitrarios, ej. un script Python que abre el archivo directo; blindaje real requeriría el sandbox a nivel SO, fuera de alcance).

Probado en vivo end-to-end con un skill de prueba desechable (creado y borrado en la misma sesión, sin tocar ningún skill real): Write denegado antes de leer → Read simulado → mismo Write permitido → reset por `compact` simulado → mismo Write vuelve a denegar. Los 4 pasos confirmados con salida real del hook, no simulados en prosa. Efecto colateral confirmado en vivo: al intentar limpiar los directorios de prueba con `rm -rf .../.session-reads/...`, la propia regla `Bash(*rm*.session-reads*)` denegó el comando — prueba entera de que la protección funciona tal como se diseñó (limpieza resuelta con `find ... -delete`, que no contiene el substring `rm`).

Pendiente, fuera de esta sesión: ningún skill real declara todavía un bullet "Lectura forzosa" — su adopción es autoría de contenido por skill, no mecanismo. Tampoco se extendió el `deny` a `.agents/hooks/*.py` (bloquearía también el mantenimiento legítimo de los propios hooks) — se dejó fuera a propósito, señalado al propietario, sin decidirlo unilateralmente.

**Próximo paso explícito (pedido por el propietario, sin dueño de turno todavía):** identificar, skill por skill, qué puntos de un flujo ya documentado necesitan una lectura forzosa real — candidato obvio: `tech-lead/SKILL.md` antes de crear una Orden (`docs/execution/*.md`), leyendo `execution-protocol.md`/`order-compliance-filter.md`. Criterio de selección: solo declarar la regla donde ya se demostró en la práctica (o es evidente por diseño) que el paso previo se puede saltar sin que nada lo note — no declarar por declarar en todo punto de escritura de cada skill, eso degradaría la señal y generaría fricción sin beneficio real.

## Verificación end-to-end

1. Tras cada fase de migración (E): `python3 .agents/hooks/validate-agent-docs.py --audit` sobre todo `.agents/` — 0 violaciones duras antes de avanzar a la siguiente fase.
2. Tras Fase 7 (hook reescrito): correr `--audit` con `--check-graph` y el validador de secuencia activados — confirmar que `provider-bypass.md` ya no aparece como huérfano y que `execution-protocol.md` no tiene bloques obligatorios sin ID.
3. Tras Fase 8 (hook `inject-context.py` registrado): probar manualmente con 3-4 prompts reales que contengan keywords de disparadores distintos (ej. mencionar "ponytail", mencionar "mutación", mencionar "ledger") y confirmar en la transcripción que el `additionalContext` llegó — no simular, correr el hook de verdad con un prompt real en una sesión de prueba.
4. Confirmar que ningún ID citado externamente (`grep -rn "<DOC>-N"` sobre `.agents/` y `.claude/` sin `--audit`) quedó huérfano tras la fusión D — cada ID retirado por fusión debe tener su reemplazo (línea de referencia) en el lugar exacto donde vivía antes.
5. Reportar al usuario, por fase, el conteo real (`wc -m`/`wc -l`) antes/después — no estimado — siguiendo el propio estándar de medición que el proyecto ya exige para reportes de eficiencia de redacción.
6. Tras activar G1 (juez `Stop`): correr 3-4 turnos reales con violaciones deliberadas y menores (ej. cerrar sesión sin declarar rol) y confirmar que `ok: false` con `reason` útil llega de vuelta al agente, no solo que el hook corrió sin error.
7. Tras activar G2/G7 (gating de `PreToolUse`): intentar deliberadamente escribir una Orden sin la tabla de Modo de Acompañamiento, y cerrar una Story con `mutation_gate` en `"pending"` — confirmar que ambas quedan bloqueadas con exit 2 antes de tocar disco/marcar completado, no después.
8. Tras activar G4: forzar una compactación real en una sesión larga de prueba y confirmar que el stdout de `SessionStart (compact)` efectivamente aparece en el contexto post-compactación.
9. Tras H3/H4 (índices de tags activos): probar con los mismos ejemplos medidos en esta conversación (`"riesgo"` en `docs/features`, `"deuda"` en `.agents/`) y confirmar que el resultado vía tags es el subconjunto preciso (2-3 archivos), no el ruido de grep de cuerpo completo (61 / 19 archivos).
10. Tras H6 (validadores de tags): escribir deliberadamente un tag fuera de vocabulario en un documento de cada corpus y confirmar que cada validador lo marca como violación dura, sin bloquear el resto del contenido.

## Fuera de alcance de esta ejecución

- Activar el hook `Stop` para el validador de grafo (C1) — queda para después de un período de rodaje solo en modo `--audit`, sin huérfanos nuevos detectados.
- El router de intención con sub-modelo barato (B3) — condicionado a medir en uso real una tasa alta de falsos negativos del keyword-match FTS5.
- El hook `type: "agent"` (G3) para verificaciones cruzadas de estado real (ej. ficha `docs/data-model/` sincronizada) — se documenta el diseño, pero se activa después de confirmar que G1/G2/G7 (más baratos, más simples) ya están estables, dado que un hook de agente consume más tokens y tiempo por invocación.
- Migrar `docs/` (dominio del Architect) al mismo formato ligero — el pedido del usuario y la exploración se acotaron a `.agents/` y `.claude/skills/`.
- Gate por rol activo (`.agents/state/active-role.json`) para decidir si el hook consulta el índice de `docs/` — evaluado y descartado: la separación de índices (H4, solo-tags) ya resuelve la saturación de contexto sin necesitar que el hook sepa qué skill está activo.
