# STORY-033 — Enriched Domain Events (cimiento #6 del substrato — raíz del pilar de Cuentas Verificadas)

| Campo | Valor |
|---|---|
| **ID** | STORY-033 |
| **Tipo** | Story (código — sexto cimiento del substrato de pricing/SaaS) |
| **Épica (Fase)** | Cimientos de Monetización (transversal, greenfield) |
| **Sprint** | substrato-monetizacion |
| **Estado** | ✅ Completada (event-store local append-only atómico; bus fan-out y envío al proveedor diferidos) |
| **Creada** | 2026-07-05 |
| **Feature** | [`enriched-domain-events`](../features/enriched-domain-events.md) |
| **ADRs** | ADR-0144 (cimiento #6) · ADR-0145 (flujo de capital + snapshot de cuenta + refuerzo de orden) · ADR-0143 (supresión por tier) · ADR-0137 (puertos) · ADR-0141 (append-only + montos ×10⁸) · ADR-0020 (Perfil D) · ADR-0093 (secretos) · ADR-0142 (CLI) |

## 1. Objetivo llano

Construir la **raíz del substrato**: el event-store append-only de **eventos de dominio inmutables y ricos** que el motor emitirá por cada acción significativa, con los datos que los productos de monetización (medición, agregación, reportes, Cuentas Verificadas) necesitarán después. Sin estos eventos estructurados desde hoy, cada producto futuro exige reabrir la capa de ejecución (justo lo que ADR-0144 evita). Incluye los **3 enriquecimientos de ADR-0145** (flujo de capital, snapshot de estado de cuenta, orden-con-fricción reforzada) que hacen reconstruible el track record del pilar de Cuentas Verificadas.

**Alcance ahora vs. después (ADR-0144 "puerto + esquema ahora, adaptador después"):**
- **Ahora (esta Story):** catálogo de tipos de evento (Core) + serialización canónica + hash encadenado + **event-store append-only atómico** + decisión de replicación desde el `ExecutionGate` real de #2 + puerto `event_out` + CLI verify.
- **Después (diferidos):** el **fan-out al bus (ADR-0085)** — no existe infraestructura de bus en el código todavía; y el **envío real a la Cabina de Mando** (adaptador de red). Ahora la decisión de replicar se computa y se expone como flag; no hay envío por red.

## 2. Agentes y Modo de Acompañamiento (ADR-0120/0122)

| Agente | Modelo | Modo |
|---|---|---|
| Rust-Engineer | Sonnet | **Docente** |

Docente: es un cimiento nuevo y sustancial (catálogo de eventos heterogéneos en un log append-only, decisión de supresión). Escribe la lección en `docs/lessons/rust/STORY-033-enriched-domain-events.md`.

## 3. Gate de Coherencia — resultado (contraste bidireccional)

- **Stack:** limpio. Crate `crates/shared` (plomería crosscutting, excepción bendecida ADR-0137: `EnrichedDomainEvent` es tipo técnico del catálogo, consumido por ≥2 dominios — `usage-metering`, `data-aggregation`, `institutional-report-engine`, `verified-account-registry`).
- **Event-store APPEND-ONLY ATÓMICO (ADR-0141 + regla DEBT-001, obligatorio #1):** tabla `domain_events` **append-only** — `event_sequence_id INTEGER NOT NULL UNIQUE` + triggers `BEFORE UPDATE`/`BEFORE DELETE` que abortan + `audit_chain_hash` encadenado (NULL solo génesis). **El append DEBE nacer atómico** (regla nueva `rust-engineer` §4): `leer-cola + INSERT` dentro de **una transacción `BEGIN IMMEDIATE`** con reintento acotado + `WriteContention` tipado. **Prueba de 2 escritores obligatoria** (qa §2). Copia el patrón ya construido de `persistence/consent_registry.rs` / `persistence/audit_log.rs`.
- **Log heterogéneo de un solo tipo de tabla (event-sourcing):** una sola tabla con `event_type` (`TEXT` + `CHECK IN (...)`) + `payload` (`TEXT` con `CHECK(json_valid(payload))`). NO una tabla por tipo de evento. El Core tiene un enum `EnrichedDomainEvent` con variantes que serializa a un payload JSON **canónico y determinista** (claves ordenadas, `BTreeMap` — patrón de `consent_registry`). El `CHECK` de `event_type` enumera el catálogo.
- **Catálogo de eventos (Core) — incluye los 3 de ADR-0145 (obligatorio #2):**
  - `OrderExecuted` — instrumento, lado, cantidad, precio, slippage, tiempo de fill, bróker, nocional, **+ refuerzo ADR-0145:** `account_id`, PnL realizado, MAE, MFE, duración del trade.
  - `CapitalFlow` (ADR-0145) — signo (depósito/retiro/transferencia), monto, divisa, `account_id`, timestamp.
  - `AccountSnapshot` (ADR-0145) — `account_id`, equity, balance, margen disponible, margen requerido.
  - `BacktestCompleted` — Sharpe, drawdown, PBO, régimen.
  - `RegimeDetected` / `DrawdownDetected` / `LiquidityStress` / `CorrelationChange` — payload respectivo.
- **Montos monetarios como entero ×10⁸ (obligatorio #3):** en `CapitalFlow` (monto), `AccountSnapshot` (equity/balance/márgenes) y `OrderExecuted` (nocional/PnL/MAE/MFE) los montos son **`i64` escalados ×10⁸** en los structs del Core, **NUNCA `f64`/`REAL`** (ADR-0141). Se serializan como enteros en el JSON. Sin recotización; el monto es un hecho histórico.
- **Consumo del `ExecutionGate` REAL de #2 (`gate_in`):** la decisión "¿replicar este evento a la Cabina de Mando?" se deriva del `ExecutionGate` de `licensing-system` (ya construido — `suppress_work_telemetry`). Si suprime → el evento se persiste **solo local** (flag `replicate=false`); si no → `replicate=true`. **NO hay envío por red** (adaptador diferido); solo se computa y expone el flag. Consúmelo real vía `public_interface`, no un stub.
- **Perfil ADR-0020:** Perfil D — Grupo I completo (con `event_sequence_id`) + II (`owner_id`, `institutional_tag`) + IV (`node_id`, `process_id` NOT NULL, `session_id` nullable). Campos propios marcados: `event_type` (`TEXT CHECK`), `payload` (`TEXT` `json_valid`), `replicate` (`INTEGER` 0/1 — decisión de supresión). Reutiliza el patrón de `migrations/0002_audit_log.sql` (mismos Grupo I + IV).
- **Puertos (ADR-0137):** `gate_in ← ExecutionGate` (Input, de #2), `event_out → EnrichedDomainEvent` (Output 1..N). Bajo `public_interface::enriched_domain_events`.
- **Guardarraíl ADR-0093:** ningún evento incluye secretos (credenciales, IPs live). Assert explícito sobre el payload.
- **Clasificación UI (ADR-0117) + backend-first:** plomería (Ventana de Verificación). Observable por **CLI (Canal #2)**; su ventana de verificación (conteo + último timestamp por tipo) va a la tanda de UI final (DEBT-005).
- **Bus (ADR-0085) inexistente:** no hay `EventBus` en el código. El "publicar en el bus" se difiere; ahora el evento se persiste y se expone por el puerto. Deuda de integración registrada.

## 4. Instrucciones de despacho (prompt exacto)

> Eres el **Rust-Engineer** en **Modo Docente**. Lee primero: `CLAUDE.md`, `.claude/skills/base/SKILL.md`, `.claude/skills/rust-engineer/SKILL.md` (en especial §4 "Atomicidad de ledgers append-only"), esta Orden, la feature `docs/features/enriched-domain-events.md`, la **plantilla canónica de append atómico** `crates/shared/src/persistence/consent_registry.rs`, la migración `migrations/0002_audit_log.sql` (Grupo I + IV + triggers), cómo `licensing-system` expone `ExecutionGate` en `public_interface.rs`, y los ADR-0144, ADR-0145, ADR-0143, ADR-0137, ADR-0141, ADR-0020, ADR-0093, ADR-0142. Declara que los leíste.
>
> **Gate de Lectura Pre-Código:** (a) confirma `crates/shared`; (b) lee el patrón de append atómico de `consent_registry.rs` (transacción `BEGIN IMMEDIATE` + reintento + `WriteContention`) — lo REPLICAS aquí; (c) lee cómo se obtiene el `ExecutionGate` real (`build_execution_gate`/`ExecutionGate.suppress_work_telemetry`).
>
> **Construye (event-store local append-only atómico — bus fan-out y envío por red son futuros):**
> 1. **Migración greenfield `migrations/0012_domain_events.sql`:** tabla `domain_events` **append-only** (`event_sequence_id UNIQUE` + triggers anti UPDATE/DELETE), Grupo I completo + Perfil D (`owner_id`, `institutional_tag`, `node_id`, `process_id NOT NULL`, `session_id` nullable); campos propios: `event_type TEXT CHECK IN (...catálogo...)`, `payload TEXT CHECK(json_valid(payload))`, `replicate INTEGER NOT NULL CHECK(replicate IN (0,1))`. `STRICT`, PK `TEXT` UUIDv7, timestamps `INTEGER` ns UTC, `audit_chain_hash` encadenado. Índices: `event_sequence_id`, `(event_type)`, `owner_id`.
> 2. **Core (`domain/enriched_domain_events.rs`, lógica pura):** enum `EnrichedDomainEvent` con las variantes del catálogo (§3), montos como `i64` ×10⁸ (CERO `f64`); serialización canónica determinista a JSON (`BTreeMap`/claves ordenadas); `event_type` string por variante; `compute_event_audit_hash` encadenado; `decide_replication(gate) -> bool` (suprime → false). Función total, determinista.
> 3. **Shell (`persistence/enriched_domain_events.rs` + `orchestrator/enriched_domain_events.rs`):** repositorio **append-only atómico** — `record_event` con `begin_with("BEGIN IMMEDIATE")` envolviendo `load_tail` + `INSERT`, reintento acotado, `WriteContention` tipado (copia `is_transient_write_conflict`). El orchestrator compone: recibe un `EnrichedDomainEvent` + el `ExecutionGate` real de #2, deriva `replicate`, persiste. Reloj inyectado.
> 4. **`public_interface`:** submódulo `enriched_domain_events` con `event_out` (devuelve el evento persistido) y `gate_in` (consume `ExecutionGate`). Sin secretos (ADR-0093).
> 5. **CLI `verify` (Canal #2, ADR-0142):** `cargo run -p app -- verify enriched-domain-events --input '<json>'` que, dado un evento (tipo + campos) y un tier/gate, reproduce el observable (evento persistido + `replicate`) en JSON.
>
> **Pruebas discriminantes (rojo→verde, deben poder fallar):**
> - **Append atómico + concurrencia:** `#[tokio::test(flavor = "multi_thread")]`, BD en archivo temporal, N≥16 escritores en paralelo → N filas, `event_sequence_id` 1..=N denso, cadena `audit_chain_hash` íntegra/recomputable. Debe caerse sin `BEGIN IMMEDIATE`.
> - **Catálogo + serialización determinista:** cada variante serializa a un payload JSON canónico estable (mismo evento → mismo string); `event_type` correcto; `json_valid` en la BD.
> - **Los 3 de ADR-0145:** `CapitalFlow`, `AccountSnapshot`, `OrderExecuted` reforzado — montos `i64` ×10⁸ (assert de que NO hay `f64`; valores conocidos redondos).
> - **Decisión de replicación:** gate que suprime → `replicate=false`; gate que no → `true`. Consumo del `ExecutionGate` real de #2.
> - **Append-only:** UPDATE/DELETE rechazados por trigger; `event_sequence_id` UNIQUE.
> - **`audit_chain_hash`:** génesis NULL, resto encadenado.
> - **Sin secretos (ADR-0093):** assert sobre el payload.
> - Cobertura con `cargo llvm-cov --workspace --summary-only`.
>
> **Comentarios en español, identificadores en inglés (ADR-0121).** Gate: `cargo test --workspace` verde + `cargo clippy --workspace --all-targets -- -D warnings` cero warnings.
>
> **Docente:** escribe `docs/lessons/rust/STORY-033-enriched-domain-events.md` explicando cero-conocimiento: qué es un event-store heterogéneo (event-sourcing con un enum + payload JSON), por qué append-only atómico, por qué los montos son enteros ×10⁸ y qué reconstruyen (gain% excluyendo depósitos, curvas de equidad, MAE/MFE), y cómo la supresión por tier separa "emitir local" de "replicar al proveedor". Cita el código real.
>
> **NO toques `migrations/` existentes (solo crea 0012). NO hagas commits. NO toques archivos personales ni los del Architect.** Al terminar reporta: archivos, `cargo test --workspace` + `clippy` + `llvm-cov`, salida del `verify enriched-domain-events`, y tu decisión de ubicación del crate.

## 5. Criterio de aceptación (verificable)

| # | Criterio | Prueba |
|---|---|---|
| 1 | Migración `STRICT` append-only: `event_sequence_id UNIQUE` + triggers + `json_valid(payload)` + `CHECK(event_type)` | inspección + tests de rechazo |
| 2 | Append atómico (`BEGIN IMMEDIATE` + reintento + `WriteContention`) + prueba de 2 escritores | test de concurrencia (cae sin la tx) |
| 3 | Catálogo completo incl. los 3 de ADR-0145; serialización JSON canónica determinista | tests por variante |
| 4 | Montos `i64` ×10⁸, cero `f64` | test + inspección |
| 5 | `decide_replication` desde el `ExecutionGate` REAL de #2 | test suprime/no-suprime |
| 6 | Append-only (UPDATE/DELETE rechazados), `event_sequence_id` UNIQUE, `audit_chain_hash` encadenado | tests |
| 7 | Sin secretos (ADR-0093) | test + assert |
| 8 | CLI `verify enriched-domain-events` correcto | `cargo run -p app -- verify enriched-domain-events --input '…'` |
| 9 | Lección Docente escrita | existe el archivo |
| 10 | Verde + cobertura | `cargo test --workspace` + `cargo llvm-cov` |

## 6. Comandos de validación (usuario)

```bash
cargo test -p shared
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo run -p app -- verify enriched-domain-events --input '{"tier":"FREE","event":{"type":"CapitalFlow","account_id":"acc-1","sign":"DEPOSIT","amount":100000000000,"currency":"USD"}}'
```

## 7. Registro de ejecución

- 2026-07-05 · Tech-Lead · Gate corrido (contraste bidireccional). Reglas: event-store `domain_events` **append-only atómico desde el minuto cero** (regla DEBT-001); log heterogéneo (enum + payload JSON canónico); catálogo con los 3 de ADR-0145; montos `i64` ×10⁸; `replicate` derivado del `ExecutionGate` real de #2; bus (ADR-0085) y envío por red diferidos (no hay infraestructura). Reparado el título corrupto del feature doc (`ccccc#`→`#`). Orden creada, despacho a Rust-Engineer (Sonnet, **Docente**).
- 2026-07-05 · Rust-Engineer (Docente) · Implementado EN VERDE (cortado por límite de sesión a mitad del Core, reanudado y completado). Migración `0012_domain_events.sql` (append-only STRICT, `event_sequence_id UNIQUE` + triggers `no_update`/`no_delete`, Grupo I + Perfil D con `process_id`/`session_id`, `event_type CHECK`, `payload json_valid`, `replicate CHECK(0,1)`); Core `domain/enriched_domain_events.rs` (enum de 8 variantes con los 3 de ADR-0145 reforzados, montos `i64` ×10⁸ cero `f64`, serialización canónica `BTreeMap`, `decide_replication`, hash encadenado); Shell `persistence/enriched_domain_events.rs` (append atómico `begin_with("BEGIN IMMEDIATE")` + reintento + `WriteContention`, patrón de `consent_registry`) + `orchestrator/enriched_domain_events.rs` (compone evento + `ExecutionGate` real → deriva `replicate`); puerto `event_out`/`gate_in` en `public_interface`; CLI `verify enriched-domain-events`. Crate `crates/shared`. 343 tests workspace, cobertura Core 99.58%/orch 100%/persist 95.50%. Lección Docente `docs/lessons/rust/STORY-033-enriched-domain-events.md`.
- 2026-07-05 · Tech-Lead · Auditoría independiente (reproducida): 343 tests, 0 fallos; `persistence` con `begin_with("BEGIN IMMEDIATE")` (l.207) envolviendo `load_tail` (l.218) + `INSERT` (l.278) + reintento/`WriteContention`; cero `f64`/`f32` en montos del Core; migración con triggers `no_update`/`no_delete` (`RAISE(ABORT)`) + todos los CHECK. Clippy limpio. Verde.
- 2026-07-05 · QA-Engineer (Sonnet) · **APTO.** 5 pruebas de mutación (revertidas byte a byte), cada una tumbó una prueba concreta: quitar `BEGIN IMMEDIATE` → cae `concurrent_record_events...` (`WriteContention{attempts:5}`, no verde-trivial); invertir `decide_replication` → caen las 4 de replicación; monto desde `f64` → cae `capital_flow_preserves_exact_integer_amount`; quitar trigger UPDATE → cae `update_is_rejected_by_trigger`; génesis no-NULL → cae la de cadena. CLI real con `ExecutionGate` construido de verdad (identidad+licencia+heartbeat), no stub: FREE→replicate=true, PAID→replicate=false, input inválido→exit 1. Sin secretos (ADR-0093). 2 observaciones no bloqueantes (cobertura de ramas de error de SQLite, simétrica a #5; el CLI reconstruye el gate completo, intencional). Árbol intacto.
- 2026-07-05 · Tech-Lead · Gate QA cerrado con APTO. **STORY-033 completada** (raíz del substrato instrumentada). Feature `enriched-domain-events` 🟡 Parcial (bus fan-out + envío a la Cabina de Mando + mapeo de acciones reales de `execute` diferidos). Desbloquea #7/#9/#10.

## 8. Deudas / diferidos registrados

- **Fan-out al bus (ADR-0085):** no existe `EventBus` en el código; el evento se persiste y se expone por el puerto. Se cablea cuando exista la infra de bus.
- **Envío real a la Cabina de Mando:** el flag `replicate` se computa; el adaptador de red es diferido (repo aparte / gateway). Reconciliación = servidor autoritativo.
- **Ventana de Verificación (Canal #1):** conteo + último timestamp por tipo → tanda de UI final (DEBT-005).
- **Mapeo de las acciones reales del motor (`execute`/EPIC-5):** hoy los eventos se construyen desde entradas modeladas; cuando `execute` exista, se cablea la emisión real por acción.
