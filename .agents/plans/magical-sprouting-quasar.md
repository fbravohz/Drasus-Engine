# Plan de Implementación — Auditoría Retroactiva de la Fundación (EPIC-0)

> Estado: **DIAGNÓSTICO COMPLETO (6/6 lotes).** Fase de plan de corrección. Nada del repo modificado aún (modo plan).

## Contexto — por qué esta auditoría

El substrato de monetización de Drasus Engine está **14/14 cimientos completos y commiteados** (`crates/shared/`), sobre la plomería de EPIC-0. Cada cimiento se cerró con QA por mutación, pero **en momentos distintos y bajo reglas que fueron endureciéndose sobre la marcha** (atomicidad de ledgers, gate de mutación 0-survivors, dos ejes de cuentas verificadas, checks ADR-0141). Riesgo estructural: decisiones de arquitectura que se tomaron pero pudieron no aplicarse uniformemente al código ya escrito, y features cuyo código pudo desviarse de su spec.

La auditoría recorrió el código y lo contrastó **pieza por pieza** contra (a) las 149 decisiones de arquitectura y (b) la spec funcional de cada feature, con **contraste bidireccional** (el código, el ADR o la spec pueden estar mal u obsoletos). Se hizo antes de que GREENFIELD se congele a BROWNFIELD (hoy el baseline SQL se edita in-situ barato; tras el primer release las migraciones son forward-only).

## Veredicto global del diagnóstico

**El substrato es arquitectónicamente sano — sin deriva sistémica.** FCIS (núcleo puro sin I/O), hexagonalidad, idioma, Inundación de Fundaciones, secretos-nunca-salen (ADR-0093), criptografía real (#11), dos ejes ortogonales (#10), doble atestación (#12), gate compuesto y "último admin en pie" (#14): todo **Cumplido con evidencia de código**. `cargo test -p shared`: **630 verdes**. CLI de verificación (Canal #2): **15/15 cimientos**.

**Los hallazgos se concentran en el esquema base de EPIC-0** (migraciones `0001–0006` + `pool.rs`), construido antes de ADR-0141, más deuda de cobertura de mutación ya anticipada (DEBT-018) y desincronizaciones de documentación. Las migraciones `0007–0020` (los 14 cimientos) pasan los checks M1–M12 en verde.

## Hallazgos consolidados (priorizados por severidad)

### 🔴 Críticos — corrompen datos / violan invariante (baratos en greenfield)

| # | Hallazgo | Evidencia | Acción | Lote |
|---|---|---|---|---|
| C1 | **`PRAGMA foreign_keys=ON` nunca se activa.** La única FK real (`job_results.job_uuid→jobs.id`) es inerte: SQLite acepta inserts huérfanos. ADR-0141 R1 ya lo marca "urgente". | `persistence/pool.rs:29-42` | Añadir `.foreign_keys(true)` a `SqliteConnectOptions` + test de insert huérfano que debe fallar. | 1, 3 |
| C2 | **`STRICT` ausente en las 6 tablas del baseline** (`0001–0006`). Las `0007–0020` sí lo tienen. | `migrations/0001..0006*.sql` | Recrear las 6 tablas con `STRICT` (edición in-situ greenfield). | 1 |

### 🟠 Medios — fallo seguro con pérdida de función / cobertura crítica

| # | Hallazgo | Evidencia | Acción | Lote |
|---|---|---|---|---|
| M1 | **Carrera lectura-luego-escritura sin `BEGIN IMMEDIATE`** en 3 ledgers append-only de EPIC-0. La solución correcta ya existe en `audit_log.rs::try_append_once` pero no se replicó. **NO cubierto por DEBT-018** (que empieza en #5). | `persistence/job.rs:439-487` (`record_result`); `features/data/sovereign-data-fetcher/src/persistence.rs:72-129` (`record`); `orchestrator/mcp_server.rs:94-141` (`check_and_record`) | Replicar patrón `BEGIN IMMEDIATE`+reintento+`WriteContention`. Ampliar DEBT-018 o abrir deuda propia. | 1 |
| M2 | **DEBT-018 (cobertura de mutación del patrón append-only) confirmada empíricamente** en #4, #5, #6, **#7 (nuevo, DEBT.md lo omite)**, #9, #10, #11, #12. Faltan los 3 tests companion validados en #13. | ausencia verificada en `persistence/{usage_metering,consent_registry,enriched_domain_events,institutional_report_engine,data_aggregation,verified_account_registry,instance_continuity,master_account_hierarchy}.rs` | Aplicar patrón de 3 tests de `data_portability.rs` (#13) a cada uno → `cargo mutants` a 0 survivors. Ampliar alcance de DEBT-018 con **#4 y #7**. | 2, 3, 4 |
| M3 | **UUID v4 en vez de v7** en toda la plomería EPIC-0 (7 sitios). Los cimientos #1–#14 sí usan v7. La feature `v7` ya está en `shared/Cargo.toml` pero falta en `sovereign-data-fetcher/Cargo.toml`. ADR-0141 M3. | `persistence/{audit_log,job}.rs`, `domain/mcp_gateway.rs`, `orchestrator/{telemetry,worker_runner,mcp_server}.rs`, `features/data/.../persistence.rs` | Pase `Uuid::new_v4()`→`Uuid::now_v7()` en los 7 sitios + habilitar `v7` en el Cargo del fetcher. | 1 |
| M4 | **`permission_decisions` sin triggers append-only a nivel BD** (a diferencia de `audit_events`/`job_results`). Solo la disciplina del repo lo protege. | `migrations/0005_mcp_gateway.sql` | Añadir triggers `BEFORE UPDATE/DELETE` con `RAISE(ABORT)`. | 1 |
| M5 | **Sentinel `"genesis"` en `permission_decisions.audit_chain_hash`** (anomalía A4). ADR-0141 M10 exige NULL en génesis. | `migrations/0005:25`, `domain/mcp_gateway.rs:100,212` | Columna nullable; `audit_chain_hash: Option<String>`; `None` en génesis. | 1 |
| M6 | **`sovereign_download_records.event_sequence_id` sin `UNIQUE`** pese a ser append-only. ADR-0141 M9. | `migrations/0006:28` | Añadir `UNIQUE`. | 1 |
| M7 | **`proptest` (ADR-0133 Capa 3) nunca activado** pese a funciones cuantitativas financieras. **Contraste bidireccional no resuelto:** el Lote 4 lo lee como decisión documentada y consistente (enumeración exhaustiva manual); el Lote 6 lo lee como violación del texto del ADR. → **decisión del Architect** (ADR-0133 es suyo). | `usage_metering.rs:113,145`, `data_aggregation.rs:200`; 0 deps `proptest` en el workspace | Escalar: añadir `proptest` a las funciones cuantitativas **o** enmendar ADR-0133 para aceptar enumeración exhaustiva. | 6 |
| M8 | **Duplicación de tokens en UI** (`settings_drawer.dart` repite 2 colores que ya son token). base.md §8.1 (bypass del provider). | `settings_drawer.dart:20,22,43,44` vs `gx_tokens.dart:40,43` | Migrar a `Gx.optimaCyan`/`Gx.transitionIndigo`. | 5 |

### 🟡 Bajos — pulido de esquema greenfield / cosmético / DRY

- **A3:** `jobs.event_sequence_id` → renombrar a `row_version` (es contador de versión de tabla mutable, nombre prohibido). `migrations/0003:58`, `persistence/job.rs:322,392`. [1]
- **PRAGMAs faltantes** en `pool.rs`: `synchronous=NORMAL`, `journal_size_limit`, `wal_autocheckpoint`. [1,3]
- **M4 CHECK enums** ausente (`jobs.state` y otros TEXT discretos del baseline). [1]
- **M5 `json_valid` CHECK** ausente en columnas JSON del baseline (`jobs.parameters/result_data`, `audit.details_json`, `telemetry.details_json`). [1]
- **M6 FK `ON DELETE RESTRICT` explícito** en `job_results→jobs`. [1]
- **M11 formato Parquet** sin CHECK/comentario en `data_snapshot_id`. [1]
- **`MAX_RECORD_ATTEMPTS=5` duplicado** en 12 archivos de `persistence/` (nombrado, no mágico, pero centralizable). [6]
- **8 `pub fn` sin doc-comment** (constructores/getters triviales). [6]
- **Bridge `jobs.rs` usa SQL directo** bypassando `JobRepository` (documentado en comentario, no registrado como deuda). `bridge/src/api/jobs.rs:85-96`. [5]

### 📄 Desincronización documental → Escalamiento al Architect (no son bugs de código)

- **ADR-0137 nunca enmendado para #11–#14:** los 4 feature docs citan una "enmienda de ADR-0137" que no existe en el documento (la ubicación en `shared` sí es correcta). [4]
- **`CLAUDE.md:15` lista solo 6 features bendecidas;** ADR-0137 ya se enmendó a 14 cimientos (mismo tema que el anterior, dirección inversa — reconciliar juntos). [6]
- **Comentarios de migraciones `0007/0008/0009`** prometen auditoría vía `audit_events` pero el código usa (bien) hash-chain propio por fila. [2]
- **`licensing-system.md` TTR-001** describe HMAC-SHA256 (es la huella de `node_id`, no la firma de licencia — no contradice ADR-0093, pero confunde). [2]
- **`institutional-report-engine.md` cita ADR-0101** (que es transpilación AST→MQL4/5, no reportes) para su render Tera. [3]
- **`verified-account-registry.md` se contradice:** el banner dice "pagado (STORY-041)", el cuerpo (línea 95) dice "retrabajo pendiente". [3]
- **ADR-0136 dice "Forge/Reactor — TBD"** pero el código ya fijó "Forge". [5]
- **DEBT-005** escrita "#1–#9"; el substrato cerró 14/14 → ampliar a "#1–#14". [5]
- **DEBT-004** causa raíz obsoleta: la **infra genérica de Canvas YA existe** (`canvas_tab.dart` con drag-drop/nodos); solo faltan los nodos por-feature. Esto **reduce el frente D** (el escalamiento de Canvas ya no es "construir la infra", sino "reconciliar ADR-0117/0136 + acotar el remanente"). [5]
- **`owner_id` no es FK física a `accounts`** en `usage_records`/`consent_records` (y sistémico en el substrato); la dependencia vive solo en prosa. **Decisión canónica única del Architect** (relacionada con activar C1). [2]
- **DEBT-005(c):** el "bug" de la SVF del fetcher (no muestra respuesta del servidor) parece ya resuelto en el código actual — confirmar/reformular. [5]

## Estructura de corrección propuesta (Stories)

> **Decisiones confirmadas por el usuario (esta sesión):** alcance = **backlog completo** (STORY-045→048 + paquete de escalamiento); Modo de Acompañamiento = **Autónomo** (el Tech-Lead despacha subagentes Sonnet, reproduce evidencia y cierra con QA por mutación).
> Cada Story que toque `domain/`/`persistence/` cierra con `cargo test` verde + `cargo mutants` acotado a 0 survivors + `clippy` limpio, reproducido por el Tech-Lead.

- **STORY-045 — Endurecimiento del esquema base de la Fundación (greenfield, in-situ).** C1 (foreign_keys=ON + test huérfano), C2 (STRICT en `0001–0006`), M3 (UUIDv7 en 7 sitios + Cargo del fetcher), M4 (triggers `permission_decisions`), M5 (audit_chain_hash NULL / A4), M6 (UNIQUE en `sovereign_download_records`), + los 🟡 de esquema (A3 rename, PRAGMAs, CHECK enums/json_valid/FK explícita/Parquet). Toca migraciones selladas → QA + mutación.
- **STORY-046 — Atomicidad append-only en la plomería EPIC-0.** M1: `BEGIN IMMEDIATE`+reintento+`WriteContention` en `record_result`, `DownloadRepository::record`, `McpGatewayRepository::append`. Incluye los 3 tests companion. Amplía DEBT-018 a la plomería.
- **STORY-047 — Retrofit de cobertura de mutación DEBT-018.** M2: los 3 tests companion (contención sostenida `busy_timeout=0`, clasificador directo con UNIQUE de PK, fidelidad de fila devuelta) aplicados a #4, #5, #6, #7, #9, #10, #11, #12 → 0 survivors cada uno. Mecánico, patrón dorado en `data_portability.rs`. Despachable en paralelo por cimiento. Actualizar DEBT-018 (añadir #4/#7) y saldarla.
- **STORY-048 — Pulido de UI + deuda de puente.** M8 (tokens `settings_drawer`), registrar la deuda del SQL directo en `bridge/jobs.rs`.
- **Paquete de escalamiento al Architect (TASK, no código).** Todos los ítems "Desincronización documental" + M7 (proptest vs enumeración) + `owner_id` FK sistémica + reconciliación de Canvas (ADR-0117/0136, frente D — reducido porque la infra ya existe). El Tech-Lead lo redacta con evidencia; el Architect edita `docs/`. Se entrega al usuario para que invoque al Architect.

## Secuencia recomendada

1. **STORY-045** (esquema base — incluye los dos 🔴) primero: es la corrección más barata, más atrasada y de mayor riesgo si se congela a BROWNFIELD.
2. **STORY-046** (atomicidad EPIC-0) — cierra el hueco de correctitud nuevo antes de que aparezca carga concurrente.
3. **STORY-047** (retrofit DEBT-018) — mecánico, paralelizable por cimiento; alto rendimiento de tokens en esta ventana.
4. **STORY-048** (UI/puente) — rápido, en paralelo.
5. **Paquete de escalamiento** — se entrega en paralelo; el Architect decide fuera de esta sesión.

## Verificación

- Por corrección de código: `cargo test -p shared` verde + `cargo mutants -p shared --file <archivos>` a **0 survivors** + `cargo clippy` limpio. El TL reproduce (no confía en el reporte del subagente).
- Por corrección de esquema (greenfield): re-aplicar migración desde cero y confirmar M1–M12 sobre la tabla resultante; test específico por hallazgo (ej. insert huérfano que debe fallar tras C1).
- CLI Canal #2: `cargo run -p app -- verify <feature> --input '<json>' | jq .` para features con puerto tocado.
- Barrido de cierre documental: sellos de feature, DEBT.md (saldar/ampliar DEBT-018, corregir DEBT-004/005), TEST.md, PROGRESS.md (corregir la nota rezagada de #14), memoria.

## Registro del diagnóstico

Los 6 lotes escribieron su tabla íntegra en `.agents/plans/magical-sprouting-quasar-agent-<id>.md` (el modo plan bloqueó el scratchpad). Al ejecutar se consolidan en la Orden de Trabajo formal `docs/execution/` de la auditoría y esos archivos temporales se limpian.
