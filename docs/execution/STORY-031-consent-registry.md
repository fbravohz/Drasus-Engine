# STORY-031 — Consent Registry / Registro de Consentimiento ToS (cimiento #5 del substrato de monetización)

| Campo | Valor |
|---|---|
| **ID** | STORY-031 |
| **Tipo** | Story (código — quinto cimiento del substrato de pricing/SaaS) |
| **Épica (Fase)** | Cimientos de Monetización (transversal, greenfield — va antes de la auditoría retroactiva) |
| **Sprint** | substrato-monetizacion |
| **Estado** | ✅ Completada (registro append-only local, ledger atómico bajo concurrencia; sincronización con la Cabina de Mando y UI diferidos) |
| **Creada** | 2026-07-04 |
| **Feature** | [`consent-registry`](../features/consent-registry.md) |
| **ADRs** | ADR-0144 (cimiento #5) · ADR-0143 (firehose gratuito) · ADR-0137 (puerto `ConsentVerdict`) · ADR-0141 (append-only + `event_sequence_id`) · ADR-0020 (Perfil D) · ADR-0093 (secretos) · ADR-0142 (CLI verify) · ADR-0145 (opt-in del track record consume este veredicto) |

## 1. Objetivo llano

Construir el **registro append-only y versionado de aceptación de Términos y Condiciones**, con granularidad opt-in/opt-out por tipo de dato: la migración de la tabla de consentimiento (inmutable), la lógica pura que decide "¿este tipo de dato está cubierto por un consentimiento vigente?" (respetando la versión aceptada vs. la vigente y el mapa de opt-outs), y el puerto `consent_out` que produce `ConsentVerdict`. Es la **columna vertebral legal** del substrato: el firehose del tier gratuito (ADR-0143), la venta de datos agregados (`data-aggregation`/#9) y el opt-in del track record publicable (ADR-0145) son legales **solo si** este registro devuelve "cubierto".

**Alcance ahora vs. después (ADR-0144 "puerto + esquema ahora, adaptador después"):**
- **Ahora (esta Story):** esquema append-only + Core (comparación de versión + resolución de cobertura respetando opt-outs) + puerto `consent_out` + CLI verify.
- **Después (diferidos):** la sincronización real con la Cabina de Mando (replica de la prueba legal central); la pantalla de aceptación de ToS + panel de opt-outs granulares (UI); el cableado real desde `data-aggregation`/firehose como consumidores.

## 2. Agentes y Modo de Acompañamiento (ADR-0120/0122)

| Agente | Modelo | Modo |
|---|---|---|
| Rust-Engineer | Sonnet | **Docente** |

En Docente el ingeniero implementa el bloque completo y escribe la lección cero-conocimiento en `docs/lessons/rust/STORY-031-consent-registry.md` (ADR-0124).

## 3. Gate de Coherencia — resultado (contraste bidireccional)

- **Stack:** limpio (Rust + CLI). Crate `crates/shared` (plomería crosscutting, excepción bendecida ADR-0137: tipo técnico `ConsentVerdict` análogo a `UsageRecord`/`AuditEvent`, consumido por ≥2 dominios — `data-aggregation` + firehose + track record #10 — y sin puerto de Alpha en el canvas).
- **Esquema APPEND-ONLY (ADR-0141) — regla obligatoria #1:** la tabla es **append-only** (feature §Restricciones: "El registro de consentimiento es append-only (inmutable, auditable)"). Lleva **`event_sequence_id INTEGER NOT NULL UNIQUE`** (posición monótona), **NO `row_version`**, + **triggers `BEFORE UPDATE`/`BEFORE DELETE` que abortan** (patrón `migrations/0002_audit_log.sql` / `0010_usage_metering.sql`). `audit_chain_hash` encadenado (NULL solo en génesis).
- **Opt-outs mutables sobre tabla inmutable — regla obligatoria #2 (EL punto de modelado crítico):** los opt-outs cambian con el tiempo, pero el registro NO se edita. Modélalo **event-sourced con snapshot completo**: **cada cambio de consentimiento es una fila-evento nueva que captura el estado COMPLETO** (versión aceptada + mapa de opt-outs entero en ese momento). El **estado vigente** de un usuario = la fila con el **`event_sequence_id` máximo para su `owner_id`** (o `MAX(created_at)` como desempate determinista). NO se hace fold parcial ni se muta la fila anterior. Prueba discriminante: cambiar un opt-out inserta fila nueva; la anterior queda intacta; la resolución de cobertura lee la última.
- **Resolución de cobertura — regla obligatoria #3 (correctitud legal):** `resolve_coverage(estado_vigente, tipo_dato, versión_vigente)` devuelve `Covered` **solo si** (a) la versión aceptada == la versión de ToS vigente (consentimiento no obsoleto) **y** (b) el tipo de dato NO está en opt-out. En cualquier otro caso `NotCovered` con razón (`StaleVersion` | `OptedOut` | `NoConsent`). El default es **negar** (feature §Restricciones: "NUNCA se asume consentimiento por defecto"). Un error aquí es ilegalidad (GDPR).
- **Re-aceptación forzada (FIJO):** `REACCEPT_ON_VERSION_CHANGE = true` es invariante (feature §Parámetros). Si la versión aceptada ≠ la vigente → `needs_reacceptance` = true y toda cobertura cae a `StaleVersion` hasta re-aceptar. No es configurable.
- **Perfil ADR-0020:** Perfil D — Grupo I completo (con `event_sequence_id`) + II (`owner_id`, `institutional_tag`) + IV (`node_id`) + subset V (`compliance_status_id` nullable). Campos propios marcados: `tos_version` (`TEXT`, versión aceptada), `consent_action` (`TEXT` + `CHECK IN ('ACCEPT','REACCEPT','OPTOUT_CHANGE')`), `optout_map` (`TEXT` JSON con `CHECK(json_valid(optout_map))` — mapa `{tipo_dato: bool}`, true = opted-out), `accepted_at` (`INTEGER` ns UTC, momento de dominio de la acción vía reloj inyectado).
- **Puerto (ADR-0137):** `consent_out` → `ConsentVerdict` (`Covered` / `NotCovered{reason}`), tipo técnico nuevo del catálogo (feature §Puertos). Consumido por `data-aggregation` y el firehose. Bajo `public_interface::consent_registry`. Confirma la ubicación leyendo el patrón ya construido (`usage_metering`); si tu lectura contradice → **párate y escálame**.
- **Depende de `central-identity` (#1, ya construido):** `owner_id` viene de la identidad. Consúmelo como el patrón ya establecido; no reinventes identidad.
- **Guardarraíl ADR-0093:** el registro JAMÁS almacena secretos — ninguna columna para credenciales, tokens ni IPs. Solo versión de ToS, opt-outs y metadatos de auditoría.
- **Clasificación UI (ADR-0117) + backend-first (decisión del usuario 2026-07-04):** la feature tiene Superficie propia (pantalla ToS + panel de opt-outs), pero por backend-first su **SVF (Canal #1) + galería** van a la **tanda de UI final del substrato** (harness SVF genérico) — deuda **rastreada y autorizada**, NO silenciosa. Para ESTA Story el observable se verifica por **CLI (Canal #2, ADR-0142)**.
- **SAD:** SAD-22 ya cubre el substrato. Desalineamiento → escala.

## 4. Instrucciones de despacho (prompt exacto)

> Eres el **Rust-Engineer** en **Modo Docente**. Lee primero: `CLAUDE.md`, `.claude/skills/base/SKILL.md`, `.claude/skills/rust-engineer/SKILL.md`, esta Orden completa, la feature `docs/features/consent-registry.md`, la feature ya construida `docs/features/central-identity.md` (de donde viene `owner_id`), las migraciones `migrations/0002_audit_log.sql` y `migrations/0010_usage_metering.sql` (patrón append-only + triggers + `json_valid`), y los ADR-0144, ADR-0143, ADR-0137, ADR-0141, ADR-0020 (§ADR.md perfiles), ADR-0093, ADR-0142, ADR-0145 (por qué el opt-in del track record consume este veredicto). Declara que los leíste.
>
> **Gate de Lectura Pre-Código:** (a) confirma la ubicación del crate (`crates/shared`); (b) lee cómo `usage-metering` (#4) expone su puerto append-only en `crates/shared/src/{domain,persistence,orchestrator,public_interface}` y `crates/app/src/main.rs` — replica ESE patrón exacto (repo append-only, submódulo en `public_interface`, harness CLI); (c) lee cómo `central-identity` expone `owner_id`.
>
> **Construye (registro local append-only — la sincronización con la Cabina de Mando y la UI son futuros):**
> 1. **Migración greenfield 0011** de la tabla de consentimiento (`consent_records`): **append-only** — `event_sequence_id INTEGER NOT NULL UNIQUE` (NO `row_version`) + triggers `BEFORE UPDATE`/`BEFORE DELETE` que abortan (patrón `0010_usage_metering.sql`). Grupo I completo + Perfil D + `owner_id`/`institutional_tag`/`node_id` + `compliance_status_id` nullable; campos propios marcados (`tos_version TEXT`, `consent_action TEXT CHECK IN ('ACCEPT','REACCEPT','OPTOUT_CHANGE')`, `optout_map TEXT CHECK(json_valid(...))`, `accepted_at INTEGER`). `STRICT`, PK `TEXT` UUIDv7, timestamps `INTEGER` ns UTC, `audit_chain_hash` encadenado. Índices: `event_sequence_id` (obligatorio append-only) + `owner_id` + `(owner_id, event_sequence_id)` para la consulta del estado vigente.
> 2. **Core (lógica pura, sin I/O):** (a) `needs_reacceptance(accepted_version, current_version) -> bool` (true si difieren — `REACCEPT_ON_VERSION_CHANGE` es FIJO); (b) `resolve_coverage(consent_state, data_type, current_version) -> ConsentVerdict` que devuelve `Covered` **solo si** versión aceptada == vigente **y** el tipo NO está en opt-out; si no, `NotCovered{reason: StaleVersion | OptedOut | NoConsent}`. Default = **negar**. (c) parseo puro y determinista del `optout_map` (JSON → mapa). CERO asunción de consentimiento por defecto.
> 3. **Shell:** repositorio **append-only** (solo INSERT; nada de update/delete — los triggers lo garantizan a nivel BD); **cada cambio = fila-evento nueva con snapshot completo** (versión + optout_map entero); consulta del **estado vigente** = fila con `MAX(event_sequence_id)` por `owner_id`; `accepted_at` derivado del reloj inyectado (no `SystemTime`); `owner_id` consumido de `central-identity`.
> 4. **`public_interface`:** el puerto `consent_out` que devuelve `ConsentVerdict` para un `(owner_id, data_type)`. **Sin secretos** (ADR-0093).
> 5. **CLI `verify` (Canal #2, ADR-0142):** subcomando que, dada una secuencia de acciones de consentimiento (aceptar versión, cambiar opt-out) y una consulta `(data_type, current_version)`, reproduce el observable (`ConsentVerdict`) en JSON, ejecutable por `cargo run -p app -- verify consent-registry --input '<json>'`.
>
> **Pruebas discriminantes (rojo→verde, deben poder fallar):**
> - **Cobertura básica:** usuario aceptó la versión vigente, sin opt-out del tipo X → `Covered`. Debe fallar si devuelve NotCovered.
> - **Opt-out granular manda:** mismo usuario con opt-out del tipo X → `NotCovered{OptedOut}`; el tipo Y (no opted-out) sigue `Covered`. Debe fallar si ignora el opt-out.
> - **Versión obsoleta:** aceptó v1, la vigente es v2 → `NotCovered{StaleVersion}` para TODO tipo, hasta re-aceptar. Debe fallar si cubre con versión vieja.
> - **Sin consentimiento = negar (default):** usuario sin ninguna fila → `NotCovered{NoConsent}`. Debe fallar si asume consentimiento.
> - **Append-only (patrón audit_log):** `UPDATE`/`DELETE` sobre `consent_records` → rechazados por trigger. Debe fallar si permite mutar.
> - **Snapshot event-sourced:** cambiar un opt-out inserta fila NUEVA (event_sequence_id+1); la fila anterior queda intacta; la resolución lee la última. Debe fallar si muta o si lee una fila vieja.
> - **`event_sequence_id` monótono y UNIQUE:** inserciones consecutivas → 1,2,3…; duplicar → rechazado.
> - **`optout_map` es JSON válido:** `CHECK(json_valid)` rechaza JSON corrupto.
> - **Guardarraíl ADR-0093:** el payload de `ConsentVerdict` NO contiene secretos (assert explícito).
> - **`audit_chain_hash`:** encadenado entre filas; NULL solo en génesis.
> - Cobertura del criterio con `cargo llvm-cov --workspace --summary-only`.
>
> **Comentarios en español, identificadores en inglés (ADR-0121).** Entrega en verde con mapeo criterio→prueba.
>
> **Docente:** escribe `docs/lessons/rust/STORY-031-consent-registry.md` (enlace a esta Orden al inicio) explicando cero-conocimiento: qué es un registro de consentimiento y por qué es la columna vertebral legal (GDPR); por qué es append-only y cómo se modela un estado MUTABLE (opt-outs) sobre una tabla INMUTABLE (event-sourcing con snapshot + última fila gana); por qué el default es negar; qué es la re-aceptación forzada por versión. Cita el código real.
>
> **NO hagas commits** (los hace el Tech-Lead). Al terminar reporta: archivos creados, salida de `cargo test` + `cargo llvm-cov`, salida del `cargo run -p app -- verify consent-registry`, y tu decisión de ubicación del crate con su justificación.

## 5. Criterio de aceptación (verificable)

| # | Criterio | Prueba |
|---|---|---|
| 1 | Migración `STRICT` append-only: `event_sequence_id UNIQUE` + triggers anti UPDATE/DELETE + `json_valid(optout_map)` | inspección del `.sql` + test de rechazo |
| 2 | `resolve_coverage` cubre solo con versión vigente + sin opt-out; default niega | tests discriminantes (4 casos: Covered / OptedOut / StaleVersion / NoConsent) |
| 3 | Estado vigente = última fila; cambio de opt-out inserta fila nueva sin mutar | test event-sourced |
| 4 | Re-aceptación forzada al cambiar versión (FIJO) | test StaleVersion |
| 5 | `event_sequence_id` monótono y UNIQUE | test |
| 6 | `ConsentVerdict` sin secretos (ADR-0093) | test + assert |
| 7 | `audit_chain_hash` encadenado (NULL solo génesis) | test |
| 8 | CLI `verify consent-registry` devuelve el JSON correcto | `cargo run -p app -- verify consent-registry --input '…'` |
| 9 | Lección Docente escrita | existe `docs/lessons/rust/STORY-031-consent-registry.md` |
| 10 | Verde + cobertura de cada criterio | `cargo test` + `cargo llvm-cov` |

## 6. Comandos de validación (usuario)

```bash
cargo test -p shared
cargo llvm-cov --workspace --summary-only
cargo run -p app -- verify consent-registry --input '{"current_version":"v2","actions":[{"action":"ACCEPT","tos_version":"v2","optout_map":{"aggregation":false}}],"query":{"data_type":"aggregation"}}'
```

## 7. Registro de ejecución

- 2026-07-04 · Tech-Lead · Gate corrido (contraste bidireccional). Reglas: tabla **append-only** (`event_sequence_id UNIQUE` + triggers, NO `row_version`); opt-outs mutables modelados **event-sourced con snapshot completo** (última fila por `owner_id` gana); cobertura **niega por default** y exige versión vigente + sin opt-out; re-aceptación forzada FIJA; puerto `consent_out`→`ConsentVerdict`; consumo de `owner_id` de #1; SVF/galería a la tanda de UI final (deuda rastreada). Orden creada, pendiente de despacho a Rust-Engineer (Sonnet, **Docente**).
- 2026-07-04 · Rust-Engineer (Docente) · Implementación completa en verde: migración `0011_consent_registry.sql`, `domain/persistence/orchestrator::consent_registry` en `crates/shared`, puerto `consent_out` + CLI `verify consent-registry` en `public_interface`/`crates/app/src/main.rs`. `cargo test -p shared` (285 pruebas, 30 nuevas) + `cargo clippy --workspace --all-targets -- -D warnings` (cero warnings) + `cargo llvm-cov --workspace --summary-only` (consent_registry: domain 100%, orchestrator 99.01%, persistence 98.50% de líneas) todos en verde. CLI verificado end-to-end con el comando exacto de §6 y variantes (OptedOut/StaleVersion/NoConsent). Lección Docente: `docs/lessons/rust/STORY-031-consent-registry.md`. Feature doc sellada 🟡 Parcial.
- 2026-07-04 · Rust-Engineer (Docente) · Reapertura DEBT-001 ("Atomicidad de ledgers append-only", regla nueva del SKILL §4). `ConsentRepository::record_action` reescrito: el read-then-write (leer estado del dueño + `MAX(event_sequence_id)` + `INSERT`) ahora ocurre en UNA transacción `BEGIN IMMEDIATE` (`try_record_action_once`) con reintento acotado (`MAX_RECORD_ATTEMPTS = 5`) ante contención transitoria y error tipado `WriteContention` si se agota (nunca pérdida silenciosa). `busy_timeout = 5s` añadido a `persistence/pool.rs` (ADR-0141 R2, faltaba). Nueva prueba `#[tokio::test(flavor = "multi_thread")]` con 16 escritores concurrentes: afirma (a) las 16 filas persistidas, (b) `event_sequence_id` = 1..=16 densos, (c) cadena `audit_chain_hash` íntegra + `audit_hash` recomputable, y (bonus) snapshot final acumula las 16 claves de opt-out. Falsación empírica confirmada: sin la transacción la prueba cae con `UNIQUE constraint failed: consent_records.event_sequence_id`. `cargo test -p shared` 286 verde (x5 sin flakes), `cargo test --workspace` sin regresión (incl. durabilidad kill-9 y telemetría), `cargo clippy --workspace --all-targets -- -D warnings` cero warnings. Solo se tocó `consent-registry` + `pool.rs` (infra, no ledger); los otros ledgers (`audit_log`/`usage_records`) quedan para STORY-032. Lección Docente ampliada.
- 2026-07-04 · Tech-Lead · Auditoría independiente (reproducida, no tomada del reporte): `cargo test -p shared consent_registry` → 31/31 verde (incluida la de concurrencia); `cargo clippy -p shared -p app --all-targets -- -D warnings` limpio. Inspección de `persistence/consent_registry.rs`: `begin_with("BEGIN IMMEDIATE")` envuelve las dos lecturas + INSERT; bucle de reintento solo ante conflicto transitorio (`is_transient_write_conflict`: "database is locked" o UNIQUE sobre `event_sequence_id`), `WriteContention` tipado al agotar; la prueba de concurrencia usa BD en **archivo temporal** (no `:memory:`), 16 tareas, asserts densos 1..=N + cadena recomputable. FCIS: Core sin I/O. Verde.
- 2026-07-04 · QA-Engineer (Sonnet) · **APTO.** Lógica línea por línea + **10 pruebas de mutación** (aplicadas y revertidas byte a byte), cada una tumbó una prueba concreta: invertir la puerta de opt-out (4 pruebas), `!=`→`==` en `needs_reacceptance` (7), default→`Covered` (2), **quitar `BEGIN IMMEDIATE`** → cae `concurrent_record_actions...` con `WriteContention{attempts:5}` (no verde-trivial), quitar trigger UPDATE / `UNIQUE` / `CHECK(json_valid)` / `CHECK(consent_action)` / génesis no-NULL / inyectar `api_key` en el JSON del veredicto (guardarraíl ADR-0093). `cargo test -p shared` 286/286, clippy limpio, CLI 4 variantes correctas. Árbol intacto (git diff --stat idéntico). **Observación no bloqueante:** una `OPTOUT_CHANGE` como PRIMERA acción de un `owner_id` no tiene guarda explícita — inofensivo (cae a `StaleVersion`, falla-seguro); una guarda explícita va a STORY-032 (registrada como DEBT-007).
- 2026-07-04 · Tech-Lead · Gate QA cerrado con APTO. **STORY-031 completada** (registro local append-only, ledger atómico bajo concurrencia). Feature `consent-registry` 🟡 Parcial por diseño (sincronización con la Cabina de Mando + UI diferidas).

## 8. Deudas / diferidos registrados

- **Sincronización con la Cabina de Mando (diferida):** el consentimiento es prueba legal central; se replica al servidor cuando exista el adaptador de red (repo aparte / gateway). Ahora solo local.
- **Cableado real de consumidores (diferido):** `data-aggregation` (#9) y el firehose de `enriched-domain-events` consultarán `consent_out` cuando se construyan; el opt-in del track record (ADR-0145/#10) también. Ahora el puerto existe y se verifica por CLI.
- **SVF (Canal #1) + galería con mocks:** pantalla de aceptación de ToS + panel de opt-outs → tanda de UI final del substrato (harness SVF genérico). Deuda rastreada y autorizada (backend-first).
