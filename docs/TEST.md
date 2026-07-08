# TEST.md — Índice de Comandos de Prueba por Feature

> **Propósito:** probar cada feature **sin leer código**. Un solo lugar con el comando exacto y el JSON de entrada por cada puerto/feature.
> **Mantenimiento:** cada Story que entregue un observable **añade aquí su bloque** (el Tech-Lead lo hace al cerrar). Si una feature no aparece, es que aún no tiene canal de prueba manual (solo tests automatizados).

## Canales de verificación

| Canal | Qué es | Estado |
|---|---|---|
| **#1 — Flutter / SVF** | Tab en el *Banco de Verificación* de la app que ejercita el backend **real por FFI**; se prueba con el ratón. | Parcial (solo `sovereign-data-fetcher`) |
| **#2 — CLI `verify`** (ADR-0142) | `cargo run -p app -- verify <feature> --input '<json>'`; imprime el observable en JSON. | Activo (12 cimientos del substrato + `sovereign-data-fetcher`) |
| **#3 — API de red / Postman** | Colección Postman/grpcurl contra el servidor gRPC público. | ⛔ **No implementado** (ver §Pendientes) |
| **Automatizado** | `cargo test` + `cargo llvm-cov`; la red de seguridad, no la prueba manual. | Activo |

---

## Comandos globales

```bash
# Toda la suite de Rust (crate shared = EPIC-0 + substrato)
cargo test -p shared
cargo test --workspace                      # incluye features/ y app/

# Lint estricto (gate obligatorio: cero warnings)
cargo clippy --workspace --all-targets -- -D warnings

# Cobertura por archivo
cargo llvm-cov --workspace --summary-only

# App / CLI
cargo run -p app -- version
cargo run -p app -- start                   # arranca + apagado limpio

# Flutter (desde ui/)
cd ui && flutter build linux                # gate obligatorio de QA Flutter
cd ui && flutter test
cd ui && flutter run -d linux               # app con el Banco de Verificación (Canal #1)
```

> ⚠️ **Prerequisito del bridge antes de `flutter run` (importante):** `flutter run` carga `target/release/libbridge.so`. Tras **cualquier** cambio a `crates/bridge` o a sus dependencias Rust (`shared`, `features/*`), o una regeneración de bindings, recompila la librería nativa **antes** de correr la app:
> ```bash
> cargo build --release -p bridge     # desde la raíz del workspace
> cd ui && flutter run -d linux
> ```
> Si te la saltas y el `.so` quedó viejo, la app **arranca pero no abre ventana** y el log muestra `Bad state: Content hash ... out-of-sync` (NO es Wayland ni se arregla reiniciando). Comparar `rustContentHash` en `ui/lib/src/rust/frb_generated.dart` vs `FLUTTER_RUST_BRIDGE_CODEGEN_CONTENT_HASH` en `crates/bridge/src/frb_generated.rs`: si las **fuentes** coinciden, el desajuste está en el `.so` compilado → recompila release.

**Convención Canal #2:** `cargo run -p app -- verify <feature-id> --input '<json>'`. La salida es **siempre** JSON válido con `"ok": true|false`; no requiere `jq`. Sin `--input` cada feature usa defaults de humo.

---

## Substrato de Monetización (ADR-0144)

### `central-identity` — cimiento #1  ✅ backend · ⏳ SVF/galería

**Puerto:** `identity_out` → `AccountIdentity`.

```bash
# Humo mínimo (usa hostname como huella)
cargo run -p app -- verify central-identity --input '{"email":"a@b.com"}'

# Normalización de correo (trim + lowercase)
cargo run -p app -- verify central-identity --input '{"email":"  Case@Example.COM  "}'
#   → "email": "case@example.com"

# Login federado + huella de hardware explícita
cargo run -p app -- verify central-identity --input '{"email":"a@b.com","oauth_provider":"Google","machine_identifiers":["cpu-123","mb-456"],"institutional_tag":"personal"}'
```

Campos del JSON: `email` (req) · `oauth_provider?` · `machine_identifiers?: string[]` · `institutional_tag?`.

```bash
cargo test -p shared central_identity      # tests unitarios/integración de la feature
```

- **Canal #1 (SVF):** ⏳ pendiente — panel de cuenta/sesión (por construir, ver plan de UI del substrato).
- **Canal #3 (Postman):** N/A — sin API de red aún.

### `licensing-system` — cimiento #2  ✅ backend · ⏳ SVF/galería

**Puerto:** `execution_gate_out` → `ExecutionGate {Allow/Deny/UpgradeRequired}` + orden de supresión de telemetría.

```bash
# Sovereign al corriente → Allow, suprime telemetría de trabajo
cargo run -p app -- verify licensing-system --input '{"tier":"SOVEREIGN"}'
#   → "verdict":"Allow", "suppress_work_telemetry": true

# Explorer (gratuito) → Allow, emite telemetría (firehose)
cargo run -p app -- verify licensing-system --input '{"tier":"EXPLORER"}'
#   → "suppress_work_telemetry": false

# Sin --input (defaults) y tier inválido (exit code 1)
cargo run -p app -- verify licensing-system
cargo run -p app -- verify licensing-system --input '{"tier":"BOGUS"}'
```

Campos del JSON: `tier?` (`SOVEREIGN`|`EXPLORER`, default `SOVEREIGN`) · `owner_email?` (cuenta a vincular vía `central-identity`).

```bash
cargo test -p shared licensing_system
```

- **Canal #1 (SVF):** ⏳ pendiente — panel de licencia/tier.
- **Canal #3 (Postman):** N/A.

### `plan-tier-quota` — cimiento #3  ✅ backend · ⏳ SVF/galería

**Puerto:** `plan_limits_out` → `PlanLimits` por tier. Cuotas: `notional_limit`, `max_activations`, y **`max_child_accounts`** (cuentas maestras hijas bajo #12, STORY-042 — FREE=`0`, PAID=`5` en el stub; fijado por el proveedor, no por el fondo). `0` es válido (tier sin cuentas hijas).

```bash
# PAID expone max_child_accounts:5; FREE expone 0
cargo run -p app -- verify plan-tier-quota --input '{"tier":"PAID"}'
cargo test -p shared plan_tier_quota
```

Campos del JSON: `tier?` (default `FREE`). Salida: `notional_limit`, `max_activations`, `max_child_accounts`, `features_enabled`.

- **Canal #1 (SVF):** ⏳ pendiente (DEBT-005). · **Canal #3:** N/A.

### `usage-metering` — cimiento #4  ✅ backend · ⏳ SVF/galería

**Puerto:** `usage_out` → libro de nocional. Montos **enteros ×10⁸** (ADR-0141, cero `f64`). Requiere `--input`.

```bash
# size y price en enteros ×10⁸ (250000000 = 2.5 unidades; 4000000000000 = 40000.0)
cargo run -p app -- verify usage-metering --input '{"tier":"FREE","operations":[{"size":250000000,"price":4000000000000}]}'
cargo test -p shared usage_metering
```

Campos: `tier?` · `operations: [{size, price}]` (req, enteros ×10⁸).

- **Canal #1 (SVF):** ⏳ pendiente (DEBT-005). · **Canal #3:** N/A.

### `consent-registry` — cimiento #5  ✅ backend · ⏳ SVF/galería

**Puerto:** `consent_out` → veredicto (default-deny GDPR: `NoConsent`/`StaleVersion`/`OptedOut`/`Covered`). Requiere `--input`.

```bash
cargo run -p app -- verify consent-registry --input '{"current_version":"v2","actions":[{"action":"ACCEPT","tos_version":"v2","optout_map":{"aggregation":false}}],"query":{"data_type":"aggregation"}}'
cargo test -p shared consent_registry
```

Campos: `current_version` · `actions: [{action, tos_version?, optout_map?}]` · `query: {data_type}`.

- **Canal #1 (SVF):** ⏳ pendiente (DEBT-005). · **Canal #3:** N/A.

### `enriched-domain-events` — cimiento #6  ✅ backend · ⏳ SVF/galería

**Puerto:** `event_out` → evento inmutable rico (8 variantes; consume `gate_in` de #2 para `replicate`). Montos enteros ×10⁸. Requiere `--input`.

```bash
cargo run -p app -- verify enriched-domain-events --input '{"tier":"FREE","event":{"type":"CapitalFlow","account_id":"acc-1","sign":"DEPOSIT","amount":100000000000,"currency":"USD"}}'
cargo test -p shared enriched_domain_events
```

Campos: `tier?` (deriva la supresión) · `event: {type, ...}` (variante ADR-0145: `CapitalFlow`/`AccountSnapshot`/`OrderExecuted`, montos ×10⁸).

- **Canal #1 (SVF):** ⏳ pendiente (DEBT-005). · **Canal #3:** N/A.

### `institutional-report-engine` — cimiento #7  ✅ backend · ⏳ SVF/galería

**Puerto:** `report_out` → reporte con **firma reproducible** (`signature_hash` ≠ `audit_hash`). `metrics` enteros ×10⁸.

```bash
cargo run -p app -- verify institutional-report-engine --input '{"report_type":"VALIDATION","metrics":{"sharpe_e8":150000000,"max_drawdown_e8":-8000000},"source_event_refs":["evt-1","evt-2"]}'
cargo test -p shared institutional_report_engine
```

Campos: `report_type` · `metrics: {clave: entero ×10⁸}` · `source_event_refs: string[]` (trazabilidad a #6).

- **Canal #1 (SVF):** ⏳ pendiente (DEBT-005). Render Tera→PDF/HTML diferido (DEBT-010). · **Canal #3:** N/A.

### `third-party-api-gateway` — cimiento #8  ✅ backend · ⏳ SVF/galería

**Puerto:** `api_request_in`/`api_response_out` → autentica (hash SHA-256, revocación gana), rate-limit de borde exacto, gate de `consent_out` **real** de #5. Credencial **nunca en claro** (ADR-0093).

```bash
# En el límite (99<100) → ALLOWED; en el borde (100) → RATE_LIMITED
cargo run -p app -- verify third-party-api-gateway --input '{"credential":"sk-demo-123","endpoint":"CERTIFY","rate_limit_per_window":100,"requests_in_window":99}'
cargo run -p app -- verify third-party-api-gateway --input '{"credential":"sk-demo-123","endpoint":"CERTIFY","rate_limit_per_window":100,"requests_in_window":100}'
cargo test -p shared third_party_api_gateway
```

Campos: `credential` (secreto en claro, solo se hashea) · `endpoint` · `rate_limit_per_window` · `requests_in_window`.

- **Canal #1 (SVF):** ⏳ pendiente (DEBT-005) — panel de administración de API. · **Canal #3 (servidor gRPC público):** ⛔ diferido al ROADMAP (tonic/mTLS/protos = Canal #3, ADR-0142); esta feature ES su futuro proveedor.

### `data-aggregation` — cimiento #9  ✅ backend · ⏳ SVF/galería

**Puerto:** `event_in`/`consent_in`/`aggregate_out` → índice anonimizado (ruido DP **determinista con semilla** + k-anonimato + hash unidireccional). Consume el `consent_out` **real** de #5. Datos crudos **nunca** salen (ADR-0093/0102). Requiere `--input`.

```bash
# min_cohort no alcanzado (1 evento < 5) → supresión; sube el nº de eventos para publicar
cargo run -p app -- verify data-aggregation --input '{"seed":42,"min_cohort":5,"external_sale_enabled":false,"events":[{"metric_e8":150000000,"consent":"COVERED"}]}'
cargo test -p shared data_aggregation
```

Campos: `seed` (semilla del RNG, determinismo) · `min_cohort` (k-anonimato) · `external_sale_enabled` · `events: [{metric_e8 (×10⁸), consent}]`.

- **Canal #1 (SVF):** ⏳ pendiente (DEBT-005) — panel de índices agregados + tamaño de cohorte. · **Canal #3:** vía #8 (diferido).

### `verified-account-registry` — cimiento #10 (pilar Cuentas Verificadas)  ✅ backend · ⏳ SVF/galería

**Puerto:** `event_in`/`consent_in`/`registry_out`/`track_record_out` → track record verificado por cuenta. **gain% EXCLUYE el flujo de capital** (diferenciador ADR-0145); **dos ejes ortogonales:** Eje A `SOVEREIGN` (atestado por Drasus) vs `BROKER_READONLY` (reportado por bróker) **inviolable** (`is_attested_by_drasus`), cruzado con Eje B realidad de capital `LIVE`/`PAPER`/`DEMO`/`CHALLENGE` (`is_real_capital`, `true` solo LIVE) — **el Eje B vive en `institutional_tag`** (Grupo II, ADR-0020, sin columna `capital_reality` — consolidado por STORY-041/DEBT-016); un track `SOVEREIGN`+`PAPER` es atestado pero de capital virtual, nunca se muestra sin ambas etiquetas; publicación opt-in con `consent_out` real de #5 (default PRIVATE); secretos nunca en el registro (ADR-0093). Requiere `--input`.

```bash
# Depósito NO cuenta como ganancia; track PRIVATE por default. Eje A SOVEREIGN (is_attested_by_drasus:true) × Eje B PAPER (is_real_capital:false) — atestado pero capital virtual. El Eje B se pasa como institutional_tag
cargo run -p app -- verify verified-account-registry --input '{"account":{"broker":"ICMarkets","currency":"USD","account_type":"OWN","institutional_tag":"PAPER"},"scope":"SOVEREIGN","consent":"COVERED","events":[{"type":"CapitalFlow","sign":"DEPOSIT","amount_e8":35000000000},{"type":"OrderExecuted","pnl_e8":15000000000}]}'
cargo test -p shared verified_account
```

Campos: `account: {broker, currency, account_type (FUNDED/PROP/OWN), institutional_tag (LIVE|PAPER|DEMO|CHALLENGE = Eje B realidad de capital, default LIVE)}` · `scope (SOVEREIGN|BROKER_READONLY = Eje A)` · `consent` · `events: [{type: CapitalFlow|OrderExecuted|AccountSnapshot, ...}]` (montos ×10⁸). La salida expone la etiqueta `capital_reality` + `is_real_capital` derivadas de `institutional_tag`.

- **Canal #1 (SVF):** ⏳ pendiente (DEBT-005) — panel de cuentas verificadas (**Superficie propia**, no plomería). · **Canal #3:** vía #8 (diferido). Portal público = repo aparte (ADR-0145).

---

### `instance-continuity` — cimiento #11 (respaldo cifrado + maestro itinerante)  ✅ backend · ⏳ SVF

**Puerto:** `identity_in`/`backup_blob_out`/`custody_status_out`. **Cripto client-side real** (ADR-0093): KDF `Argon2id` desde el secreto maestro + cifrado autenticado **AES-256-GCM** con **nonce sembrado inyectado** (determinista en test, aleatorio en prod); la clave y el secreto maestro **jamás** se persisten ni salen. `compute_backup_delta` **excluye** secretos de bróker/IPs live. **Gate de titularidad exclusiva** (`custody_epoch`→`CustodyConflict`): exactamente una máquina escritora de la cadena a la vez. Dos tablas: `instance_backups` append-only atómica + `custody_state` mutable. Requiere `--input`.

```bash
# Round-trip de cifrado (round_trip_ok:true) + reclamo de titularidad (CLAIMED/CustodyConflict); jamás emite la clave ni el secreto
cargo run -p app -- verify instance-continuity --input '{"master_secret":"correct horse battery staple","plaintext":"snapshot-bytes","nonce_seed":42,"custody":{"titular_node_id":"node-A","custody_epoch":3},"my_node_id":"node-A"}'
cargo test -p shared instance_continuity
```

Campos: `master_secret` (nunca se re-emite) · `plaintext` · `nonce_seed` (siembra determinista) · `custody: {titular_node_id, custody_epoch}` · `my_node_id`. Salida: `round_trip_ok`, `nonce_hex`, `blob_hash`, `custody_claim_outcome`, `is_titular_before_claim` — sin clave ni secreto.

- **Canal #1 (SVF):** ⏳ pendiente (DEBT-005) — toggle de respaldo + indicador de titularidad en el cajón de ajustes (**Superficie propia**). · Adaptador de almacén de objetos (S3/R2) y liberación forzada central diferidos.

---

### `master-account-hierarchy` — cimiento #12 (fondo raíz → cuentas maestras hijas)  ✅ backend · ⏳ SVF

**Puerto:** `identity_in`/`consent_in`/`override_command_out`/`override_attestation_out`. **Gate de consentimiento** (`decide_override_authorization`): un override del fondo sobre una hija se ejecuta **solo si** el `ConsentVerdict` **real** de #5 es `Covered` — sin opt-in vigente se **deniega** y ambos lados atestan el intento denegado. **Doble atestación**: `issue_override` (lado fondo, `ISSUER`) + `receive_override` (lado hija, `EXECUTOR`) encadenan cada uno en su propia cadena append-only (`override_attestations`, atómica) — **cada lado re-consulta el consentimiento por su cuenta**, nunca se pasan el veredicto (la hija conserva su Plano de Control). **"Eliminar" = archivar** (`LocalEffect` sin variante de borrado, ADR-0141). Jerarquía = `parent_owner_id` **nullable** cacheado (puntero anti-`tenant_id`, no árbol). El relé cifrado (ADR-0143) que transporta el comando fondo→hija es adaptador de red **diferido**. Requiere `--input`.

```bash
# Override ARCHIVE con consentimiento COVERED → outcome EXECUTED + local_effect ARCHIVED + ambas atestaciones (ISSUER/EXECUTOR); con NotCovered → DENIED en ambos lados
cargo run -p app -- verify master-account-hierarchy --input '{"parent_owner_id":"fund-X","child_owner_id":"trader-7","node_id":"node-A","consent":"COVERED","command_kind":"ARCHIVE","target_ref":"strategy-42","justification":"riesgo excedido"}'
cargo test -p shared master_account_hierarchy
```

Campos: `parent_owner_id` (fondo) · `child_owner_id` (hija) · `node_id` · `consent` (verdict de #5) · `command_kind` (`ARCHIVE`|`MODIFY`|`REQUEST_AUDIT_REPORT`) · `target_ref` · `justification`. Salida: `outcome` (`EXECUTED`|`DENIED`), `denial_reason`, `local_effect` (`ARCHIVED`|`NO_EFFECT`), `issuer_*`/`executor_audit_hash` + `event_sequence_id` — sin secretos (ADR-0093).

- **Canal #1 (SVF):** ⏳ pendiente (DEBT-005) — panel consolidado de cuentas hijas del fondo (auditoría + emisión de override) + indicador "reporta a `<Fondo>`" en la hija (**Superficie propia**). · Relé cifrado (adaptador de red) y venta a fondos diferidos.

---

## Ingesta de Datos (EPIC-1)

### `sovereign-data-fetcher`  ✅ backend · ✅ SVF

**Puerto:** descarga soberana de barras OHLCV.

```bash
cargo run -p app -- verify sovereign-data-fetcher --input '{"symbol":"BTCUSDT","interval":"1h"}'
cargo test -p sovereign-data-fetcher
```

- **Canal #1 (SVF):** ✅ `ui/lib/tabs/verification_bank/sovereign_data_fetcher_section.dart` — sección en el Banco de Verificación con datos reales por FFI (broker/símbolo/rango/timeframe + job + historial). Abrir con `cd ui && flutter run -d linux`.

---

## Plomería EPIC-0 (crate `shared`)

Estas features son infraestructura crosscutting; **no exponen subcomando `verify`** (se validan por tests). Filtro por módulo:

```bash
cargo test -p shared clock                      # reloj determinista + auditoría
cargo test -p shared audit_log                  # cadena de hash append-only
cargo test -p shared job                         # async-job-executor
cargo test -p shared worker                       # worker-isolation-orchestrator
cargo test -p shared telemetry                    # buffer + heartbeat
cargo test -p shared mcp                          # agentic-mcp-gateway

# Gate de recuperación tras crash (kill -9) — vive en crates/app/tests
cargo test -p app

# Servidor MCP (stdio) — arranque manual
cargo run -p app -- run-mcp-server
```

---

## Pendientes (canales aún no construidos)

- **Canal #3 — API de red / Postman:** **no implementado.** El **backend** del cimiento #8 `third-party-api-gateway` (auth por hash, rate-limit, gate de consentimiento) ya existe y se prueba por Canal #2 (CLI), pero **no existe `.proto` ni servidor gRPC/`tonic`** que lo exponga a la red — eso (más el **SaaS gateway** headless, ADR-0142) está diferido al ROADMAP. Cuando el servidor exista, este bloque documentará: arranque, autenticación mTLS, y la colección Postman/grpcurl **por puerto de feature**.
- **SVF de los cimientos del substrato (#1–#8):** hoy verifican por Canal #2 (CLI). Sus tabs SVF + componentes de galería con mocks son la **tanda de UI final** planificada (backend-first, DEBT-005): la fontanería primero, la UI de verificación después, antes de la UI productiva real.
