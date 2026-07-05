# STORY-034 — Institutional Report Engine (cimiento #7 del substrato)

| Campo | Valor |
|---|---|
| **ID** | STORY-034 |
| **Tipo** | Story (código — séptimo cimiento del substrato de pricing/SaaS) |
| **Épica (Fase)** | Cimientos de Monetización (transversal, greenfield) |
| **Sprint** | substrato-monetizacion |
| **Estado** | ✅ Completada (puerto + ensamblado + firma reproducible + persistencia append-only atómica; render Tera y catálogo de productos diferidos) |
| **Creada** | 2026-07-05 |
| **Feature** | [`institutional-report-engine`](../features/institutional-report-engine.md) |
| **ADRs** | ADR-0144 (cimiento #7) · ADR-0101 (plantillas Tera — render diferido) · ADR-0027 (trazabilidad al audit-log) · ADR-0137 (puertos) · ADR-0141 (append-only, montos ×10⁸) · ADR-0020 (Perfil D + subset V) · ADR-0093 (secretos) · ADR-0142 (CLI) |

## 1. Objetivo llano

Construir el **puerto de reportes institucionales**: dado un resultado del guantelete (validación/backtest/ejecución), ensambla un **documento institucional firmado y trazable** — con una **firma de integridad criptográfica REPRODUCIBLE** (mismo resultado → misma firma) y enlaces a los eventos fuente del event-store (#6) / audit-log para trazabilidad (ADR-0027). Habilita los productos institucionales de 5–6 cifras que son subproductos naturales del guantelete.

**Alcance ahora vs. después (ADR-0144 "puerto + esquema ahora, adaptador después"):**
- **Ahora (esta Story):** puerto `generate_report` + Core (ensamblado del reporte + firma reproducible) + persistencia append-only del reporte generado + enlaces de trazabilidad a los eventos fuente + CLI verify. Salida canónica en JSON.
- **Después (diferidos):** el **render con Tera (ADR-0101) a PDF/HTML** (Tera NO está en el workspace todavía); el **white-label/branding**; el **catálogo de productos** (moonshot `institutional-report-products`); la exposición por la **API de terceros** (#8); el **mapeo del `BacktestResult`/`RobustnessScore` reales** (hoy placeholders `pub struct X;`).

## 2. Agentes y Modo de Acompañamiento (ADR-0120/0122)

| Agente | Modelo | Modo |
|---|---|---|
| Rust-Engineer | Sonnet | **Docente** |

Docente: concepto nuevo relevante — la **firma reproducible** (determinismo → misma entrada, misma firma). Lección en `docs/lessons/rust/STORY-034-institutional-report-engine.md`.

## 3. Gate de Coherencia — resultado (contraste bidireccional)

- **Stack:** limpio. Crate `crates/shared` (plomería crosscutting, excepción ADR-0137: `InstitutionalReport` es tipo técnico del catálogo, consumido por los adaptadores de producto y la API de terceros #8).
- **Firma reproducible — regla obligatoria #1 (EL punto de correctitud):** `compute_report_signature(report)` debe ser **determinista**: mismo contenido de reporte → misma firma, bit a bit (feature §"determinismo"). Serialización **canónica** (claves ordenadas, `BTreeMap` — patrón de `enriched_domain_events`), hash sobre esa forma canónica. Cero `f64` en montos (métricas monetarias como `i64` ×10⁸ o representación entera/textual estable; NUNCA `REAL` — ADR-0141). Prueba discriminante: generar dos veces sobre el mismo input → misma firma; cambiar un dato → firma distinta.
- **Persistencia APPEND-ONLY ATÓMICA (regla DEBT-001, obligatoria #2):** tabla `generated_reports` **append-only** (`event_sequence_id UNIQUE` + triggers anti UPDATE/DELETE) — cada generación es un registro inmutable (audit trail de qué se generó y cuándo). El append **nace atómico**: `BEGIN IMMEDIATE` + reintento + `WriteContention` (copia el patrón de `persistence/enriched_domain_events.rs`). **Prueba de 2 escritores obligatoria** (qa §2).
- **Trazabilidad (ADR-0027) — regla obligatoria #3:** el reporte enlaza a los **ids de los eventos fuente** (del event-store #6 / audit-log). Modela `source_event_refs: Vec<String>` (ids que el reporte cita) en el input y persístelos con el reporte. NUNCA el reporte altera los datos fuente (feature §Restricciones): solo los presenta.
- **Perfil ADR-0020:** Perfil D — Grupo I completo (con `event_sequence_id`) + II (`owner_id`, `institutional_tag`) + IV (`node_id`) + **subset V forense** (`signature_hash`, `compliance_status_id`). Campos propios marcados: `report_type` (`TEXT CHECK`), `source_result_ref` (`TEXT` — referencia al resultado fuente), `source_event_refs` (`TEXT` JSON `json_valid` — ids de eventos citados), `report_body` (`TEXT` JSON `json_valid` — el contenido canónico del reporte). **Distinción clave:** `audit_hash`/`audit_chain_hash` (Grupo I, integridad de la fila en el ledger) es DISTINTO de `signature_hash` (firma reproducible del CONTENIDO del reporte). Lleva ambos.
- **Entrada placeholder (`result_in`):** `BacktestResult`/`RobustnessScore` son `pub struct X;` vacíos en `types/mod.rs` (como `Order` en #4). Modela la **entrada mínima de reporte** (un conjunto de métricas nombradas + metadatos), NO inventes el `BacktestResult` completo. Deja nota de que el mapeo real es futuro.
- **Puertos (ADR-0137):** `result_in ← BacktestResult`/`RobustnessScore` (Input, placeholders), `report_out → InstitutionalReport` (Output). Bajo `public_interface::institutional_report_engine`.
- **Guardarraíl ADR-0093:** ningún reporte incluye secretos. Assert explícito.
- **Render Tera diferido:** Tera NO está en el workspace; NO lo agregues. El Core produce el `InstitutionalReport` estructurado + su firma; la "plantilla base" es la **serialización canónica** (JSON) por ahora. El render a PDF/HTML es adaptador posterior.
- **Clasificación UI (ADR-0117):** plomería con salida documental (Ventana de Verificación). Observable por **CLI (Canal #2)**; su ventana va a la tanda de UI final (DEBT-005).

## 4. Instrucciones de despacho (prompt exacto)

> Eres el **Rust-Engineer** en **Modo Docente**. Lee primero: `CLAUDE.md`, `.claude/skills/base/SKILL.md`, `.claude/skills/rust-engineer/SKILL.md` (§4 atomicidad de ledgers), esta Orden, la feature `docs/features/institutional-report-engine.md`, la **plantilla de append atómico + serialización canónica** `crates/shared/src/persistence/enriched_domain_events.rs` y `crates/shared/src/domain/enriched_domain_events.rs`, cómo `types/mod.rs` declara `BacktestResult`/`RobustnessScore` (placeholders), y los ADR-0144, ADR-0101, ADR-0027, ADR-0137, ADR-0141, ADR-0020, ADR-0093, ADR-0142. Declara que los leíste.
>
> **Gate de Lectura Pre-Código:** (a) confirma `crates/shared`; (b) copia el patrón de append atómico + serialización canónica `BTreeMap` de `enriched_domain_events`; (c) confirma que modelas entrada mínima de reporte, no el `BacktestResult` real; (d) confirma que NO agregas Tera al workspace.
>
> **Construye (puerto + ensamblado + firma reproducible + persistencia append-only atómica; render Tera y catálogo diferidos):**
> 1. **Migración greenfield `migrations/0013_generated_reports.sql`:** tabla `generated_reports` **append-only** (`event_sequence_id UNIQUE` + triggers anti UPDATE/DELETE), Grupo I + Perfil D + subset V (`signature_hash TEXT NOT NULL`, `compliance_status_id TEXT` nullable); campos propios: `report_type TEXT NOT NULL CHECK(...)`, `source_result_ref TEXT`, `source_event_refs TEXT NOT NULL CHECK(json_valid(source_event_refs))`, `report_body TEXT NOT NULL CHECK(json_valid(report_body))`. `STRICT`, PK `TEXT` UUIDv7, `audit_chain_hash` encadenado. Índices: `event_sequence_id`, `report_type`, `owner_id`.
> 2. **Core `domain/institutional_report_engine.rs` (lógica pura):** struct `InstitutionalReport` (métricas nombradas + metadatos + `source_event_refs`); `assemble_report(input) -> InstitutionalReport`; `compute_report_signature(&report) -> String` **determinista** (serialización canónica `BTreeMap` → hash, misma entrada → misma firma); `report_type()`; `compute_report_audit_hash` encadenado. Cero `f64` en montos. Todo determinista.
> 3. **Shell `persistence/institutional_report_engine.rs` + `orchestrator/institutional_report_engine.rs`:** repo append-only **atómico** (`record_report` con `begin_with("BEGIN IMMEDIATE")` + reintento + `WriteContention`, copia `is_transient_write_conflict`); el orchestrator compone: ensambla el reporte (Core), computa firma, persiste con los `source_event_refs`. Reloj inyectado. El reporte NUNCA muta datos fuente.
> 4. **`public_interface`:** submódulo `institutional_report_engine` con `report_out` (reporte + firma) y `result_in`. Sin secretos (ADR-0093).
> 5. **CLI `verify`:** `cargo run -p app -- verify institutional-report-engine --input '<json>'` que, dado un resultado (métricas + refs de eventos), reproduce el observable (reporte + firma) en JSON.
>
> **Pruebas discriminantes (rojo→verde, deben poder fallar):**
> - **Firma reproducible (CRÍTICO):** dos `assemble_report`+`compute_report_signature` sobre el mismo input → **misma firma**; cambiar un dato → firma distinta. Debe fallar si la serialización no es canónica (orden no determinista) o usa `f64`.
> - **Append atómico + concurrencia:** `#[tokio::test(flavor = "multi_thread")]`, BD en archivo temporal, N≥16 escritores → N filas, `event_sequence_id` 1..=N denso, cadena íntegra. Cae sin `BEGIN IMMEDIATE`.
> - **Trazabilidad:** el reporte persiste sus `source_event_refs` y no altera nada del input (assert de que el input queda intacto / el reporte solo lo presenta).
> - **Append-only:** UPDATE/DELETE rechazados por trigger; `event_sequence_id` UNIQUE; `json_valid` rechaza JSON corrupto.
> - **`signature_hash` ≠ `audit_hash`:** ambos presentes y distintos en su rol (test que verifica que existen las dos columnas y que la firma es del contenido).
> - **`audit_chain_hash`:** génesis NULL, resto encadenado.
> - **Sin secretos (ADR-0093):** assert sobre el reporte.
> - Cobertura con `cargo llvm-cov --workspace --summary-only`.
>
> **Comentarios en español, identificadores en inglés (ADR-0121).** Gate: `cargo test --workspace` verde + `cargo clippy --workspace --all-targets -- -D warnings` cero warnings.
>
> **Docente:** escribe `docs/lessons/rust/STORY-034-institutional-report-engine.md` cero-conocimiento: qué es una firma reproducible y por qué el determinismo la hace verificable por terceros, por qué la serialización canónica es la clave, la diferencia entre `audit_hash` (integridad de la fila) y `signature_hash` (integridad del contenido del reporte), y qué es la trazabilidad al audit-log. Cita el código real.
>
> **NO agregues Tera. NO toques migraciones existentes (solo crea 0013). NO hagas commits. NO toques archivos personales ni los 6 del Architect** (`docs/ROADMAP.md`, `docs/features/licensing-system.md`, `docs/features/usage-metering.md`, `docs/features/central-identity.md`, `docs/README.md`, `docs/moonshots/encrypted-local-backup.md`). Al terminar reporta: archivos, `cargo test --workspace` + `clippy` + `llvm-cov`, salida del `verify`, y tu decisión de crate.

## 5. Criterio de aceptación (verificable)

| # | Criterio | Prueba |
|---|---|---|
| 1 | Migración `STRICT` append-only + triggers + `json_valid` + `CHECK(report_type)` | inspección + tests de rechazo |
| 2 | Firma **reproducible** (mismo input → misma firma; cambio → distinta) | test discriminante |
| 3 | Append atómico (`BEGIN IMMEDIATE` + reintento + `WriteContention`) + 2 escritores | test de concurrencia (cae sin la tx) |
| 4 | Trazabilidad (`source_event_refs` persistidos; sin alterar datos fuente) | test |
| 5 | `signature_hash` (contenido) distinto de `audit_hash` (fila) | test + inspección |
| 6 | `audit_chain_hash` encadenado; `event_sequence_id` UNIQUE | tests |
| 7 | Sin secretos (ADR-0093); cero `f64` en montos | test + inspección |
| 8 | CLI `verify institutional-report-engine` | `cargo run -p app -- verify institutional-report-engine --input '…'` |
| 9 | Lección Docente | existe el archivo |
| 10 | Verde + cobertura | `cargo test --workspace` + `cargo llvm-cov` |

## 6. Comandos de validación (usuario)

```bash
cargo test -p shared
cargo clippy --workspace --all-targets -- -D warnings
cargo run -p app -- verify institutional-report-engine --input '{"report_type":"VALIDATION","metrics":{"sharpe_e8":150000000,"max_drawdown_e8":-8000000},"source_event_refs":["evt-1","evt-2"]}'
```

## 7. Registro de ejecución

- 2026-07-05 · Tech-Lead · Gate corrido. Reglas: firma **reproducible** (serialización canónica `BTreeMap` → hash, cero `f64`); tabla `generated_reports` **append-only atómica** (regla DEBT-001); trazabilidad por `source_event_refs` a los eventos de #6; `signature_hash` (contenido) ≠ `audit_hash` (fila); entrada placeholder (`BacktestResult` vacío) → entrada mínima; render Tera diferido (no está en el workspace). Orden creada, despacho a Rust-Engineer (Sonnet, **Docente**).
- 2026-07-05 · Rust-Engineer (Docente) · Implementado EN VERDE: migración `0013_generated_reports.sql` (append-only STRICT, `event_sequence_id UNIQUE` + triggers, Grupo I + Perfil D + subset V `signature_hash`/`compliance_status_id`, `report_type CHECK`, `source_event_refs`/`report_body` `json_valid`); Core `domain/institutional_report_engine.rs` (`ReportType`, `InstitutionalReport`, `assemble_report`, `compute_report_signature` reproducible con `BTreeMap`+`Sha256`, `compute_report_audit_hash`; `metrics: BTreeMap<String,i64>` cero `f64`); Shell `persistence/` (repo append-only atómico `BEGIN IMMEDIATE`+reintento+`WriteContention`) + `orchestrator/` (`generate_report` con reloj inyectado, nunca muta el input); puerto `report_out`/`result_in`; CLI `verify institutional-report-engine`. Crate `crates/shared`. 342 tests shared (26 nuevas), cobertura Core 99.75%/orch 99.19%/persist 95.51%. Lección Docente escrita.
- 2026-07-05 · Tech-Lead · Auditoría independiente (reproducida): 369 tests workspace, 0 fallos; `persistence` con `begin_with("BEGIN IMMEDIATE")` (l.202) + reintento/`WriteContention`; Core con `BTreeMap`+`Sha256` (firma canónica), `metrics: BTreeMap<String,i64>` sin `f64`; migración STRICT append-only con `signature_hash` (subset V) distinto de `audit_hash`. Clippy limpio. Verde.
- 2026-07-05 · QA-Engineer (Sonnet) · **APTO.** 5+ pruebas de mutación (revertidas byte a byte), cada una tumbó una prueba concreta: `BTreeMap`→`HashMap` → cae `compute_report_signature_is_reproducible...`; `begin()` DEFERRED en vez de `BEGIN IMMEDIATE` → cae `concurrent_record_reports...` (3/3 corridas); omitir `metrics` de la firma → cae `compute_report_signature_changes_when_a_metric_changes`; quitar trigger UPDATE / `CHECK(json_valid)` → caen las de rechazo; génesis no-NULL → cae la de cadena. CLI real: firma≠audit_hash, sin decimales, sin secretos; `report_type` inválido → exit 1. 2 observaciones no bloqueantes (heurística IP no exhaustiva pero la garantía ADR-0093 es estructural — el Core no modela ningún campo secreto; `public_interface.rs` 0% cobertura = patrón preexistente de todos los cimientos, solo se ejercita por CLI). Árbol intacto.
- 2026-07-05 · Tech-Lead · Gate QA cerrado con APTO. **STORY-034 completada.** Feature `institutional-report-engine` 🟡 Parcial (render Tera→PDF/HTML, catálogo de productos, mapeo `BacktestResult` real, API de terceros diferidos).

## 8. Deudas / diferidos registrados

- **Render con Tera (ADR-0101) a PDF/HTML + white-label:** Tera no está en el workspace; el Core produce el reporte + firma; el render es adaptador posterior.
- **Catálogo de productos institucionales:** moonshot `institutional-report-products` (adaptadores sobre este puerto).
- **Mapeo del `BacktestResult`/`RobustnessScore` reales:** hoy placeholders; se cablea cuando el guantelete los produzca.
- **Exposición por la API de terceros:** cimiento #8 `third-party-api-gateway`.
- **Ventana de Verificación (Canal #1):** tanda de UI final (DEBT-005).
