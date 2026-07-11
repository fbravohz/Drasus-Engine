# STORY-045…048 · Remediación de la Auditoría Retroactiva de la Fundación

> Orden umbrella de la remediación derivada de la auditoría retroactiva EPIC-0. Agrupa cuatro historias coordinadas (STORY-045/046/047/048) que comparten origen (el plan de auditoría) y ventana de ejecución. Cada historia conserva su identidad y su criterio de cierre; la trazabilidad fina vive aquí.

| Campo | Valor |
|---|---|
| **ID** | STORY-045 (umbrella 045–048) |
| **Título** | Remediación de la auditoría retroactiva de la Fundación |
| **Tipo** | Story (×4 coordinadas) + Task (escalamiento) |
| **Épica (Fase)** | EPIC-0 — Fundación (retroactiva) |
| **Sprint** | Auditoría retroactiva |
| **Estado** | En curso |
| **Responsable** | Rust-Engineer (Sonnet) + Flutter-Engineer (Sonnet) · audita Tech-Lead + QA |
| **Creada** | 2026-07-10 |
| **Completada** | — |

## 0. Resumen ejecutivo

- **Qué problema resuelve:** la auditoría retroactiva EPIC-0 (6 lotes de diagnóstico) confirmó que el substrato es arquitectónicamente sano, pero halló que **el esquema base de la Fundación** (migraciones `0001–0006` + `pool.rs`) se construyó antes de que ADR-0141 se endureciera, y que la cobertura de mutación del patrón de ledger append-only (DEBT-018) no se aplicó uniformemente.
- **Qué se construye:** (045) endurecimiento del esquema base greenfield; (046) atomicidad append-only en la plomería EPIC-0; (047) retrofit de los 3 tests companion de mutación en los cimientos #4–#12; (048) pulido de UI (tokens).
- **Por qué ahora:** GREENFIELD permite editar el baseline in-situ sin costo de migración; tras el primer release distribuido se congela a BROWNFIELD y estas correcciones se vuelven caras o imposibles.

---

## 1. Especificación de origen

- **Plan de auditoría (spec de origen):** [`.agents/plans/magical-sprouting-quasar.md`](../../.agents/plans/magical-sprouting-quasar.md) — hallazgos consolidados C1/C2/M1–M8 + 🟡.
- **Memoria:** [[auditoria-retroactiva-epic0]], [[pricing-foundations-saas]], [[debt-registry-y-atomicidad]].
- **Deuda:** `docs/DEBT.md` — DEBT-018 (retrofit de mutación), DEBT-004/005 (a reformular).
- **ADR(s):** ADR-0141 (M1–M12 / R1 / A2–A6), ADR-0133 (capa 8 mutación + capa 3 proptest), ADR-0020, ADR-0093, ADR-0123, ADR-0006 (greenfield in-situ).

## 2. Objetivo (una frase llana)

Dejar la Fundación uniformemente conforme a las reglas de esquema, atomicidad y cobertura que el substrato ya cumple, aprovechando que la fase greenfield permite corregir el baseline sin costo.

## 3. Agentes y Modo de Acompañamiento (ADR-0120)

| Agente | Etapa | Historia | Depende de | Modo |
|---|---|---|---|---|
| Rust-Engineer | Etapa 2 | STORY-045 + 046 | ninguno | Autónomo |
| Rust-Engineer | Etapa 2 | STORY-047 (retrofit) | 045+046 en verde | Autónomo |
| Flutter-Engineer | Etapa 4 | STORY-048 | ninguno | Autónomo |
| **QA-Engineer** | **Etapa 5 — gate obligatorio** | todas | ingeniero implementador | Autónomo |
| Tech-Lead | Gate de mutación (capa 8) | todas | QA | — (lo corre el TL) |

## 4. Instrucciones de despacho

Los prompts exactos se despacharon vía la herramienta Agent en Modo Autónomo (registrados en la bitácora de sesión y resumidos en §7). Cada uno ordena: leer `CLAUDE.md` + `base.md` + el SKILL del rol + el plan de auditoría, e implementar el alcance de su historia con pruebas discriminantes (rojo→verde). El patrón dorado de atomicidad/tests companion es `crates/shared/src/persistence/data_portability.rs`; la referencia de ledger atómico es `audit_log.rs::try_append_once`.

- **STORY-045 (esquema base):** C1 (foreign_keys=ON + PRAGMAs + test de insert huérfano), C2 (STRICT en 0001–0006), M3 (UUIDv7 en 7 sitios + Cargo del fetcher), M4 (triggers `permission_decisions`), M5 (audit_chain_hash NULL / A4), M6 (UNIQUE en `sovereign_download_records`), 🟡 de esquema (A3 rename, PRAGMAs, CHECK enums/json_valid, FK explícita, formato Parquet).
- **STORY-046 (atomicidad EPIC-0):** M1 (`BEGIN IMMEDIATE`+reintento+`WriteContention` en `job.rs::record_result`, fetcher `persistence.rs::record`, `mcp_server.rs::check_and_record`) + 3 tests companion cada uno.
- **STORY-047 (retrofit DEBT-018):** los 3 tests companion en #4,#5,#6,#7,#9,#10,#11,#12 → 0 survivors. Despacho en Ola 2.
- **STORY-048 (UI):** M8 (literales → `Gx.optimaCyan`/`Gx.transitionIndigo` en `settings_drawer.dart`).

## 5. Criterio de aceptación (cada criterio ↔ su prueba)

| # | Criterio verificable | Prueba |
|---|---|---|
| 1 | Un insert de `job_results` con `job_uuid` inexistente falla por FK | test de insert huérfano (045/C1) |
| 2 | Las 6 tablas del baseline son `STRICT` | migración corre desde cero + inspección de esquema (045/C2) |
| 3 | Ningún `Uuid::new_v4()` en la plomería EPIC-0 | grep = 0 + tests verdes (045/M3) |
| 4 | UPDATE/DELETE directo sobre `permission_decisions` aborta | test de trigger (045/M4) |
| 5 | `audit_chain_hash` es NULL en génesis (sin sentinel) | test de génesis (045/M5) |
| 6 | `sovereign_download_records.event_sequence_id` es UNIQUE | test de duplicado rechazado (045/M6) |
| 7 | Los 3 ledgers EPIC-0 sobreviven contención concurrente sin perder eventos | 3 tests companion ×3 ledgers (046/M1) |
| 8 | Cada cimiento #4–#12 mata los mutantes del patrón append-only | `cargo mutants` a 0 survivors (047/M2) — lo corre el TL |
| 9 | UI sin literales de color duplicados; `flutter analyze` limpio | inspección + build (048/M8) |

## 6. Comandos de validación (para el usuario)

```bash
cargo test -p shared
cargo clippy --workspace --all-targets -- -D warnings
# gate de mutación (lo corre el Tech-Lead, --output dedicado para no colisionar):
cargo mutants -p shared --file crates/shared/src/persistence/<archivo>.rs --output /tmp/mutants-<story>
# esquema desde cero:
rm -f ui/drasus.db && cargo run -p app -- verify clock --input '{}'   # re-aplica migraciones
# UI:
cd ui && flutter analyze
```

## 7. Registro de ejecución (bitácora cronológica)

- 2026-07-10 · Diagnóstico (6 lotes Sonnet) · COMPLETO · substrato sano; hallazgos concentrados en esquema base EPIC-0 + DEBT-018. Tablas de veredictos en `.agents/plans/magical-sprouting-quasar-agent-*.md`.
- 2026-07-10 · Rust-Engineer (Sonnet) STORY-045+046 · APROBADO · +11 tests `shared` (641), +3 fetcher (24). Auditoría TL: verificación estática del esquema (foreign_keys+PRAGMAs con test que los asevera, STRICT×6, triggers, `audit_chain_hash` nullable+`Option`, UNIQUE, rename `row_version`); **1 defecto cazado** (M3 se saltó `audit_log.rs:192`, `new_v4` residual → rematado por el TL con `now_v7`). Reproducción TL: `cargo test -p shared` 641 verde, fetcher 24, kill9 verde, clippy limpio. Gate de mutación TL sobre pool/job/mcp: atomicidad cubierta (retry `+=→*=` = TIMEOUT detectable; append cazado); 20 survivors ortogonales + fidelidad de `update_progress` → **DEBT-019**.
- 2026-07-10 · Flutter-Engineer (Sonnet) STORY-048 · APROBADO · 4 líneas literal→token (valores idénticos verificados byte a byte), `flutter build linux --release` verde. Reproducción TL: grep 0 literales duplicados, diff acotado a 4 líneas.
- 2026-07-10 · Rust-Engineer (Sonnet) STORY-047 · APROBADO · retrofit DEBT-018: +23 tests (664), luego +1 (665) al cerrar 5 survivors de fidelidad mutable (`update_parent_and_consent` #12, `update_publication_and_scopes` #10) devueltos por SendMessage. Gate de mutación TL: 1ª corrida 5 `missed`; 2ª corrida (post-fix) **0 `missed`** (8 `timeout` del contador de reintento, aceptables).
- 2026-07-10 · QA-Engineer (Sonnet) · **APTO** · revisión de correctitud línea por línea de las 4 historias sobre el `git diff` (28 archivos); reprodujo `cargo test` (shared 665 / fetcher 24 / kill9 1) + clippy limpio. Confirmó: patrón de atomicidad correcto (tx envuelve tail+insert, reintento solo ante conflicto transitorio, cero doble-INSERT), STORY-047 solo tests (cero producción colada), 048 sin cambio de valor. 2 observaciones no bloqueantes (comentarios desactualizados por el rename/nuevo camino de escritura) → corregidas por el TL.

## 8. Pendientes derivados / decisiones

- **Paquete de escalamiento al Architect (Task aparte):** desincronizaciones documentales (ADR-0137 sin enmienda para #11–#14; CLAUDE.md §1 lista solo 6 features bendecidas; comentarios de migraciones 0007–0009; cita errónea de ADR-0101 en `institutional-report-engine.md`; contradicción interna de `verified-account-registry.md`; ADR-0136 "TBD" vs "Forge"), M7 (proptest vs enumeración exhaustiva — ADR-0133), `owner_id` FK física sistémica, y reconciliación de Canvas (ADR-0117/0136 — la infra genérica YA existe). El TL redacta la evidencia; el Architect edita `docs/`.
- **DEBT.md (hecho):** DEBT-018 saldada (STORY-047, 0 `missed`); nueva **DEBT-019** (cobertura de mutación de la plomería EPIC-0 — 3 killables de `update_progress` + 17 ortogonales preexistentes de manejadores MCP/telemetría/boilerplate); DEBT-004 reformulada (infra Canvas ya existe), DEBT-005 ampliada a #1–#14.
- **PROGRESS.md (hecho):** corregida la nota rezagada que decía "#14 sin commitear" (sí está commiteado).
- **Observaciones de QA (hechas):** comentarios desactualizados corregidos por el rename `row_version` (`job_executor.rs:187`) y el nuevo camino de escritura `record_decision` (`mcp_gateway.rs:73`).

---

## 9. Cierre ejecutivo (para el usuario — CEO)

```
ESTADO: 🟢 COMPLETADO

PROGRESO MACRO:
- La base de la aplicación (la parte más antigua, construida antes de que se
  endurecieran las reglas de la base de datos) quedó al mismo nivel de rigor
  que el resto: la integridad referencial de la base de datos ahora SÍ se
  aplica de verdad (antes se aceptaban registros huérfanos en silencio), las
  tablas son estrictas de tipos, y las tres bitácoras que registran trabajo
  concurrente ya no pueden perder un registro bajo carga. Además, las ocho
  piezas del sistema de cobro que tenían un hueco de pruebas quedaron
  blindadas contra ese modo de falla. Todo verificado con la herramienta de
  mutación (que sabotea el código a propósito para comprobar que las pruebas
  lo atrapan): cero huecos.

FRICCIONES Y DEUDA:
- Queda un hueco de PRUEBAS (no de correctitud) en la plomería base: algunos
  manejadores y rutas de lectura no tienen prueba que ejercite cada línea. No
  puede corromper datos —lo crítico ya está probado— y se paga barato en la
  misma tanda de endurecimiento. Está anotado para no perderse.
- El diagnóstico destapó varios documentos de diseño desactualizados respecto
  al código (nombres, catálogos, una decisión de modelado de base de datos).
  No son errores de código; los reuní para que el arquitecto decida.

INPUT REQUERIDO DEL CEO:
- Dos cosas, ninguna urgente: (1) autorizar los commits de esta remediación
  (agrupados por tipo); (2) cuando quieras, invocar al arquitecto con el
  paquete de decisiones que preparé (documentos de diseño a reconciliar +
  una decisión sobre pruebas basadas en propiedades + la FK de identidad).
```
