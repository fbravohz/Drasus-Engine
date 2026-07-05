# STORY-032 — Endurecimiento de atomicidad de ledgers append-only (DEBT-001 + DEBT-007)

| Campo | Valor |
|---|---|
| **ID** | STORY-032 |
| **Tipo** | Story (código — hardening dedicado, salda deuda rastreada) |
| **Épica (Fase)** | Cimientos de Monetización (transversal, greenfield) |
| **Sprint** | substrato-monetizacion |
| **Estado** | ✅ Completada (ledgers `audit_events`/`usage_records` con append atómico; DEBT-001 y DEBT-007 saldadas) |
| **Creada** | 2026-07-05 |
| **Deudas que salda** | [DEBT-001](../DEBT.md) (ledgers append-only sin transacción atómica) · [DEBT-007](../DEBT.md) (guarda `OPTOUT_CHANGE`-primera) |
| **ADRs** | ADR-0141 (append-only + `event_sequence_id`) · ADR-0137 (puertos) · ADR-0002/0004 (determinismo) · ADR-0142 (CLI) |

## 1. Objetivo llano

Aplicar **retroactivamente** el patrón de *append atómico* (ya construido y probado en `consent-registry`/STORY-031) a los dos ledgers append-only que se commitearon **antes** de que la regla existiera: `audit_events` (`persistence/audit_log.rs`) y `usage_records` (`persistence/usage_metering.rs`). Hoy asignan `event_sequence_id` con `load_tail` + `INSERT` en **sentencias separadas** sobre `self.pool` → bajo escritura concurrente dos escritores derivan el mismo `event_sequence_id` y uno pierde su evento (fallo seguro, pero pérdida). Además, cerrar **DEBT-007**: una `OPTOUT_CHANGE` como primera acción de un `owner_id` en `consent-registry` debe fallar con error tipado, no depender del efecto colateral `StaleVersion`.

**NO hay cambio de esquema** (las tablas ya tienen `event_sequence_id UNIQUE` + triggers). Todo el arreglo es en el código Rust del Shell. **Sin cambio de comportamiento para un solo escritor** — las 286 pruebas existentes (incl. recuperación kill-9) deben seguir verdes.

## 2. Agentes y Modo de Acompañamiento (ADR-0120/0122)

| Agente | Modelo | Modo |
|---|---|---|
| Rust-Engineer | Sonnet | **Autónomo** |

Autónomo (no Docente): el patrón ya se enseñó en la lección `docs/lessons/rust/STORY-031-consent-registry.md`; esta Story lo **reaplica**. La plantilla canónica es `crates/shared/src/persistence/consent_registry.rs`.

## 3. Gate de Coherencia — resultado (contraste bidireccional)

- **Plantilla canónica (copiar el patrón, no inventar):** `persistence/consent_registry.rs` (STORY-031) ya tiene la forma correcta: `record_action` = bucle de reintento acotado (`MAX_*_ATTEMPTS`) que delega en `try_*_once`, donde `self.pool.begin_with("BEGIN IMMEDIATE")` envuelve las lecturas + el `INSERT`; `is_transient_write_conflict` (mensajes "database is locked"/"database table is locked" o `UNIQUE` sobre `event_sequence_id`); error tipado `WriteContention { attempts }` al agotar; `busy_timeout` ya vive en `persistence/pool.rs` (compartido). **Replica esa forma exacta** en los dos ledgers.
- **`audit_log.rs::append` (línea ~88):** hoy hace `load_tail(self.pool)` + `INSERT execute(self.pool)` sueltos. Envuélvelos en UNA transacción `BEGIN IMMEDIATE` (leer cola DENTRO de la tx) + reintento. Añade `AuditLogError::WriteContention`.
- **`usage_metering.rs::record_operation` (línea ~171):** OJO — aquí el read-then-write es **más ancho**: lee la cola global (`load_tail`) **y** la acumulación del ciclo (`.fetch_one` de la suma de `notional_per_op` del `(owner_id, billing_cycle_id)`), y luego INSERTA con `cycle_accumulated`. **Las DOS lecturas + el INSERT** deben ir dentro de la MISMA transacción `BEGIN IMMEDIATE`, o dos operaciones concurrentes del mismo dueño derivarían el mismo acumulado (además del mismo `event_sequence_id`). Reintento + `UsageRepositoryError::WriteContention`.
- **DEBT-007 (`consent-registry`):** en `domain::consent_registry::apply_consent_action` (o en la Shell `try_record_action_once`), una acción `OptoutChange` sin estado previo (primer evento del dueño) debe producir un **error tipado** (p. ej. `ConsentActionError::OptoutBeforeAccept`) en vez de caer al efecto colateral `accepted_version=""` → `StaleVersion`. Mantén la función total donde aplique, pero la Shell rechaza la secuencia inválida antes de persistir. NO rompas las pruebas existentes de `consent-registry` (las que hacen `OPTOUT_CHANGE` siempre lo hacen tras un `ACCEPT`).
- **Sin migración:** el esquema ya es correcto (append-only, `UNIQUE`, triggers). NO toques `migrations/`.
- **Sin regresión:** las 286 pruebas de `shared` (incl. `kill9_recovery` de `crates/app`) deben seguir verdes. El cambio es transparente para un solo escritor.
- **Determinismo (ADR-0002/0004):** reloj inyectado, sin `SystemTime`. El orden lo fija `event_sequence_id`, no el reloj.

## 4. Instrucciones de despacho (prompt exacto)

> Eres el **Rust-Engineer** en **Modo Autónomo**. Lee primero: `CLAUDE.md`, `.claude/skills/base/SKILL.md`, `.claude/skills/rust-engineer/SKILL.md` (en especial §4 "Atomicidad de ledgers append-only"), esta Orden, la lección `docs/lessons/rust/STORY-031-consent-registry.md`, y la **plantilla canónica** `crates/shared/src/persistence/consent_registry.rs`. Declara que los leíste.
>
> **Construye (reaplica el patrón de append atómico; sin cambio de esquema):**
> 1. **`crates/shared/src/persistence/audit_log.rs`:** refactoriza `append` para envolver `load_tail` + `INSERT` en una transacción `BEGIN IMMEDIATE` (`self.pool.begin_with("BEGIN IMMEDIATE")`), con bucle de reintento acotado ante conflicto transitorio y error tipado `AuditLogError::WriteContention { attempts }`. Copia `is_transient_write_conflict` de la plantilla.
> 2. **`crates/shared/src/persistence/usage_metering.rs`:** refactoriza `record_operation` para envolver **las dos lecturas** (cola global + acumulación del ciclo `(owner_id, billing_cycle_id)`) **y** el `INSERT` en una sola transacción `BEGIN IMMEDIATE` + reintento + `UsageRepositoryError::WriteContention`. Crítico: la acumulación se lee DENTRO de la tx.
> 3. **DEBT-007 en `consent-registry`:** haz que una `OPTOUT_CHANGE` como PRIMER evento de un `owner_id` devuelva un error tipado explícito (no el efecto colateral `StaleVersion`). Ubícalo donde no rompa las pruebas existentes.
>
> **Pruebas discriminantes (rojo→verde, deben poder fallar):**
> - **Concurrencia `audit_events`:** `#[tokio::test(flavor = "multi_thread")]`, BD en archivo temporal (NO `:memory:`), N≥16 escritores en paralelo llamando `append`. Afirma: (a) las N filas persistidas (ningún evento perdido), (b) `event_sequence_id` = 1..=N densos sin huecos/duplicados, (c) cadena `audit_chain_hash` íntegra + recomputable. Debe caerse si quitas la transacción.
> - **Concurrencia `usage_records`:** idem, N escritores del **mismo dueño y ciclo** llamando `record_operation`. Afirma además que `cycle_accumulated` de la última fila == suma exacta de todos los `notional_per_op` (ninguna acumulación pisada). Debe caerse sin la transacción.
> - **DEBT-007:** `OPTOUT_CHANGE` sin `ACCEPT` previo → error tipado (assert del variante). Debe caerse si se permite silenciosamente.
> - **No regresión:** `cargo test -p shared` (286) y `cargo test -p app` (kill-9) verdes.
> - Cobertura con `cargo llvm-cov --workspace --summary-only`.
>
> **Comentarios en español, identificadores en inglés (ADR-0121).** Gate: `cargo test --workspace` verde + `cargo clippy --workspace --all-targets -- -D warnings` cero warnings.
>
> **NO toques `migrations/`. NO toques estos 6 archivos (otro Architect los está editando):** `docs/ROADMAP.md`, `docs/features/licensing-system.md`, `docs/features/usage-metering.md`, `docs/features/central-identity.md`, `docs/README.md`, `docs/moonshots/encrypted-local-backup.md`. **NO hagas commits. NO toques archivos personales de la raíz.** Al terminar reporta: diffs conceptuales de `append` y `record_operation`, salida de las pruebas de concurrencia nuevas, y `cargo test --workspace` + `clippy`.

## 5. Criterio de aceptación (verificable)

| # | Criterio | Prueba |
|---|---|---|
| 1 | `audit_log::append` atómico (BEGIN IMMEDIATE + reintento + WriteContention) | inspección + prueba de concurrencia |
| 2 | `usage_metering::record_operation` atómico incluyendo la acumulación del ciclo | inspección + prueba de concurrencia (acumulado exacto) |
| 3 | DEBT-007: `OPTOUT_CHANGE`-primera → error tipado | test discriminante |
| 4 | Sin cambio de esquema (migraciones intactas) | `git status migrations/` limpio |
| 5 | Sin regresión: 286 shared + kill-9 verdes | `cargo test --workspace` |
| 6 | Clippy cero warnings + cobertura | `cargo clippy` + `cargo llvm-cov` |

## 6. Comandos de validación (usuario)

```bash
cargo test -p shared
cargo test -p app
cargo clippy --workspace --all-targets -- -D warnings
```

## 7. Registro de ejecución

- 2026-07-05 · Tech-Lead · Gate corrido. Reaplica el patrón de `consent_registry.rs` (STORY-031) a `audit_log`+`usage_metering`; en usage_metering la tx debe envolver también la lectura de acumulación del ciclo. DEBT-007 = error tipado para `OPTOUT_CHANGE`-primera. Sin migración, sin regresión. Orden creada, despacho a Rust-Engineer (Sonnet, Autónomo).
- 2026-07-05 · Rust-Engineer (Autónomo) · Implementado EN VERDE: `audit_log.rs::append` → bucle `try_append_once` con `begin_with("BEGIN IMMEDIATE")` + `MAX_APPEND_ATTEMPTS`/`AuditLogError::WriteContention`; `usage_metering.rs::record_operation` → `try_record_operation_once` con las DOS lecturas (suma del ciclo + cola global) **y** el INSERT dentro de la tx + `UsageRepositoryError::WriteContention`; `consent_registry.rs` DEBT-007 → guarda `OptoutBeforeAccept` tras leer el estado previo (antes de fusionar). 4 pruebas nuevas (2 de concurrencia con BD en archivo temporal, 2 de DEBT-007). Sin tocar `migrations/`. **Falsación empírica:** revertir solo el cuerpo transaccional a sentencias sueltas tumba ambas pruebas de concurrencia con `WriteContention{attempts:5}` — el reintento solo NO basta. `cargo test -p shared` 290, `cargo test -p app` kill-9 ok, `cargo test --workspace` verde, clippy cero warnings.
- 2026-07-05 · Tech-Lead · Auditoría independiente (reproducida): 290 tests verdes (4 nuevas por nombre); `usage_metering::try_record_operation_once` con `begin_with("BEGIN IMMEDIATE")` (l.225) + `fetch_one(&mut *tx)` acumulación (l.238) + `fetch_optional(&mut *tx)` cola (l.257) + `execute(&mut *tx)` (l.307) — las dos lecturas y el INSERT en la MISMA tx; `audit_log::append` con reintento+`WriteContention`; guarda `OptoutBeforeAccept` (l.308). Clippy limpio; solo 3 archivos tocados; migraciones y los 6 archivos del Architect intactos. Verde.
- 2026-07-05 · QA-Engineer (Sonnet) · **APTO.** Lógica línea por línea + mutaciones (revertidas byte a byte): quitar `BEGIN IMMEDIATE` en audit_log → cae `concurrent_appends...`; quitarlo en usage_metering → cae `concurrent_record_operations...`; **mover SOLO la lectura de acumulación fuera de la tx → TAMBIÉN cae** (cero hueco de cobertura); deshabilitar la guarda → cae `optout_change_as_first_action...` sin falso positivo en `optout_change_after_accept_still_succeeds`. 290 shared + kill-9 verdes, clippy limpio. Observación no bloqueante: cobertura `audit_log.rs` 84.87% (líneas sin cubrir = boilerplate de error + rama de agotamiento de reintentos + `load_tail` directo), **simétrica a la plantilla `consent_registry.rs` ya aceptada en STORY-031** — el criterio crítico (atomicidad) sí está cubierto y es discriminante por mutación. Árbol intacto.
- 2026-07-05 · Tech-Lead · Gate QA cerrado con APTO. **STORY-032 completada.** **DEBT-001 y DEBT-007 → Pagadas** en `docs/DEBT.md`.

## 8. Deudas / diferidos registrados

- Al cerrar en verde con QA APTO: mover **DEBT-001** y **DEBT-007** a "Pagadas" en `docs/DEBT.md` con enlace a esta Orden.
