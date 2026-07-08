# STORY-037 — Verified Account Registry (cimiento #10 del substrato · pilar Cuentas Verificadas)

| Campo | Valor |
|---|---|
| **ID** | STORY-037 |
| **Tipo** | Story (código — décimo y último cimiento del substrato de pricing/SaaS) |
| **Épica (Fase)** | Cimientos de Monetización (transversal, greenfield) |
| **Sprint** | substrato-monetizacion |
| **Estado** | 🟡 En curso (registro + track record + firma + esquema + puertos; portal, contrato de reporte al servidor central y conexión read-only real al bróker diferidos) |
| **Creada** | 2026-07-06 |
| **Feature** | [`verified-account-registry`](../features/verified-account-registry.md) |
| **ADRs** | ADR-0145 (pilar #10 — rector) · ADR-0143 (tres planos + telemetría clase 5) · ADR-0093 (secretos nunca salen) · ADR-0137 (puertos) · ADR-0141 (append-only + row_version + ×10⁸) · ADR-0020 (Perfil D + subset V) · ADR-0002 (FCIS) |

## 1. Objetivo llano

Construir el **registro multi-cuenta con track record verificado** — el pilar análogo a myFXbook/MT5 Signals, con el diferenciador soberano: Drasus atestigua criptográficamente lo que **su propio motor ejecutó** (cadena de hash + append-only), no solo lo que el bróker reporta. Se entrega el **Core (track record por ámbito de atestación + gain% que excluye flujo de capital + firma reproducible) + esquema (registro de cuentas + track atestado) + consumo real de #6/#5/#1 + puertos + CLI**; el portal público, el contrato de reporte al servidor central y la conexión read-only real al bróker son adaptadores posteriores.

**Alcance ahora vs. después (ADR-0145 "contrato + esquema ahora, portal después"):**
- **Ahora (esta Story):** Core (cálculo del track record por ámbito, gain% sin depósitos, drawdown, estadística, firma) + esquema (`verified_accounts` mutable + `attested_track_records` append-only) + consumo real de `EnrichedDomainEvent` (#6), `ConsentVerdict` (#5) y `owner_id` (#1) + puertos `event_in`/`consent_in`/`registry_out`/`track_record_out` + CLI verify.
- **Después (diferidos):** el **portal público** (repo aparte, stack libre — ADR-0145); el **contrato/adaptador de reporte** del track publicado hacia el servidor central (adaptador de red); la **conexión read-only real al bróker** (`broker-connector` en el Plano de Ejecución — aquí se modela el ámbito, no se cablea el fetch); el panel de UI (SVF).

## 2. Agentes y Modo de Acompañamiento (ADR-0120/0122)

| Agente | Modelo | Modo |
|---|---|---|
| Rust-Engineer | Sonnet | **Docente** |

Docente: conceptos nuevos — atestación soberana (por qué la cadena de hash prueba "lo ejecutó Drasus"), la distinción inviolable ámbito soberano vs read-only del bróker, y el cálculo del gain% que **excluye** el flujo de capital. Lección en `docs/lessons/rust/STORY-037-verified-account-registry.md`.

## 3. Gate de Coherencia — resultado (contraste bidireccional)

- **Stack:** limpio. Crate `crates/shared` (plomería crosscutting, excepción ADR-0137: tipos técnicos nuevos del pilar — registro de cuenta verificada + track record atestado, catálogo ADR-0137 enmienda 2026-07-04).
- **Dos ámbitos de atestación por cuenta — regla obligatoria #1 (ADR-0145 FIJO):** cada cuenta modela **dos ámbitos coexistentes**: (i) **soberano** (`SOVEREIGN` — ejecución propia atestada por la cadena de hash del audit-log) y (ii) **read-only del bróker** (`BROKER_READONLY` — cuenta-completa reportada, computada localmente). La distinción atestado/no-atestado es **inviolable y visible**: NUNCA se presenta un dato `BROKER_READONLY` como `SOVEREIGN`. El ámbito es una columna/campo estructural del track, no un adorno. Prueba: un track soberano y uno read-only de la misma cuenta quedan etiquetados distinto y el soberano lleva firma.
- **gain% excluye el flujo de capital — regla obligatoria #2 (ADR-0145, EL diferenciador de cálculo):** el gain% se calcula sobre el crecimiento **excluyendo depósitos/retiros/transferencias** (consume los eventos `CapitalFlow` de #6). Prueba precisa con el ejemplo del ADR: un depósito y un retiro NO cuentan como ganancia. Sin esto el track está mal (confunde capital aportado con beneficio). Core puro, determinista.
- **Secretos jamás en el registro — regla obligatoria #3 (ADR-0093/0145):** NUNCA vive una credencial de bróker ni una investor password en `verified_accounts`/`attested_track_records`. La cuenta se referencia por un identificador **no secreto** (`broker_connection_ref`); los secretos siguen en `broker_connections` (cifrados, locales — fuera del alcance de esta Story). Guardarraíl estructural con test (como #8/#9): ninguna columna ni salida porta secretos.
- **Publicación opt-in por cuenta — regla obligatoria #4 (ADR-0145, `consent-registry` #5):** el default es **privado** (`PUBLICATION_DEFAULT = privado`, FIJO). Publicar exige consentimiento vigente por cuenta, consultado al `consent_out` **real** de #5 (`resolve_consent_verdict`, default-deny). Sin opt-in vigente → NUNCA se marca publicable. No un stub.
- **Firma reproducible del track soberano — regla obligatoria #5 (ADR-0020 subset V):** el track atestado lleva `signature_hash` reproducible (serialización canónica determinista + SHA-256, patrón de `institutional-report-engine` #7). Regenerar el mismo track sobre los mismos eventos → misma firma. `signature_hash` (contenido del track) ≠ `audit_hash` (fila).
- **Dos tablas — regla obligatoria #6 (ADR-0141):**
  - **`verified_accounts` MUTABLE** (estado de publicación/ámbitos cambian) → `row_version` (concurrencia optimista → `VersionConflict`). Grupo I + Perfil D. Campos propios: `broker` (TEXT), `leverage` (INTEGER), `currency` (TEXT), `account_type` (TEXT CHECK `FUNDED`/`PROP`/`OWN`), `publication_status` (TEXT CHECK `PRIVATE`/`PUBLIC`), `attestation_scopes` (TEXT json_valid — conjunto de `SOVEREIGN`/`BROKER_READONLY`), `broker_connection_ref` (TEXT — referencia NO secreta, nullable).
  - **`attested_track_records` APPEND-ONLY ATÓMICA** (cada track calculado es un snapshot inmutable firmado) → `event_sequence_id UNIQUE` + triggers + `BEGIN IMMEDIATE`+reintento+`WriteContention`. Grupo I + Perfil D + subset V. Campos propios: `verified_account_id` (referencia), `scope` (TEXT CHECK `SOVEREIGN`/`BROKER_READONLY`), `time_window` (TEXT), `signature_hash` (TEXT NOT NULL), y las métricas del track como enteros ×10⁸. **Prueba de 2 escritores obligatoria** (qa §2).
- **Enteros ×10⁸ — regla obligatoria #7 (ADR-0141):** todo monto (equity, balance, drawdown, gain%, PnL) como entero ×10⁸. Ninguna columna `REAL`. El gain% como fracción ×10⁸ (ej. 4.41 → 441000000). STRICT, UUIDv7.
- **Anti-`tenant_id` (ADR-0144):** multi-cuenta bajo `owner_id`/`institutional_tag`; PROHIBIDO calcar `tenant_id`. Las N cuentas son sub-entidades 1:N bajo la identidad de #1.
- **NO es #9:** el track publicable es **identificable por diseño** (el usuario quiere ser visto); NO pasa por la anonimización de `data-aggregation` (#9). Son canales distintos.
- **Perfil ADR-0020:** Perfil D — Grupo I + II (`owner_id`, `institutional_tag`) + IV (`node_id`) + subset V (`signature_hash` en el track). `verified_accounts` con `row_version`; `attested_track_records` con `event_sequence_id`.
- **Puertos (ADR-0137):** `event_in ← EnrichedDomainEvent` (Input 0..N), `consent_in ← ConsentVerdict` (Input 1..N), `registry_out → <registro de cuenta>` (Output 1..N), `track_record_out → <track atestado>` (Output 1..N). Bajo `public_interface::verified_account_registry`. Nombres de `struct` los fija el ingeniero.
- **Diferidos:** portal público (repo aparte), contrato de reporte al servidor central (adaptador de red), conexión read-only real al bróker (`broker-connector`). NO los cablees.

## 4. Instrucciones de despacho (prompt exacto)

> Eres el **Rust-Engineer** en **Modo Docente**. Lee primero: `CLAUDE.md`, `.claude/skills/base/SKILL.md`, `.claude/skills/rust-engineer/SKILL.md` (§4 atomicidad de ledgers + placeholders→DEBT), esta Orden, la feature `docs/features/verified-account-registry.md`, el ADR-0145 (`docs/adr/ADR-0145.md`), cómo se define `EnrichedDomainEvent` (#6) en `crates/shared/src/domain/enriched_domain_events.rs` (en particular las variantes `CapitalFlow`, `AccountSnapshot` y la orden reforzada con `account_id`/PnL), el patrón **firma reproducible** de `crates/shared/src/domain/institutional_report_engine.rs`, el patrón **append atómico** `crates/shared/src/persistence/enriched_domain_events.rs`, el patrón **`row_version` mutable** `crates/shared/src/persistence/central_identity.rs`, cómo `consent-registry` (#5) expone su veredicto real en `orchestrator::consent_registry::resolve_consent_verdict`, y los ADR-0145, ADR-0143, ADR-0093, ADR-0137, ADR-0141, ADR-0020, ADR-0002. Declara que los leíste.
>
> **Gate de Lectura Pre-Código:** (a) confirma `crates/shared`; (b) confirma las variantes reales de `EnrichedDomainEvent` de #6 que vas a consumir (`CapitalFlow` para excluir del gain%, `AccountSnapshot` para las curvas, orden reforzada para la estadística); (c) copia el patrón de firma reproducible (#7) y el de append atómico (#6) y el de `row_version` (#1); (d) confirma cómo consumir el veredicto de consentimiento real de #5; (e) confirma que NO cableas portal, contrato de red ni fetch read-only del bróker.
>
> **Construye (Core + esquema + puertos; adaptadores diferidos):**
> 1. **Migración `migrations/0016_verified_account_registry.sql`** con DOS tablas: (a) `verified_accounts` MUTABLE con `row_version` (Grupo I + Perfil D; `broker`, `leverage`, `currency`, `account_type CHECK(FUNDED,PROP,OWN)`, `publication_status CHECK(PRIVATE,PUBLIC)`, `attestation_scopes` json_valid, `broker_connection_ref` nullable NO secreto); (b) `attested_track_records` APPEND-ONLY (`event_sequence_id UNIQUE` + triggers anti UPDATE/DELETE; `verified_account_id`, `scope CHECK(SOVEREIGN,BROKER_READONLY)`, `time_window`, `signature_hash NOT NULL`, métricas enteros ×10⁸, `audit_chain_hash`). `STRICT`, UUIDv7.
> 2. **Core `domain/verified_account_registry.rs`:** enums `AccountType`, `PublicationStatus`, `AttestationScope`; cálculo puro del track record a partir de una lista de eventos de #6 — curvas de equity/balance (de `AccountSnapshot`), drawdown máximo, **gain% que EXCLUYE `CapitalFlow`** (depósitos/retiros no cuentan como ganancia), % de trades rentables, tiempo medio de espera, días de trading; `compute_track_record_signature` (reproducible, serialización canónica + SHA-256); `compute_track_record_audit_hash` encadenado; tipos de los puertos `registry_out`/`track_record_out`. Montos ×10⁸. Determinista.
> 3. **Shell:** `persistence/verified_account_registry.rs` — repo de cuentas con **`row_version`** (registrar, actualizar publicación/ámbitos con concurrencia optimista) + repo de track **append-only atómico** (`BEGIN IMMEDIATE`+reintento+`WriteContention`); `orchestrator/verified_account_registry.rs` — flujo: registrar cuenta (default PRIVATE) → agrupar eventos de #6 por cuenta → calcular track por ámbito → firmar el soberano → **gate de publicación con consentimiento real de #5** (sin opt-in vigente, no publica) → persistir. Reloj inyectado.
> 4. **`public_interface`:** submódulo `verified_account_registry` con los cuatro puertos. Sin secretos (ADR-0093): `broker_connection_ref` es no-secreto.
> 5. **CLI `verify`:** `cargo run -p app -- verify verified-account-registry --input '<json>'` que, dada una cuenta + eventos + consentimiento, reproduce el observable (track record por ámbito con gain% y estado de publicación) en JSON.
>
> **Pruebas discriminantes (rojo→verde):**
> - **gain% excluye flujo de capital:** con un depósito + un retiro + operaciones, el gain% refleja SOLO el beneficio, no el capital aportado. Debe fallar si el depósito se contara como ganancia (usa el ejemplo del ADR-0145).
> - **Ámbito inviolable:** un track `BROKER_READONLY` NUNCA se marca como `SOVEREIGN`; el soberano lleva firma, el read-only no se presenta como verificado. Assert estructural.
> - **Secretos nunca en el registro:** ninguna columna ni el struct de salida porta credencial/investor password; `broker_connection_ref` es no-secreto. Assert estructural (ADR-0093).
> - **Publicación opt-in real de #5:** default PRIVATE; sin consentimiento vigente por cuenta no se publica; con opt-in vigente sí. Usa el veredicto real, no un stub.
> - **Firma reproducible:** regenerar el track sobre los mismos eventos → mismo `signature_hash`; cambiar una métrica → cambia la firma.
> - **row_version:** actualizar publicación con `WHERE row_version=?`; dos actualizaciones concurrentes → una gana, la otra `VersionConflict`.
> - **Append atómico + concurrencia:** 16 escritores sobre `attested_track_records` (archivo temporal) → N filas, `event_sequence_id` 1..=N denso. Cae sin `BEGIN IMMEDIATE`.
> - **Append-only** (UPDATE/DELETE del track rechazados), `event_sequence_id` UNIQUE, `audit_chain_hash` encadenado, CHECKs de `account_type`/`publication_status`/`scope`, `json_valid` de `attestation_scopes`.
> - **Enteros ×10⁸:** métricas enteras; ninguna columna `REAL`.
> - Cobertura con `cargo llvm-cov --workspace --summary-only`.
>
> **Comentarios en español, identificadores en inglés (ADR-0121).** Gate: `cargo test --workspace` verde + `cargo clippy --workspace --all-targets -- -D warnings` cero warnings. Si modelas algún tipo de #6/#1 como placeholder, cruza a DEBT (regla del skill).
>
> **Docente:** `docs/lessons/rust/STORY-037-verified-account-registry.md` cero-conocimiento: qué es la atestación soberana y por qué la cadena de hash prueba "lo ejecutó Drasus" (vs. myFXbook que solo confía en el bróker), por qué el gain% debe excluir el flujo de capital (separar beneficio de capital aportado), la diferencia entre ámbito soberano y read-only y por qué NUNCA se confunden, y por qué la publicación es opt-in consultando el consentimiento real. Cita el código real.
>
> **NO construyas el portal, ni contrato de red al servidor central, ni el fetch read-only del bróker. NO toques migraciones existentes (solo crea 0016). NO hagas commits. NO toques archivos personales ni los 6 del Architect** (`docs/ROADMAP.md`, `docs/features/licensing-system.md`, `docs/features/usage-metering.md`, `docs/features/central-identity.md`, `docs/README.md`, `docs/moonshots/encrypted-local-backup.md`). Al terminar reporta: archivos, `cargo test --workspace` + `clippy` + `llvm-cov`, salida del `verify`, y tu decisión de crate.

## 5. Criterio de aceptación (verificable)

| # | Criterio | Prueba |
|---|---|---|
| 1 | Migración `STRICT`: `verified_accounts` (`row_version`) + `attested_track_records` append-only (triggers + UNIQUE) + Grupo I + Perfil D + subset V | inspección + tests |
| 2 | gain% EXCLUYE el flujo de capital (depósitos/retiros no son ganancia) | test discriminante con el ejemplo del ADR |
| 3 | Ámbito soberano vs read-only inviolable (nunca se confunden; soberano firmado) | assert estructural |
| 4 | Secretos nunca en el registro (`broker_connection_ref` no secreto) | assert estructural (ADR-0093) |
| 5 | Publicación opt-in con `consent_out` REAL de #5 (default PRIVATE) | test opt-in/sin-consentimiento |
| 6 | Firma del track reproducible (`signature_hash` ≠ `audit_hash`) | test de reproducibilidad |
| 7 | `row_version` (concurrencia optimista → VersionConflict) | test |
| 8 | Append atómico del track + 2 escritores | test de concurrencia (cae sin la tx) |
| 9 | `audit_chain_hash` encadenado; `event_sequence_id` UNIQUE; enteros ×10⁸ | tests |
| 10 | CLI `verify verified-account-registry` | `cargo run -p app -- verify verified-account-registry --input '…'` |
| 11 | Lección Docente | existe el archivo |
| 12 | Verde + cobertura | `cargo test --workspace` + `cargo llvm-cov` |

## 6. Comandos de validación (usuario)

```bash
cargo test -p shared
cargo clippy --workspace --all-targets -- -D warnings
cargo run -p app -- verify verified-account-registry --input '{"account":{"broker":"ICMarkets","currency":"USD","account_type":"OWN"},"consent":"COVERED","events":[{"type":"CapitalFlow","sign":"DEPOSIT","amount_e8":35000000000},{"type":"OrderExecuted","pnl_e8":15000000000}]}'
```

## 7. Registro de ejecución

- 2026-07-06 · Tech-Lead · Gate de Coherencia corrido (contraste bidireccional). Reglas obligatorias: (1) dos ámbitos de atestación por cuenta, distinción soberano/read-only inviolable; (2) gain% excluye el flujo de capital (diferenciador, consume `CapitalFlow` de #6); (3) secretos jamás en el registro (`broker_connection_ref` no secreto, ADR-0093); (4) publicación opt-in con consentimiento real de #5 (default privado); (5) firma reproducible del track soberano (subset V); (6) dos tablas (`verified_accounts` mutable `row_version` + `attested_track_records` append-only atómica); (7) enteros ×10⁸. Perfil D + subset V. Anti-`tenant_id`. Portal + contrato de red + fetch read-only del bróker diferidos. Orden creada, despacho a Rust-Engineer (Sonnet, **Docente**).
- 2026-07-06 · Rust-Engineer (Sonnet, Docente) · Entregado (1er despacho detenido por el usuario sin código; re-despacho completo). Migración `0016_verified_account_registry.sql` (dos tablas STRICT), Core `domain/verified_account_registry.rs` (`compute_track_record` gain%-sin-flujo-de-capital, `AttestationScope::is_sovereign_attestation`, `compute_track_record_signature` reproducible, hashes encadenados, `decide_publication`), persistencia `persistence/verified_account_registry.rs` (`row_version`/`VersionConflict` + append atómico `BEGIN IMMEDIATE`+reintento+`WriteContention` + prueba de 16 escritores en archivo), orquestador `orchestrator/verified_account_registry.rs` (`register_account`/`attest_track_record`/`request_publication` con `consent_out` real de #5), CLI `verify verified-account-registry`, lección `docs/lessons/rust/STORY-037-verified-account-registry.md`. Sin placeholders (consumió variantes reales de #6). 40 tests nuevos de la feature.
- 2026-07-06 · Tech-Lead · **Auditoría independiente APROBADA** (reproducción: clippy 0 warnings, 483 tests verdes; el `match` de `compute_track_record` separa estructuralmente `OrderExecuted`/PnL de `CapitalFlow` — imposible que un depósito infle el gain%; el flujo de capital solo participa como denominador/capital-base; ámbito inviolable vía `is_sovereign_attestation`; firma reproducible; guardarraíl de secretos con test; consentimiento real; migración STRICT+triggers+Perfil D+subset V).
- 2026-07-06 · QA (mutación, ejecutada por el TL vía `cargo-mutants`) · **APTO**. 118 mutantes: 84 cazados, 18 inviables, 16 sobrevivientes; mutación manual de `BEGIN IMMEDIATE`→`begin()` tumba la prueba de 16 escritores. Críticos cazados (gain%-sin-flujo vía contraprueba de suma ingenua, ámbito inviolable, consentimiento, firma, `row_version`, triggers). Sobrevivientes = huecos no bloqueantes (ruta de reintento; campos de struct de retorno tras update; bordes de `compute_track_record`) → **DEBT-013**.
- 2026-07-06 · Tech-Lead · **CIMIENTO #10 CERRADO — SUBSTRATO 10/10 COMPLETO.** Feature sellada 🟡 Parcial. Pendiente de autorización: commit agrupado.

## 8. Deudas / diferidos registrados

- **Portal público de Cuentas Verificadas (repo aparte, stack libre):** la vista publicada del track record vive fuera del monolito (ADR-0145); aquí solo se cimenta lo que le reportará.
- **Contrato/adaptador de reporte al servidor central:** la emisión del track publicado hacia la Cabina de Mando es adaptador de red diferido; aquí se modela el estado de publicación y el puerto, no el envío.
- **Conexión read-only real al bróker (`broker-connector`):** el fetch de la cuenta-completa corre en el Plano de Ejecución del usuario; aquí se modela el ámbito `BROKER_READONLY`, no el fetch (la investor password nunca sale, ADR-0093).
- **Superficie propia (Canal #1):** panel de cuentas verificadas → tanda de UI final (DEBT-005). Nota: esta feature tiene Superficie propia (no plomería), su SVF es parte de esa tanda.
- **Huecos de cobertura del QA → DEBT-013:** ruta de reintento no ejercitada (sistémico con DEBT-011/012); campos del struct de retorno tras `update` sin aserción; bordes de `compute_track_record` (filtro cross-cuenta del snapshot, `>`vs`>=` en drawdown/win-rate, signo del capital-base de respaldo). No bloqueante.
