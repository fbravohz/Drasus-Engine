# STORY-038 — Verified Account Registry · Eje B (realidad de capital) — retrabajo de #10

| Campo | Valor |
|---|---|
| **ID** | STORY-038 |
| **Tipo** | Story (código — **retrabajo** de STORY-037, cimiento #10; paga DEBT-014) |
| **Épica (Fase)** | Cimientos de Monetización (transversal, greenfield) |
| **Sprint** | substrato-monetizacion |
| **Estado** | 🟡 En curso (retrabajo del código sellado de #10: añadir el Eje B ortogonal al Eje A ya existente) |
| **Creada** | 2026-07-06 |
| **Feature** | [`verified-account-registry`](../features/verified-account-registry.md) |
| **Paga** | [DEBT-014](../DEBT.md) |
| **ADRs** | ADR-0145 (pilar #10 — rector, **corregido 2026-07-06**: dos ejes ortogonales) · ADR-0143 (tres planos + telemetría clase 5) · ADR-0093 (secretos nunca salen) · ADR-0137 (puertos) · ADR-0141 (append-only + row_version + ×10⁸) · ADR-0020 (Perfil D + subset V) · ADR-0002 (FCIS) |

## 1. Objetivo llano

STORY-037 modeló **un solo eje** de atestación (`SOVEREIGN`/`BROKER_READONLY` — quién ejecutó). La corrección del Architect a ADR-0145 (2026-07-06) señaló que faltaba un **segundo eje ortogonal**: la **realidad del capital** (`LIVE`/`PAPER`/`DEMO`/`CHALLENGE` — qué se arriesgó). Una cuenta en `PAPER`/`DEMO`/`CHALLENGE` corre en el **mismo entorno determinista de ejecución** que `LIVE` (NO es backtesting) — por lo tanto **sí es atestiguable** por Drasus, solo que con capital virtual. "Atestable" (Eje A) y "capital real" (Eje B) son ejes **distintos**.

Este retrabajo **añade** el Eje B al código ya sellado de #10, sin tocar el Eje A (que es correcto). El punto de correctitud nuevo: **un track `SOVEREIGN`+`PAPER` se atestigua con firma (Eje A = atestado) pero jamás se presenta sin la etiqueta de capital virtual (Eje B = PAPER)**. Omitir el Eje B es, según ADR-0145 corregido, "tan grave como confundir los valores del Eje A".

**Alcance:** modelado (Core + esquema + puertos + CLI + tests). NO se construye superficie de publicación (sigue diferida, DEBT-005) ni portal.

## 2. Agentes y Modo de Acompañamiento (ADR-0120/0122)

| Agente | Modelo | Modo |
|---|---|---|
| Rust-Engineer | Sonnet | **Revisión** |

Modo Revisión (no Docente): el patrón de #10 ya está enseñado en `docs/lessons/rust/STORY-037-verified-account-registry.md`; esto es un retrabajo mecánico y quirúrgico sobre código conocido. No se exige lección nueva; sí una **nota breve** al final de la lección existente sobre por qué el Eje B es ortogonal al Eje A (2-4 líneas), si el ingeniero lo estima útil — opcional.

## 3. Gate de Coherencia — resultado (contraste bidireccional)

- **ADR-0145 corregido → código:** el ADR ahora exige **dos ejes ortogonales por cuenta y por track**. Eje A ya existe (`AttestationScope`). Falta Eje B (`CapitalReality`). El ADR dice: "Cada track atestado lleva ambos ejes"; "todo dato publicado lleva **ambas** etiquetas, nunca una sola". → El código debe portar `CapitalReality` en la cuenta **y** estamparlo en cada track, y la proyección de puerto debe exponer **ambos ejes siempre**.
- **Código → ADR:** el Eje A (`SOVEREIGN`/`BROKER_READONLY`, `is_sovereign_attestation`) es correcto y **no se toca su semántica**. El Eje B es aditivo y ortogonal: no reemplaza ni condiciona al Eje A. Las combinaciones realistas del ADR: `SOVEREIGN`+`LIVE` (buque insignia), `SOVEREIGN`+`PAPER/DEMO/CHALLENGE` (potencial atestiguable antes de arriesgar capital real), `BROKER_READONLY`+`LIVE` (estilo myFXbook).
- **Naturaleza del Eje B — valor único en la cuenta, estampado por track.** A diferencia del Eje A (`attestation_scopes` es un **conjunto** coexistente en la cuenta; cada track elige UNO), el Eje B es un **valor único** por cuenta (una cuenta es LIVE, o PAPER, o DEMO, o CHALLENGE — no varias a la vez). Cada track **estampa** la realidad de capital de su cuenta al momento del cálculo (igual que estampa el `scope`). Fuente de verdad: `verified_accounts.capital_reality`; el orquestador `attest_track_record` lo copia de la cuenta al track (el llamador no puede mislabelar).
- **Greenfield (ADR-0006):** la migración `0016` se edita **in situ** (añadir columna + CHECK a ambas tablas); NO se crea una migración incremental. Fase GREENFIELD, baseline editable.
- **Firma y audit_hash deben incluir el Eje B (integridad):** `compute_track_record_signature` debe distinguir `LIVE` de `PAPER` con métricas idénticas (igual que hoy distingue el `scope`) — de lo contrario un track PAPER y uno LIVE de las mismas cifras colisionarían en firma, y la etiqueta de capital sería falsificable. Ambos `audit_hash` (cuenta y track) deben encadenar el nuevo campo.
- **Puertos (ADR-0137) — ambos ejes siempre visibles:** `VerifiedAccountRecord` gana `capital_reality: String`; `AttestedTrackRecord` gana `capital_reality: String` **y** `is_real_capital: bool` (derivado de `CapitalReality::is_real_capital`, paralelo a `is_attested_by_drasus`). La proyección NUNCA emite un track con el Eje A pero sin el Eje B — ambos campos son estructurales en el struct de salida.
- **Sin secretos (ADR-0093):** el Eje B es una etiqueta de enum, jamás un secreto. Guardarraíl existente intacto.
- **`CAPITAL_MODES` (CONFIG, ya en la Feature):** conjunto `LIVE/PAPER/DEMO/CHALLENGE` soportado; ortogonal a `ATTESTATION_SCOPES`.
- **Regla del skill (rust-engineer §4):** si algún tipo de #6/#1 quedara como placeholder, cruza a DEBT. Aquí no debería aplicar (el Eje B es propio de #10).

## 4. Cambios exigidos (mapa quirúrgico sobre el código sellado)

**Core — `crates/shared/src/domain/verified_account_registry.rs`:**
1. Enum nuevo `CapitalReality { Live, Paper, Demo, Challenge }` con `as_str` (`LIVE`/`PAPER`/`DEMO`/`CHALLENGE`), `from_str_value`, e `is_real_capital(&self) -> bool` (`true` SOLO para `Live`) — mismo estilo que `AttestationScope`. Documenta que es el **Eje B**, ortogonal al Eje A.
2. `compute_verified_account_audit_hash`: nuevo parámetro `capital_reality: CapitalReality`, empujado al buffer (posición fija, documentada).
3. `compute_track_record_audit_hash`: nuevo parámetro `capital_reality: &str`, empujado al buffer.
4. `metrics_to_canonical_map` + `compute_track_record_signature`: nuevo parámetro `capital_reality: CapitalReality`, incluido en el mapa canónico (clave `capital_reality`) — la firma debe cambiar entre `LIVE` y `PAPER` con métricas idénticas.
5. `VerifiedAccountRecord`: campo `capital_reality: String`. `AttestedTrackRecord`: campos `capital_reality: String` **y** `is_real_capital: bool`.

**Shell — `crates/shared/src/persistence/verified_account_registry.rs`:**
6. `NewVerifiedAccount` + `VerifiedAccountRow`: campo `capital_reality: CapitalReality`. `create`/`find_by_id`/`update_publication_and_scopes`/`row_to_verified_account` + INSERT/SELECT/UPDATE columns + llamadas a `compute_verified_account_audit_hash`. Nueva variante de error `UnknownCapitalReality(String)`.
7. `RecordTrackRecordInput` + `AttestedTrackRecordRow`: campo `capital_reality: CapitalReality`. `record_track_record`/`try_record_once`/`load_chain`/`row_to_track_record` + INSERT/SELECT columns + llamada a `compute_track_record_audit_hash`.
8. `From<&VerifiedAccountRow> for VerifiedAccountRecord` y `From<&AttestedTrackRecordRow> for AttestedTrackRecord`: proyectan `capital_reality` (y `is_real_capital` derivado en el track).

**Migración — `migrations/0016_verified_account_registry.sql` (editar in situ):**
9. `verified_accounts`: columna `capital_reality TEXT NOT NULL CHECK (capital_reality IN ('LIVE','PAPER','DEMO','CHALLENGE'))`.
10. `attested_track_records`: misma columna + CHECK. Actualiza los comentarios de cabecera (documentar el Eje B).

**Orquestador — `crates/shared/src/orchestrator/verified_account_registry.rs`:**
11. `attest_track_record`: estampa `capital_reality` del track **desde `account.capital_reality`** (no lo recibe por parámetro — la cuenta es la fuente de verdad); pásalo a `compute_track_record_signature` y a `RecordTrackRecordInput`.

**CLI harness — `crates/shared/src/public_interface.rs`:**
12. El input de `verify_verified_account_registry` (`VerifiedAccountVerifyInput` / su sub-struct de cuenta) gana `capital_reality` (con `#[serde(default)]` → `LIVE` si se omite, para no romper invocaciones previas); la salida refleja ambos ejes. Re-export del nuevo `CapitalReality` en el submódulo `verified_account_registry` de `public_interface`.

**Tests (rojo→verde) — actualiza los existentes y añade el discriminante:**
13. Todos los constructores de struct en tests (`sample_new_account`, `record_input`, `sample_metrics` no aplica, los `AttestedTrackRecordRow`/`VerifiedAccountRecord`/`AttestedTrackRecord` literales) ganan el campo. Los tests de columnas de la migración (`migration_creates_*`) añaden `capital_reality` a la lista esperada.
14. **Test discriminante nuevo (EL punto de DEBT-014):** un track `SOVEREIGN` + `PAPER`:
    - `is_attested_by_drasus == true` (ES atestado — mismo entorno determinista);
    - `is_real_capital == false` y `capital_reality == "PAPER"` (jamás se presenta como LIVE).
    Demuestra que los dos ejes son **ortogonales**: atestable ≠ capital real.
15. **Firma distingue el Eje B:** mismas métricas + mismo `scope`, `LIVE` vs `PAPER` → firmas distintas (paralelo al test `compute_track_record_signature_differs_by_scope`).
16. **CHECK de BD rechaza un `capital_reality` fuera del catálogo** (paralelo a `database_check_rejects_unknown_scope`).

## 5. Instrucciones de despacho (prompt exacto)

> Eres el **Rust-Engineer** en **Modo Revisión**. Lee primero: `CLAUDE.md`, `.claude/skills/base/SKILL.md`, `.claude/skills/rust-engineer/SKILL.md` (§4), **esta Orden completa (STORY-038)**, el ADR-0145 corregido (`docs/adr/ADR-0145.md` — banner 🔶 2026-07-06 + sección "Modelo de confianza" con los dos ejes), la Feature `docs/features/verified-account-registry.md` (banner 🔶 + parámetro `CAPITAL_MODES`), y **el código sellado que vas a modificar**: `crates/shared/src/domain/verified_account_registry.rs`, `crates/shared/src/persistence/verified_account_registry.rs`, `crates/shared/src/orchestrator/verified_account_registry.rs`, `migrations/0016_verified_account_registry.sql`, y el harness `verify_verified_account_registry` en `crates/shared/src/public_interface.rs`. Declara que los leíste.
>
> **Este es un RETRABAJO quirúrgico, no una reescritura.** El Eje A (`AttestationScope`, `is_sovereign_attestation`) es correcto y su semántica NO se toca. Añades el **Eje B ortogonal** (`CapitalReality`), siguiendo el mapa de la §4 de esta Orden **al pie de la letra** (los 16 puntos). Edita con `Edit` en bloques pequeños; no reescribas archivos enteros.
>
> **El invariante de correctitud nuevo (no lo pierdas de vista):** un track puede ser **atestado** (Eje A = `SOVEREIGN`) y a la vez de **capital virtual** (Eje B = `PAPER`/`DEMO`/`CHALLENGE`). Son ejes independientes. `is_real_capital` NO se deriva de `is_attested_by_drasus`; se deriva SOLO de `CapitalReality`. El struct de salida `AttestedTrackRecord` lleva SIEMPRE ambos ejes.
>
> **Decisiones de modelado ya tomadas (no las re-litigues):** (a) `CapitalReality` es un **valor único** por cuenta (no un conjunto como `attestation_scopes`); (b) vive en `verified_accounts` **y** se estampa en cada `attested_track_records`; (c) `attest_track_record` lo copia de `account.capital_reality` (no lo recibe por parámetro); (d) la migración `0016` se edita **in situ** (greenfield); (e) la firma y ambos `audit_hash` incluyen el Eje B.
>
> **Pruebas discriminantes obligatorias (§4 puntos 14-16):** el test `SOVEREIGN`+`PAPER` (atestado pero capital virtual), la firma que distingue `LIVE`/`PAPER`, y el CHECK de BD. Además, actualiza TODOS los tests existentes que construyen los structs afectados y las listas de columnas esperadas de las migraciones.
>
> **Comentarios en español, identificadores en inglés (ADR-0121).** Gate: `cargo test --workspace` verde + `cargo clippy --workspace --all-targets -- -D warnings` cero warnings.
>
> **NO construyas la superficie de publicación, ni portal, ni contrato de red. NO toques otras migraciones (solo edita 0016 in situ). NO hagas commits. NO toques archivos personales ni los 6 del Architect** (`docs/ROADMAP.md`, `docs/features/licensing-system.md`, `docs/features/usage-metering.md`, `docs/features/central-identity.md`, `docs/README.md`, `docs/moonshots/encrypted-local-backup.md`). Al terminar reporta: archivos tocados, `cargo test --workspace` + `clippy`, la salida del `verify verified-account-registry` mostrando ambos ejes, y confirma explícitamente que el test discriminante `SOVEREIGN`+`PAPER` pasa con `is_attested_by_drasus=true` e `is_real_capital=false`.

## 6. Criterio de aceptación (verificable)

| # | Criterio | Prueba |
|---|---|---|
| 1 | `CapitalReality` (LIVE/PAPER/DEMO/CHALLENGE) con `is_real_capital` (true solo LIVE) | round-trip + unit test |
| 2 | Eje B en ambas tablas (columna + CHECK), migración STRICT editada in situ | tests de columnas + CHECK |
| 3 | **Discriminante: `SOVEREIGN`+`PAPER` → atestado (Eje A) pero capital virtual (Eje B)** | test discriminante nuevo |
| 4 | Firma reproducible distingue `LIVE`/`PAPER` con métricas idénticas | test de firma por Eje B |
| 5 | `audit_hash` (cuenta y track) encadena el Eje B | tests de hash |
| 6 | Proyección de puerto expone SIEMPRE ambos ejes (`capital_reality` + `is_real_capital`) | assert estructural |
| 7 | Orquestador estampa el Eje B desde la cuenta (no mislabela) | test end-to-end |
| 8 | CLI `verify` muestra ambos ejes; `#[serde(default)]`=LIVE no rompe invocaciones previas | `cargo run -p app -- verify …` |
| 9 | Eje A intacto (semántica de `is_sovereign_attestation` sin cambios) | tests existentes verdes |
| 10 | Verde + clippy limpio | `cargo test --workspace` + `clippy` |

## 7. Comandos de validación (usuario)

```bash
cargo test -p shared
cargo clippy --workspace --all-targets -- -D warnings
cargo run -p app -- verify verified-account-registry --input '{"account":{"broker":"ICMarkets","currency":"USD","account_type":"OWN","capital_reality":"PAPER"},"scope":"SOVEREIGN","consent":"COVERED","events":[{"type":"OrderExecuted","pnl_e8":15000000000}]}'
```

## 8. Registro de ejecución

- 2026-07-06 · Tech-Lead · Gate de Coherencia del retrabajo corrido (contraste bidireccional ADR-0145 corregido ↔ código sellado de #10). Eje B (`CapitalReality`) ortogonal al Eje A (`AttestationScope`): valor único por cuenta, estampado por track; firma + ambos `audit_hash` lo incluyen; proyección de puerto expone ambos ejes siempre; migración `0016` editada in situ (greenfield). Mapa quirúrgico de 16 puntos (§4). Orden creada, despacho a Rust-Engineer (Sonnet, **Revisión**). Paga DEBT-014.
- 2026-07-06 · Rust-Engineer (Sonnet, Revisión) · Entregado. Enum `CapitalReality` (Eje B) con `is_real_capital` (true solo `Live`); `capital_reality` añadido a `compute_verified_account_audit_hash`, `compute_track_record_audit_hash`, `metrics_to_canonical_map`/`compute_track_record_signature`; `VerifiedAccountRecord`/`AttestedTrackRecord` ganan `capital_reality` (+ `is_real_capital` derivado); `NewVerifiedAccount`/`VerifiedAccountRow`/`RecordTrackRecordInput`/`AttestedTrackRecordRow` + INSERT/SELECT/UPDATE + variante `UnknownCapitalReality`; migración `0016` editada in situ (columna + CHECK en ambas tablas); `attest_track_record` estampa desde `account.capital_reality`; harness CLI con `#[serde(default)]`=LIVE. Sin placeholders. Discriminante `SOVEREIGN`+`PAPER` en 3 capas.
- 2026-07-06 · Tech-Lead · **Auditoría independiente APROBADA** (reproducción propia: `cargo test -p shared` 465 verdes, clippy 0 warnings; verificado en el código real: `is_real_capital` deriva SOLO de `CapitalReality` — nunca del Eje A; firma y ambos `audit_hash` encadenan el Eje B; orquestador estampa desde la cuenta; proyección de puerto expone ambos ejes de forma independiente; Eje A intacto en semántica).
- 2026-07-06 · QA (mutación, `cargo-mutants` sobre el Core, ejecutada por el TL) · **APTO**. 92 mutantes: 80 cazados, 6 inviables, 6 sobrevivientes — los 6 sobrevivientes son EXACTAMENTE los bordes ya rastreados en DEBT-013 (filtro cross-cuenta del snapshot, `>`vs`>=` en drawdown/peak/win-rate, signo del capital-base de respaldo), todos en `compute_track_record`. **Ninguno toca la lógica nueva del Eje B** (`CapitalReality`/`is_real_capital`/firma/hash/proyección = 100% cazados). El retrabajo no introdujo deuda nueva; DEBT-013 re-confirmada (sigue abierta, no bloqueante).
- 2026-07-06 · Tech-Lead · **RETRABAJO CERRADO — DEBT-014 PAGADA.** El cimiento #10 ahora modela ambos ejes ortogonales (Eje A autoría × Eje B realidad de capital). Feature `verified-account-registry.md` actualizada (banner 🔶 resuelto). Pendiente de autorización: commit agrupado.

## 9. Deudas / diferidos

- Al cerrar (auditoría TL APROBADA + QA APTO): **DEBT-014 → Pagada** por esta Story; el banner 🔶 de la Feature `verified-account-registry.md` se retira (queda solo el sello de estado, ya con ambos ejes); DEBT-013 (huecos de cobertura de #10) sigue vigente salvo que el QA de este retrabajo lo cubra incidentalmente.
- Superficie de publicación (panel de cuentas verificadas con ambos ejes visibles) sigue diferida → tanda de UI (DEBT-005).
