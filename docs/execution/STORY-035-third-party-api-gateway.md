# STORY-035 — Third-Party API Gateway (cimiento #8 del substrato)

| Campo | Valor |
|---|---|
| **ID** | STORY-035 |
| **Tipo** | Story (código — octavo cimiento del substrato de pricing/SaaS) |
| **Épica (Fase)** | Cimientos de Monetización (transversal, greenfield) |
| **Sprint** | substrato-monetizacion |
| **Estado** | 🟡 En curso (Core auth + rate-limit + esquema + puertos; servidor gRPC/tonic, mTLS y protos diferidos) |
| **Creada** | 2026-07-05 |
| **Feature** | [`third-party-api-gateway`](../features/third-party-api-gateway.md) |
| **ADRs** | ADR-0144 (cimiento #8) · ADR-0142 (gRPC/CLI) · ADR-0093 (seguridad — credencial nunca en claro) · ADR-0137 (puertos) · ADR-0141 (append-only + row_version) · ADR-0020 (Perfil D) |

## 1. Objetivo llano

Construir la **puerta de entrada autenticada para terceros**: el Core que valida una solicitud externa (autenticación por credencial de API + endpoint habilitado + no revocada) y computa la **ventana de rate-limit**, el esquema (credenciales de API + registro de uso), y los puertos. El gateway **no computa**: delega en los puertos internos respetando el consentimiento (`consent-registry` #5). Convierte cada capacidad interna (certificación, feeds, ruteo) en un producto vendible por API sin reabrir el core.

**Alcance ahora vs. después (ADR-0144 "contratos + auth ahora, servidor después"):**
- **Ahora (esta Story):** Core (validación de solicitud + ventana de rate-limit + gate de consentimiento) + esquema (`api_credentials` mutable + `api_usage_records` append-only) + consumo real de `consent_out` de #5 + puertos `api_request_in`/`api_response_out` + CLI verify.
- **Después (diferidos):** el **servidor gRPC (tonic, ADR-0142)** — no existe en el workspace; el **mTLS** (transporte); los **protos por dominio** (`.proto`); la **delegación real** a todos los puertos internos (certificación #7, feeds #9, ruteo `execute`).

## 2. Agentes y Modo de Acompañamiento (ADR-0120/0122)

| Agente | Modelo | Modo |
|---|---|---|
| Rust-Engineer | Sonnet | **Docente** |

Docente: conceptos nuevos — autenticación por hash de credencial (nunca en claro) y ventana de rate-limit. Lección en `docs/lessons/rust/STORY-035-third-party-api-gateway.md`.

## 3. Gate de Coherencia — resultado (contraste bidireccional)

- **Stack:** limpio. Crate `crates/shared` (plomería crosscutting, excepción ADR-0137: `ThirdPartyRequest`/`ThirdPartyResponse` son tipos técnicos del catálogo).
- **Credencial NUNCA en claro — regla obligatoria #1 (ADR-0093):** `api_credentials` guarda un **hash** de la credencial de API (SHA-256), jamás el secreto en texto plano. Autenticar = hashear la credencial presentada y comparar contra el hash almacenado (patrón de `central-identity`/`licensing-system` con secretos). El puerto y el registro de uso guardan una **referencia** (id de credencial), nunca el secreto.
- **Dos tablas — regla obligatoria #2:**
  - **`api_credentials` MUTABLE** (se revoca/rota) → lleva **`row_version`** (concurrencia optimista, patrón de `central-identity`/`plan-tier-quota`: `UPDATE ... WHERE id=? AND row_version=?` + chequeo `rows_affected()==0` → `VersionConflict`). Grupo I + Perfil D (`owner_id`, `access_token_id`, `node_id`). Campos propios: `credential_hash` (TEXT), `status` (TEXT CHECK `ACTIVE`/`REVOKED`), `rate_limit_per_window` (INTEGER), `window_seconds` (INTEGER), `endpoints_enabled` (TEXT JSON `json_valid`).
  - **`api_usage_records` APPEND-ONLY ATÓMICA** (regla DEBT-001) → `event_sequence_id UNIQUE` + triggers + `BEGIN IMMEDIATE`+reintento+`WriteContention`. Grupo I + Perfil D. Campos propios: `credential_id` (referencia), `endpoint` (TEXT), `outcome` (TEXT CHECK `ALLOWED`/`RATE_LIMITED`/`DENIED`). **Prueba de 2 escritores obligatoria** (qa §2).
- **Ventana de rate-limit — regla obligatoria #3 (Core puro):** `compute_rate_limit(requests_in_window, limit) -> Allow | RateLimited` — cuenta las solicitudes de una credencial en la ventana vigente (leídas de `api_usage_records`) y compara contra `rate_limit_per_window`. Determinista, reloj inyectado para la ventana. Prueba de borde exacto (en el límite → permitido; +1 → rechazado).
- **Gate de consentimiento (ADR + feature §Restricciones) — regla obligatoria #4:** antes de delegar datos, el gateway consulta `consent_out` de `consent-registry` (#5) **real** (vía `public_interface::consent_registry`/`resolve_consent_verdict`). Si el consentimiento no cubre el tipo de dato → `DENIED`, sin delegar. NUNCA expone datos crudos que violen consentimiento.
- **El gateway NO computa (feature §Restricciones):** valida + rate-limita + verifica consentimiento + registra uso, y **decide delegar**; la delegación real a los puertos internos (#7 report, #9 feeds, `execute`) es futura. Modela la **decisión de delegación** (a qué puerto interno iría) + un `ThirdPartyResponse` resultante; no cablees todos los puertos internos.
- **Perfil ADR-0020:** Perfil D — Grupo I + II (`owner_id`, `access_token_id`) + IV (`node_id`). `api_credentials` con `row_version` (mutable); `api_usage_records` con `event_sequence_id` (append-only).
- **Puertos (ADR-0137):** `api_request_in ← ThirdPartyRequest` (Input 0..N), `api_response_out → ThirdPartyResponse` (Output 0..N). Bajo `public_interface::third_party_api_gateway`.
- **Servidor gRPC/tonic + mTLS + protos diferidos:** no existe tonic en el workspace; NO lo agregues. El Core y el esquema son el contrato; el servidor es adaptador posterior.

## 4. Instrucciones de despacho (prompt exacto)

> Eres el **Rust-Engineer** en **Modo Docente**. Lee primero: `CLAUDE.md`, `.claude/skills/base/SKILL.md`, `.claude/skills/rust-engineer/SKILL.md` (§4 atomicidad de ledgers), esta Orden, la feature `docs/features/third-party-api-gateway.md`, el patrón **append atómico** `crates/shared/src/persistence/enriched_domain_events.rs`, el patrón **`row_version` mutable** `crates/shared/src/persistence/central_identity.rs`, cómo `consent-registry` (#5) expone su veredicto en `public_interface`/`orchestrator::consent_registry`, y los ADR-0144, ADR-0142, ADR-0093, ADR-0137, ADR-0141, ADR-0020. Declara que los leíste.
>
> **Gate de Lectura Pre-Código:** (a) confirma `crates/shared`; (b) copia el patrón append atómico (usage) y el patrón `row_version` (credentials); (c) confirma cómo consumir el veredicto de consentimiento real de #5; (d) confirma que NO agregas tonic ni protos.
>
> **Construye (Core auth + rate-limit + esquema + puertos; servidor gRPC diferido):**
> 1. **Migración `migrations/0014_api_gateway.sql`** con DOS tablas: (a) `api_credentials` MUTABLE con `row_version` (Grupo I + Perfil D; `credential_hash`, `status CHECK(ACTIVE,REVOKED)`, `rate_limit_per_window`, `window_seconds`, `endpoints_enabled` json_valid); (b) `api_usage_records` APPEND-ONLY (`event_sequence_id UNIQUE` + triggers anti UPDATE/DELETE; `credential_id`, `endpoint`, `outcome CHECK(ALLOWED,RATE_LIMITED,DENIED)`, `audit_chain_hash`). `STRICT`, UUIDv7. Índices apropiados (incl. `(credential_id, created_at)` para la ventana de rate-limit).
> 2. **Core `domain/third_party_api_gateway.rs`:** `hash_api_credential`; `authenticate(presented, stored_hash, status) -> AuthVerdict` (revocada → denegado); `compute_rate_limit(requests_in_window, limit) -> Allow|RateLimited`; `is_endpoint_enabled`; tipos `ThirdPartyRequest`/`ThirdPartyResponse`/`GatewayOutcome`. Determinista.
> 3. **Shell:** `persistence/third_party_api_gateway.rs` — repo de credenciales con **`row_version`** (crear, revocar con concurrencia optimista, cargar por hash) + repo de uso **append-only atómico** (`BEGIN IMMEDIATE`+reintento+`WriteContention`); `orchestrator/third_party_api_gateway.rs` — flujo del gateway: autenticar → rate-limit (contar uso en ventana) → **consultar consentimiento real de #5** → decidir delegación → registrar uso con `outcome`. Reloj inyectado.
> 4. **`public_interface`:** submódulo `third_party_api_gateway` con `api_request_in`/`api_response_out`. Sin secretos en claro (ADR-0093).
> 5. **CLI `verify`:** `cargo run -p app -- verify third-party-api-gateway --input '<json>'` que, dada una credencial + solicitud + historial, reproduce el observable (`outcome`: ALLOWED/RATE_LIMITED/DENIED) en JSON.
>
> **Pruebas discriminantes (rojo→verde):**
> - **Credencial nunca en claro:** la tabla guarda hash; autenticar con la credencial correcta → OK, incorrecta → denegado; el `ThirdPartyResponse` y el registro NO contienen el secreto (assert ADR-0093).
> - **Rate-limit de borde exacto:** en el límite → ALLOWED; +1 en la ventana → RATE_LIMITED. Debe fallar si el umbral se ignora.
> - **Revocación (row_version):** revocar una credencial (UPDATE con `WHERE row_version=?`) → siguiente auth denegada; dos revocaciones concurrentes → una gana, la otra `VersionConflict`.
> - **Append atómico + concurrencia:** 16 escritores sobre `api_usage_records` (archivo temporal) → N filas, `event_sequence_id` 1..=N denso. Cae sin `BEGIN IMMEDIATE`.
> - **Gate de consentimiento real de #5:** consentimiento que no cubre → DENIED sin delegar; que cubre → delega. Usa el veredicto real, no un stub.
> - **Append-only** (UPDATE/DELETE de usage rechazados), `event_sequence_id` UNIQUE, `audit_chain_hash` encadenado, `json_valid`.
> - Cobertura con `cargo llvm-cov --workspace --summary-only`.
>
> **Comentarios en español, identificadores en inglés (ADR-0121).** Gate: `cargo test --workspace` verde + `cargo clippy --workspace --all-targets -- -D warnings` cero warnings.
>
> **Docente:** `docs/lessons/rust/STORY-035-third-party-api-gateway.md` cero-conocimiento: por qué una credencial se guarda hasheada y nunca en claro, qué es una ventana de rate-limit y cómo se computa determinísticamente, la diferencia entre tabla mutable (`row_version`) y append-only (`event_sequence_id`) — por qué credenciales es mutable y uso es append-only, y por qué el gateway consulta consentimiento antes de delegar. Cita el código real.
>
> **NO agregues tonic ni protos. NO toques migraciones existentes (solo crea 0014). NO hagas commits. NO toques archivos personales ni los 6 del Architect** (`docs/ROADMAP.md`, `docs/features/licensing-system.md`, `docs/features/usage-metering.md`, `docs/features/central-identity.md`, `docs/README.md`, `docs/moonshots/encrypted-local-backup.md`). Al terminar reporta: archivos, `cargo test --workspace` + `clippy` + `llvm-cov`, salida del `verify`, y tu decisión de crate.

## 5. Criterio de aceptación (verificable)

| # | Criterio | Prueba |
|---|---|---|
| 1 | Migración `STRICT`: `api_credentials` con `row_version` + `api_usage_records` append-only (triggers + UNIQUE) | inspección + tests |
| 2 | Credencial hasheada, nunca en claro (ADR-0093) | test auth + assert sin secreto |
| 3 | Rate-limit de borde exacto (límite → ALLOWED, +1 → RATE_LIMITED) | test discriminante |
| 4 | Revocación con `row_version` (concurrencia optimista → VersionConflict) | test |
| 5 | Append atómico de uso + 2 escritores | test de concurrencia (cae sin la tx) |
| 6 | Gate de consentimiento con `consent_out` REAL de #5 | test cubre/no-cubre |
| 7 | `audit_chain_hash` encadenado; `event_sequence_id` UNIQUE | tests |
| 8 | CLI `verify third-party-api-gateway` | `cargo run -p app -- verify third-party-api-gateway --input '…'` |
| 9 | Lección Docente | existe el archivo |
| 10 | Verde + cobertura | `cargo test --workspace` + `cargo llvm-cov` |

## 6. Comandos de validación (usuario)

```bash
cargo test -p shared
cargo clippy --workspace --all-targets -- -D warnings
cargo run -p app -- verify third-party-api-gateway --input '{"credential":"sk-demo-123","endpoint":"CERTIFY","rate_limit_per_window":100,"requests_in_window":100}'
```

## 7. Registro de ejecución

- 2026-07-05 · Tech-Lead · Gate corrido. Reglas: credencial **hasheada nunca en claro** (ADR-0093); dos tablas (`api_credentials` mutable con `row_version` + `api_usage_records` append-only atómica); ventana de rate-limit determinista; gate de consentimiento con `consent_out` REAL de #5; el gateway no computa (decide delegar); servidor gRPC/tonic + mTLS + protos diferidos (no existen en el workspace). Orden creada, despacho a Rust-Engineer (Sonnet, **Docente**).
- 2026-07-06 · Rust-Engineer (Sonnet, Docente) · Entregado. Migración `0014_api_gateway.sql` (dos tablas STRICT), Core `domain/third_party_api_gateway.rs`, persistencia `persistence/third_party_api_gateway.rs` (append atómico `BEGIN IMMEDIATE`+reintento+`WriteContention` + `row_version`/`VersionConflict`), orquestador `orchestrator/third_party_api_gateway.rs` (`handle_gateway_request` con `consent_out` real de #5), CLI `verify third-party-api-gateway`, lección `docs/lessons/rust/STORY-035-third-party-api-gateway.md`. 43 tests nuevos.
- 2026-07-06 · Tech-Lead · **Auditoría independiente APROBADA** (reproducción: clippy 0 warnings, 412 tests verdes; patrón atómico fiel a #6/#4, prueba de 16 escritores en archivo temporal, triggers/CHECK/UNIQUE probados por comportamiento; migración limpia, ningún archivo fuera de alcance ni de los 6 del Architect). Decisión aceptada: credencial desconocida → `Denied` sin persistir uso (no hay `credential_id` que atribuir; probada).
- 2026-07-06 · QA-Engineer (Sonnet, adversarial) · **APTO**. Mutación con `cargo-mutants` (71 mutantes: 45 cazados, 16 inviables, 10 sobrevivientes) — todos los mutantes críticos de seguridad cazados (revocación-gana, borde `<` de rate-limit, cuatro puertas). Mutación manual: quitar `BEGIN IMMEDIATE` tumba la prueba de 16 escritores 3/3. Huecos no bloqueantes (ruta de reintento no ejercitada por WAL+busy_timeout; `audit_hash` de `revoke` sin aserción) → **DEBT-011**.
- 2026-07-06 · Tech-Lead · **CIMIENTO #8 CERRADO.** Feature sellada 🟡 Parcial; substrato **8/10**. Pendiente de autorización: commit agrupado.

## 8. Deudas / diferidos registrados

- **Servidor gRPC (tonic, ADR-0142) + mTLS + protos por dominio:** no existen en el workspace; el Core + esquema son el contrato; el servidor es adaptador posterior. Es el **Canal #3 (Postman/grpcurl)** de `TEST.md`, aún no construido.
- **Delegación real a los puertos internos:** certificación (#7), feeds (#9), ruteo (`execute`/EPIC-5); ahora se modela la decisión de delegación.
- **Ventana de Verificación (Canal #1):** panel de administración de API → tanda de UI final (DEBT-005).
- **Huecos de cobertura del QA → DEBT-011:** la ruta de reintento del append (`is_transient_write_conflict`/`WriteContention`) no la ejercita ninguna prueba (WAL+`busy_timeout` hace esperar, no fallar); `revoke` no asevera su `audit_hash` recalculado. No bloqueante (la propiedad crítica de no-pérdida sí está probada).
