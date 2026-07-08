# STORY-041 — Retrabajo #10: consolidar el Eje B en `institutional_tag` (eliminar `capital_reality`)

> **Orden de retrabajo del Tech-Lead** · Ingeniero: Rust-Engineer (Sonnet, Revisión) · Paga **DEBT-016** · Corrige la violación de Inundación de Fundaciones (ADR-0020) detectada por el Architect en ADR-0145 (banner 🔶 2026-07-07) y SAD-22 §22.6.
>
> **Toca código SELLADO de #10** (`verified-account-registry`, STORY-037/038) → auditoría TL independiente + QA por mutación **obligatorios**, igual que STORY-038.

## 1. El problema (qué está mal hoy)

STORY-038 implementó el Eje B (realidad de capital) como una **columna nueva** `capital_reality` (`LIVE`/`PAPER`/`DEMO`/`CHALLENGE`) en `verified_accounts` y `attested_track_records`. Pero esas tablas **ya tienen** `institutional_tag` (Grupo II — Soberanía & Propiedad, "Environment", ADR-0020), obligatorio por su Perfil D, poblado hoy con un placeholder genérico (`"DRASUS_LOCAL"`). **Dos columnas con el mismo dominio de valores en la misma fila** = violación directa de "reutilización antes que creación" (ADR-0144 FIJO) y del vocabulario canónico único de ADR-0020.

**Corrección ratificada por el propietario (ADR-0145):** el Eje B **NO es un campo nuevo** — es `institutional_tag`, extendiendo su vocabulario de `PROD`/`PAPER`/`CHALLENGE` a `LIVE`/`PAPER`/`DEMO`/`CHALLENGE` (`LIVE` reemplaza `PROD` como sinónimo más claro en contexto de trading; `DEMO` es valor nuevo del mismo campo).

## 2. El objetivo (qué debe quedar)

- **Una sola columna** para el Eje B: `institutional_tag`, con `CHECK (institutional_tag IN ('LIVE','PAPER','DEMO','CHALLENGE'))` en **ambas** tablas de #10.
- **La columna `capital_reality` desaparece** de la migración `0016` y de todo el código.
- **Cero cambio de comportamiento observable del Eje B:** `is_real_capital` sigue siendo `true` solo para `LIVE`; el gain% sigue excluyendo el flujo de capital (Eje A intacto); la doble etiqueta (Eje A autoría × Eje B capital) se sigue exponiendo siempre; el discriminante `SOVEREIGN`+`PAPER` (atestado pero capital virtual) se conserva. **Solo cambia la COLUMNA/fuente del Eje B, no la semántica.**

## 3. Fase greenfield (ADR-0006)

`0016` es editable **in situ** (no hay migración incremental). Edita la migración existente directamente: quita `capital_reality`, añade el `CHECK` a `institutional_tag`. No crees una migración nueva.

## 4. Mapa quirúrgico

### 4.1 `migrations/0016_verified_account_registry.sql`
- **`verified_accounts`** (línea ~130 y ~150): elimina la columna `capital_reality` y su `CHECK`. En `institutional_tag` (línea ~130) añade `CHECK (institutional_tag IN ('LIVE','PAPER','DEMO','CHALLENGE'))`.
- **`attested_track_records`** (línea ~175 y ~192): idem — elimina `capital_reality`, añade el `CHECK` a `institutional_tag`.
- Actualiza los comentarios de cabecera (líneas ~33, ~70, ~88) para que describan `institutional_tag` como portador del Eje B (realidad de capital), citando la corrección ADR-0145 2026-07-07. No dejes referencias a `capital_reality`.

### 4.2 `crates/shared/src/domain/verified_account_registry.rs`
- **Conserva el tipo `CapitalReality`** (enum `Live`/`Paper`/`Demo`/`Challenge` con `as_str`/`from_str_value`/`is_real_capital`) — es la abstracción de dominio que **interpreta** el valor de `institutional_tag`. No lo elimines; es lo que da tipado fuerte y `is_real_capital` a partir de un `String`.
- Donde el hash/firma consuma el Eje B, la **fuente** pasa a ser el `institutional_tag` de la cuenta/track (interpretado como `CapitalReality`), no una columna `capital_reality` separada. La clave canónica en `metrics_to_canonical_map`/firma puede seguir llamándose `capital_reality` **como nombre lógico del dato firmado** si eso preserva la firma — pero el valor proviene de `institutional_tag`. Documenta esa equivalencia en un comentario. (Si prefieres renombrar la clave canónica a `institutional_tag`, es aceptable siempre que actualices los tests de firma; decide tú, pero deja el porqué en comentario.)
- El comentario de sección (línea ~222 "columna `capital_reality`") debe reescribirse: el Eje B vive en `institutional_tag`.

### 4.3 `crates/shared/src/persistence/verified_account_registry.rs`
- Elimina el campo `capital_reality: CapitalReality` de `NewVerifiedAccount`, `VerifiedAccountRow`, `RecordTrackRecordInput`, `AttestedTrackRecordRow` y el error `UnknownCapitalReality` si queda huérfano.
- El Eje B se lee/escribe desde `institutional_tag`. Los INSERT/SELECT/UPDATE dejan de mencionar `capital_reality`.
- La proyección de puerto sigue exponiendo `is_real_capital` — ahora derivado de `CapitalReality::from_str_value(&row.institutional_tag).is_real_capital()`. **Mantén la validación:** un `institutional_tag` fuera del vocabulario debe fallar tipado (reutiliza/renombra el error, ej. `UnknownInstitutionalTag`), no asumir un default.

### 4.4 `crates/shared/src/orchestrator/verified_account_registry.rs`
- Donde hoy estampa `institutional_tag: "DRASUS_LOCAL"` (líneas ~179, ~385) y `capital_reality` por separado: el `institutional_tag` pasa a ser el valor del Eje B (`LIVE`/`PAPER`/…). Ya no existe `capital_reality` como campo aparte. `attest_track_record` estampa el `institutional_tag` de la cuenta en el track (igual que antes estampaba `capital_reality`).

### 4.5 `crates/shared/src/public_interface.rs` + `crates/app/src/main.rs`
- El input del CLI deja de recibir `account.capital_reality`; ahora recibe el Eje B como `account.institutional_tag` (vocabulario `LIVE`/`PAPER`/`DEMO`/`CHALLENGE`, **default `LIVE`** — reemplaza el `default_verified_account_capital_reality`). El output sigue exponiendo `is_real_capital` + la etiqueta de capital, ahora leídos de `institutional_tag`.
- Actualiza el doc-comment del subcomando en `main.rs` si menciona `capital_reality`.

## 5. Invariantes que NO cambian (verifícalos con tests existentes)
- gain% EXCLUYE el flujo de capital (Eje A, diferenciador ADR-0145).
- `AttestationScope` `SOVEREIGN`/`BROKER_READONLY` inviolable (`is_sovereign_attestation`) — Eje A intacto.
- `is_real_capital` = `true` solo para `LIVE`.
- Ambos ejes se exponen SIEMPRE juntos en la proyección de puerto.
- Secretos nunca en el registro (ADR-0093).
- Ledger append-only atómico de `attested_track_records` intacto (`BEGIN IMMEDIATE` + `WriteContention` + 16 escritores).

## 6. Tests obligatorios (ADR-0133)
- **Todos los tests de #10 existentes deben seguir pasando** tras el cambio de columna (adáptalos a `institutional_tag`, no los borres).
- El discriminante `SOVEREIGN`+`PAPER` (atestado pero capital virtual) sigue probado en las 3 capas.
- **Guardarraíl anti-regresión:** un test que verifique que **no existe** ninguna columna `capital_reality` en el esquema (query a `pragma_table_info` o equivalente) y que `institutional_tag` sí porta el vocabulario del Eje B con su `CHECK`.
- JSON no filtra secretos (ADR-0093), test existente intacto.

## 7. Verificación antes de reportar
```bash
cargo test -p shared verified_account
cargo clippy -p shared --all-targets -- -D warnings
cargo run -p app -- verify verified-account-registry --input '{"account":{"broker":"ICMarkets","currency":"USD","account_type":"OWN","institutional_tag":"PAPER"},"scope":"SOVEREIGN","consent":"COVERED","events":[{"type":"CapitalFlow","sign":"DEPOSIT","amount_e8":35000000000},{"type":"OrderExecuted","pnl_e8":15000000000}]}'
```
El CLI debe imprimir el track con `is_real_capital:false` (PAPER) + `is_attested_by_drasus:true` (SOVEREIGN), sin columna ni clave `capital_reality` en el esquema.

## 8. Docente (ADR-0122)
Añade a `docs/lessons/rust/STORY-037-verified-account-registry.md` (o crea `STORY-041-...`) la lección: por qué reutilizar el campo canónico del catálogo (Grupo II `institutional_tag`) en vez de crear una columna nueva, y cómo un tipo de dominio (`CapitalReality`) puede interpretar un campo canónico sin duplicarlo.

## 9. Prohibiciones
- **NO** commitees nada.
- **NO** toques los 6 archivos protegidos del Architect ni ninguna otra feature (#11/#12 no se tocan).
- **NO** cambies la semántica del Eje B ni del Eje A — solo la columna/fuente del Eje B.
- **NO** uses modelos/agentes Opus.

---

## §10. Registro de cierre (lo llena el Tech-Lead al auditar)
- **Ingeniero:** 2026-07-07 · Rust-Engineer (Sonnet). Migración `0016` editada in-situ (columna `capital_reality` eliminada de ambas tablas; `CHECK (institutional_tag IN ('LIVE','PAPER','DEMO','CHALLENGE'))` añadido a `institutional_tag` en ambas). `CapitalReality` conservado como intérprete de `institutional_tag`. Persistence: campo `capital_reality` eliminado de las 4 structs; error `UnknownCapitalReality`→`UnknownInstitutionalTag` (fail-typed, sin default); proyección deriva `is_real_capital` de `institutional_tag`; **dos tests-guardarraíl** (`pragma_table_info` verifica ausencia de columna + CHECK del Eje B). Orquestador/public_interface/CLI: el Eje B se lee/escribe desde `institutional_tag` (input `account.institutional_tag`, default `LIVE`). Lección Docente añadida.
- **Auditoría TL independiente:** 2026-07-07 · **APROBADA**. Reproducción propia: `cargo test -p shared verified_account` 51 verdes + `cargo clippy` limpio. Verificado en esquema real: columna `capital_reality` eliminada (solo comentarios históricos); `institutional_tag` porta el CHECK del Eje B en ambas tablas; validación fail-typed `UnknownInstitutionalTag`; guardarraíl anti-regresión presente. La proyección de puerto conserva la etiqueta legible `capital_reality` derivada de `institutional_tag` (no es columna — decisión de diseño documentada).
- **QA por mutación (`cargo-mutants` sobre el Core, ejecutada por el TL):** 2026-07-07 · **APTO**. 92 mutantes: 80 cazados, 6 inviables, 6 sobrevivientes — **idénticos a STORY-038** (los mismos bordes de `compute_track_record`, DEBT-013 preexistente; **ninguno nuevo**). El retrabajo preservó la cobertura exactamente: cambió la fuente del Eje B, no la lógica.
- **Estado:** ✅ **CERRADO — DEBT-016 PAGADA.** Violación de Inundación de Fundaciones corregida en greenfield antes de commitear. Pendiente de autorización: commit agrupado.
