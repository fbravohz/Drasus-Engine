# STORY-008 · Orquestador de Aislamiento de Workers

| Campo | Valor |
|---|---|
| **ID** | STORY-008 |
| **Título** | Orquestador de Aislamiento de Workers (procesos OS + memoria compartida + watchdog) |
| **Tipo** | Story |
| **Épica (Fase)** | EPIC-0 — Fundación |
| **Sprint** | 1 |
| **Estado** | Completada |
| **Responsable** | Rust-Engineer (Sonnet) — Modo Docente · auditó Tech-Lead (pendiente) |
| **Creada** | 2026-06-19 |
| **Completada** | 2026-06-20 |

## 1. Especificación de origen

- **Feature:** [`worker-isolation-orchestrator`](../features/worker-isolation-orchestrator.md)
- **TTR(s):** TTR-001 (Bridge de Memoria Compartida), TTR-002 (Watchdog de Procesos y Graceful Shutdown)
- **Módulo:** [`shared`](../modules/) — plomería transversal (igual que `clock`, `audit-log`, `telemetry`, `async-job-executor`)
- **ADR(s):** ADR-0013 (stack Rust puro — Python rechazado permanentemente), ADR-0020 (Perfil D + linaje híbrido padre→worker), ADR-0016 (Local-First)


## 2. Objetivo

Construir el componente que lanza trabajos pesados (backtests, optimizaciones) como **procesos OS independientes**, compartiendo el buffer de datos de mercado vía memoria compartida sin copias, y supervisando que ningún proceso quede huérfano tras una cancelación o crash del orquestador.

## 3. Agentes y Modo de Acompañamiento (ADR-0120)

| Agente | Etapa del pipeline | Depende de | Modo |
|---|---|---|---|
| Rust-Engineer | Etapa 2 — Implementación Core | ninguno (STORY-005 ya en disco) | Docente |

**Modo Docente (ADR-0122):** el Rust-Engineer implementa un bloque lógico completo por su cuenta (con `Edit`/`Write`), luego se detiene, enseña cada decisión de diseño con profundidad cero-conocimiento (qué es, por qué existe, qué problema resuelve), invita preguntas del usuario sobre el código ya escrito y las responde al mismo nivel. No avanza al siguiente bloque sin agotar las preguntas del actual.

## 4. Instrucciones de despacho por agente

### 4.1 Rust-Engineer

```
Lee primero base/SKILL.md (ley suprema) y luego rust-engineer/SKILL.md (tu SKILL de rol).

Eres el Rust-Engineer de Drasus Engine en STORY-008. Tu Modo es DOCENTE (ADR-0122): implementas
un bloque lógico completo por tu cuenta (Edit/Write sin esperar al usuario), luego te detienes,
enseñas cada decisión con profundidad cero-conocimiento (qué es la herramienta/patrón, por qué
existe en Rust, qué problema resuelve), invitas preguntas del usuario sobre el código ya escrito
y las respondes al mismo nivel. No avanzas al siguiente bloque sin agotar las preguntas del actual.
Granularidad: un módulo Rust (un archivo .rs con su función o struct) por bloque.

─── CONTEXTO ───

Workspace: crates/shared (lib). Allí ya existen:
- domain/clock.rs         (trait Clock + DeterministicClock)
- domain/audit_log.rs     (registro inmutable con hash-chain)
- domain/telemetry.rs     (buffer de métricas no-bloqueante)
- domain/job.rs           (modelo de Job: estado FSM QUEUED→RUNNING→DONE/FAILED)
- persistence/audit_log.rs
- persistence/telemetry.rs
- persistence/job.rs      (JobRepository: append + estado + RECOVERY de RUNNING→QUEUED al reiniciar)
- orchestrator/job_executor.rs (cola async que despacha Jobs y los ejecuta inline vía tokio::spawn)
- migrations/0001–0004    (tabla foundation_master_fields, audit_events, jobs/job_results, telemetry)
- public_interface.rs     (re-exporta todo lo público)

La feature que implementas AHORA es `worker-isolation-orchestrator`. Va en crates/shared igual que
el resto de la plomería transversal de EPIC-0.

─── SPEC ───

Feature: docs/execution/STORY-008-worker-isolation-orchestrator.md (esta Orden)
Feature original: docs/features/worker-isolation-orchestrator.md

─── ALCANCE ───

TTR-001: Bridge de Memoria Compartida
- El orquestador mapea un buffer binario (simulado como Vec<u8> de Arrow en los tests) en un
  segmento de memoria compartida del OS (mmap anónimo o POSIX shm).
- Los procesos worker acceden al buffer sin copias; el acceso tras el montaje inicial < 1ms.
- Los workers abren el segmento como Read-Only (PROT_READ); cualquier intento de escritura
  desde el worker debe resultar en error del OS (el mapping se abre sin PROT_WRITE).
- El consumo de RAM no crece con el número de workers (un solo segmento compartido).

TTR-002: Watchdog de Procesos y Graceful Shutdown
- El orquestador mantiene un registro de los Child handles de los procesos worker lanzados.
- Al cancelar un job (señal Cancelado), envía SIGTERM a todos los hijos y espera < 2s;
  si siguen vivos tras 2s, envía SIGKILL.
- Si el proceso padre desaparece (simulado en test con drop del handle o proceso auxiliar),
  los workers deben detectarlo y terminar solos. Mecanismo sugerido: el worker abre el mismo
  segmento de memoria compartida como "latido"; cuando el padre cierra el segmento (drop), el
  mmap del worker falla o recibe SIGHUP — elige el mecanismo que más claramente enseñe Rust.
- Integración con JobRepository: los jobs de tipo WorkerProcess que queden RUNNING en SQLite
  al reiniciar el orquestador se reencolan a QUEUED (igual que STORY-005, ya implementado).

─── REGLAS DE IMPLEMENTACIÓN ───

FCIS estricto:
- domain/worker_orchestrator.rs: lógica pura — decide cuántos workers lanzar, qué job
  les asigna, cuándo matar. CERO imports de std::process, std::fs, tokio, nix, memmap2.
  Solo tipos propios + trait abstractions.
- orchestrator/worker_runner.rs: la cáscara — implementa el spawn real (Command::new),
  la memoria compartida (memmap2), la supervisión (tokio::time + señales OS).
  Aquí viven TODOS los efectos de sistema (fork, mmap, señales).
- persistence/ y migrations/ existentes: reutilizar JobRepository sin modificarla.

Migración: NO es necesaria — el esquema de jobs ya existe (0003_jobs.sql de STORY-005).
Solo añade un campo `worker_pid` nullable (INTEGER) a la tabla jobs si necesitas persistir el PID,
como migración 0005_worker_pid.sql (idempotente: IF NOT EXISTS). Si decides no persistir el PID
en DB y solo tenerlo en memoria, no hay migración — documenta la decisión.

Perfil de persistencia (ADR-0020): el job de worker hereda el perfil D ya existente en
jobs/job_results (STORY-005). Los campos `parent_id` (linaje orquestador→worker) y `process_id`
(OS PID del worker) de la spec salen de la persistencia cuando el job se cierra.

Dependencias (Cargo.toml de shared, añadir las que falten):
- memmap2 = "0.9" (o la versión más reciente estable)
- nix = { version = "0.29", features = ["process", "signal"] }  (para SIGTERM/SIGKILL en Unix)
Solo añade lo necesario.

─── CRITERIO DE ACEPTACIÓN ───

Ver §5 de esta Orden. Cada criterio tiene su prueba nombrada. Entregar verde + mapeo 1-a-1.

─── PROTOCOLO DE LECCIONES ───

Al cerrar esta Story, consolida TODO lo enseñado en UN SOLO archivo:
docs/lessons/rust/STORY-008-worker-isolation-orchestrator.md
(ADR-0124: un archivo por Story, no por tema. Enlaza a esta Orden al inicio.)

─── MODO DOCENTE — SECUENCIA SUGERIDA DE BLOQUES ───

Implementa en este orden (ajusta si el diseño lo pide):
1. domain/worker_orchestrator.rs — tipos puros + trait WorkerBackend
2. orchestrator/worker_runner.rs — spawn de proceso + setup de memoria compartida (TTR-001)
3. orchestrator/worker_runner.rs — watchdog loop (TTR-002: SIGTERM → espera → SIGKILL)
4. (opcional) migrations/0005_worker_pid.sql — si decides persistir el PID
5. tests en cada archivo — criterios del §5
6. public_interface.rs — exports nuevos

Por cada bloque: implementa → enseña → preguntas → siguiente.
```

**Plan de Implementación — Rust-Engineer (2026-06-20)**

| Bloque | Archivo | Descripción |
|---|---|---|
| 1 | `domain/worker_orchestrator.rs` | Tipos puros: `WorkerConfig`, `WorkerOrchestrator`, trait `WorkerBackend`. Sin I/O. |
| 2 | `orchestrator/worker_runner.rs` | Cáscara: `SharedMemorySegment` (mmap + keepalive), `OsWorkerBackend`, `graceful_shutdown`, `is_process_alive`. |
| 3 | `domain/mod.rs` + `orchestrator/mod.rs` | Registrar módulos nuevos. |
| 4 | `public_interface.rs` | Re-exportar tipos públicos. |

**Decisión sobre migración `worker_pid`:** la columna `process_id` (STRING) ya existe en la tabla `jobs` (migración `0003_jobs.sql`). Se almacena el PID OS como string en ese campo al transicionar a `RUNNING`. No se añade `0005_worker_pid.sql`.

---

## 5. Criterio de aceptación

| # | Criterio verificable | Prueba que lo demuestra |
|---|---|---|
| 1 | El buffer compartido se mapea; un segundo proceso (worker simulado) lo lee en < 1ms tras el montaje | `shared_memory_access_latency_under_1ms` |
| 2 | N workers (N=4 en test) acceden al mismo buffer; la RAM del proceso principal no crece linealmente | `shared_memory_ram_constant_with_n_workers` |
| 3 | Un worker que intenta escribir en el buffer recibe un error del OS (mapping Read-Only) | `shared_memory_worker_write_is_rejected` |
| 4 | Al cancelar un job con N workers activos, todos terminan en < 2s | `worker_graceful_shutdown_under_2s` |
| 5 | Si el orquestador muere (simulado), los workers se auto-terminan | `worker_terminates_when_parent_drops` |
| 6 | Los jobs de tipo WorkerProcess que queden RUNNING en SQLite al reiniciar pasan a QUEUED (igual que STORY-005) | `worker_jobs_recovered_to_queued_on_restart` |
| 7 | El orquestador no lanza más de `MAX_CONCURRENT_WORKERS` procesos simultáneos | `worker_respects_max_concurrent_workers` |
| 8 | `domain/worker_orchestrator.rs` no importa `std::process`, `tokio`, `memmap2` ni `nix` (FCIS limpio) | inspección + `grep` de imports en el dominio |

## 6. Comandos de validación (copy/paste)

```bash
cargo build --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo test -p shared                          # todos los tests del crate shared (incluye los nuevos)
cargo test -p shared worker                   # solo los tests de worker (filtro por nombre)
cargo llvm-cov --workspace --summary-only     # cobertura de líneas
```

## 7. Registro de ejecución

**Fecha:** 2026-06-20 · **Agente:** Rust-Engineer (Sonnet) · **Modo:** Docente

| Criterio | Prueba | Estado |
|---|---|---|
| 1 · acceso < 1ms | `shared_memory_access_latency_under_1ms` | ✅ |
| 2 · RAM constante con N workers | `shared_memory_ram_constant_with_n_workers` | ✅ |
| 3 · escritura del worker rechazada | `shared_memory_worker_write_is_rejected` | ✅ |
| 4 · shutdown < 2s | `worker_graceful_shutdown_under_2s` | ✅ |
| 5 · workers terminan al morir el padre | `worker_terminates_when_parent_drops` | ✅ |
| 6 · RUNNING → QUEUED al reiniciar | `worker_jobs_recovered_to_queued_on_restart` | ✅ |
| 7 · máx. workers concurrentes respetado | `worker_respects_max_concurrent_workers` | ✅ |
| 8 · dominio sin imports de sistema | inspección + grep (cero coincidencias) | ✅ |

**Gate:** `cargo test -p shared` → **91 passed, 0 failed**.
**Lección:** `docs/lessons/rust/STORY-008-worker-isolation-orchestrator.md`

## 8. Pendientes derivados / decisiones

- **Spec corregida:** los residuos Python (Ray, `multiprocessing.shared_memory`, `ProcessPoolExecutor`, ZeroMQ, puerto 8002) fueron purgados de `worker-isolation-orchestrator.md` el 2026-06-19 (Tech-Lead, alineado con ADR-0013).
- **CPU/memoria por proceso (diferido de STORY-007):** los reportes de métricas CPU/RAM por worker hacia `telemetry` se diseñan en esta Story si el Rust-Engineer los introduce naturalmente; si no, quedan como pendiente para EPIC-2+.
- **Extensión remota (HybridComputeCooperative):** la extensión de workers a VPS/bare-metal vía gRPC es EPIC-9+; no se toca aquí.
- **SIGKILL test en CI:** el criterio #5 (padre muere → workers terminan) puede requerir un feature flag para CI (algunos runners no permiten `kill -9` entre procesos). Documentar si es el caso.
