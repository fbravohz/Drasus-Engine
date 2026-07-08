# STORY-040 — Master Account Hierarchy (cimiento #12 del substrato de monetización)

> **Orden de trabajo del Tech-Lead** · Ingeniero: Rust-Engineer (Sonnet) · ADR-0147 (cimiento #12) · Depende de #1 `central-identity`, #5 `consent-registry`, #10 `verified-account-registry`, ADR-0119, ADR-0143, ADR-0141, ADR-0144.
>
> **Cierra el substrato de monetización (12/12).** Último cimiento antes de la auditoría retroactiva EPIC-0.

## 1. Objetivo observable

Una **cuenta maestra raíz** (fondo) agrupa **N cuentas maestras hijas**. La raíz tiene autoridad de **auditoría** y **override** (parar/archivar/modificar) sobre cada hija — pero:

- El mando **nunca** escribe directo en la DB de la hija: viaja como comando cifrado por el **relé genérico** (ADR-0143, adaptador de red **diferido**); la hija lo ejecuta **localmente**.
- Todo override exige **consentimiento vigente** de la hija (`consent-registry`, #5) — si no está `Covered`, se **rechaza** y ambos lados lo registran como intento denegado.
- Todo override queda **doblemente atestado**: el fondo encadena "emití esta orden"; la hija encadena "recibí esta orden firmada por mi padre y la ejecuté". Nunca una mutación silenciosa.
- "Eliminar" es **siempre archivar/desactivar**, jamás un DELETE físico (ADR-0141).

## 2. Ubicación (ADR-0137, excepción crosscutting)

Igual que los 11 cimientos previos: vive en **`crates/shared`**, NO como crate propio (produce tipos `textLabel` de plomería, consumido por ≥2 dominios, sin puerto de Alpha en el canvas). Tres archivos nuevos + cableado:

- `crates/shared/src/domain/master_account_hierarchy.rs` — Core puro.
- `crates/shared/src/persistence/master_account_hierarchy.rs` — repositorios.
- `crates/shared/src/orchestrator/master_account_hierarchy.rs` — composición.
- `migrations/0018_master_account_hierarchy.sql` — esquema.
- Cableado en `domain/mod.rs`, `orchestrator/mod.rs`, `persistence/mod.rs`, `public_interface.rs`, `crates/app/src/main.rs`.

## 3. Esquema — `migrations/0018_master_account_hierarchy.sql` (STRICT, UUIDv7)

Dos tablas, siguiendo EXACTAMENTE los patrones ya establecidos (ver `0016`/`0017`):

### 3.1 `account_hierarchy` — MUTABLE (ADR-0141, concurrencia optimista)

El registro que cachea el puntero al padre + referencia al consentimiento vigente. Es **el puntero, no el árbol** (anti-`tenant_id`, ADR-0144 FIJO).

- Campos Grupo I ADR-0020 (`id` UUIDv7, `created_at`, `updated_at`) + `owner_id` (la hija), `parent_owner_id` **nullable** (NULL = sin padre), `consent_ref` (referencia al consentimiento contractual vigente), `node_id`.
- `row_version INTEGER NOT NULL DEFAULT 1` — concurrencia optimista (UPDATE ... WHERE id=? AND row_version=? → `rows_affected()==0` ⇒ `VersionConflict`). Mismo patrón que `verified_accounts` de #10.
- `audit_hash` del estado.

### 3.2 `override_attestations` — APPEND-ONLY ATÓMICA (ADR-0141)

La doble atestación. Cada override produce **dos filas** encadenadas: una del lado ISSUER (fondo), una del lado EXECUTOR (hija). Una sola tabla con columna de rol.

- Grupo I + `owner_id`, `parent_owner_id`, `node_id`.
- `attestation_side TEXT NOT NULL CHECK (attestation_side IN ('ISSUER','EXECUTOR'))`.
- `command_kind TEXT NOT NULL CHECK (command_kind IN ('ARCHIVE','MODIFY','REQUEST_AUDIT_REPORT'))`.
- `target_ref TEXT NOT NULL` (qué estrategia/portafolio/parámetro), `outcome TEXT NOT NULL CHECK (outcome IN ('EXECUTED','DENIED'))`, `justification TEXT`.
- `event_sequence_id INTEGER NOT NULL UNIQUE` + `audit_hash` + `audit_chain_hash` encadenado (idéntico a `attested_track_records` de #10).
- **Triggers** `BEFORE UPDATE`/`BEFORE DELETE` que abortan (append-only inviolable) — copiar de `0016`/`0017`.

## 4. Core — `domain/master_account_hierarchy.rs` (lógica pura, ADR-0002)

1. **`OverrideCommandKind`** enum: `Archive` / `Modify` / `RequestAuditReport`. `as_str()` → `ARCHIVE`/`MODIFY`/`REQUEST_AUDIT_REPORT`, `from_str_value()`. Doc: catálogo de `OVERRIDE_COMMANDS` (ADR-0008).
2. **Gate de autorización** — función pura `decide_override_authorization(consent: &ConsentVerdict, ...) -> OverrideOutcome` donde `OverrideOutcome` = `Executed` | `Denied(reason)`. **Regla FIJA:** `Executed` **solo si** `consent.is_covered()` (reutiliza `ConsentVerdict` real de #5, NO reinventar). Consentimiento no vigente ⇒ `Denied`. Esta es la puerta del override.
3. **`compute_override_audit_hash(...)`** — hash SHA-256 **determinista** del evento (mapa canónico ordenado: `attestation_side`, `command_kind`, `target_ref`, `owner_id`, `parent_owner_id`, `outcome`, `event_sequence_id`, `previous_audit_hash`). Mismo estilo que `compute_track_record_audit_hash` de #10. Sin `f64`, sin `HashMap` sin ordenar.
4. **"Eliminar" = archivar** — la ejecución local de un `Archive` marca la fila objetivo como archivada/desactivada; **nunca** emite DELETE. Modela el efecto como una transición de estado, no como borrado.

> **Prohibido** en el Core: I/O, `thread_rng`, `HashMap` iterado sin ordenar en un hash, `f64` para montos.

## 5. Persistencia — `persistence/master_account_hierarchy.rs`

- **`AccountHierarchyRepository`** (MUTABLE): `link_child` / `load` / `update` con concurrencia optimista `row_version` → error tipado `VersionConflict`. Copiar la forma de `VerifiedAccountRepository` de #10.
- **`OverrideAttestationRepository`** (APPEND-ONLY ATÓMICA): `record_attestation` = **bucle de reintento acotado** (MAX=5) delegando a `try_record_once` que abre `pool.begin_with("BEGIN IMMEDIATE")` envolviendo load_tail(MAX event_sequence_id + audit_hash) + INSERT. Error `WriteContention { attempts }`; `is_transient_write_conflict` reutiliza el helper canónico (matches "database is locked" / UNIQUE en `event_sequence_id`). **Copiar EXACTAMENTE de `AttestedTrackRecordRepository` de #10** — es el patrón bendecido (DEBT-001).

## 6. Orquestador — `orchestrator/master_account_hierarchy.rs` (composición, sin lógica)

- `link_child_to_parent(...)` — registra la jerarquía + consentimiento contractual.
- `issue_override(...)` (lado fondo): consulta consentimiento → `decide_override_authorization` → **append fila ISSUER** (outcome según decisión) → produce el comando cifrado para el relé (el adaptador de red queda **diferido**: el orquestador devuelve el comando listo, no lo transmite).
- `receive_override(...)` (lado hija): re-valida consentimiento **localmente** → si `Executed`, ejecuta el efecto local (archivar/modificar) → **append fila EXECUTOR**; si `Denied`, append fila EXECUTOR con outcome `DENIED`. Ambos extremos siempre atestan.

## 7. Reglas FIJAS (ADR-0147 — las seis, inviolables)

1. Jerarquía central: la hija solo cachea `parent_owner_id` (puntero), nunca el árbol completo (anti-`tenant_id`).
2. Canal de mando elevado: comando cifrado por relé genérico (ADR-0143), la hija ejecuta local; jamás escritura directa remota.
3. Consentimiento contractual: override exige `ConsentVerdict::Covered` vigente (#5).
4. Doble atestación: ISSUER + EXECUTOR encadenados, siempre, incluso en `Denied`.
5. "Eliminar" = archivar (ADR-0141), nunca DELETE físico.
6. La hija conserva su Plano de Control (ADR-0119) — esta capa va **encima**, no reemplaza.

## 8. Tests obligatorios (ADR-0133)

- **Concurrencia — 16 escritores** sobre `override_attestations` en **DB de archivo temporal** (NO `:memory:`): todos los `event_sequence_id` únicos y contiguos, ninguna escritura perdida, `WriteContention` es el único error tolerado y se reintenta. (Copiar de #10/#11.)
- **Gate de consentimiento denegado:** `ConsentVerdict::NotCovered(...)` ⇒ `OverrideOutcome::Denied` y la fila EXECUTOR queda con outcome `DENIED` (intento denegado atestado en ambos lados).
- **Doble atestación:** un override `Executed` produce exactamente una fila ISSUER + una EXECUTOR, ambas con `audit_chain_hash` encadenado correcto.
- **Archivar-no-borrar:** un `Archive` deja la fila objetivo archivada y la fila original + su cadena intactas; ningún DELETE.
- **Concurrencia optimista de jerarquía:** dos updates con el mismo `row_version` ⇒ el segundo da `VersionConflict`.
- **JSON no filtra secretos** (ADR-0093): el output serializado del CLI no expone credenciales de bróker, IPs live ni secreto alguno — test que fija la lista exacta de claves permitidas (copiar el patrón `*_json_never_leaks_secret_fields` de #1/#11).
- **Hash determinista:** mismo evento ⇒ mismo `audit_hash` en corridas repetidas.
- Property/proptest donde aplique (orden de eventos, idempotencia del gate).

## 9. Cableado del CLI (Canal #2, ADR-0142)

- `public_interface.rs`: submódulo `master_account_hierarchy` (re-export domain/orchestrator/persistence) + `verify_master_account_hierarchy(input) -> output`, siguiendo la forma de `verify_instance_continuity`. Input JSON: `parent_owner_id`, `child_owner_id`, `node_id`, `consent` (verdict), `command_kind`, `target_ref`, `justification`. Output: decisión de autorización + hashes de ambas atestaciones (ISSUER/EXECUTOR) + `outcome`.
- `crates/app/src/main.rs`: rama `"master-account-hierarchy"` en el `match`, con mensaje de error si falta `--input`, y añadir la feature a la lista de "Features soportadas en Fase 1" (doc del subcomando + mensaje de feature-id no reconocido).

**Comando de ejemplo (debe funcionar tras la implementación):**

```bash
cargo run -p app -- verify master-account-hierarchy --input '{"parent_owner_id":"fund-X","child_owner_id":"trader-7","node_id":"node-A","consent":"COVERED","command_kind":"ARCHIVE","target_ref":"strategy-42","justification":"riesgo excedido"}'
```

## 10. Lección Docente (ADR-0122)

Al cerrar, escribe `docs/lessons/rust/STORY-040-master-account-hierarchy.md` con lo no obvio: reuso del patrón append-only atómico de #10 para doble atestación, el gate de consentimiento como función pura, y por qué la jerarquía es un puntero cacheado y no un árbol local.

## 11. Prohibiciones

- **NO** commitees nada (el usuario autoriza cada commit; queda todo en disco).
- **NO** toques los 6 archivos protegidos del Architect (`docs/ROADMAP.md`, `docs/features/{licensing-system,usage-metering,central-identity}.md`, `docs/README.md`, `docs/moonshots/encrypted-local-backup.md`).
- **NO** modifiques `AccountIdentity` de #1 ni los tipos de #5/#10 — solo los **consumes**.
- **NO** construyas el adaptador de red del relé ni la UI (diferidos, disparador "venta a fondos").

---

## §12. Registro de cierre (lo llena el Tech-Lead al auditar)

- **Ingeniero:** 2026-07-07 · Rust-Engineer (Sonnet, Docente) · Entregado. Migración `0018_master_account_hierarchy.sql` (dos tablas STRICT/UUIDv7: `account_hierarchy` MUTABLE `row_version`/`VersionConflict` con `parent_owner_id` nullable — puntero, no árbol; `override_attestations` APPEND-ONLY atómica `event_sequence_id UNIQUE` + triggers BEFORE UPDATE/DELETE), Core `domain/master_account_hierarchy.rs` (`OverrideCommandKind`, `AttestationSide`, gate puro `decide_override_authorization(consent: &ConsentVerdict)` que consume el `ConsentVerdict` **real** de #5, `apply_local_command_effect` con `LocalEffect` enum cerrado sin borrado — "eliminar = archivar", hashes estilo buffer-separador de #10), Shell `persistence/master_account_hierarchy.rs` (repo mutable `VersionConflict` + repo append-only atómico `BEGIN IMMEDIATE`+reintento MAX=5+`WriteContention` + prueba de 16 escritores en archivo), orquestador (`issue_override` lado fondo / `receive_override` lado hija — **cada lado re-consulta consentimiento vía `resolve_consent_verdict`**, nunca se pasan el veredicto / `execute_override` end-to-end), `public_interface::master_account_hierarchy` + `verify master-account-hierarchy`, CLI en `main.rs`, lección Docente. 28 tests de la feature. Sin placeholders.
- **Auditoría TL independiente:** 2026-07-07 · **APROBADA**. Reproducción propia: `cargo test -p shared master_account_hierarchy` 28 verdes + `cargo clippy -p shared --all-targets -D warnings` limpio. Verificado en el código real: gate `decide_override_authorization` puro que consume `ConsentVerdict` real de #5 (import, no redefinición) — `Executed` solo si `Covered`; doble atestación ISSUER (fondo) + EXECUTOR (hija) con re-consulta de consentimiento por lado (coherente con la autonomía de la hija, regla fija #6); append-only atómico `BEGIN IMMEDIATE`+`WriteContention` calcado de #10; "archivar-no-borrar" estructural (`LocalEffect` enum cerrado de 2 valores, sin variante de borrado físico).
- **QA por mutación (`cargo-mutants` sobre el Core, ejecutada por el TL):** 2026-07-07 · **APTO**. 29 mutantes: 23 cazados, 6 inviables, **0 sobrevivientes** — cobertura total del Core (gate, efecto local, hashes). **Sin deuda de mutación nueva.**
- **Barrido de Cierre Documental:** 2026-07-07 · Completo (feature sellada, PROGRESS 12/12, memoria, TEST.md).
- **Estado:** ✅ **CIMIENTO #12 CERRADO — SUBSTRATO 12/12 COMPLETO.** Pendiente de autorización: commit agrupado. Siguiente: auditoría retroactiva EPIC-0.
