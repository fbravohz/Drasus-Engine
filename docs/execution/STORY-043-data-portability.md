# STORY-043 — Data Portability (cimiento #13 del substrato de monetización)

> **Orden de trabajo del Tech-Lead** · Ingeniero: Rust-Engineer (Sonnet, Docente) · ADR-0148 (cimiento #13) · Depende de #1 `central-identity` (`owner_id`), enlaza #5 `consent-registry` · Reutiliza ADR-0020 (`owner_id` universal), ADR-0141 (append-only, pseudonimización sobre DELETE), ADR-0093 (secretos jamás salen).

## 1. Objetivo observable

Un puerto que, dado un `owner_id` autenticado, permite (a) **exportar** en formato legible los datos ligados a esa identidad (GDPR Art. 15/20) y (b) **solicitar el olvido** (Art. 17), con excepciones de retención legal. Es infraestructura de cumplimiento transversal (mismo nivel que #5), **no** dominio de trading.

Se cimenta AHORA: (1) un **catálogo declarativo** de qué tablas tienen `owner_id` (candidatas a export/olvido); (2) un **registro append-only** de solicitudes con su estado. El **generador de archivo real** (recorrer el esquema y volcar el dato) y la **UI** se **difieren** (adaptador; disparador: primera solicitud real o lanzamiento en jurisdicción GDPR).

## 2. Ubicación (ADR-0137, excepción crosscutting)

`crates/shared`, NO crate propio — infraestructura de cumplimiento transversal (produce tipos `textLabel` de plomería, consumida por ≥2 dominios, sin puerto de Alpha). Tres archivos + cableado:
- `crates/shared/src/domain/data_portability.rs` — Core puro.
- `crates/shared/src/persistence/data_portability.rs` — repositorios.
- `crates/shared/src/orchestrator/data_portability.rs` — composición.
- `migrations/0019_data_portability.sql` — esquema.
- Cableado en `domain/mod.rs`, `orchestrator/mod.rs`, `persistence/mod.rs`, `public_interface.rs`, `crates/app/src/main.rs`.

## 3. Esquema — `migrations/0019_data_portability.sql` (STRICT, UUIDv7)

### 3.1 `exportable_data_catalog` — catálogo declarativo (metadato de esquema)
Qué tablas portan `owner_id` y su clasificación de retención. Es metadato, no tabla de negocio (análogo a `foundation_master_fields` de `0001`).
- Grupo I (`id`, `created_at`, `updated_at`, `audit_hash`) + `table_name TEXT NOT NULL UNIQUE`, `feature_name TEXT NOT NULL`, `owner_id_column TEXT NOT NULL` (nombre de la columna `owner_id` en esa tabla), `retention_exempt INTEGER NOT NULL CHECK (retention_exempt IN (0,1))` (1 = obligación de retención legal → se pseudonimiza pero NO se purga el contenido).
- Concurrencia optimista `row_version` (el catálogo puede reclasificarse). `UNIQUE(table_name)` = auto-declaración idempotente.

### 3.2 `data_portability_requests` — APPEND-ONLY ATÓMICA (ADR-0141)
El registro auditable de solicitudes. Grupo I + Perfil D.
- Grupo I + `owner_id`, `institutional_tag` (Perfil D, Grupo II), `node_id`.
- `request_type TEXT NOT NULL CHECK (request_type IN ('EXPORT','FORGET'))`.
- `status TEXT NOT NULL CHECK (status IN ('RECEIVED','PROCESSING','COMPLETED'))` — el avance de estado lo emite el adaptador diferido como **nuevos eventos** (mismo `request_group_id`, nuevo `event_sequence_id`); el estado vigente = el del evento más reciente por solicitud.
- `request_group_id TEXT NOT NULL` (agrupa los eventos de UNA solicitud lógica), `disposition_detail TEXT` (JSON: qué tablas se pseudonimizaron vs. retuvieron por ley, para un FORGET — `json_valid` cuando no es NULL), `compliance_status_id TEXT` (subset V, estado de cumplimiento).
- `event_sequence_id INTEGER NOT NULL UNIQUE` + `audit_hash` + `audit_chain_hash` encadenado + **triggers** BEFORE UPDATE/DELETE que abortan. **Copiar EXACTAMENTE de `0016`/`0017`/`0018`.**

## 4. Core — `domain/data_portability.rs` (lógica pura, ADR-0002)

1. **`RequestType`** enum: `Export`/`Forget` (`as_str` → `EXPORT`/`FORGET`, `from_str_value`).
2. **`RequestStatus`** enum: `Received`/`Processing`/`Completed` (`as_str` → `RECEIVED`/`PROCESSING`/`COMPLETED`, `from_str_value`).
3. **`ForgetDisposition`** enum + **`decide_forget_disposition(retention_exempt: bool) -> ForgetDisposition`** — función pura: `retention_exempt == true` ⇒ `PseudonymizeAndRetain` (desvincula `owner_id`, **conserva** la fila/contenido por integridad del ledger); `false` ⇒ `PseudonymizeAndPurge` (desvincula `owner_id`, el contenido no-esencial puede purgarse). **En NINGÚN caso un DELETE físico de la fila** (ADR-0141) — modela ambos como transición de estado, sin variante de borrado.
4. **Filtro de exclusión de secretos** — `is_excluded_from_export(column_or_table: &str) -> bool` que EXCLUYE credenciales de bróker, claves de cifrado, IPs live (ADR-0093). Reutiliza el criterio/patrón de `instance_continuity::is_excluded_from_backup` de #11 (mismo espíritu; no dupliques la lista si puedes referenciarla, pero es aceptable un conjunto propio documentado).
5. **`build_export_manifest(owner_id, catalog_entries) -> ExportManifest`** — resolución **determinista** (ordenada) de qué tablas del catálogo aplican a un `owner_id` y qué columnas se exportarían, **excluyendo** las que filtra el punto 4. Es el manifiesto (estructura), no el dato real (que trae el adaptador diferido).
6. **`compute_request_audit_hash(...)`** — SHA-256 sobre mapa canónico ordenado (`request_group_id`, `request_type`, `status`, `owner_id`, `event_sequence_id`, `previous_audit_hash`, `disposition_detail`). Estilo `compute_override_audit_hash` de #12.

> **Prohibido** en el Core: I/O, `thread_rng`, `HashMap` sin ordenar en un hash, `f64`.

## 5. Persistencia — `persistence/data_portability.rs`

- **`ExportableDataCatalogRepository`** (MUTABLE): `declare_table` (idempotente por `table_name` — si ya existe, no duplica; mismo espíritu que `seed_default_catalog`/`activate`), `load_all`, `reclassify` con concurrencia optimista `row_version` → `VersionConflict`.
- **`DataPortabilityRequestRepository`** (APPEND-ONLY ATÓMICA): `record_event` = bucle de reintento acotado (MAX=5) → `try_record_once` con `pool.begin_with("BEGIN IMMEDIATE")` envolviendo load_tail(MAX `event_sequence_id`/`audit_hash`) + INSERT; error `WriteContention { attempts }`; `is_transient_write_conflict` reutiliza el helper canónico. **Copiar EXACTAMENTE de `AttestedTrackRecordRepository` de #10 / #12.** `latest_status_for(request_group_id)` deriva el estado vigente del evento más reciente.

## 6. Orquestador — `orchestrator/data_portability.rs` (composición, sin lógica)

- `declare_exportable_table(...)` — auto-declaración de una tabla en el catálogo.
- `seed_known_catalog(...)` — **stub**: siembra el catálogo con las tablas del substrato que ya portan `owner_id`, marcando `retention_exempt=1` las de retención legal (`audit_events`, `usage_records` de #4, `attested_track_records` de #10) y `0` el resto. Idempotente. (Es el equivalente a `seed_default_catalog` de #3 — demuestra el mecanismo sin recorrer el esquema real.)
- `request_export(identity, ...)` — registra una solicitud EXPORT append-only (estado `RECEIVED`) + arma el `ExportManifest` vía el Core.
- `request_forget(identity, catalog, ...)` — registra una solicitud FORGET append-only; por cada tabla del catálogo aplica `decide_forget_disposition` y arma el `disposition_detail` (qué se pseudonimiza-y-retiene vs. pseudonimiza-y-purga). El recorrido/pseudonimización REAL del dato queda diferido; aquí se registra la decisión auditable.

## 7. Reglas FIJAS (ADR-0148)
1. NUNCA se exportan secretos (credenciales de bróker, claves, IPs live) — ADR-0093, filtro del Core punto 4.
2. NUNCA el export incluye datos de terceros — solo el `owner_id` solicitante (el manifiesto filtra por `owner_id`).
3. NUNCA el olvido hace DELETE físico de una fila — siempre pseudonimización (ADR-0141), incluso para tablas sin retención.
4. Las tablas con retención legal (`retention_exempt=1`) se pseudonimizan pero CONSERVAN el registro.
5. Este cimiento NO es el backup de #11 (blob opaco ≠ export legible) ni cubre rectificación (Art. 16, vive en #1) — solo acceso/portabilidad/olvido (Art. 15/17/20).

## 8. Tests obligatorios (ADR-0133)
- **Concurrencia — 16 escritores** sobre `data_portability_requests` en **DB de archivo temporal** (NO `:memory:`): `event_sequence_id` únicos/contiguos, sin escrituras perdidas, `WriteContention` reintentado. (Copiar de #10/#12.)
- **`decide_forget_disposition`:** `retention_exempt=true` ⇒ `PseudonymizeAndRetain`; `false` ⇒ `PseudonymizeAndPurge`; NUNCA hay variante de DELETE.
- **Filtro de secretos:** `build_export_manifest` NUNCA incluye columnas/tablas de secretos (broker creds, keys, IPs) — test que fija que quedan fuera.
- **Catálogo idempotente:** `declare_table` dos veces con el mismo `table_name` no duplica; `reclassify` con `row_version` viejo ⇒ `VersionConflict`.
- **Estado vigente:** dos eventos del mismo `request_group_id` (RECEIVED→PROCESSING) ⇒ `latest_status_for` devuelve el más reciente.
- **JSON no filtra secretos** (ADR-0093): output del CLI sin credenciales/IPs — test de allowlist de claves (patrón `*_json_never_leaks_secret_fields`).
- **Hash determinista.** Property/proptest donde aplique.

## 9. Cableado del CLI (Canal #2, ADR-0142)
- `public_interface.rs`: submódulo `data_portability` (re-export domain/orchestrator/persistence) + `verify_data_portability(input) -> output`, forma de `verify_master_account_hierarchy`. Input JSON: `owner_id`/identidad, `request_type` (`EXPORT`|`FORGET`), y para FORGET opcionalmente el catálogo. Output: `request_group_id`, `status`, `audit_hash`, y para EXPORT el resumen del `ExportManifest` (tablas incluidas, sin dato real); para FORGET el `disposition_detail`. Sin secretos.
- `crates/app/src/main.rs`: rama `"data-portability"` en el `match` + mensaje si falta `--input` + añadir a la lista "Features soportadas en Fase 1" (doc del subcomando + mensaje de feature-id no reconocido).

**Comando de ejemplo (debe funcionar):**
```bash
cargo run -p app -- verify data-portability --input '{"owner_id":"user-42","institutional_tag":"LIVE","node_id":"node-A","request_type":"FORGET"}'
```

## 10. Lección Docente (ADR-0122)
Al cerrar, `docs/lessons/rust/STORY-043-data-portability.md`: por qué el catálogo es metadato declarativo auto-poblado (no una auditoría central recurrente), el olvido como pseudonimización sobre DELETE, y el reuso del patrón append-only atómico para el ledger de solicitudes.

## 11. Prohibiciones
- **NO** commitees nada. **NO** toques los 6 archivos protegidos del Architect ni otras features (#1–#12 no se tocan; solo se **consumen** `AccountIdentity`/`owner_id`).
- **NO** construyas el generador de export real (recorrido de esquema) ni la UI — diferidos.
- **NO** uses modelos/agentes Opus.

---

## §12. Registro de cierre (lo llena el Tech-Lead al auditar)
- **Ingeniero:** Rust-Engineer (Sonnet, Docente). Entregó Core (`domain/data_portability.rs`), persistencia (`persistence/data_portability.rs`), orquestador (`orchestrator/data_portability.rs`), cableado (`*/mod.rs`, `public_interface.rs`, `crates/app/src/main.rs`) y lección Docente. Consumió `migrations/0019_data_portability.sql` tal cual.
- **Auditoría TL independiente:** ✅ reproducida por el TL: `cargo test -p shared --lib` = **564 verdes** (33→36 en `data_portability` tras el endurecimiento de QA); `cargo clippy -p shared -p app --all-targets -- -D warnings` = 0 warnings; CLI `verify data-portability` en EXPORT y FORGET emite JSON válido y **`api_credentials` queda EXCLUIDA del manifiesto de EXPORT** (0 ocurrencias) pero presente como `PSEUDONYMIZE_AND_PURGE` en el FORGET (ADR-0093 correcto).
- **QA por mutación:** ✅ **APTO** — `cargo-mutants` sobre los 3 archivos: **73 mutantes → 53 caught + 1 timeout(caught) + 19 unviable, 0 missed (0 survivors)**. La primera corrida tenía **11 sobrevivientes**; el TL los mató con 3 tests deterministas: (1) `record_event_exhausts_exactly_max_attempts_when_write_lock_is_held` (contención sostenida vía segundo escritor con `busy_timeout=0` reteniendo `BEGIN IMMEDIATE` → agota `MAX_RECORD_ATTEMPTS`; mata los 4 del bucle de reintento + `is_transient→false` + `||→&&`); (2) `is_transient_is_false_for_a_permanent_non_sequence_unique_violation` (viola la PK `id`, no `event_sequence_id` → mata `is_transient→true` + `&&→||`); (3) `reclassify_returned_row_reflects_new_hash_chain_and_timestamp` (assertions sobre la fila DEVUELTA → mata los 3 de borrado de campo en la proyección mutable). El `TIMEOUT` de la línea 404 (`+= → *=`) es captura legítima: el mutante entra en bucle infinito bajo contención sostenida.
- **Hallazgo transversal (→ EPIC-0):** al medir, se confirmó que los mismos 7 mutantes-equivalentes (clasificador de contención + proyección mutable) **sobreviven idénticos en #10** (`verified-account-registry`, ya cerrado/commiteado) y por herencia del patrón calcado, previsiblemente en #5/#6/#9/#11/#12 y en los ledgers endurecidos por STORY-032. Registrado en **DEBT-018** para pago en la auditoría retroactiva EPIC-0, con el patrón de tests ya validado aquí. Enmienda de política: `cargo-mutants` formalizado como gate (ADR-0133 + skills).
- **Barrido de Cierre Documental:** ✅ feature `data-portability.md` sellada 🟡 Parcial (generador de export real + UI diferidos); `docs/TEST.md` con la entrada de #13; lección `docs/lessons/rust/STORY-043-data-portability.md`; DEBT-018 abierta; ADR-0133 enmendado (capa de mutación) y skills `tech-lead`/`rust-engineer`/`qa-engineer` actualizados.
- **Estado:** ✅ **CERRADA** (2026-07-08). Backend del cimiento #13 completo (catálogo + registro de solicitudes); SVF/galería y generador de export real diferidos.
