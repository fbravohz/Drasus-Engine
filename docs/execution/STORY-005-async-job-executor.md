# STORY-005 · Ejecutor de trabajos asíncrono (async-job-executor)

> Orden de Trabajo Spec-Driven. El prompt EXACTO del agente + comandos de validación + bitácora. Vive en git, no en el chat.

| Campo | Valor |
|---|---|
| **ID** | STORY-005 |
| **Título** | Ejecutor de trabajos asíncrono con recuperación tras crash |
| **Tipo** | Story |
| **Épica (Fase)** | EPIC-0 — Fundación |
| **Sprint** | 1 |
| **Estado** | ✅ Completado (alcance EPIC-0: TTR-001..006; gate de recuperación demostrado) |
| **Responsable** | Rust-Engineer (Sonnet) · auditó Tech-Lead |
| **Creada** | 2026-06-12 |
| **Completada** | 2026-06-12 |

## 1. Especificación de origen (qué specs implementa)
- **Feature:** [`async-job-executor`](../features/async-job-executor.md)
- **TTR(s) EN ALCANCE:** TTR-ASYNC-EXECUTOR-001 (job queue Tokio+SQLite), -002 (worker pool), -003 (persistencia jobs/job_results), -004 (recuperación en startup ← **criterio de salida de EPIC-0**), -005 (progreso/estimación), -006 (cancelación).
- **TTR(s) FUERA DE ALCANCE (secuenciados):** TTR-ASYNC-EXECUTOR-007 (integración con módulos costosos generate/validate/manage/incubate/feedback) — esos módulos no existen aún (EPIC-2+). Entra cuando existan.
- **ADR(s):** ADR-0011 (operaciones asíncronas), ADR-0016 (Local-First), ADR-0020 (contrato de campos por perfil), ADR-0003 (FCIS).

## 2. Objetivo (una frase llana)
Una cola de trabajos costosos que se procesan en segundo plano y que, si el sistema se cae de golpe, no pierde ningún trabajo: al reiniciar los recupera del disco y los vuelve a encolar.

## 3. Instrucciones de despacho (la spec ejecutable)
```
Eres el Rust-Engineer de Drasus Engine. Tarea STORY-005 (Épica 0, la pieza con el criterio de salida de la fase). STORY-001/002/003/004 ya aprobadas: existen el workspace FCIS, el pool SQLite con migraciones, el reloj (puerto Clock) y la bitácora append-only (AuditLogRepository).

PASOS DE ARRANQUE: 1) Lee `.claude/skills/base/SKILL.md` y declara `[base/SKILL.md leído y activo]`. 2) Lee `.claude/skills/rust-engineer/SKILL.md`. 3) Lee `docs/features/async-job-executor.md` COMPLETO. 4) Lee los ADRs citados que necesites (ADR-0011 patrón async, ADR-0016 Local-First, ADR-0020 campos por perfil). 5) Lee el código existente que vas a reusar como patrón, NO reinventes: `crates/shared/src/persistence/pool.rs` (connect/migrate), `migrations/0002_audit_log.sql` (patrón de migración por perfil), `crates/shared/src/persistence/audit_log.rs` (patrón de repositorio + uso del puerto Clock), `crates/shared/src/domain/clock.rs` y `crates/shared/src/lib.rs`/`public_interface.rs`.

ALCANCE (SOLO esto): TTR-001 (submit con persistencia antes del ack), TTR-002 (worker pool con límite max_concurrent_jobs), TTR-003 (persistencia jobs + job_results), TTR-004 (recuperación en startup), TTR-005 (progreso 0-100 + estimación), TTR-006 (cancelación). FUERA: TTR-007 (integración con módulos generate/validate/etc.) — esos módulos NO existen; NO los toques ni los crees.

UBICACIÓN: el executor es infraestructura transversal -> hogar `crates/shared` (igual que clock y audit-log), salvo que ADR-0003/SAD indiquen otra cosa (justifícalo si te desvías). FCIS estricto:
- NÚCLEO (domain): la máquina de estados del job pura — transiciones válidas (QUEUED->RUNNING->COMPLETED/FAILED/CANCELLED, RUNNING->QUEUED en recuperación), validación de transición, cálculo de estimación de tiempo. Sin I/O, sin reloj real, sin UUID generado dentro del núcleo (se inyectan, como en audit_log.rs).
- CÁSCARA (persistence/orchestrator): pool SQLite, generación de UUID, lectura del puerto Clock, el runtime Tokio, los workers, la cola en memoria, la recuperación en startup.

PERSISTENCIA: crea la migración `migrations/0003_jobs.sql` con tablas `jobs` y `job_results`. Aplica el FILTRO DE RELEVANCIA POR PERFIL (ADR-0020): Grupo I universal + EXACTAMENTE los campos que `async-job-executor.md` sección "Gobernanza y Estándares" lista (concurrencia: process_id/session_id/node_id; integridad: audit_chain_hash/logic_hash/event_sequence_id; soberanía: owner_id/access_token_id) + las columnas FUNCIONALES propias (uuid, user_id, job_type, parameters, state, progress, timestamps; y en job_results: job_uuid, result_data, error_message, completed_at). NO calques los 25 campos. `jobs` es mutable (state/progress se actualizan); `job_results` es APPEND-ONLY (sigue el patrón de triggers de `0002_audit_log.sql`: RAISE(ABORT) en UPDATE/DELETE). SQLite WAL. Migración idempotente.

REGLAS DURAS de la feature:
- El job se PERSISTE en SQLite ANTES de retornar el UUID (durabilidad; si no, un kill -9 entre el ack y el commit pierde el job).
- job_results NUNCA se modifica tras insertar (append-only por trigger, como audit_events).
- Nunca más de max_concurrent_jobs workers a la vez.
- Recuperación en startup: jobs en QUEUED se reencolan; jobs en RUNNING pasan a QUEUED (no sabemos si completaron) y se reencolan; se registra un evento de auditoría por la bitácora existente (`AuditLogRepository::append`) tipo "JOB_RECOVERED_AT_STARTUP" con job_uuid y previous_state en details_json (reusa el patrón de clock_audit.rs; NO inventes otra forma de auditar).

CRITERIO DE SALIDA (el gate de toda la Épica 0) — DEBES demostrarlo con tests:
1. OBLIGATORIO (gate): test de integración de recuperación sobre DB EN ARCHIVO (no en memoria; usa un archivo temporal, p. ej. tempfile, para que sobreviva la "caída"): (a) inicializa el executor, encola jobs, deja alguno en RUNNING; (b) suelta/cierra el executor SIN completar los jobs (simula el crash: el estado en disco es la única verdad); (c) reabre el executor sobre la MISMA DB; (d) verifica que los jobs QUEUED y RUNNING fueron recuperados (RUNNING->QUEUED), reencolados, y que se registró el evento de auditoría de recuperación. Ningún job se pierde.
2. DESEABLE (prueba de guerra real): si es viable SIN el binario raíz `app` (que aún no existe, llega en STORY-009), añade un test que lance un SUBPROCESO (un `examples/` o bin de prueba del crate que encole un job y entre en bucle), le envíe SIGKILL real (kill -9) con `std::process::Command`/nix, y luego reabra la DB y verifique la recuperación. Si NO es viable todavía sin el binario raíz, NO lo fuerces: déjalo documentado como pendiente para STORY-009 y entrega el test (1) como evidencia del gate. Repórtalo claramente.

VALIDACIÓN GENERAL: `cargo build --workspace`, `cargo clippy --workspace --all-targets -- -D warnings` (CERO warnings), `cargo test -p shared` verde. Tests unitarios del núcleo (máquina de estados: transiciones válidas e inválidas) + los de integración de recuperación.

LÍMITES: Solo STORY-005 (TTR-001..006). NO TTR-007. NO crees módulos de negocio. NO inventes campos fuera de async-job-executor.md / ADR-0020 (si una columna del perfil es ambigua para `jobs` vs `job_results`, repórtalo como BLOQUEO con cita, NO la inventes). NO modifiques `docs/` (eso lo sella el Tech-Lead). NO cambies migraciones 0001/0002. Código y comentarios en inglés.

ENTREGABLE (repórtamelo): 1) dónde ubicaste el executor y la separación núcleo/cáscara; 2) el esquema de `0003_jobs.sql` (qué columnas, por qué, qué es append-only); 3) cómo garantizas "persistir antes del ack"; 4) la lista de tests (núcleo + recuperación) y qué prueba cada uno; 5) si lograste o no el test de kill -9 real y por qué; 6) salida de build/clippy/test; 7) ambigüedades/bloqueos.
```

## 4. Criterio de aceptación
- **Gate EPIC-0:** test de integración que demuestra recuperación tras crash sobre DB en disco — ningún job QUEUED/RUNNING se pierde; RUNNING→QUEUED; evento de auditoría de recuperación registrado.
- Durabilidad: el job se persiste en SQLite **antes** de devolver el UUID.
- `job_results` append-only (UPDATE/DELETE rechazado por trigger, patrón de `0002`).
- Concurrencia acotada por `max_concurrent_jobs`.
- Migración `0003_jobs.sql` idempotente, WAL, filtro por perfil (NO 25 columnas calcadas).
- FCIS: máquina de estados pura en el núcleo; Tokio/SQLite/UUID/Clock en la cáscara.
- `cargo clippy --workspace --all-targets -- -D warnings` limpio; `cargo test -p shared` verde.

## 5. Comandos de validación (para el usuario — copy/paste)
```bash
cd /var/home/fbravohz/Documentos/Entornos/Personal/Drasus-Engine
cargo test -p shared                                   # incluye el test de recuperación tras crash
cargo clippy --workspace --all-targets -- -D warnings  # debe salir sin warnings
# La migración nueva existe y define jobs + job_results
ls migrations/0003_jobs.sql
# job_results debe ser append-only (triggers que abortan UPDATE/DELETE, patrón de 0002)
grep -nE "TRIGGER|RAISE\(ABORT" migrations/0003_jobs.sql
# FCIS: el núcleo (máquina de estados) NO debe tocar Tokio/SQLite/SystemTime (esperado: vacío)
grep -rnE "tokio|sqlx|SystemTime|Uuid::new" crates/shared/src/domain/ | grep -i job
```

## 6. Registro de ejecución (bitácora cronológica)
- 2026-06-12 · Despachada a Rust-Engineer (Sonnet) por el Tech-Lead. Alcance acotado por Regla del Tech-Lead (TTR-001..006; TTR-007 secuenciado a EPIC-2+).
- 2026-06-12 · Rust-Engineer (Sonnet) · **ENTREGA INCOMPLETA** (el reporte del agente se truncó por límite de sesión). Auditoría independiente del Tech-Lead:
  - ✅ Construido: núcleo puro `domain/job.rs` (máquina de estados, 23 tests), cáscara `persistence/job.rs` (repositorio, 9 tests) y `orchestrator/job_executor.rs` (workers, cola, `recover_at_startup`, `cancel`, progreso). Migración `0003_jobs.sql`: `jobs` mutable + `job_results` append-only por triggers (`event_sequence_id` con cadena de hash). FCIS verificado (núcleo sin tokio/sqlx/SystemTime/UUID). Build + clippy `-D warnings` limpios; 60 tests verdes.
  - ❌ **DEFECTO DE IMPLEMENTACIÓN (bloqueante del gate):** el criterio de salida de EPIC-0 — test de recuperación tras crash sobre **DB en archivo** — NO está cubierto. `recover_at_startup` tiene **0 invocaciones en tests**; los 9 tests de `persistence/job.rs` usan `sqlite::memory:` (no sobrevive a reabrir → no demuestra durabilidad); no hay test que persista en disco, suelte el executor y reabra la misma DB. "Todo verde" NO equivale a "gate demostrado".
  - **Veredicto Tech-Lead:** NO se cierra. Regresa al Rust-Engineer para añadir SOLO el test del gate (DB en archivo con `tempfile`, RUNNING→QUEUED, evento `JOB_RECOVERED_AT_STARTUP` verificado, ningún job perdido). No es defecto de diseño → no se escala al Architect.
- 2026-06-12 · Rust-Engineer (Sonnet, 2ª ronda) · **APROBADO** · Añadió el test del gate `jobs_survive_simulated_crash_and_are_recovered_on_restart` (+ `recover_at_startup_on_empty_database_is_a_noop`) sobre SQLite en archivo temporal: encola 3 jobs, lleva uno a RUNNING, cierra el pool (simula `kill -9`), reabre pool nuevo sobre el MISMO archivo y verifica (a) QUEUED recuperado, (b) RUNNING→QUEUED, (c) ningún job perdido + completado intacto, (d) evento `JOB_RECOVERED_AT_STARTUP` con `previous_state` por cada job recuperado. Solo se añadió el `mod tests`; cero cambios a lógica de producción. **Auditoría independiente del Tech-Lead:** `cargo test -p shared` 62 verdes incl. el del gate; `cargo clippy --workspace --all-targets -- -D warnings` limpio; `cargo llvm-cov --workspace --summary-only` = **90.80% líneas**. Test del gate leído e inspeccionado (usa archivo real, reabre pool, asserts completos). **CERRADO.**

## 4b. Mapeo criterio → prueba (verificado por Tech-Lead)
| Criterio | Prueba que lo demuestra | Estado |
|---|---|---|
| Recuperación tras crash (gate EPIC-0) | `jobs_survive_simulated_crash_and_are_recovered_on_restart` | ✅ |
| Durabilidad: persistir antes del ack | `submit_persists_job_in_queued_state` | ✅ |
| `job_results` append-only | `job_results_update_and_delete_are_rejected_by_triggers` | ✅ |
| Transiciones de estado válidas/ inválidas | `transition_*` + 23 tests de `domain/job.rs` | ✅ |
| Recuperación no-op sobre DB vacía | `recover_at_startup_on_empty_database_is_a_noop` | ✅ |
| Concurrencia (TTR-002) y cancelación (TTR-006) | implementadas; cobertura parcial del worker pool | 🟡 ver §7 |

## 7. Pendientes derivados / decisiones
- **Test de guerra `kill -9` real (subproceso + SIGKILL):** diferido a STORY-009 (necesita el binario raíz `app`, que aún no existe). El test de recuperación sobre DB en archivo con cierre/reapertura del pool queda como evidencia del gate en EPIC-0 (un `kill -9` no puede deshacer un commit ya en disco con WAL; lo que demuestra el gate es que el estado en disco basta para recuperar).
- **Cobertura del worker pool concurrente (TTR-002) y cancelación en RUNNING (TTR-006):** `spawn_workers`/`run_job`/`cancel` quedan con cobertura parcial (job_executor.rs 66.71%). No es el gate de EPIC-0; su comportamiento concurrente se ejerce de verdad con cargas reales al integrar TTR-007. Acción: completar estas pruebas al implementar TTR-007 (o antes si se prioriza robustez de concurrencia).
- **TTR-007 (integración con módulos costosos):** secuenciado a EPIC-2+ cuando existan `generate`/`validate`/`manage`/`incubate`/`feedback`.
