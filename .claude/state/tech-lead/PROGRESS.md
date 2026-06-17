# Bitácora Operativa — Tech-Lead (Drasus Engine)

> Memoria viva entre sesiones. El Tech-Lead la LEE al arrancar (Etapa 0) y la ACTUALIZA al cerrar cada tarea/decisión.
> Este archivo es el TABLERO/índice: dónde estamos + siguiente paso. El DETALLE de cada trabajo vive en su Orden de Trabajo (`docs/execution/<ID>-<slug>.md`); el estado de fase vive en `docs/ROADMAP.md`. No dupliques detalle aquí: apunta a la Orden.
> Sistema de seguimiento (Spec-Driven): cada trabajo se ejecuta desde una Orden de Trabajo con el prompt exacto + comandos de validación. Plantilla: `docs/execution/_TEMPLATE.md`.

---

## Estado actual

- **Fase activa:** EPIC-0 — Fundación.
- **Última sesión:** 2026-06-16.
- **✅ TASK-004 (Auditoría Inundación de Fundaciones) CERRADA** (2026-06-13). Fases 1-4 completas y auditadas (ver `docs/execution/TASK-004-...md`). 137 features + 8 módulos auditados; perfiles reasignados, contratos diseñados, Grupo I completo en todo el corpus, ADR-0020 expone los 3 campos transversales (conteo se mantiene en 25), TEMPLATES arreglado (causa raíz P1). Commits: `bace15c` (fase 1), `4bf76b3` (decisiones fase 2), `ef6ca36` (fase 3). **Mantra del usuario** grabado en base/SKILL.md ("ante la duda, prefiero tenerlo y no necesitarlo").
- **⚠️ Cambio de rumbo del ROADMAP (2026-06-16, ADR-0118):** `crash-recovery` (antes "STORY-006") **YA NO es de EPIC-0** — pertenece a `execute`/EPIC-5 (necesita el conector de bróker, que no existe hasta entonces). El gate de recuperación tras `kill -9` que EPIC-0 sí exige ya está cubierto por `async-job-executor` (STORY-005, cerrado). El ROADMAP se reescribió a v3.0 (guía de orden + estado simple, sin bitácora narrativa — el detalle vive en las Órdenes de Trabajo).
- **🎚️ Nuevo mecanismo (ADR-0120, 2026-06-16):** cada Agente de una Orden declara un **Modo de Acompañamiento** — Autónomo (despacho yo vía `Agent`) / Mentor (el usuario teclea, el Ingeniero dicta bloque a bloque) / Revisión (el usuario entrega código, el Ingeniero audita). Se declara en la Orden, nunca en el chat. Bajo Mentor/Revisión yo NO despacho: redacto la Orden y me detengo; el usuario invoca el skill del Ingeniero directamente.
- **➡️ SIGUIENTE PASO CONCRETO:** Orden **STORY-007 (`telemetry`)** redactada y lista en `docs/execution/STORY-007-telemetry.md` — Rust-Engineer en **Modo Mentor** (decisión del usuario). Alcance: TTR-001 (buffer no bloqueante + heartbeat + persistencia + poda); TTR-002 y los sub-comportamientos avanzados (Builder ETA, Heap Monitor, Best Strategy Tracker, CPU/mem) quedan diferidos (detalle en §8 de la Orden). **Falta:** que el usuario invoque `/rust-engineer` pasándole esa Orden para arrancar la sesión de Mentor; yo audito y cierro cuando termine. Después: STORY-008 (`worker-isolation`), STORY-009 (CLI + binario raíz `app`). Transversal: los 6 spikes de gates (SPIKE-001–006) antes de cerrar EPIC-0 / arrancar EPIC-1 — solo SPIKE-001 (smoke test NautilusTrader) sin validar de fondo.
- **Pendiente diferido:** auditoría de Inundación de Fundaciones en los 41 moonshots (misma estrategia, TASK futura — tarea #6 del tablero).
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

## Pendientes / vigilancia

- **Sprint 1:** STORY-003 ✅ → STORY-004 🟡 (TTR-001 hecho; TTR-002 a EPIC-2+) → STORY-005 ✅ → **STORY-007 (telemetry, Orden lista en Modo Mentor, SIGUIENTE: usuario invoca `/rust-engineer`)** → STORY-008 (worker-isolation) → STORY-009 (CLI + binario raíz `app`). `crash-recovery` (antes "STORY-006") salió de EPIC-0 por ADR-0118 → ahora es trabajo de EPIC-5.
- **`kill -9` real (subproceso + SIGKILL):** diferido a STORY-009 (necesita binario raíz). El gate de STORY-005 ya está demostrado con el test de cierre/reapertura de DB en archivo.
- **Spikes de gates SPIKE-001–SPIKE-006:** aún no despachados (se decidió arrancar por cimientos). SPIKE-001 (smoke test NautilusTrader) es el único sin validar de fondo; SPIKE-002–SPIKE-006 tienen veredicto en ADR, resta validación residual. Bloquean el inicio de EPIC-1.
- **Git:** el árbol tiene cambios sin commitear (Orden STORY-007 nueva, ediciones a ROADMAP.md y PROGRESS.md). No se ha commiteado nada (regla: git solo si el usuario lo pide).
