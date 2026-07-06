# STORY-036 — Data Anonymization & Aggregation (cimiento #9 del substrato)

| Campo | Valor |
|---|---|
| **ID** | STORY-036 |
| **Tipo** | Story (código — noveno cimiento del substrato de pricing/SaaS) |
| **Épica (Fase)** | Cimientos de Monetización (transversal, greenfield) |
| **Sprint** | substrato-monetizacion |
| **Estado** | 🟡 En curso (Core anonimización + agregación + esquema + puertos; pipeline de venta externa y API de red diferidos) |
| **Creada** | 2026-07-06 |
| **Feature** | [`data-aggregation`](../features/data-aggregation.md) |
| **ADRs** | ADR-0144 (cimiento #9) · ADR-0102 (anonimización DP + hash unidireccional) · ADR-0143 (tiers/canal interno vs externo) · ADR-0137 (puertos) · ADR-0141 (append-only + enteros ×10⁸) · ADR-0020 (Perfil B) · ADR-0002 (FCIS — Core puro, aleatoriedad sembrada) · ADR-0093 (datos crudos nunca salen) |

## 1. Objetivo llano

Construir el puerto que toma eventos de ejecución enriquecidos (de #6), los **anonimiza** (ruido gaussiano de privacidad diferencial + hash unidireccional de la topología, ADR-0102) y los **agrega** en índices vendibles (sentimiento, régimen, fricción de bróker, correlación) donde **ningún usuario es reconocible**. Sin esto, cada producto de datos futuro exige reabrir la captura desde el usuario #1. Se entrega el **Core (anonimización + agregación + k-anonimato) + esquema + gate de consentimiento real de #5 + puertos + CLI**; el pipeline de venta externa y la exposición por API de red son adaptadores posteriores.

**Alcance ahora vs. después (ADR-0144 "puerto + esquema ahora, adaptador después"):**
- **Ahora (esta Story):** Core (DP con RNG **sembrado inyectado**, hash unidireccional, k-anonimato) + esquema (`aggregated_indexes` append-only atómica) + consumo real de `event_in` (#6) y `consent_in` (#5) + puertos `event_in`/`consent_in`/`aggregate_out` + CLI verify.
- **Después (diferidos):** el pipeline productivo de venta externa (moonshot `aggregated-data-feeds`), la exposición por la API de terceros (#8, servidor de red diferido), y el cableado del canal interno crudo del tier gratuito hacia la Cabina de Mando (adaptador de red).

## 2. Agentes y Modo de Acompañamiento (ADR-0120/0122)

| Agente | Modelo | Modo |
|---|---|---|
| Rust-Engineer | Sonnet | **Docente** |

Docente: conceptos nuevos — privacidad diferencial (ruido gaussiano **determinista** con semilla), k-anonimato (tamaño mínimo de cohorte), hash unidireccional de topología, y por qué un Core puro (FCIS) NUNCA llama a un RNG del sistema. Lección en `docs/lessons/rust/STORY-036-data-aggregation.md`.

## 3. Gate de Coherencia — resultado (contraste bidireccional)

- **Stack:** limpio. Crate `crates/shared` (plomería crosscutting, excepción ADR-0137: `AggregatedIndex` es tipo técnico del catálogo ADR-0137/0144).
- **Determinismo del ruido — regla obligatoria #1 (ADR-0002 FCIS + ADR-0004 patrón `Clock`):** el ruido gaussiano de privacidad diferencial se genera con un **RNG sembrado e inyectado** (un `seed: u64` o un puerto RNG, igual que el `Clock` se inyecta). El Core es **puro**: NUNCA llama a `rand::thread_rng()` ni a una fuente de entropía del sistema. Misma semilla → mismo valor con ruido (reproducible en tests). Sin esto no hay forma de probar la anonimización ni de auditar el linaje. Defecto si el Core toma aleatoriedad sin semilla.
- **Enteros ×10⁸, cero `f64` en lo persistido — regla obligatoria #2 (ADR-0141):** el `noise_level`, el valor de la métrica agregada y cualquier monto se guardan como **entero ×10⁸**. El cálculo interno del ruido gaussiano puede usar `f64` **transitorio** (Box-Muller / `rand_distr::Normal`), pero el resultado se **redondea a `i64` ×10⁸** antes de persistir; ninguna columna es `REAL`.
- **k-anonimato — regla obligatoria #3 (feature §Parámetros, `MIN_COHORT_SIZE` FIJO):** un agregado cuya cohorte tiene **menos de `MIN_COHORT_SIZE`** contribuyentes **NUNCA se publica** (se suprime → el Core devuelve `None`/`Suppressed`, no una fila). Prueba de borde exacto: en el mínimo → publica; uno menos → suprime. Es invariante FIJO, no configurable.
- **Hash unidireccional de topología — regla obligatoria #4 (ADR-0102):** la topología/firma de estrategia se comprime a `SHA-256` **antes** de entrar a cualquier agregado; jamás se guardan los parámetros/fórmulas crudos. Patrón `hash_*` de `central-identity`/#8.
- **Gate de consentimiento real de #5 — regla obligatoria #5 (feature §Restricciones):** NUNCA se agrega un dato sin consentimiento vigente que lo cubra. El orquestador consulta el `consent_out` **real** de `consent-registry` (#5) vía `resolve_consent_verdict` (default-deny GDPR); un evento sin cobertura o con opt-out **se excluye** del agregado. No un stub.
- **Datos crudos nunca salen — regla obligatoria #6 (ADR-0093/0102, feature §Regla de oro):** el `AggregatedIndex` de salida y la fila persistida NO contienen balances en dólares crudos, llaves, IPs, ni parámetros de estrategia identificables — solo métricas con ruido + hashes + tamaño de cohorte. Guardarraíl estructural con test (como el de secreto de #8).
- **Separación de canales — regla obligatoria #7 (feature §Restricciones, ADR-0143):** el canal **interno** (crudo, tier gratuito, uso lícito por ToS) queda **separado** del canal **externo** (agregado, consentido). Columna `channel CHECK(INTERNAL, EXTERNAL)`; `EXTERNAL_SALE_ENABLED=false` (default) → no se produce agregado de canal externo.
- **Tabla append-only atómica — regla obligatoria #8 (ADR-0141, DEBT-001):** cada índice publicado es un **snapshot inmutable** → `aggregated_indexes` append-only con `event_sequence_id UNIQUE` + triggers anti UPDATE/DELETE + `BEGIN IMMEDIATE`+reintento+`WriteContention`. **Prueba de 2 escritores obligatoria** (qa §2).
- **Perfil ADR-0020:** Perfil B (IA/R&D) — Grupo I + Soberanía II (`owner_id`/`institutional_tag`) + subset de Linaje III (`data_snapshot_id` al conjunto fuente) + Hardware IV (`node_id`). El agregado es un producto de datos derivado (linaje obligatorio).
- **Puertos (ADR-0137):** `event_in ← EnrichedDomainEvent` (Input 0..N), `consent_in ← ConsentVerdict` (Input 1..N), `aggregate_out → AggregatedIndex` (Output 1..N). Bajo `public_interface::data_aggregation`.
- **Diferidos:** pipeline de venta externa, exposición por API de red (#8), adaptador de red del canal interno. NO los cablees.

## 4. Instrucciones de despacho (prompt exacto)

> Eres el **Rust-Engineer** en **Modo Docente**. Lee primero: `CLAUDE.md`, `.claude/skills/base/SKILL.md`, `.claude/skills/rust-engineer/SKILL.md` (§4 atomicidad de ledgers + determinismo), esta Orden, la feature `docs/features/data-aggregation.md`, el ADR-0102 (`docs/adr/ADR-0102.md`), el patrón **append atómico** `crates/shared/src/persistence/enriched_domain_events.rs`, cómo se define `EnrichedDomainEvent` (#6) en `crates/shared/src/domain/enriched_domain_events.rs`, cómo `consent-registry` (#5) expone su veredicto real en `orchestrator::consent_registry::resolve_consent_verdict`, y los ADR-0144, ADR-0143, ADR-0137, ADR-0141, ADR-0020, ADR-0002, ADR-0093. Declara que los leíste.
>
> **Gate de Lectura Pre-Código:** (a) confirma `crates/shared`; (b) copia el patrón append atómico (`enriched_domain_events`); (c) confirma cómo consumir el `EnrichedDomainEvent` real de #6 y el veredicto de consentimiento real de #5; (d) confirma el patrón de inyección del `Clock` para replicarlo con un **RNG sembrado**; (e) confirma que NO agregas API de red ni pipeline de venta externa.
>
> **Construye (Core anonimización + agregación + esquema + puertos; adaptadores diferidos):**
> 1. **Migración `migrations/0015_data_aggregation.sql`** con `aggregated_indexes` APPEND-ONLY (`event_sequence_id UNIQUE` + triggers anti UPDATE/DELETE + `audit_chain_hash`). Grupo I + **Perfil B** (`owner_id`, `institutional_tag`, `node_id`, `data_snapshot_id`). Campos propios: `index_type` (TEXT CHECK `SENTIMENT`/`REGIME`/`BROKER_FRICTION`/`CORRELATION`), `time_window` (TEXT), `cohort_size` (INTEGER), `noise_level` (INTEGER ×10⁸), `metric_value` (INTEGER ×10⁸), `channel` (TEXT CHECK `INTERNAL`/`EXTERNAL`). `STRICT`, UUIDv7. Índice en `event_sequence_id` + el que sirva las consultas por `(index_type, time_window)`.
> 2. **Core `domain/data_aggregation.rs`:** `apply_differential_privacy(raw_value_e8: i64, noise_level_e8: i64, seed: u64) -> i64` (ruido gaussiano determinista por semilla, resultado entero ×10⁸); `hash_strategy_topology(topology: &str) -> String` (SHA-256 hex); `meets_k_anonymity(cohort_size: i64, min_cohort: i64) -> bool`; `aggregate_index(inputs, min_cohort, seed) -> Option<AggregatedIndex>` (anonimiza los cubiertos, suma, verifica cohorte; `None` si suprime por k-anonimato); enum `IndexType`, `Channel`; tipo `AggregatedIndex`; `compute_aggregate_audit_hash` encadenado. Determinista, cero aleatoriedad sin semilla.
> 3. **Shell:** `persistence/data_aggregation.rs` — repo append-only atómico (`BEGIN IMMEDIATE`+reintento+`WriteContention`, patrón `enriched_domain_events`) + `load_chain`; `orchestrator/data_aggregation.rs` — flujo: por cada evento, **consultar consentimiento real de #5** (excluir no-cubiertos/opt-out) → anonimizar los cubiertos → agregar → **verificar k-anonimato** (suprimir si no llega) → persistir el snapshot con su linaje. Reloj y semilla inyectados.
> 4. **`public_interface`:** submódulo `data_aggregation` con `event_in`/`consent_in`/`aggregate_out`. Sin datos crudos identificables en la salida (ADR-0093/0102).
> 5. **CLI `verify`:** `cargo run -p app -- verify data-aggregation --input '<json>'` que, dado un lote de eventos + consentimiento + semilla, reproduce el observable (índice agregado publicado o supresión por cohorte) en JSON.
>
> **Pruebas discriminantes (rojo→verde):**
> - **Ruido determinista:** misma semilla + mismo valor → mismo resultado con ruido (reproducible); y el resultado con ruido **difiere** del valor crudo (privacidad real). Debe fallar si el Core usa RNG sin semilla.
> - **k-anonimato de borde exacto:** cohorte `== MIN_COHORT_SIZE` → publica; `== MIN-1` → suprime (`None`). Debe fallar si publica bajo el mínimo.
> - **Gate de consentimiento real de #5:** evento sin cobertura → excluido del agregado; opt-out → excluido; cubierto → incluido. Usa el veredicto real, no un stub.
> - **Datos crudos nunca salen:** el `AggregatedIndex` serializado NO contiene el balance crudo, la topología cruda ni identificadores — solo métrica con ruido + hash + cohorte (assert estructural ADR-0093/0102).
> - **Separación de canales:** `EXTERNAL_SALE_ENABLED=false` → no se produce agregado `EXTERNAL`; el interno sí. `channel` persistido correcto.
> - **Append atómico + concurrencia:** 16 escritores sobre `aggregated_indexes` (archivo temporal) → N filas, `event_sequence_id` 1..=N denso. Cae sin `BEGIN IMMEDIATE`.
> - **Append-only** (UPDATE/DELETE rechazados por trigger), `event_sequence_id` UNIQUE, `audit_chain_hash` encadenado, CHECKs de `index_type`/`channel`.
> - **Enteros ×10⁸:** `noise_level`/`metric_value` enteros; ninguna columna `REAL`.
> - Cobertura con `cargo llvm-cov --workspace --summary-only`.
>
> **Comentarios en español, identificadores en inglés (ADR-0121).** Gate: `cargo test --workspace` verde + `cargo clippy --workspace --all-targets -- -D warnings` cero warnings.
>
> **Docente:** `docs/lessons/rust/STORY-036-data-aggregation.md` cero-conocimiento: qué es privacidad diferencial y por qué el ruido debe ser **determinista con semilla** (no puede probarse ni auditarse un RNG del sistema), qué es k-anonimato y por qué una cohorte pequeña se suprime, por qué la topología se guarda hasheada y nunca cruda, y por qué el agregado consulta consentimiento antes de sumar cada dato. Cita el código real.
>
> **NO agregues API de red, tonic ni pipeline de venta externa. NO toques migraciones existentes (solo crea 0015). NO hagas commits. NO toques archivos personales ni los 6 del Architect** (`docs/ROADMAP.md`, `docs/features/licensing-system.md`, `docs/features/usage-metering.md`, `docs/features/central-identity.md`, `docs/README.md`, `docs/moonshots/encrypted-local-backup.md`). Al terminar reporta: archivos, `cargo test --workspace` + `clippy` + `llvm-cov`, salida del `verify`, y tu decisión de crate.

## 5. Criterio de aceptación (verificable)

| # | Criterio | Prueba |
|---|---|---|
| 1 | Migración `STRICT`: `aggregated_indexes` append-only (triggers + UNIQUE) + Grupo I + Perfil B (`data_snapshot_id` linaje) | inspección + tests |
| 2 | Ruido DP **determinista con semilla** + difiere del crudo | test de reproducibilidad + test de privacidad |
| 3 | k-anonimato de borde exacto (mínimo → publica; -1 → suprime) | test discriminante |
| 4 | Gate de consentimiento con `consent_out` REAL de #5 (excluye no-cubiertos/opt-out) | test cubre/excluye |
| 5 | Datos crudos nunca en la salida (ADR-0093/0102) | assert estructural |
| 6 | Separación de canales INTERNAL/EXTERNAL + `EXTERNAL_SALE_ENABLED` | test |
| 7 | Append atómico + 2 escritores | test de concurrencia (cae sin la tx) |
| 8 | `audit_chain_hash` encadenado; `event_sequence_id` UNIQUE; enteros ×10⁸ | tests |
| 9 | CLI `verify data-aggregation` | `cargo run -p app -- verify data-aggregation --input '…'` |
| 10 | Lección Docente | existe el archivo |
| 11 | Verde + cobertura | `cargo test --workspace` + `cargo llvm-cov` |

## 6. Comandos de validación (usuario)

```bash
cargo test -p shared
cargo clippy --workspace --all-targets -- -D warnings
cargo run -p app -- verify data-aggregation --input '{"seed":42,"min_cohort":5,"external_sale_enabled":false,"events":[{"metric_e8":150000000,"consent":"COVERED"}]}'
```

## 7. Registro de ejecución

- 2026-07-06 · Tech-Lead · Gate de Coherencia corrido (contraste bidireccional). Reglas obligatorias: (1) ruido DP **determinista con RNG sembrado inyectado** (Core puro FCIS, cero aleatoriedad sin semilla); (2) enteros ×10⁸ en lo persistido; (3) k-anonimato FIJO con supresión; (4) hash unidireccional de topología (ADR-0102); (5) gate de consentimiento **real** de #5; (6) datos crudos nunca salen (ADR-0093/0102); (7) separación de canales interno/externo; (8) tabla append-only atómica (DEBT-001). Perfil B (linaje `data_snapshot_id`). API de red + venta externa diferidas. Orden creada, despacho a Rust-Engineer (Sonnet, **Docente**).
- 2026-07-06 · Rust-Engineer (Sonnet, Docente) · Entregado (en dos tramos — dos cortes por error de API/límite de sesión; el TL reanudó vía SendMessage y el código quedó íntegro antes del último corte). Migración `0015_data_aggregation.sql` (`aggregated_indexes` STRICT append-only, Perfil B), Core `domain/data_aggregation.rs` (`apply_differential_privacy` RNG sembrado Box-Muller, `meets_k_anonymity`, `hash_strategy_topology`, `aggregate_index`, hash encadenado), persistencia `persistence/data_aggregation.rs` (append atómico `BEGIN IMMEDIATE`+reintento+`WriteContention` + prueba de 16 escritores en archivo), orquestador `orchestrator/data_aggregation.rs` (`run_aggregation` con `consent_out` real de #5 + separación de canales), CLI `verify data-aggregation`, lección `docs/lessons/rust/STORY-036-data-aggregation.md`. 31 tests nuevos de la feature.
- 2026-07-06 · Tech-Lead · **Auditoría independiente APROBADA** (reproducción: clippy 0 warnings, 443 tests verdes; RNG sembrado inyectado verificado, k-anon verificado ANTES de sumar, ruido una vez sobre la suma, guardarraíl estructural de datos crudos con test, gate de consentimiento real excluye opt-out/no-cubiertos, migración STRICT+triggers+Perfil B). Observación: en `run_aggregation` la topología se hashea y descarta (`let _ = ...`) sin persistir columna → placeholder ADR-0102, registrado en DEBT-012.
- 2026-07-06 · QA (mutación, ejecutada por el TL vía `cargo-mutants` ante caída del subagente por límite de sesión) · **APTO**. 61 mutantes: 35 cazados, 8 inviables, 18 sobrevivientes; mutación manual de `BEGIN IMMEDIATE`→`begin()` tumba la prueba de 16 escritores 5/5. Críticos cazados (k-anon `>=`, consentimiento, canales, guardarraíl, hash de auditoría). Sobrevivientes = huecos no bloqueantes (fórmula del ruido sin valor-dorado; ruta de reintento no ejercitada; hash de topología descartado) → **DEBT-012**.
- 2026-07-06 · Tech-Lead · **CIMIENTO #9 CERRADO.** Feature sellada 🟡 Parcial; substrato **9/10**. Pendiente de autorización: commit agrupado.

## 8. Deudas / diferidos registrados

- **Pipeline de venta externa (moonshot `aggregated-data-feeds`):** el canal externo produce el snapshot agregado; el pipeline productivo de distribución/venta es adaptador posterior.
- **Exposición por la API de terceros (#8):** los índices se venden por el `third-party-api-gateway`, cuyo servidor de red está diferido (Canal #3).
- **Adaptador de red del canal interno (tier gratuito → Cabina de Mando):** el firehose crudo del tier gratuito hacia el proveedor (ADR-0143) es adaptador de red diferido; aquí solo se modela la separación de canales.
- **Ventana de Verificación (Canal #1):** panel de índices agregados + tamaño de cohorte → tanda de UI final (DEBT-005).
- **Huecos de cobertura del QA → DEBT-012:** fórmula del ruido Box-Muller sin test de valor-dorado; ruta de reintento no ejercitada (mismo problema sistémico que DEBT-011); hash de topología calculado y descartado (placeholder hasta que la topología sea dimensión real). No bloqueante.
