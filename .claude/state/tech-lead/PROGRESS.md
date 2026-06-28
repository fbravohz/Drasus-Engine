# Bitácora Operativa — Tech-Lead (Drasus Engine)

> Memoria viva entre sesiones. El Tech-Lead la LEE al arrancar (Etapa 0) y la ACTUALIZA al cerrar cada tarea/decisión.
> Este archivo es el TABLERO/índice: dónde estamos + siguiente paso. El DETALLE de cada trabajo vive en su Orden de Trabajo (`docs/execution/<ID>-<slug>.md`); el estado de fase vive en `docs/ROADMAP.md`. No dupliques detalle aquí: apunta a la Orden.
> Sistema de seguimiento (Spec-Driven): cada trabajo se ejecuta desde una Orden de Trabajo con el prompt exacto + comandos de validación. Plantilla: `docs/execution/_TEMPLATE.md`.

---

## Estado actual

- **Fase activa:** EPIC-1 — Soberanía de Datos (`ingest`). EPIC-0 cerrada 2026-06-27.
- **Última sesión:** 2026-06-27.
- **✅ STORY-024 (`sovereign-data-fetcher`) CERRADA (2026-06-27, QA APTO).** Primer crate hexagonal de dominio (`crates/features/data/`). Modo Docente. Ciclo completo: 1ª entrega → verificación TL → QA NO APTO (concurrencia falsa, `Semaphore` decorativo) → regresión (`JoinSet` + `Arc<dyn BulkSource>`, test honesto `peak 2..=3`) → re-auditoría TL → QA APTO. Migración 0006 (Perfil A). Lección en `docs/lessons/rust/`. ROADMAP actualizado.
- **✅ ADR-0141 "Modelado Relacional Soberano" RATIFICADO Y FORMALIZADO (2026-06-28).** Cierra el vacío de modelado relacional (FK/índices/tipos de precio/PK/integridad SQLite↔Parquet/evolución Parquet/pool/retención). Decisiones ratificadas por el usuario: `audit_chain_hash`=NULL canónico; `STRICT` en TODAS las tablas; UUIDv7 en TODAS las PKs; sesiones de mercado derivadas en runtime; umbral "crece con mercado→Parquet, con usuario→SQLite". Archivos: ADR-0141 creado + índice; ADR-0006 enmendado (greenfield/brownfield); CLAUDE.md §1 (flag de fase); SAD-08/11/20 actualizados. Checks M1–M12/R1–R7/F1–F3 incorporados al Gate del TL **por referencia** a ADR-0141.
- **🌱 FASE DEL PROYECTO: GREENFIELD** (CLAUDE.md §1 / ADR-0006 enmendado). Monolito de escritorio; "producción" = instancia en la máquina de cada usuario. Baseline de migraciones editable in-situ hasta el primer release distribuido.
- **🖥️ Entrega de UI pendiente — inspector panel del `sovereign-data-fetcher` (corrección 2026-06-28):** STORY-024 entregó el MOTOR de descarga (backend). El fetcher NO es plomería: tiene Superficie propia = Inspector Panel (broker/símbolo/fechas/timeframe, ADR-0136). Falta una **Story de UI** (UI-Designer → Bridge → Flutter) para ese panel. Progreso → `background-download-manager`; exploración → `canvas-navigation`. Error de clasificación corregido en feature doc, Orden §8 y `tech-lead/SKILL.md`.
- **➡️ SIGUIENTE PASO (pendiente de luz verde del usuario):** **auditoría retroactiva desde STORY-001/EPIC-0** con ADR-0141 + contraste bidireccional como vara. Lista de trabajo de esquema heredada (anomalías del Architect): (A5) implementar PRAGMAs en `pool.rs` — URGENTE, primer ítem; (A6) editar baseline 0001–0006 a `STRICT` + UUIDv7 (legítimo en greenfield); (A4) `permission_decisions.audit_chain_hash` → NULL; (A3) `jobs.event_sequence_id` → renombrar a `row_version`; (A2) decidir destino de `foundation_master_fields.event_sequence_id`.
- **🔧 Reglas nuevas del usuario (2026-06-27/28) grabadas en skills:** (1) **contraste bidireccional en el Gate** (retar feature/ADR/SAD, no obediencia ciega) — `tech-lead`. (2) **Etapa 7 "¿Qué aprendimos y cómo mejoramos?"** al cerrar cada iteración → traducir errores en mejoras de skill — `tech-lead`. (3) **TDD/prueba discriminante** (la prueba debe poder fallar; medir el comportamiento) — `tech-lead`/`rust-engineer`/`qa-engineer`. (4) **Ventana de Verificación** obligatoria al sellar plomería (ADR-0117) — `tech-lead`. (5) **trazabilidad ADR→checks del Gate** — `tech-lead`. (6) **fase greenfield/brownfield** en recomendaciones — `architect`.
- **🔧 Corrección de criterio (2026-06-27, por el usuario):** (1) **Docente lo despacha el Tech-Lead**, igual que Autónomo — el Ingeniero implementa solo y escribe la lección; NO es el usuario quien invoca (eso es Mentor/Revisión). Yo me había equivocado. Corregido en `tech-lead/SKILL.md`. (2) El **Gate de Coherencia** ahora exige barrido ADR COMPLETO (índice + ADRs candidatos bajo demanda, no solo los citados) e impacto en el SAD. Ambas reglas grabadas en `tech-lead/SKILL.md` §"Gate de Coherencia Pre-Despacho".
- **✅ TASK-006 (Auditoría Inundación de Fundaciones) CERRADA** (2026-06-13; renumerada desde TASK-004 el 2026-06-18, ver entrada de esa fecha). Fases 1-4 completas y auditadas (ver `docs/execution/TASK-006-...md`). 137 features + 8 módulos auditados; perfiles reasignados, contratos diseñados, Grupo I completo en todo el corpus, ADR-0020 expone los 3 campos transversales (conteo se mantiene en 25), TEMPLATES arreglado (causa raíz P1). Commits: `bace15c` (fase 1), `4bf76b3` (decisiones fase 2), `ef6ca36` (fase 3). **Mantra del usuario** grabado en base/SKILL.md ("ante la duda, prefiero tenerlo y no necesitarlo").
- **⚠️ Cambio de rumbo del ROADMAP (2026-06-16, ADR-0118):** `crash-recovery` (antes "STORY-006") **YA NO es de EPIC-0** — pertenece a `execute`/EPIC-5 (necesita el conector de bróker, que no existe hasta entonces). El gate de recuperación tras `kill -9` que EPIC-0 sí exige ya está cubierto por `async-job-executor` (STORY-005, cerrado). El ROADMAP se reescribió a v3.0 (guía de orden + estado simple, sin bitácora narrativa — el detalle vive en las Órdenes de Trabajo).
- **🎚️ Nuevo mecanismo (ADR-0120, 2026-06-16):** cada Agente de una Orden declara un **Modo de Acompañamiento** — Autónomo (despacho yo vía `Agent`) / Mentor (el usuario teclea, el Ingeniero dicta bloque a bloque) / Revisión (el usuario entrega código, el Ingeniero audita). Se declara en la Orden, nunca en el chat. Bajo Mentor/Revisión yo NO despacho: redacto la Orden y me detengo; el usuario invoca el skill del Ingeniero directamente.
- **✅ STORY-007 (`telemetry`, TTR-001) CERRADA y auditada** (2026-06-18). Ver entrada de esa fecha más abajo.
- **➡️ SIGUIENTE PASO CONCRETO:** despachar **STORY-008 (`worker-isolation-orchestrator`)** — preguntar al usuario el Modo de Acompañamiento (ADR-0120) antes de redactar la Orden, igual que se hizo para STORY-007. Después: STORY-009 (CLI + binario raíz `app`), STORY-010 (`agentic-mcp-gateway`, núcleo MCP + evaluador de permisos, ADR-0123). Transversal: los 6 spikes de gates (SPIKE-001–006) antes de cerrar EPIC-0 / arrancar EPIC-1 — solo SPIKE-001 (smoke test NautilusTrader) sin validar de fondo.
- **Pendiente diferido:** auditoría de Inundación de Fundaciones en los 41 moonshots (misma estrategia, TASK futura).
- **🔢 Corrección de integridad de numeración (2026-06-18):** TASK-004 colisionaba con STORY-004 (ambos usaban "4" en el contador global Story/Spike/Task/Bug). Renumerada a **TASK-006** (siguiente número global libre, cronológicamente correcto). Protocolo de numeración aclarado en `tech-lead/SKILL.md` §"Vocabulario Ágil e Identificadores" (ver entrada de hoy abajo).
- **Nomenclatura:** ya NO se usan códigos F/W/G. Identificadores estilo Jira: EPIC-n, SPRINT-n, STORY-###, SPIKE-###, TASK-###, BUG-###. Cada Story se ejecuta desde su Orden de Trabajo en `docs/execution/`.

## Reglas activas confirmadas con el usuario

- **Modelos (actualizado 2026-06-25 — dual-platform):**
  - **En Claude Code:** ingenieros NUNCA en Opus. Sonnet por defecto / tareas críticas; Haiku solo mecánicas. El Architect SÍ en Opus (necesita mucho contexto). El Tech-Lead despacha subagentes vía herramienta Agent (subagent_type `general-purpose`) cargando `CLAUDE.md` + `base/SKILL.md` + el SKILL del rol.
  - **En opencode:** agentes configurados en `.opencode/agents/` con modelo fijo. Ingenieros de código (Rust, Flutter, Bridge, QA, Quant, Refactoring) → `qwen3.7-plus`; UI-Designer → `deepseek-v4-flash`; Architect → `qwen3.7-max`. El Tech-Lead despacha vía herramienta `task` con `subagent_type: <nombre-del-agente>`. Cada agente lee `CLAUDE.md` + `base/SKILL.md` + su SKILL de rol al arrancar.
- **Verificación independiente:** el Tech-Lead reproduce la evidencia él mismo (build/test/grep/inspección) antes de cerrar. No cierra sobre el reporte del ingeniero.
- **Política de pruebas y QA (actualizada 2026-06-21):** cada ingeniero escribe y corre sus propias pruebas cubriendo CADA criterio, entrega ya en verde con mapeo criterio→prueba + cobertura. **QA-Engineer es gate obligatorio en TODAS las Stories desde EPIC-0 sin excepción** — revisa lógica del código antes de correr tests, detecta bugs que los tests no cubren (ver qa-engineer/SKILL.md §1c). El Tech-Lead NO cierra ningún ticket sin veredicto APTO del QA. Herramienta de cobertura: `cargo llvm-cov --workspace --summary-only`. **Addendum Flutter (lección STORY-015):** el QA no puede aprobar código Flutter sin ejecutar `flutter build <platform>` — la revisión manual de código Dart no detecta errores de tipos entre bindings generados y código escrito. El gate de QA para Stories Flutter exige `flutter build` verde como condición mínima. Política de comentarios descriptivos obligatoria en todos los ingenieros (ver base/SKILL.md §"Política de Comentarios").
- **Comunicación:** "Habla en cristiano" — traducir códigos internos (EPIC-0, Wn, Gate, TTR…) a lenguaje llano (regla en `base/SKILL.md`).
- **Cambios documentales:** si se modifica algo, propagarlo coherentemente a TODOS los docs afectados; nada de cambios cosméticos.

## Bitácora cronológica

### 2026-06-12 — Arranque de EPIC-0
- **Plan de EPIC-0:** unificado dentro de `docs/ROADMAP.md` v2.0 (se borró `EXECUTION-PLAN.md`). Dos pistas paralelas: spikes de gates SPIKE-001–SPIKE-006 + tandas de cimentación (STORY-001→STORY-002→STORY-003-9→TASK-001-12). Usuario eligió arrancar por los cimientos.
- **STORY-001 (esqueleto) — Completado y auditado.** Rust-Engineer (Sonnet). Workspace Cargo con 8 crates de módulo (`ingest, generate, validate, incubate, manage, execute, feedback, withdraw`) + `shared`, patrón FCIS, cajas vacías. Auditoría Tech-Lead: `cargo build`/`cargo test` 9/9 verdes, 0 warnings, FCIS verificado por inspección.
  - Descubrimiento: el binario raíz `app` (SAD §4.2) NO se creó (criterio literal de STORY-001 no lo pedía). Decisión: se crea en **STORY-009** junto a la CLI. Anotado en ROADMAP.
- **STORY-002 (base de datos) — Completado y auditado.** Rust-Engineer (Sonnet). `migrations/0001_foundation_master_fields.sql` crea los 25 campos maestros exactos (ADR-0020 V2), SQLite WAL, idempotente. Test en `crates/shared/src/persistence/pool.rs`. Auditoría Tech-Lead: 25 campos verificados contra el ADR, test verde.
- **Escalamiento a Architect (Opus) — Contrato de 25 campos + `transformation_id`.** Veredicto:
  - Los 25 campos son **contrato lógico/vocabulario obligatorio**, NO 25 columnas calcadas en cada tabla. Grupo I (Identidad) universal; grupos II–V por **Filtro de Relevancia por Perfil** (`architect/SKILL.md`, `TEMPLATES.md`). La tabla ancla de EPIC-0 es correcta y se queda.
  - `transformation_id` = identificador (TEXT/UUID), no flag booleano.
  - **Propagado** (aprobado por usuario): ADR-0020 V2 "Aplicación" reescrita; SAD §17.9 y §20 alineados; glosa de `transformation_id` corregida en ADR + 8 módulos; typos "V2 V2" (7 features) y comillas sueltas (2 features) limpiados. Verificado por grep.
  - **Implicación para Sprint 1:** las tablas de STORY-003–STORY-008 aplican el filtro por perfil; NO copian 25 columnas.

### 2026-06-12 — STORY-003 (`clock`) completado y auditado
- Rust-Engineer (Sonnet). Reloj en `crates/shared`: `domain/clock.rs` (trait `Clock` + `DeterministicClock`, núcleo puro), `orchestrator.rs` (`SystemClock`, cáscara con `SystemTime::now()`), re-exportado en `public_interface.rs`.
- Auditoría Tech-Lead: build + clippy 0 warnings; 10 tests verdes incl. determinismo bit-a-bit (`deterministic_clock_same_seed_produces_identical_sequence`); FCIS verificado por grep (núcleo sin acceso a reloj real, cáscara sí).
- **Pendiente diferido a STORY-004:** auditoría del reloj (NTP offset, virtual_process_id, delta real/virtual). Requiere `audit-log` (STORY-004) + que el Architect defina el perfil de persistencia de `clock`. El engineer NO inventó campos (correcto). Anotado en bitácora del ROADMAP.

### 2026-06-12 — STORY-004 (`audit-log`) TTR-001 completado y auditado
- Rust-Engineer (Sonnet). Registro inmutable en `crates/shared`: `domain/audit_log.rs` (encadenamiento + verificación de hash, núcleo puro), `persistence/audit_log.rs` (repositorio append-only, sin update/delete), migración `0002_audit_log.sql` (tabla `audit_events` perfil Auditoría + triggers que abortan UPDATE/DELETE).
- Auditoría Tech-Lead: build + clippy `-D warnings` limpios; 22 tests verdes incl. detección de mutación de evento histórico, rechazo de UPDATE/DELETE por trigger, determinismo de cadena; FCIS verificado. Decisiones aceptadas: dep `uuid` (Rust puro), perfil Auditoría de `architect/SKILL.md`.
- TTR-002 (reconciliación Nautilus) fuera de alcance → EPIC-2+.

### 2026-06-12 — STORY-003 cerrado (Fase 2: rastro de auditoría del reloj)
- **Escalamiento al Architect (Opus):** las postcondiciones de `clock.md` citaban `ntp_sync_offset`/`virtual_process_id`/delta real-virtual como campos del catálogo ADR-0020 V2, pero NO existen ahí (referencia huérfana). Veredicto del Architect: son **payload de evento** (`details_json`), no columnas; el reloj emite a la bitácora existente, Perfil D; 3 eventos auditables (`CLOCK_NTP_SYNC`, `CLOCK_MODE_TRANSITION`, `CLOCK_SESSION_CLOSE`); sin cambios a ADR-0020 V2. Editó `clock.md`. Verificado por grep.
- **Rust-Engineer (Sonnet):** módulo de cáscara `crates/shared/src/clock_audit.rs` (`emit_ntp_sync`/`emit_mode_transition`/`emit_session_close` vía `AuditLogRepository::append`); dep `serde_json`. Auditoría Tech-Lead: clippy `-D warnings` limpio, 28 tests verdes, FCIS (núcleo `clock.rs` intacto) y granularidad del hot-path verificados. STORY-003 → ✅.

### 2026-06-12 — STORY-005 cerrado (async-job-executor) + política de pruebas
- **1ª ronda Rust-Engineer (Sonnet):** construyó núcleo `domain/job.rs`, `persistence/job.rs`, `orchestrator/job_executor.rs`, migración `0003_jobs.sql` (`jobs` mutable + `job_results` append-only). 60 tests verdes, clippy limpio. **PERO** la auditoría independiente del Tech-Lead detectó que el GATE de EPIC-0 (recuperación tras crash) no estaba probado: `recover_at_startup` con 0 invocaciones en tests y todo sobre `:memory:` (no demuestra durabilidad). Defecto de implementación → regresado.
- **Pregunta del usuario sobre testing → decisión:** se formalizó la política de pruebas (ver "Reglas activas"); se instaló `cargo-llvm-cov`. NO QA-Engineer dedicado.
- **2ª ronda Rust-Engineer (Sonnet):** añadió `jobs_survive_simulated_crash_and_are_recovered_on_restart` (DB en archivo, cierra/reabre pool, verifica QUEUED recuperado + RUNNING→QUEUED + nada perdido + evento `JOB_RECOVERED_AT_STARTUP`). Auditoría Tech-Lead: 62 tests verdes, clippy limpio, cobertura 90.80%, test del gate leído e inspeccionado. STORY-005 → ✅ (alcance EPIC-0). TTR-007 y cobertura del worker pool concurrente → EPIC-2+.
- **Nota:** la herramienta de subagentes de esta sesión NO tiene `SendMessage`, así que las correcciones se hacen con un Agent nuevo (contexto fresco) apuntando al código ya escrito, no continuando el agente previo.

### 2026-06-12 — Metodología: rename masivo F/W/G → Jira + sistema de Órdenes de Trabajo
- Se eliminaron los códigos F/W/G de TODO el repo (24 archivos). Nuevo esquema: EPIC-n, SPRINT-n, STORY-###, SPIKE-###, TASK-###, BUG-###. Archivos de ejecución renombrados a `docs/execution/STORY-00n-*.md`.
- Sistema spec-driven: cada trabajo tiene una Orden de Trabajo (`docs/execution/`) con el prompt exacto + comandos de validación + bitácora. Plantilla `_TEMPLATE.md`. Reglas en `base/SKILL.md` (sellado + comandos de validación) y `tech-lead/SKILL.md` (flujo de Órdenes).
- Build sigue verde; código intacto (el rename fue documental).

### 2026-06-16 — Reanudación de sesión: ROADMAP v3.0, ADR-0120 (Modos), Orden STORY-007 redactada
- **Contexto recuperado al arrancar:** entre la sesión anterior (2026-06-12/13) y esta, el usuario hizo varios commits documentales por su cuenta. Releí `docs/ROADMAP.md`, los ADR nuevos y los `SKILL.md` afectados antes de tocar nada (regla de reanudación de `tech-lead/SKILL.md`).
- **ADR-0118 (entrega por módulo completo):** reescribió `docs/ROADMAP.md` a v3.0 — ya no es bitácora, solo guía de orden + estado simple. Reasignó `crash-recovery` fuera de EPIC-0 (va a `execute`/EPIC-5). Esto invalidaba el "siguiente paso" que dejé anotado (STORY-006 crash-recovery) — corregido.
- **ADR-0119:** separación Plano de Control/Ejecución para operación distribuida (EPIC-9+). No afecta el trabajo inmediato de EPIC-0.
- **ADR-0120 (Modos de Acompañamiento):** nuevo mecanismo Autónomo/Mentor/Revisión por Agente, declarado en la Orden de Trabajo. Pregunté al usuario el Modo para el Rust-Engineer de la siguiente Story; eligió **Mentor**.
- **Orden `STORY-007-telemetry.md` creada** (`docs/execution/STORY-007-telemetry.md`): alcance TTR-001 de `telemetry` (buffer no bloqueante + heartbeat + persistencia SQLite con poda), Rust-Engineer en Modo Mentor. Diferidos documentados en §8 de la Orden: TTR-002 (→EPIC-7, necesita `feedback`), Builder ETA/gRPC/WebSocket y Best Strategy Tracker (→EPIC-3/EPIC-8, necesitan `generate`/UI headless), CPU/memoria por proceso (→STORY-008, mismo dominio que `worker-isolation-orchestrator`). Diseño de esquema (`metric_name`/`details_json` fuera del contrato de 25 campos) aplicado por precedente directo de `audit_events` (STORY-004), sin necesidad de escalar al Architect.
- **Bajo Modo Mentor, yo NO despacho** (ADR-0120): la Orden queda lista; el usuario decide cuándo invoca `/rust-engineer` pasándole la ruta de la Orden. Yo retomo auditoría y cierre cuando esa sesión termine.
- ROADMAP actualizado: fila STORY-007 → "en curso (Modo Mentor, Orden lista, pendiente invocación)" con enlace a la Orden.

### 2026-06-18 — Corrección de integridad de numeración (TASK-004→TASK-006) + protocolo aclarado
- **Origen:** el usuario notó que STORY-004 y TASK-004 compartían el número "4", violando la regla "secuencial global" de `tech-lead/SKILL.md`. Investigación con `git log` (fechas reales de creación de archivos y commits) para reconstruir la cronología antes de tocar nada.
- **Hallazgo:** STORY-001 a 005 se crearon juntas en un solo commit (2026-06-13 02:07). TASK (auditoría inundación) se creó 42 min después (02:49) — le correspondía el **6**, no el 4. "STORY-006" (crash-recovery) nunca tuvo archivo real: solo fue una mención de "siguiente paso" en un commit (17:11, ese mismo día) y una nota histórica en ADR-0118 — no es una colisión real, es una reserva fantasma. STORY-007 (`telemetry`) es trabajo ACTIVO (último commit real del repo) citado textualmente por ADR-0124 (protocolo de lecciones) — moverlo tenía mucho mayor radio de impacto que la única colisión real.
- **Decisión del usuario (Opción A, de un menú de 2):** renumerar SOLO TASK-004 → **TASK-006**. STORY-007 no se toca. `crash-recovery` sigue sin número de Story (correcto: ADR-0118 ya no pre-numera trabajo de épicas futuras).
- **Ejecutado:** `git mv` del archivo de la Orden + ID interno; referencias corregidas en ADR-0020 (línea del Registro de Mantenimiento), ADR-0120 (lista de compatibilidad retroactiva), ADR-0118 (nota histórica de "STORY-006" ahora explícita: nunca tuvo archivo, número liberado y reusado), `CONTENT-STRATEGY.md` (6 menciones, incl. una fila de tabla con placeholder obsoleto "STORY-006 — Próxima" corregida a STORY-007), y esta bitácora. Verificado con `grep -rn "TASK-004"` → 0 resultados reales restantes.
- **Protocolo de numeración aclarado** (pregunta del usuario: "¿por qué los Spikes no respetan la numeración global?" + "quiero poder insertar un Task entre épicas sin renumerar"). Se actualizó `tech-lead/SKILL.md` §"Vocabulario Ágil e Identificadores" con dos reglas que ya eran la práctica real pero no estaban escritas:
  1. **SPIKE-### tiene su PROPIO contador, independiente de Story/Task/Bug.** Los Spikes son una lista fija de 6 riesgos de viabilidad definida de antemano en el ROADMAP §6 (no se despachan incrementalmente como Stories/Tasks) — por diseño no comparten el contador global. Story/Task/Bug SÍ comparten un único contador secuencial entre ellos.
  2. **Solo la épica activa tiene numeración real.** Las épicas futuras (EPIC-1+) se listan en el ROADMAP por nombre de Feature/módulo, SIN número de Story pre-asignado — el número se asigna recién cuando el trabajo se despacha de verdad (se crea su Orden en `docs/execution/`). Esto es intencional: deja espacio para insertar un Task/Bug/Spike entre épicas sin tener que renumerar nada. `crash-recovery` (EPIC-5) es el ejemplo correcto: no tiene número todavía.
- **No se renumeraron los Spikes** (SPIKE-001-006): tienen su propio contador por diseño, no son una colisión real con Story/Task/Bug pese a reusar 1-6; renumerarlos habría sido una cascada masiva (citados en ADR-0107, ADR-0112 a ADR-0117, ROADMAP §6) para corregir algo que ya es coherente bajo la regla aclarada.

### 2026-06-18 — STORY-007 (`telemetry`, TTR-001) cerrada y auditada
- **Lo que pasó fuera de esta sesión del Tech-Lead:** el usuario arrancó en Modo Mentor (tecleó `TelemetrySampleContent`, defecto detectado en relectura: `process_id` duplicado), luego cambió la Story a **Modo Docente** (ADR-0122, nuevo) y el Rust-Engineer terminó la implementación completa con explicación profunda bloque a bloque. Lecciones formales en `docs/lessons/rust/STORY-007-telemetry.md` (ADR-0124: un archivo por Story, no por tema).
- **Auditoría independiente del Tech-Lead (reproducida, no tomada del reporte):** `cargo build --workspace` limpio; `cargo clippy --workspace --all-targets -- -D warnings` 0 warnings; `cargo test -p shared` → 76/76 verdes, verifiqué por nombre los 8 tests mapeados 1-a-1 contra los 8 criterios de la Orden (§5); `cargo llvm-cov --workspace --summary-only` → `domain/telemetry.rs` 100%, `orchestrator/telemetry.rs` 93.19% (coinciden exacto con lo reportado), `persistence/telemetry.rs` 94.29% (reportado 93.55%, variación menor). Inspección manual: `domain/telemetry.rs` sin `SystemTime`/`sqlx` (FCIS limpio); `migrations/0004_telemetry.sql` con columnas exactas del perfil + 2 índices justificados, sin triggers append-only (correcto, esta tabla SÍ borra por poda); 8 exports verificados en `public_interface.rs`.
- **Diseño de esquema** (`metric_name`/`details_json` fuera del contrato de 25 campos): aplicado por precedente directo de `audit_events` (STORY-004) — sin necesidad de escalar al Architect, ya resuelto.
- **Veredicto: APROBADO.** Sellado `docs/features/telemetry.md` (banner 🟡 Parcial, TTR-002 → EPIC-7) y `docs/ROADMAP.md` (fila STORY-007 → "parcial (TTR-002 → EPIC-7)").
- **Diferidos confirmados** (§8 de la Orden, sin cambios): TTR-002 → EPIC-7; Builder ETA/gRPC/WebSocket + Best Strategy Tracker → EPIC-3/EPIC-8; CPU/memoria por proceso → STORY-008.

### 2026-06-19 — Revisión de commits y staged; ROADMAP actualizado a v3.1

**Commits revisados (HEAD hasta 6353961):**
- `d852540` — auditoría de integridad ADR: 42 hallazgos corregidos en 124 ADRs (documental).
- `c879749` — ADR-0124 lecciones: un archivo por Story/Task, no por tema (ya estaba en PROGRESS).
- `32fcb2e` — `agentic-mcp-gateway` dual-cabine (documental).
- `c03ec68` — STORY-007 (`telemetry`) ya auditada y cerrada (registrada en entrada 2026-06-18).
- `7741e46` — ADR-0122 Modo Docente + protocolo de lecciones (ya en PROGRESS).
- Y commits anteriores ya documentados.

**Staged (nuevas decisiones arquitectónicas del Architect):**
- **ADR-0125-0128** — Capa de datos fundamentales:
  - ADR-0125: Event Study + Surprise como métodos canónicos; NLP a `moonshots`.
  - ADR-0126: Sourcing externo de hecho crudo + scoring 100% propio (Soberanía).
  - ADR-0127: PIT de eventos — arrival timestamp + vintage/as-of (first-print vs revisiones).
  - ADR-0128: Mapa de exposición evento→activo + normalización por instrumento.
- **ADR-0129** — Entradas concurrentes no bloqueantes por defecto + de-duplicación de señal (extiende ADR-0004 y ADR-0081).
- **ADR-0130** — Frecuencia/horizonte de operación como objetivo declarable + agnosticismo de temporalidad (extiende NSGA-II y backtest-engine).
- **4 features nuevas:** `fundamental-event-store` (→ `ingest`/EPIC-1), `event-impact-scorer` + `asset-exposure-map` + `fundamental-indicator-projector` (→ `generate`/EPIC-3).
- **Features modificadas:** `order-fsm`, `advanced-trade-management`, `backtest-engine`, `nsga2-optimizer` (extendidas por ADR-0129/0130).
- **Módulos actualizados:** `ingest`, `generate`, `validate`, `execute`, `manage` (nuevos TTRs incorporados).
- **SAD-21.md** — nueva sección del SAD.
- **Índices actualizados por el Architect:** `docs/ADR.md` (ADR-0125-0130 registrados), `docs/README.md` (4 features nuevas con módulo asignado).

**Qué actualizó el Tech-Lead:**
- `docs/ROADMAP.md` → v3.1 (2026-06-19):
  - EPIC-1: añadido `fundamental-event-store` (ADR-0126/0127).
  - EPIC-2: añadidos ADR-0129 (N posiciones concurrentes) y ADR-0130 (agnosticismo de temporalidad).
  - EPIC-3: añadidas 3 features fundamentales + objetivo de frecuencia/horizonte (ADR-0130).
  - EPIC-5: `order-fsm` y `advanced-trade-management` referenciados con ADR-0129.

**✅ STORY-008 CERRADA Y AUDITADA (2026-06-20).** Ver entrada de hoy abajo.
**✅ TASK-011 CERRADA (2026-06-20):** Architect enmendó ADR-0003 con la Regla de Tabla Única (una Feature → una tabla → un módulo dueño; TTR de Integración ≠ TTR de Construcción; "Consumido por" = accede al puerto). También actualizó ADR-0118 (referencia cruzada) y `docs/templates/FEATURE.md` (nota en "Dependencias y Bloqueantes"). El Gate de Coherencia Pre-Despacho del tech-lead ya incorporaba la regla; ahora puede citar **ADR-0003 §"Persistencia en Features Multi-Consumidor"**.
**✅ STORY-009 CERRADA Y AUDITADA (2026-06-20).** Ver entrada de hoy abajo.
**✅ STORY-010 CERRADA Y AUDITADA (2026-06-20).** Ver entrada de hoy abajo. 🟡 Parcial (TTR-001 UI + TTR-004 SaaS diferidos).
**✅ TASK-012 CERRADA (2026-06-20):** Architect creó ADR-0134 — matriz de plataformas (Windows/Linux/macOS nativos; iOS/Android cliente delgado gRPC; Web futuro incierto). Decisión prctl: optimización Linux-only; macOS y Windows usan keepalive file. ADR-0016/0030/0033 actualizados con refs cruzadas.
**✅ BUG-013 CERRADO (2026-06-20):** `prctl` gateado bajo `#[cfg(target_os = "linux")]` en `worker_runner.rs`. QA veredicto APTO. 103/103 tests verdes. Primera vez que el QA-Engineer actúa como gate obligatorio (nueva política de sesión).
**✅ STORY-014 CERRADA Y AUDITADA (2026-06-21):** Smoke test NautilusTrader. `nautilus-model =0.58.0` compila en el workspace. Crate stub `nautilus_compat` creado. Test `nautilus_crates_compile_and_basic_type_is_accessible` verde. Ningún tipo NT fuera del stub. QA APTO. 110 tests workspace verdes. Gate SPIKE-001 cerrado.
**✅ STORY-015 CERRADA Y AUDITADA (2026-06-21):** Panel Operativo Fundacional. Bridge (`crates/bridge`) compila limpio. `flutter build linux` verde (`build/linux/x64/release/bundle/drasus_ui`). `flutter test` verde (1 test, `pumpAndSettle` fix aplicado para animación TabBarView). QA APTO — 8/8 criterios. SPIKE-006 cerrado. Flutter SDK: `~/.flutter` v3.44.2 (git clone stable). Lección: bindings generados (`flutter_rust_bridge_codegen generate`) son la fuente de verdad de tipos — `u64` Rust = `BigInt` Dart, no `int`. Políticas actualizadas en qa-engineer/SKILL.md y tech-lead/SKILL.md: `flutter build` es gate obligatorio y prerequisito de SDK antes de despachar QA Flutter.
**➡️ SIGUIENTE PASO:** SPIKE-002-005 (marcar cerrados por ADR — los crates rechazados nunca estuvieron en el workspace, verificación trivial con `grep`) y arrancar EPIC-1.

### 2026-06-20 — STORY-010 (`agentic-mcp-gateway`) cerrada y auditada

**Auditoría independiente del Tech-Lead:**
- `cargo build --workspace` ✅ limpio.
- `cargo clippy --workspace --all-targets -- -D warnings` ✅ 0 warnings.
- `cargo test --workspace` → **103 verdes** (91 previos + 12 nuevos MCP).
- 12 tests MCP verificados por nombre (9 requeridos + 3 bonus: withdraw_denied, audit_hash_deterministic, audit_hash_differs).
- FCIS: `grep -n "sqlx\|tokio\|std::io" domain/mcp_gateway.rs` → 0 resultados. ✅
- Cobertura: `domain/mcp_gateway.rs` 93.18% · `persistence/mcp_gateway.rs` 97.91% · `orchestrator/mcp_server.rs` 9.60% (esperado — el servidor stdio no tiene test de integración sin cliente MCP real).
- Lección `docs/lessons/rust/STORY-010-agentic-mcp-gateway.md` creada (202 líneas, 5 bloques de enseñanza con código real).
- Crate MCP elegido: `rmcp` 1.7.0 (SDK oficial modelcontextprotocol, versión estable, 13M descargas).
- ADR-0123 sellado (✅), `agentic-mcp-gateway.md` sellado (🟡 Parcial — TTR-001 UI real + TTR-004 SaaS diferidos).
- **Veredicto: APROBADO.** ROADMAP fila STORY-010 → "🟡 parcial".

### 2026-06-20 — STORY-009 (`cli-app`) cerrada y auditada

**Auditoría independiente del Tech-Lead:**
- `cargo build --workspace` ✅ limpio.
- `cargo clippy --workspace --all-targets -- -D warnings` ✅ 0 warnings.
- `cargo test --workspace` → **93 verdes** (91 previos STORY-001-008 + 1 nuevo gate `kill -9` en `crates/app/tests/` + compilación de 8 módulos).
- Gate EPIC-0 verificado: `job_survives_kill9_and_recovers_on_restart` → OK. Salida real observada: "Recuperados 1 jobs del crash anterior." + evento `JOB_RECOVERED_AT_STARTUP` en BD.
- `drasus version` → `drasus v0.1.0` ✅; `drasus start` → arranca + apagado limpio ✅.
- FCIS: grep de `domain::\|persistence::\|orchestrator::` en `main.rs` → solo líneas de comentarios (7 y 93). ✅
- Cobertura: `app/src/main.rs` 95.77% líneas / 100% funciones. 8 regiones no cubiertas = rama `#[cfg(not(unix))]` (Windows) — inaccesible en Linux, aceptable.
- Lección `docs/lessons/rust/STORY-009-cli-app.md` creada (254 líneas), enlaza a la Orden y cita código real de esta Story.
- Decisión no especificada aceptada: re-exportaciones `create_pool`/`run_migrations` en `shared/public_interface.rs` para mantener ADR-0003 (el binario accede a `shared` solo por su `public_interface`).
- **Veredicto: APROBADO.** ROADMAP fila STORY-009 → "✅ terminado".

### 2026-06-20 — TASK-011 cerrada (Architect: enmienda ADR-0003)

**Escalamiento resuelto:**
- **ADR-0003** enmendado: sección "Persistencia en Features Multi-Consumidor (Regla de Tabla Única)" añadida. Fija 4 reglas FIJO: una Feature → una tabla → un módulo dueño (migración única); TTR de Integración = enchufar puerto, NUNCA migrar; "Consumido por" = accede al puerto; datos propios del consumidor van en tablas propias con referencia.
- **ADR-0118**: referencia cruzada bidireccional con ADR-0003 (Construcción vs Integración y su cara de persistencia).
- **`docs/templates/FEATURE.md`**: nota en "Dependencias y Bloqueantes" aclarando el significado de "Consumido por" para la persistencia.
- **Hallazgo del Architect:** ADR-0006 (Migraciones Centralizadas) ya hacía estructuralmente imposible la "tabla por consumidor" — la regla de ADR-0003 ahora lo hace explícito.
- **ADR a citar en Órdenes de Trabajo:** ADR-0003 §"Persistencia en Features Multi-Consumidor".

### 2026-06-20 — STORY-008 (`worker-isolation-orchestrator`) cerrada + SKILL.md actualizado

**STORY-008 — Auditoría independiente del Tech-Lead:**
- `cargo build --workspace` ✅ limpio.
- `cargo clippy --workspace --all-targets -- -D warnings` ✅ 0 warnings.
- `cargo test -p shared` → **91/91 verdes** (76 previos de STORY-001-007 + 16 nuevos de worker).
- Los 8 criterios del §5 tienen prueba nombrada presente y en verde: `shared_memory_access_latency_under_1ms`, `shared_memory_ram_constant_with_n_workers`, `shared_memory_worker_write_is_rejected`, `worker_graceful_shutdown_under_2s`, `worker_terminates_when_parent_drops`, `worker_jobs_recovered_to_queued_on_restart`, `worker_respects_max_concurrent_workers`, FCIS grep (0 imports de sistema en domain).
- Cobertura: `domain/worker_orchestrator.rs` 92.71% · `orchestrator/worker_runner.rs` 76.89% (aceptable, rutas OS-level parcialmente no ejercibles en CI sin procesos reales).
- FCIS verificado por grep: `domain/worker_orchestrator.rs` — cero imports de `std::process`, `tokio`, `memmap2`, `nix`. ✅
- Decisión de migración verificada: `process_id` ya existía en `jobs` (migración 0003) — no se añadió `0005_worker_pid.sql`. Correcto.
- Feature sellada por el Rust-Engineer durante la sesión Docente. ROADMAP actualizado a "terminado".
- **Modo Docente funcionó:** el Rust-Engineer completó bloques 1-4 con enseñanza bloque a bloque. Lección en `docs/lessons/rust/STORY-008-worker-isolation-orchestrator.md`.

**SKILL.md (tech-lead) — 3 actualizaciones de esta sesión (pendiente autorización para commit):**
1. Regla de git: autorización explícita por turno, sin herencia de sesiones previas.
2. Gate de Coherencia Pre-Despacho: auditoría ADR-0020 en 4 pasos (Grupo I, Perfil, Grupos coherentes, catálogo de 25) + claridad sobre variantes locales de latencia.
3. Modelo de tabla única por feature: una Feature → una tabla → un módulo dueño; consumidores usan el puerto, no duplican esquema.

**Auditoría ADR-0020 (143 features barridas):** catálogo sigue en **25 campos exactos** (confirmado en texto ADR + conteo + SQL migración 0001). Campos fuera del catálogo encontrados: 4 falsos positivos del parser (constantes/parámetros CONFIG) + campos locales legítimos ya documentados (agentic-mcp-gateway, variantes de latencia) + 2 sin clasificar explícita (`active_genome_domain`, `phase_id`) → locales válidos por ser de dominio único. Único candidato a vigilar: `latency_ns` en pre-trade-validator (1 feature; necesita 3+ para promover al catálogo).

### 2026-06-23 — Repaso de EPIC-0 contra la arquitectura hexagonal (ADR-0135/0136/0137)

**Contexto:** el usuario pidió repasar toda la Épica 0 tras el giro arquitectónico de los últimos commits (`d9d44bf`→`134c21d`). Releí base/tech-lead SKILL, ROADMAP, ADR-0135/0136/0137, estructura física de crates y los 15 commits.

**Verificación independiente reproducida:**
- `cargo test --workspace` → **verde tras la reestructura**: 103 (`shared`) + 1 (kill-9 en `app`) + 1 (smoke Nautilus) + 1 (preset) = 106 tests. `app`/`bridge`/`nautilus_compat` compilan.
- Estructura nueva confirmada: los 8 crates de módulo (cascarones de STORY-001) **fueron demolidos** por `98a8e7c`. Hoy el workspace es `shared` + `crates/features/<dominio>/` (7 dominios vacíos + `_TEMPLATE`) + `crates/presets/standard-pipeline` + `app` + `bridge` + `nautilus_compat`.
- **Todo el código de EPIC-0 sobrevivió** porque vive en `crates/shared` (clock, audit_log, job, telemetry, worker_orchestrator, mcp_gateway) + 5 migraciones. No se tocó.

**Desajustes encontrados (lo construido vs ADR-0137):**
1. **🔑 (duda madre, ESCALADA al Architect):** las 6 features de Fundación son módulos dentro de `shared`, no crates hexagonales bajo `crates/features/infrastructure/`. ADR-0137 + invariante de CLAUDE.md exigen "un crate hexagonal por feature", pero la *nota* de ADR-0137 bendice que los *tipos* de infra vivan en `shared` y calla sobre las *features*. Decisión del usuario: **escalar al Architect** para que enmiende el ADR y zanje si migran o se quedan.
2. Ninguna de las 6 features de Fundación tiene `## Puertos de Integración` (verificado con grep). Pendiente backfill — **bloqueado por el veredicto** (#1 define en qué contexto se declaran).
3. Ficha EPIC-0 del ROADMAP desactualizada ("8 módulos + shared", "monolito modular FCIS"). Pendiente reescritura — se bundle con el post-veredicto para no editar dos veces.
4. **SPIKE-002/003 cerrados hoy** (erradicación de `tch`/`libtorch`/`pysr`/`python` verificada ausente por grep; ROADMAP §6 actualizado). **SPIKE-004/005:** su validación residual NO es ejecutable en EPIC-0 (desempeño del motor → EPIC-2; LLM → EPIC-8). Esto choca con el criterio de salida literal de EPIC-0 ("los 6 SPIKE con su validación residual ejecutada") → señalado al usuario como wrinkle del criterio.

**Decisión del usuario (2 preguntas):** (a) duda madre → escalar al Architect; (b) alcance → re-alinear docs + plan de migración. El plan de migración queda **bloqueado** hasta el veredicto (no se especula su contenido).

**VEREDICTO DEL ARCHITECT (subagente Opus, 2026-06-23) — auditado y APROBADO por el Tech-Lead:**
- **Problema 1 → SE QUEDAN EN `shared`.** Las 6 features de infra crosscutting NO migran. ADR-0137 enmendado (bloque "Enmienda 2026-06-23 — Excepción bendecida", línea 106) con criterio anti-coladero de 3 condiciones (produce tipo `textLabel` del catálogo + consumida por ≥2 dominios + sin puerto de Alpha en canvas). `crates/features/infrastructure/` queda vacía a propósito. **La Orden de migración queda DESCARTADA** (no hay nada que migrar).
- **Problema 2 → criterio de salida de EPIC-0 reformulado** (ROADMAP línea 73): gate de Fundación = veredicto en ADR para los 6 + residual ejercible en EPIC-0 (smoke SPIKE-001/006, erradicación grep SPIKE-002/003); desempeño (SPIKE-004) → EPIC-2, LLM (SPIKE-005) → EPIC-8.
- **Archivos del Architect:** `docs/adr/ADR-0137.md`, `CLAUDE.md` §1, `docs/ADR.md`, `docs/ROADMAP.md` línea 73. **Auditoría:** grep G1=0 contradicciones, G2=enmienda en los 3 archivos, alcance respetado (cero cambios en `crates/`/código/git).
- **Tech-Lead además:** selló SPIKE-002/003 en ROADMAP §6; reescribió ficha EPIC-0 (Objetivo + Alcance hexagonal) + nota de reestructura bajo la tabla de estado.

**✅ Backfill de puertos COMPLETADO y auditado (2026-06-23, subagente Sonnet).** Las 6 features de Fundación (`clock`, `audit-log`, `async-job-executor`, `telemetry`, `worker-isolation-orchestrator`, `agentic-mcp-gateway`) ya tienen `## Puertos de Integración` (posición correcta Tareas→Puertos→Gobernanza/Persistencia en las 6, verificado por grep). Puertos OUTPUT declarados con tipos válidos del catálogo: `timestamp_ns` (clock), `AuditEvent` (audit-log, agentic-mcp-gateway), `Job` (async-job-executor out / worker-isolation in), `TelemetrySample` (telemetry).

**✅ HUECO RESUELTO (2026-06-24, Architect Sonnet, auditado y APROBADO):** los 6 puertos sin tipo de catálogo NO eran un defecto sino una categoría faltante. ADR-0137 enmendado (Enmienda 2026-06-24, línea 165) crea la categoría **"puerto interno"**: la firma Rust que una feature `textLabel` de infraestructura expone a sus llamadores directos, que NO es un conector del canvas y por tanto NO requiere tipo de catálogo (se documenta con `(interno)`). Tres criterios para calificar + regla de promoción (3+ features de dominios distintos → se promueve al catálogo). Regla FIJO reescrita: "todo puerto DE CANVAS requiere tipo" (antes "todo puerto"); puertos internos exentos; un nodo sin puertos de canvas no conecta (los internos no cuentan); ante la duda → puerto de canvas (más restrictivo). Las 6 features actualizadas con sus puertos internos completos; flags de escalamiento eliminados. Auditoría: grep A1=0 flags obsoletos, A2=6 puertos internos presentes, A3=regla antigua reemplazada + 1 sola sección FIJO, alcance respetado (cero cambios en `ui/`/`crates/`). Seguimiento natural: cuando se construya `agentic-mcp-gateway` en su épica, el Rust-Engineer confirma que `mcp_call_in` coincide con la firma real del protocolo MCP (ajustable sin enmienda, es firma interna).

**➡️ SIGUIENTE PASO:** EPIC-0 queda coherente con la arquitectura hexagonal (código verde + docs realineados). Pendiente para cerrar formalmente EPIC-0: nada bloqueante. Listo para arrancar EPIC-1 (`ingest`) cuando el usuario lo autorice. Recordatorio: hay cambios sin commitear (regla: git solo si el usuario lo pide; agrupar por tipo `docs`/`chore`).

### 2026-06-25 — Sub-fase de Estandarización de la Biblioteca de Componentes UI (formalización + arranque)

- **Origen:** el usuario reportó que la galería (biblioteca de componentes de producción) se construyó sin estándar entre varios modelos: colores/radios/padding hardcodeados, "glass mejorado" no seleccionable globalmente, color de fuente no configurable (texto invisible sobre fondo claro), bordes con glow fijo en vez del énfasis, cero comentarios de bloque, y bugs de interacción al hacer clic. Pidió estandarizar componente por componente (cobertura 100%) y formalizarlo dentro de EPIC-0 con el flujo completo de Tech-Lead.
- **Hallazgo de gobernanza:** el plan previo `tengo-feedback-1-en-peaceful-breeze.md` (2026-06-24) había definido STORY-016 (tema dinámico), 017 (Dashboard Shell), 018 (Canvas Shell) y 019 (Design System, ADR-0138) — **todas construidas en código pero nunca registradas** en el ROADMAP ni con Orden. Por eso ADR-0138 citaba "STORY-019" inexistente. El usuario eligió **formalización completa**.
- **Decisiones del usuario:** (1) modos de superficie N-extensibles (no un 4º fijo); (2) color de fuente auto por paleta + override manual; (3) selector de color híbrido (swatches + rueda HSV) uniforme; (4) refactor en 4 lotes Sonnet en paralelo, Modo **Autónomo**; (5) cobertura 100% sin muestreo + arreglo de bugs de interacción; (6) grabar la disciplina en los skills; (7) autorización al Tech-Lead para enmendar ADR-0138 sin escalar.
- **Hecho en esta sesión (gobernanza):**
  - Enmienda ADR-0138 (2026-06-25): "Tema Extensible — registro abierto de N propiedades". Índice `docs/ADR.md` completado (faltaban 0138 y 0139).
  - Órdenes retro `STORY-016..019` creadas en `docs/execution/` (estado real 🟡 parcial, nota ad-hoc).
  - Órdenes nuevas `STORY-020` (contrato de tokens, prompt de despacho listo) y `STORY-021` (estandarización 4 lotes + bugs, prompt común listo).
  - ROADMAP ficha EPIC-0: filas STORY-016 a 021 + nota de registro retroactivo.
  - Plan de trabajo: `.claude/plans/estamos-teniendo-problemas-importantes-hazy-cloud.md`.
- **✅ STORY-020 CERRADA (2026-06-25):** contrato de tokens congelado. 1ª entrega Flutter (Sonnet) → QA NO APTO (2 footguns `const` de superficie preexistentes, 1 borde hardcodeado introducido en el drawer, comentario inexacto, duplicación de mapa de paletas) → 5 correcciones aplicadas → Tech-Lead APROBADO (build linux verde reproducido; grep confirma 0 const de superficie, drawer usa `Gx.borderBase`, `kPalettes` única fuente). STORY-016 marcada ✅ (su pendiente de color de fuente lo entregó STORY-020). Artefactos nuevos: enum `enhancedGlass` + `kSurfaceModeRegistry` (registro N-extensible), `kTextDefaults` (texto oscuro para slate/paper), espejos estáticos `_globalAccent`/`_globalTextColor`, tokens `Gx.textBase*`/`borderBase`/`accentDynamic`/`borderHairline`/`borderFocus`/`space4..64`, widget `ui/lib/widgets/color_picker.dart` (swatches + rueda HSV reusable).

- **✅ STORY-021 CERRADA Y AUDITADA (2026-06-25):** estandarización total de las 13 secciones de la galería contra el contrato de tokens, build linux verde, QA APTO, cobertura 100%. Recorrido: 1ª tanda de 4 lotes cortada por límite de sesión a media tarea (recuperada sin perder trabajo) → re-tanda de 3 lotes sobre lo incompleto → QA NO APTO (hallazgo clave que el grep de literales NO ve: ~19 `Gx.borderPanel` estático en bordes de chrome → `Gx.borderBase`; tokens de texto estáticos en chrome; 3 radios sin token) → remediación → Tech-Lead APROBADO (greps de verificación en 0; 3 residuos `textLabel` en botones cerrados por el Tech-Lead). Skills actualizados (tarea de vigilancia): `flutter-engineer/SKILL.md` §"Biblioteca de Componentes — Contrato de Tokens" y `qa-engineer/SKILL.md` §"Gate de UI". **Registro histórico del corte (referencia):**
  - **Estado medido por sección** (grep de hardcodes residuales `Colors.white/black|Color(0x` y `BorderRadius.circular(` literal ≠999, y presencia de tokens nuevos `Gx.textBase/borderBase/accentDynamic/space`):
    - ✅ Avanzadas: `section_dataviz_quant.dart` (tokens=11, 0 hardcodes), `section_dataviz_extended.dart` (tokens=6, 0), `section_dag_nodes.dart` (tokens=11; quedan 3 radios literales).
    - 🟡 Parciales: `section_nav.dart` (tokens=2; 2 radios), `section_buttons_extended.dart` (1 Colors.white residual en loading button), `section_inputs_extended.dart` (3 radios), `section_data_display_extended.dart` (2 radios).
    - ❌ Sin tocar / casi nada: `section_dataviz_new.dart` (6 radios, 0 tokens), `section_std_missing.dart` (6 radios, 0 tokens), `section_feedback_extended.dart` (1 color, 2 radios, 0 tokens), `section_drasus_core_extended.dart` (1 color, 5 radios, 0 tokens), `section_animations.dart` (2 colores hardcoded — AccentAbSection, 0 tokens), `section_trade_tape.dart` (2 Colors.black en gradiente, 0 tokens).
  - (Resuelto: el reinicio re-despachó re-lotes A/B/C sobre lo incompleto; QA + remediación cerraron la Story. Lo de arriba queda como registro de cómo se recuperó el corte.)
  1. `grep -rnE "Colors\.(white|black)|Color\(0x" ui/lib/gallery/sections/` y `grep -rn "BorderRadius.circular(" ui/lib/gallery/sections/` (literales ≠ 999) → ver qué secciones ya están limpias y cuáles no.
  2. Por cada lote incompleto, re-despacha usando el prompt YA ESCRITO en `docs/execution/STORY-021-component-standardization.md` §4 (lotes: L1 inputs/buttons/std-missing · L2 nav/feedback/data-display · L3 las 3 dataviz · L4 dag-nodes/animations/trade-tape/drasus-core + widgets de componente de gallery_fx.dart).
  3. `cd ui && export PATH="$HOME/.flutter/bin:$PATH" && flutter analyze && flutter build linux --debug` definitivos (yo, no el reporte del agente).
  4. Gate de QA de STORY-021 (obligatorio): cobertura nominal 100% (checklist por lote vs código), hardcodes cero, reactividad a los N modos sobre paleta `paper` (claro) y `bunker` (oscuro), bordes/títulos en énfasis, bugs de interacción corregidos.
  5. Sellar STORY-021 (Estado ✅ + §7) y actualizar fila del ROADMAP.
  6. **Tarea pendiente 8:** grabar la disciplina en `.claude/skills/flutter-engineer/SKILL.md` y `.claude/skills/qa-engineer/SKILL.md` (contrato de tokens + comentarios + interacción probada + cobertura 100% sin muestreo; gate de UI de QA).
  - Contexto recuperable sin re-derivar: la Orden STORY-021 tiene los prompts completos y el mecanismo de checklist nominal permite ver qué falta.
- **➡️ SIGUIENTE PASO:** sub-fase de UI (STORY-016 a 021) **COMPLETA y auditada**. Biblioteca de componentes estandarizada y formalizada en EPIC-0. Pendiente: (a) decisión del usuario sobre commit (ver Git); (b) residuos de estilos no estandarizados (el usuario reporta que aún hay componentes fuera del estándar — pendiente diagnóstico con grep); (c) reanudar EPIC-1 (`ingest`) cuando el usuario lo autorice.
- **STORY-022 — Galería navegable y aislable (✅ cerrada 2026-06-26):** el usuario reportó que la galería de componentes era "un monolito" que impedía depurar componentes individuales. Diagnóstico real: no era un solo archivo (ya eran 14), sino que `gallery_tab.dart` renderizaba las 21 categorías (~150 widgets) juntas en un scroll único → imposible aislar. Solución (Opción 1, aprobada por el usuario): catálogo navegable maestro-detalle. Creado `lib/gallery/gallery_registry.dart` (modelo `GalleryEntry`/`GalleryCategory` + `buildGalleryCatalog`, builders bajo demanda); `gallery_tab.dart` reescrito de 1200→393 líneas (cáscara con panel lateral + buscador + detalle aislado). Despachado a Flutter-Engineer (Autónomo, Sonnet). En auditoría salió que las pruebas quedaron rojas por (1) overflow PREEXISTENTE de `AccentAbSection` (lo verifiqué aislando el widget a 380px → desborda 66px; no lo causó el refactor), (2) smoke test atado al diseño viejo, (3) goldens por regenerar. Re-despacho (alcance ampliado con autorización del usuario): overflow corregido (Row mainAxisSize.min + Flexible + ellipsis), smoke test reescrito con navegación real, goldens regenerados. Re-auditado: `flutter test` 3/3 verde, `flutter build linux` verde, `flutter analyze` 0 errores nuevos. Gate QA-Engineer → **APTO**. Orden `docs/execution/STORY-022-gallery-isolation.md` sellada ✅. **Fase 2 (planificada, NO ejecutada):** extraer ~40-50 componentes reutilizables a `lib/widgets/` (inventario en §8 de la Orden). **Pendiente git:** todo sin commitear; worktree huérfano `.claude/worktrees/agent-a1a9cd6a95fe207bf` por limpiar (requiere autorización).
- **Configuración de agentes opencode (2026-06-25):** creados 8 agentes en `.opencode/agents/` (rust-engineer, flutter-engineer, bridge-engineer, qa-engineer, quant-engineer, refactoring-engineer, ui-designer, architect) con modelos baratos asignados. SKILL de tech-lead actualizado con doble bloque de despacho (Bloque A = Claude Code, Bloque B = opencode). Ambos bloques incluyen lectura de `CLAUDE.md` + `base/SKILL.md` + SKILL de rol.
- **Git:** árbol sucio a propósito (ADR-0138, índice ADR, 6 Órdenes STORY-016..021, ROADMAP, PROGRESS, plan, 2 skills, tech-lead SKILL, 8 agentes opencode, + código UI de STORY-020/021). NADA commiteado. Git solo con autorización explícita del usuario, agrupando por tipo (`docs` para ADR/Órdenes/ROADMAP/skills, `feat(ui)` para el código).

### 2026-06-27 — Cierre de EPIC-0 + arranque de EPIC-1 (STORY-024 Sovereign Fetcher)

- **Worktrees de git revisados (a petición del usuario):** un solo worktree huérfano `.claude/worktrees/agent-a1a9cd6a95fe207bf` en commit `61c6d3a` (ancestro de `main` → ya integrado, cero commits propios, solo una carpeta `ui/lib/gallery/` sin seguimiento duplicada de lo que `main` ya tiene). **Eliminado con autorización del usuario** (`git worktree remove --force`). Árbol principal limpio.
- **EPIC-0 cerrada:** todos los criterios de salida cumplidos (esqueleto verde, migración de 25 campos, recuperación tras `kill -9`, 6 spikes con veredicto en ADR, Panel Operativo Fundacional). Corregida fila obsoleta de TASK-011 en el ROADMAP (decía "pendiente"; estaba cerrada desde 2026-06-20). EPIC-0 marcada ✅ en el mapa de entregas.
- **EPIC-1 arrancada.** Primera historia por la cadena de precondiciones de `ingest`: **STORY-024 `sovereign-data-fetcher`** (descarga híbrida Bulk+Delta, módulo TTR-006 = feature TTR-001+TTR-002). Es el primer crate hexagonal de dominio del proyecto (puebla `crates/features/data/`).
- **Gate de Coherencia Pre-Despacho corrido sobre `sovereign-data-fetcher.md`** (correcciones in-situ, ninguna requirió escalar al Architect):
  1. Persistencia: eliminada fila duplicada de `data_snapshot_id`; el "URL/endpoint de la fuente" pasó a campo propio `source_endpoint` (provenance, fuera del catálogo de 25). Eliminado `execution_latency_ms` (Grupo V, ajeno al Perfil A) — la duración la lleva el `Job`/telemetría. Nota de perfil añadida.
  2. Puertos: añadida `## Puertos de Integración` (faltaba). Salidas `ticks_out` (`Tick`) / `bars_out` (`Bars`); nodo fuente sin input de canvas. Tipos ya en el catálogo ADR-0137 → sin escalamiento.
  3. Decisión de tipos: `Tick`/`Bars` ya existen como STUBS en `crates/shared/src/types/mod.rs`; se completan ahí (no en el crate de la feature) por el invariante "cada crate de feature depende solo de `shared`".
- **Modo de Acompañamiento (decisión del usuario):** Rust-Engineer en **Modo Docente** → **lo despacho yo** (subagente Sonnet); el Ingeniero implementa y escribe la lección en `docs/lessons/rust/STORY-024-...md`. Etapas que no aplican: 0.5 + 3-4 (sin superficie UI propia; la barra de progreso es de `background-download-manager`) y 1+6 (Quant — no es fórmula/estrategia/métrica).
- **Barrido ADR ampliado del Gate (2026-06-27):** incorporados a la Orden ADR-0011 (descarga = Job durable), ADR-0105 + SAD §8 (datos crudos crudos → Polars en el transformador; **corrigió que la Orden original pedía parsear a structs**, contradiciendo el SAD), ADR-0093 (credenciales diferidas — datos públicos), ADR-0008/0012 (config + concurrencia consciente). SAD sin cambio (la Orden se alineó hacia él).
- **Diferidos** (§8 de la Orden): TTR-003 (webhook listener) + TTR-004 (conversor de datos alternativos) → datos alternativos, alcance mayor, EPIC-3+. Prueba de integración contra Binance Vision real → manual opcional (no CI); las pruebas usan adaptadores falsos de los puertos.
- **Pendiente git:** cambios sin commitear de esta sesión (feature spec corregida, Orden STORY-024 nueva, ROADMAP, este PROGRESS). Git solo con autorización explícita, agrupando por tipo (`docs`).

## Pendientes Windows (validación cruzada de plataforma)

- **✅ CERRADO (2026-06-20, commit `baaaac2`):** Al correr `cargo test` desde el toolchain MSVC de Windows via PowerShell/WSL2 se descubrió que `worker_runner.rs` y `kill9_recovery.rs` no compilaban en Windows (APIs `nix`, `pre_exec`, `/proc/stat` sin gate). Corregidos con `#[cfg(unix)]` / `#[cfg(not(unix))]`. Resultado: Linux 93/93 ✅ · Windows 89/89 ✅ (4 tests Unix-only excluidos). La rama `#[cfg(not(unix))]` de `main.rs` (SIGTERM fallback) sigue sin cobertura medible en Linux — sin impacto funcional, despliegue real es Linux.

## Pendientes / vigilancia

- **Sprint 1:** STORY-003 ✅ → STORY-004 🟡 (TTR-001 hecho; TTR-002 a EPIC-2+) → STORY-005 ✅ → STORY-007 🟡 (TTR-001 ✅ auditado 2026-06-18; TTR-002 a EPIC-7) → STORY-008 ✅ → STORY-009 ✅ → **STORY-010 (`agentic-mcp-gateway`, SIGUIENTE)**. `crash-recovery` (mencionado informalmente como "STORY-006" en su momento, sin archivo real) salió de EPIC-0 por ADR-0118 → ahora es trabajo de EPIC-5, sin número de Story asignado aún.
- **`kill -9` real (subproceso + SIGKILL):** diferido a STORY-009 (necesita binario raíz). El gate de STORY-005 ya está demostrado con el test de cierre/reapertura de DB en archivo.
- **Spikes de gates SPIKE-001–SPIKE-006:** aún no despachados (se decidió arrancar por cimientos). SPIKE-001 (smoke test NautilusTrader) es el único sin validar de fondo; SPIKE-002–SPIKE-006 tienen veredicto en ADR, resta validación residual. Bloquean el inicio de EPIC-1.
- **Git:** el árbol tiene cambios sin commitear (Orden STORY-007 nueva, ediciones a ROADMAP.md y PROGRESS.md). No se ha commiteado nada (regla: git solo si el usuario lo pide).
