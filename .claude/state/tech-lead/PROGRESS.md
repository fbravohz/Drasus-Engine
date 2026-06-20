# Bitácora Operativa — Tech-Lead (Drasus Engine)

> Memoria viva entre sesiones. El Tech-Lead la LEE al arrancar (Etapa 0) y la ACTUALIZA al cerrar cada tarea/decisión.
> Este archivo es el TABLERO/índice: dónde estamos + siguiente paso. El DETALLE de cada trabajo vive en su Orden de Trabajo (`docs/execution/<ID>-<slug>.md`); el estado de fase vive en `docs/ROADMAP.md`. No dupliques detalle aquí: apunta a la Orden.
> Sistema de seguimiento (Spec-Driven): cada trabajo se ejecuta desde una Orden de Trabajo con el prompt exacto + comandos de validación. Plantilla: `docs/execution/_TEMPLATE.md`.

---

## Estado actual

- **Fase activa:** EPIC-0 — Fundación.
- **Última sesión:** 2026-06-16.
- **✅ TASK-006 (Auditoría Inundación de Fundaciones) CERRADA** (2026-06-13; renumerada desde TASK-004 el 2026-06-18, ver entrada de esa fecha). Fases 1-4 completas y auditadas (ver `docs/execution/TASK-006-...md`). 137 features + 8 módulos auditados; perfiles reasignados, contratos diseñados, Grupo I completo en todo el corpus, ADR-0020 expone los 3 campos transversales (conteo se mantiene en 25), TEMPLATES arreglado (causa raíz P1). Commits: `bace15c` (fase 1), `4bf76b3` (decisiones fase 2), `ef6ca36` (fase 3). **Mantra del usuario** grabado en base/SKILL.md ("ante la duda, prefiero tenerlo y no necesitarlo").
- **⚠️ Cambio de rumbo del ROADMAP (2026-06-16, ADR-0118):** `crash-recovery` (antes "STORY-006") **YA NO es de EPIC-0** — pertenece a `execute`/EPIC-5 (necesita el conector de bróker, que no existe hasta entonces). El gate de recuperación tras `kill -9` que EPIC-0 sí exige ya está cubierto por `async-job-executor` (STORY-005, cerrado). El ROADMAP se reescribió a v3.0 (guía de orden + estado simple, sin bitácora narrativa — el detalle vive en las Órdenes de Trabajo).
- **🎚️ Nuevo mecanismo (ADR-0120, 2026-06-16):** cada Agente de una Orden declara un **Modo de Acompañamiento** — Autónomo (despacho yo vía `Agent`) / Mentor (el usuario teclea, el Ingeniero dicta bloque a bloque) / Revisión (el usuario entrega código, el Ingeniero audita). Se declara en la Orden, nunca en el chat. Bajo Mentor/Revisión yo NO despacho: redacto la Orden y me detengo; el usuario invoca el skill del Ingeniero directamente.
- **✅ STORY-007 (`telemetry`, TTR-001) CERRADA y auditada** (2026-06-18). Ver entrada de esa fecha más abajo.
- **➡️ SIGUIENTE PASO CONCRETO:** despachar **STORY-008 (`worker-isolation-orchestrator`)** — preguntar al usuario el Modo de Acompañamiento (ADR-0120) antes de redactar la Orden, igual que se hizo para STORY-007. Después: STORY-009 (CLI + binario raíz `app`), STORY-010 (`agentic-mcp-gateway`, núcleo MCP + evaluador de permisos, ADR-0123). Transversal: los 6 spikes de gates (SPIKE-001–006) antes de cerrar EPIC-0 / arrancar EPIC-1 — solo SPIKE-001 (smoke test NautilusTrader) sin validar de fondo.
- **Pendiente diferido:** auditoría de Inundación de Fundaciones en los 41 moonshots (misma estrategia, TASK futura).
- **🔢 Corrección de integridad de numeración (2026-06-18):** TASK-004 colisionaba con STORY-004 (ambos usaban "4" en el contador global Story/Spike/Task/Bug). Renumerada a **TASK-006** (siguiente número global libre, cronológicamente correcto). Protocolo de numeración aclarado en `tech-lead/SKILL.md` §"Vocabulario Ágil e Identificadores" (ver entrada de hoy abajo).
- **Nomenclatura:** ya NO se usan códigos F/W/G. Identificadores estilo Jira: EPIC-n, SPRINT-n, STORY-###, SPIKE-###, TASK-###, BUG-###. Cada Story se ejecuta desde su Orden de Trabajo en `docs/execution/`.

## Reglas activas confirmadas con el usuario

- **Modelos:** ingenieros NUNCA en Opus. Sonnet por defecto / tareas críticas; Haiku solo mecánicas. El Architect SÍ en Opus (necesita mucho contexto). El Tech-Lead despacha subagentes vía herramienta Agent (subagent_type `general-purpose`) cargando `base/SKILL.md` + el SKILL del rol.
- **Verificación independiente:** el Tech-Lead reproduce la evidencia él mismo (build/test/grep/inspección) antes de cerrar. No cierra sobre el reporte del ingeniero.
- **Política de pruebas (decidida 2026-06-12):** cada ingeniero escribe y corre sus propias pruebas unitarias/integración cubriendo CADA criterio de aceptación, y entrega ya en verde con el mapeo criterio→prueba + cobertura. El Tech-Lead verifica COBERTURA DEL CRITERIO (no solo "tests verdes"): cada criterio crítico debe tener una prueba que lo ejerza de verdad (ej. durabilidad/recuperación → DB en archivo, NUNCA `:memory:`). NO se usa un QA-Engineer dedicado en EPIC-0 (queda disponible para escalar casos puntuales). Herramienta de cobertura: `cargo llvm-cov --workspace --summary-only` (instalada). Formalizado en `rust-engineer/SKILL.md` §6, `tech-lead/SKILL.md` (Verificación Independiente) y la plantilla `docs/execution/_TEMPLATE.md` (tabla criterio↔prueba + comando de cobertura).
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
**⚠️ TASK-011 ABIERTA:** escalamiento al Architect — enmienda ADR-0003 (tabla única por feature + TTRs de integración vs construcción). Pendiente invocación del Architect.
**➡️ SIGUIENTE PASO:** invocar Architect para TASK-011 (ADR-0003 enmienda) → luego STORY-009 (CLI + binario `app`). Secuencia EPIC-0 restante: STORY-009 → STORY-010 (`agentic-mcp-gateway`). Luego SPIKE-001-006.

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

## Pendientes / vigilancia

- **Sprint 1:** STORY-003 ✅ → STORY-004 🟡 (TTR-001 hecho; TTR-002 a EPIC-2+) → STORY-005 ✅ → STORY-007 🟡 (TTR-001 ✅ auditado 2026-06-18; TTR-002 a EPIC-7) → **STORY-008 (worker-isolation, SIGUIENTE)** → STORY-009 (CLI + binario raíz `app`) → STORY-010 (`agentic-mcp-gateway`, ADR-0123). `crash-recovery` (mencionado informalmente como "STORY-006" en su momento, sin archivo real) salió de EPIC-0 por ADR-0118 → ahora es trabajo de EPIC-5, sin número de Story asignado aún.
- **`kill -9` real (subproceso + SIGKILL):** diferido a STORY-009 (necesita binario raíz). El gate de STORY-005 ya está demostrado con el test de cierre/reapertura de DB en archivo.
- **Spikes de gates SPIKE-001–SPIKE-006:** aún no despachados (se decidió arrancar por cimientos). SPIKE-001 (smoke test NautilusTrader) es el único sin validar de fondo; SPIKE-002–SPIKE-006 tienen veredicto en ADR, resta validación residual. Bloquean el inicio de EPIC-1.
- **Git:** el árbol tiene cambios sin commitear (Orden STORY-007 nueva, ediciones a ROADMAP.md y PROGRESS.md). No se ha commiteado nada (regla: git solo si el usuario lo pide).
