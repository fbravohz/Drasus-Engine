# STORY-007 · Telemetría técnica (buffer de alta velocidad + señal de vida)

> **Plantilla de Orden de Trabajo (Spec-Driven).** Ver `docs/execution/_TEMPLATE.md`.
> La Orden de Trabajo es la **especificación ejecutable**: contiene la instrucción EXACTA que recibe el agente,
> los comandos para que el usuario valide por su cuenta, y el registro de lo que pasó. Vive en git, NO en el chat.

| Campo | Valor |
|---|---|
| **ID** | STORY-007 |
| **Título** | Telemetría técnica (buffer de alta velocidad + señal de vida) |
| **Tipo** | Story |
| **Épica (Fase)** | EPIC-0 — Fundación |
| **Sprint** | 1 |
| **Estado** | 🟡 Parcial (TTR-001 ✅ implementado y auditado; TTR-002 → EPIC-7) |
| **Responsable** | Rust-Engineer (Sonnet, Modo Mentor→Docente) · auditó Tech-Lead |
| **Creada** | 2026-06-16 |
| **Completada** | 2026-06-18 |

## 1. Especificación de origen

- **Feature:** [`telemetry`](../features/telemetry.md) — solo **TTR-001** (Buffer de Alta Velocidad). TTR-002 (Diseñador de Vistas de Correlación) queda fuera: necesita el módulo `feedback`, que no existe aún (ver §8).
- **Módulo:** plomería transversal en `crates/shared` (ADR-0003) — mismo patrón que `clock` y `audit-log`, sin módulo de pipeline dueño.
- **ADR(s):** ADR-0015 (Causalidad — la telemetría es evidencia de infraestructura, distinta del audit-log de negocio), ADR-0020 (contrato de persistencia/perfiles), ADR-0003 (crate `shared`).

## 2. Objetivo (una frase llana)

Que el sistema capture su propio pulso de rendimiento — cuánto tardan las rutas críticas y si los procesos de fondo siguen vivos — sin frenar el trabajo real, y lo guarde en disco con poda automática para no acumular basura.

## 3. Agentes y Modo de Acompañamiento (ADR-0120 + ADR-0122)

| Agente | Etapa del pipeline | Depende de | Modo |
|---|---|---|---|
| Rust-Engineer | Etapa 2 (Implementación Core) | Ninguno | ~~Mentor~~ → **Docente** (cambio del usuario, 2026-06-17, tras Bloque 1) |

**Cambio de Modo registrado (2026-06-17):** el Bloque 1 (`TelemetrySampleContent`) se ejecutó en Modo Mentor (el usuario tecleó, con un defecto — `process_id` duplicado — detectado en la relectura). A partir de aquí el usuario pasó la Story a **Modo Docente** (ADR-0122): el Rust-Engineer implementa directamente con `Edit`/`Write` sobre `domain/`, `orchestrator/`, `persistence/`, `schemas.rs`, y se detiene a explicar cada bloque con profundidad cero-conocimiento, invitando preguntas, antes de avanzar al siguiente (contrato completo en `rust-engineer/SKILL.md` §"Modos de Acompañamiento" y en `base/SKILL.md`). Las lecciones de esta sesión se registran formalmente en `docs/lessons/rust/`.

## 4. Instrucciones de despacho

### 4.1 Rust-Engineer

```
Eres el Rust-Engineer de Drasus Engine. Antes de actuar:
1. Lee completo `.claude/skills/base/SKILL.md`.
2. Lee completo `.claude/skills/rust-engineer/SKILL.md`, en particular la sección
   "MODOS DE ACOMPAÑAMIENTO DE IMPLEMENTACIÓN (ADR-0120)" — esta Orden te declara
   en Modo MENTOR (tabla §3 de esta misma Orden).
3. Lee esta Orden completa (STORY-007).
4. Lee `docs/features/telemetry.md` completo — en particular "Comportamientos
   Observables" (solo los 3 primeros puntos aplican, ver alcance abajo),
   "Restricciones", "Parámetros Configurables", TTR-001 y la tabla "Persistencia".
5. Si tienes duda sobre el contrato de 25 campos, la tabla "Persistencia" de la
   Feature ya es el resumen aplicado — no hace falta releer `docs/adr/ADR-0020.md`
   completo salvo que algo no cuadre.

ALCANCE de esta Story (NO construyas más que esto):
- Capturar latencia de una operación nombrada (hot-path: "tiempo de ejecución
  desde señal hasta orden en el puerto" es el caso de uso citado por la Feature,
  pero la API debe ser genérica: cualquier módulo puede medir cualquier tramo).
- Capturar una "señal de vida" (heartbeat) nombrada, sin valor de latencia.
- Persistir ambas en SQLite a través de un buffer NO bloqueante (cola en memoria
  + escritura por lotes en un hilo/tarea de fondo) — el llamador nunca espera al
  disco.
- Poda automática: borrar muestras más viejas que `RETENTION_DAYS` (parámetro
  configurable, default 7, rango 1-30 según la tabla de la Feature).

FUERA de alcance (no lo construyas, ya está decidido y registrado en §8 de esta
Orden): TTR-002 (vistas de correlación para `feedback`), "Builder Telemetry & ETA
Prediction" (gRPC/WebSocket, necesita el proceso de generación de `generate`),
"Heap Memory Monitor" con endpoint `/api/system/gc` (necesita la capa headless
gRPC, EPIC-8), "Best Strategy Tracker" (necesita ranking de estrategias de
`generate`), monitoreo de CPU/memoria por proceso (se construye junto a
`worker-isolation-orchestrator`, STORY-008, mismo dominio de aislamiento de
procesos).

DISEÑO DE PERSISTENCIA (decisión del Tech-Lead, por precedente directo de
`migrations/0002_audit_log.sql` — revísalo como referencia de formato):
- Tabla `telemetry_samples`, migración nueva `migrations/0004_telemetry.sql`,
  idempotente (`CREATE TABLE IF NOT EXISTS` / `CREATE INDEX IF NOT EXISTS`).
- Columnas del contrato canónico (ADR-0020), exactamente las que la tabla
  "Persistencia" de `telemetry.md` ya declara — NO copies grupos completos, solo
  estas:
  - Grupo I (universal): `id`, `created_at`, `updated_at`, `audit_hash`,
    `audit_chain_hash`, `event_sequence_id`.
  - Grupo II: `institutional_tag`.
  - Grupo III: `logic_hash`, `session_id`.
  - Grupo IV: `node_id`, `process_id`, `execution_latency_ms` (nullable: solo
    se llena en muestras de latencia, NULL en heartbeats).
- Columnas propias de la Feature (fuera del contrato de 25 campos, mismo patrón
  que `action_type`/`entity_type`/`entity_id`/`details_json` en `audit_events`):
  `metric_name` (TEXT NOT NULL — qué se midió, ej. "ingest.hot_path_latency",
  "job_executor.heartbeat") y `details_json` (TEXT, nullable — contexto extra
  opcional).
- Índice por `metric_name` + `created_at` (acceso por serie temporal, citado en
  "Comportamientos Observables": "permite consultar series temporales").
- La poda (`DELETE FROM telemetry_samples WHERE created_at < ?`) NO es un
  trigger: es una función de `persistence/telemetry.rs` que el caller invoca
  (no inventes un scheduler aquí — la periodicidad real se conecta en
  STORY-008/async-job-executor más adelante; este TTR solo entrega la función
  y su prueba).

ESTRUCTURA DE CÓDIGO (sigue el patrón ya usado por `clock`/`audit-log` en
`crates/shared/src/`):
- `domain/telemetry.rs`: lógica pura — construir una muestra de latencia o de
  heartbeat ya validada (sin tocar el reloj real ni el disco), y la función
  pura que decide qué filas quedan fuera de la ventana de retención dado un
  corte de tiempo. Reusa el puerto `Clock` existente (`domain/clock.rs`) para
  el timestamp, igual que hace `audit_log.rs` — NO uses `SystemTime::now()`
  aquí.
- `orchestrator/telemetry.rs` (o el archivo equivalente que ya exista en
  `orchestrator/`, revisa la carpeta antes de crear uno nuevo): la cáscara con
  el buffer no bloqueante (cola en memoria) y el hilo/tarea que vacía la cola a
  SQLite por lotes. Revisa qué dependencia de concurrencia ya usa
  `async-job-executor` antes de añadir una nueva — reusa si aplica, y si hace
  falta una nueva dependencia, dila explícitamente antes de tocar `Cargo.toml`.
- `persistence/telemetry.rs`: repositorio (insertar lote, purgar por corte,
  consultar por `metric_name` — lo mínimo para probar lo anterior).
- Exporta lo necesario en `public_interface.rs`.

RESTRICCIONES NO NEGOCIABLES (de la Feature):
- "ALTA EFICIENCIA": la llamada que un módulo hace para registrar una muestra
  (encolar, NO el flush a disco) debe tardar menos de 50 microsegundos. Esto se
  demuestra con una prueba/benchmark real, no se asume.
- "DETERMINISMO NO AFECTADO": el núcleo (`domain/`) no debe leer el reloj real
  ni el disco — mismo patrón FCIS que `clock`/`audit-log`.

Bajo Modo MENTOR: documenta aquí mismo (§4.1, debajo de este bloque, sección
"Plan de Implementación") la secuencia de bloques antes de dictarlos en el chat:
concepto Rust → fragmento exacto a teclear (archivo + ubicación) → punto de
verificación. Bloques pequeños: una función o struct por vez.
```

**Plan de Implementación** (lo llena el Rust-Engineer al ser invocado en Modo Mentor):

Decisión de diseño previa (no estaba en la Orden, la resuelve este plan): `audit_hash`/
`audit_chain_hash` se mantienen como una cadena **en memoria** (estado compartido protegido
por un `std::sync::Mutex`, sin I/O en el camino caliente), sembrada UNA SOLA VEZ al iniciar
el proceso leyendo la última fila persistida (mismo patrón que `recover_at_startup` del
ejecutor de jobs) — evita colisión de `event_sequence_id` entre arranques sin pagar una
lectura a disco por cada muestra registrada. Ningún criterio de aceptación (§5) exige
verificación de cadena tipo `verify_chain`; no se construye.

| # | Bloque | Archivo | Concepto Rust a explicar | Verificación |
|---|---|---|---|---|
| 1 | `TelemetrySampleContent` (struct) | `domain/telemetry.rs` | Campos `Option<T>` para expresar "no aplica" (heartbeat vs. latencia) | lectura conjunta |
| 2 | `TelemetrySample` (struct, envuelve Grupo I + content) | `domain/telemetry.rs` | Composición de structs (mismo patrón que `AuditEvent` envolviendo `AuditEventContent`) | lectura conjunta |
| 3 | `compute_sample_hash` (función pura SHA-256) | `domain/telemetry.rs` | Serialización canónica con separador (`\u{1F}`), igual que `canonical_bytes` en `audit_log.rs` | `cargo build` |
| 4 | `build_sample` (encadena con `previous: Option<&TelemetrySample>`) | `domain/telemetry.rs` | `match` sobre `Option`, mismo patrón que `chain_event` | `cargo build` |
| 5 | `expired_sample_ids` (filtro puro de poda) | `domain/telemetry.rs` | Iteradores `.filter().map().collect()` | `cargo build` |
| 6 | Tests del núcleo (criterios #1, #2, #8) | `domain/telemetry.rs` | `#[cfg(test)]`, `assert_eq!`, por qué NO hay `SystemTime` ni `sqlx` en este archivo | `cargo test -p shared` |
| 7 | Migración `0004_telemetry.sql` (tabla + índice) | `migrations/0004_telemetry.sql` | `CREATE TABLE IF NOT EXISTS` idempotente, índice compuesto `(metric_name, created_at)` | `cargo test -p shared` (test de `pool.rs` aplica todas las migraciones) |
| 8 | `TelemetryError` (enum + `Display`/`Error`/`From<sqlx::Error>`) | `persistence/telemetry.rs` | Mismo patrón que `AuditLogError` | lectura conjunta |
| 9 | `TelemetryRepository` + `new` + `load_tail` | `persistence/telemetry.rs` | Por qué `load_tail` existe (sembrar la cadena en memoria al iniciar, una sola vez) | `cargo build` |
| 10 | `insert_batch` (transacción SQLx, varias filas) | `persistence/telemetry.rs` | `pool.begin()` / `tx.commit()`, por qué un lote = una transacción (menos fsync) | `cargo build` |
| 11 | `purge_older_than(cutoff_ns)` | `persistence/telemetry.rs` | `DELETE ... WHERE created_at < ?`, `rows_affected()` | `cargo build` |
| 12 | `query_by_metric(metric_name, from_ns, to_ns)` | `persistence/telemetry.rs` | Mapeo fila->struct (`row_to_sample`, igual que `row_to_event`) | `cargo build` |
| 13 | Tests de persistencia (criterios #5, #6, #7) | `persistence/telemetry.rs` | DB en archivo temporal (no `:memory:`) para probar durabilidad tras reabrir | `cargo test -p shared` |
| 14 | `TelemetryBufferConfig` | `orchestrator/telemetry.rs` | Mismo patrón que `JobExecutorConfig` (+ `Default`) | lectura conjunta |
| 15 | `ChainState` + `Shared` (estado compartido) | `orchestrator/telemetry.rs` | `std::sync::Mutex` (sección crítica síncrona, sin `.await` dentro) vs. `tokio::Mutex` — por qué aquí toca el primero | lectura conjunta |
| 16 | `TelemetryBuffer::new` + `bootstrap` (siembra desde `load_tail`) | `orchestrator/telemetry.rs` | `Arc<Shared>`, handle clonable barato (igual que `JobExecutor`); `bootstrap` es async y se llama UNA VEZ, antes de registrar nada | `cargo build` |
| 17 | `record_latency` / `record_heartbeat` (síncronas, no `async`) | `orchestrator/telemetry.rs` | Por qué estas dos funciones NO son `async fn`: encolar en `mpsc::UnboundedSender` y tomar el `Mutex` síncrono no esperan a nada — esto es lo que hace medible el límite de 50µs | `cargo build` |
| 18 | `spawn_flush_task` (tarea de fondo) | `orchestrator/telemetry.rs` | `tokio::spawn` + `tokio::time::interval` + drenar el canal con `try_recv` en bucle antes de `insert_batch` | `cargo build` |
| 19 | `purge` (wrapper que usa el `Clock` inyectado) | `orchestrator/telemetry.rs` | Cómputo de `cutoff_ns` a partir de `RETENTION_DAYS` + el `Clock` (nunca `SystemTime::now()`) | `cargo build` |
| 20 | Tests de la cáscara (criterios #3, #4) | `orchestrator/telemetry.rs` | Benchmark con `std::time::Instant` para el límite de 50µs; prueba de no-bloqueo durante un flush deliberadamente lento | `cargo test -p shared` |
| 21 | Export en `public_interface.rs` | `public_interface.rs` | — (edición de re-exports, no lógica nueva) | `cargo build` |
| 22 | Cierre: `cargo clippy --workspace --all-targets -- -D warnings`, `cargo llvm-cov`, mapeo criterio→test en §5, llenar §7 | — | — | comandos de §6 |

Bloques pequeños, uno por turno. Empezamos por el Bloque 1.

## 5. Criterio de aceptación

| # | Criterio verificable | Prueba que lo demuestra |
|---|---|---|
| 1 | Una muestra de latencia se construye en el núcleo puro sin tocar reloj real ni disco | `domain::telemetry::tests::build_sample_constructs_a_latency_sample` |
| 2 | Una muestra de heartbeat (sin valor de latencia) se construye y persiste correctamente | `domain::telemetry::tests::build_sample_constructs_a_heartbeat_sample` (construcción) + `persistence::telemetry::tests::heartbeat_sample_persists_with_null_execution_latency` (persistencia, NULL ida y vuelta) |
| 3 | El registro de una muestra (encolar) tarda menos de 50µs | `orchestrator::telemetry::tests::record_heartbeat_enqueues_in_under_50_microseconds` (promedio real sobre 1000 llamadas con `Instant`) |
| 4 | El buffer no bloquea al llamador mientras el flush a disco está en curso | `orchestrator::telemetry::tests::record_does_not_block_while_a_slow_flush_is_in_progress` (sostiene el lock de escritura real de SQLite con `BEGIN IMMEDIATE` 150ms; 100 llamadas mientras tanto tardan una fracción de eso) |
| 5 | Las muestras persisten tras reabrir la base de datos (DB en archivo, no `:memory:`) | `persistence::telemetry::tests::samples_persist_after_reopening_the_database` |
| 6 | La poda elimina solo las muestras más viejas que el corte de retención, conserva el resto | `domain::telemetry::tests::expired_sample_ids_returns_only_samples_older_than_cutoff` (decisión pura) + `persistence::telemetry::tests::purge_older_than_deletes_only_samples_before_the_cutoff` (DELETE real) |
| 7 | Consulta por `metric_name` + rango de tiempo devuelve la serie esperada | `persistence::telemetry::tests::query_by_metric_filters_by_name_and_range_and_orders_by_time` |
| 8 | El núcleo (`domain/telemetry.rs`) no importa `SystemTime` ni el pool de SQLite | inspección (`grep` sin resultados de import) + `cargo clippy --workspace --all-targets -- -D warnings` limpio |

Cobertura de líneas nueva (`cargo llvm-cov --workspace --summary-only`): `domain/telemetry.rs` 100.00%, `persistence/telemetry.rs` 93.55%, `orchestrator/telemetry.rs` 93.19%.

## 6. Comandos de validación (para el usuario)

```bash
cargo test -p shared
cargo clippy --workspace --all-targets -- -D warnings
cargo llvm-cov --workspace --summary-only
```

## 7. Registro de ejecución

- **2026-06-16, Bloque 1 (Modo Mentor):** el usuario tecleó `TelemetrySampleContent` en `domain/telemetry.rs`. Defecto detectado en la relectura: `process_id: String` duplicado (dos campos con el mismo nombre, no compila). Quedó pendiente de corrección.
- **2026-06-17, cambio de Modo:** el usuario pasó la Story de Mentor a **Docente** (ADR-0122) y autorizó al Rust-Engineer a corregir el defecto del Bloque 1 directamente y terminar la Story completa.
- **2026-06-17, implementación (Modo Docente):**
  - `domain/telemetry.rs`: corregido el `process_id` duplicado; agregado `TelemetrySample`, `compute_sample_hash` (SHA-256 canónico, reusa `GENESIS_PREVIOUS_HASH` de `audit_log.rs`), `build_sample` (encadena con `previous: Option<&TelemetrySample>`) y `expired_sample_ids` (filtro puro de poda). 6 tests, 100% de líneas cubiertas.
  - `migrations/0004_telemetry.sql`: tabla `telemetry_samples` (Grupo I + II + III + IV aplicables + `metric_name`/`details_json`) con dos índices — `(metric_name, created_at)` para series temporales y `(created_at)` para la poda. Sin triggers de append-only (a diferencia de `audit_events`/`job_results`): la poda borra a propósito.
  - `persistence/telemetry.rs`: `TelemetryError`, `TelemetryRepository` (`new`, `load_tail`, `insert_batch` en una sola transacción SQLx, `purge_older_than`, `query_by_metric`). 6 tests con DB en archivo temporal donde el criterio lo exige.
  - `orchestrator/telemetry.rs`: `TelemetryBufferConfig`, `TelemetryBuffer` (reusa `ExecutorIdentity` del Async Job Executor en vez de duplicar un struct de identidad idéntico). `record_latency`/`record_heartbeat` son funciones **síncronas** (no `async fn`) — encolan en un `mpsc::UnboundedSender` y solo tocan un `std::sync::Mutex` breve para la cadena en memoria; nunca esperan al disco. `bootstrap` siembra esa cadena leyendo `load_tail` una sola vez al iniciar. `spawn_flush_task` vacía el canal por lotes cada `flush_interval_ms`. `purge` calcula el corte desde el `Clock` inyectado. 3 tests, incluido uno que sostiene el lock real de escritura de SQLite para demostrar que el buffer no espera al disco.
  - **Desviación menor del Plan de Implementación (§4.1):** el Bloque 15 planeaba un struct `ChainState` separado; se simplificó a un campo `chain_state: std::sync::Mutex<Option<TelemetrySample>>` directamente en `Shared` — un struct envoltorio de un solo campo no agregaba nada.
  - **Dependencia tocada:** se agregó el feature `time` a `tokio` en `[dependencies]` de `crates/shared/Cargo.toml` (ya estaba en `[dev-dependencies]`) — lo necesita `tokio::time::interval` en `spawn_flush_task`. No es un crate nuevo.
  - `public_interface.rs`: exportados `TelemetryBuffer`, `TelemetryBufferConfig`, `TelemetryError`, `TelemetryRepository`, `TelemetrySample`, `TelemetrySampleContent`, `build_sample`, `expired_sample_ids`.
- **Verde final:** `cargo build --workspace`, `cargo clippy --workspace --all-targets -- -D warnings` (0 warnings tras corregir 2 hallazgos `clippy::cloned_ref_to_slice_refs`), `cargo test -p shared` (76 tests, 0 fallos), `cargo llvm-cov --workspace --summary-only` (ver §5 para cobertura de los archivos nuevos).
- **Lecciones formales:** consolidadas en [`docs/lessons/rust/STORY-007-telemetry.md`](../lessons/rust/STORY-007-telemetry.md) — un solo archivo por Story (ADR-0124, corrige la regla "un archivo por tema" de ADR-0122), con los conceptos de Rust de esta sesión anclados al código real de arriba.
- **2026-06-18, auditoría independiente (Tech-Lead):** reproduje los 3 comandos de §6 yo mismo (no me basé en el reporte del ingeniero): `cargo build --workspace` limpio; `cargo clippy --workspace --all-targets -- -D warnings` 0 warnings; `cargo test -p shared` → 76/76 verdes, confirmé por nombre los 8 tests citados en §5 (mapeo 1-a-1 contra cada criterio, incluidos los compuestos de los criterios #2 y #6); `cargo llvm-cov --workspace --summary-only` → `domain/telemetry.rs` 100.00% líneas (coincide con lo reportado), `orchestrator/telemetry.rs` 93.19% líneas (coincide exacto), `persistence/telemetry.rs` 94.29% líneas (reportado 93.55%, variación menor sin impacto). Inspeccioné `domain/telemetry.rs`: sin `SystemTime` ni `sqlx` importados (criterio #8, FCIS limpio). Inspeccioné `migrations/0004_telemetry.sql`: columnas exactas del perfil declarado, idempotente, dos índices justificados (serie temporal + poda), sin triggers append-only (correcto, esta tabla sí borra). Confirmé los 8 exports en `public_interface.rs`. **Veredicto: APROBADO.** Sellado `docs/features/telemetry.md` (banner 🟡 Parcial) y `docs/ROADMAP.md` (fila STORY-007).
- **Pendiente para el Tech-Lead:** ~~auditar este registro contra §5 (mapeo 1-a-1), reproducir los comandos de §6, y sellar `docs/features/telemetry.md` TTR-001 + el ROADMAP si corresponde.~~ Hecho (ver entrada de arriba).

## 8. Pendientes derivados / decisiones

- **TTR-002 de `telemetry`** (vistas de correlación para `feedback`) → diferido a EPIC-7, cuando exista el módulo `feedback`.
- **"Builder Telemetry & ETA Prediction"** (throughput de generación, gRPC/WebSocket) → diferido a EPIC-3 (`generate`), necesita el proceso de generación de candidatas que aún no existe.
- **"Heap Memory Monitor"** con endpoint `/api/system/gc` → diferido a EPIC-8 (necesita la capa headless gRPC, ADR-0116).
- **"Best Strategy Tracker"** (evento `best_strategy_update` + minigráfico) → diferido a EPIC-3/EPIC-8 (necesita ranking de `generate` + superficie UI).
- **Monitoreo de CPU/memoria por proceso** → se construye en STORY-008 (`worker-isolation-orchestrator`), mismo dominio de aislamiento de procesos; evita duplicar el concern aquí.
- **Decisión de esquema (columnas `metric_name`/`details_json` fuera del contrato de 25 campos):** aplicado por precedente directo de `audit_events` (`action_type`/`entity_type`/`entity_id`/`details_json`, ya aceptado por el Architect en STORY-004). No se trató como ambigüedad nueva que requiera escalar — si el usuario lo objeta, se corrige antes de implementar.
- **Wiring a daemons existentes** (que `clock`/`async-job-executor` emitan heartbeats reales): no incluido en esta Story — la API queda lista pero conectar cada daemon existente es trabajo de integración posterior, evita scope creep. Si conviene hacerlo ya, decisión del usuario para ampliar esta Orden antes de despachar.
