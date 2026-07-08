# DEBT.md — Registro Canónico de Deuda Técnica Rastreada

> **Propósito:** el **único lugar descubrible** donde vive la lista de deudas técnicas conocidas y **deliberadamente aplazadas**. Una deuda rastreada es lo opuesto a deuda oculta: está escrita con su causa, su impacto y su disparador de pago. En greenfield es sano — permite avanzar en el camino crítico sin frenar por cosas que aún no muerden, **siempre que** queden registradas aquí.
> **Relación con `PROGRESS.md`:** la bitácora del Tech-Lead (`.claude/state/tech-lead/PROGRESS.md`) *narra* cuándo se halló una deuda; **este archivo es el registro canónico**. Las entradas de PROGRESS apuntan a los `DEBT-XXX` de aquí. Si una deuda no está aquí, no está rastreada.
> **Mantenimiento:** el Tech-Lead añade/cierra filas al abrir o pagar una deuda. Estado: `Abierta` · `En pago` · `Pagada` (con enlace a la Story que la saldó).

## Convención de severidad

| Severidad | Significado |
|---|---|
| 🔴 Alta | Puede corromper datos o violar un invariante bajo condiciones alcanzables; pagar pronto. |
| 🟠 Media | Fallo seguro (sin corrupción) pero con pérdida de función o correctitud bajo condiciones aún no presentes; pagar antes de que la condición llegue. |
| 🟡 Baja | Cosmético / diferido por decisión de secuenciación; sin riesgo de correctitud. |

---

## Deudas abiertas

### DEBT-001 · Ledgers append-only sin transacción atómica ni reintento
- **Severidad:** 🟠 Media
- **Origen:** observación de QA en STORY-030 (`usage-metering`); patrón preexistente desde `audit_log` (EPIC-0).
- **Descripción:** los ledgers append-only asignan `event_sequence_id` con `SELECT MAX(...)+1` e `INSERT` en **sentencias separadas**, sin envolverlas en una transacción `BEGIN IMMEDIATE`. Bajo escritura concurrente, dos escritores pueden derivar el mismo `event_sequence_id`; el `UNIQUE` rechaza a uno (fallo seguro) pero **el evento perdedor se pierde** si no hay reintento.
- **Impacto actual:** nulo — SQLite serializa escritores a nivel de archivo, el motor es local/monoproceso, y los tests corren monohilo sobre `:memory:` (nunca ejercen concurrencia). Se vuelve real con jobs concurrentes (`async-job-executor`, ejecución de varias estrategias).
- **Causa raíz (instrucciones):** el skill `rust-engineer` exigía los invariantes de tamper-evidence (UNIQUE, triggers, hash chain) pero **no** exigía (a) atomicidad transaccional en *read-then-write*, ni (b) prueba de 2 escritores. Vacío de plantilla, no descuido del agente.
- **Disparador de pago / plan:**
  1. Regla permanente en skills `rust-engineer` + `qa-engineer` (transacción `BEGIN IMMEDIATE` + `busy_timeout` + reintento acotado; prueba de 2 escritores obligatoria en todo ledger). → **hecho 2026-07-04**.
  2. `consent-registry` (#5) nace correcto (arreglado en STORY-031 antes de cerrar).
  3. **STORY-032 de endurecimiento** para los ledgers ya commiteados (`audit_log` #0002, `usage_records` #0010), con su propio QA. Recomendado: entre #5 y #6.
- **Estado:** ✅ **Pagada** — [STORY-032](./execution/STORY-032-ledger-atomicity-hardening.md) (2026-07-05). Los 3 puntos del plan completados: regla permanente en skills (2026-07-04), `consent-registry` (#5) nació correcto (STORY-031), y `audit_events`+`usage_records` endurecidos con append atómico (`BEGIN IMMEDIATE` + reintento + `WriteContention`), QA APTO por mutación (quitar la transacción tumba las pruebas de concurrencia).

### DEBT-002 · `PlanLimits` duplicado (stub sellado de #2 vs. real de #3)
- **Severidad:** 🟡 Baja
- **Origen:** STORY-029 (`plan-tier-quota`).
- **Descripción:** conviven dos `PlanLimits` en namespaces distintos — el stub sellado dentro de `licensing-system` (#2) y el real de `plan-tier-quota` (#3).
- **Impacto actual:** ninguno de correctitud (namespaces separados); es deuda de unificación.
- **Disparador de pago:** mini-Story de re-cableado de #2 (toca código sellado → exige su propio QA).
- **Estado:** Abierta.

### DEBT-003 · Gaps de backend de `sovereign-data-fetcher`
- **Severidad:** 🟡 Baja
- **Origen:** STORY-024.
- **Descripción:** (G1) no existe submit de job en background → la SVF usa await/spinner; (G2) `sovereign_download_records` (migr. 0006) no guarda `symbol`/`bytes_total`/`status`; (G3) el estado `retrying` no existe en `JobState`.
- **Impacto actual:** limita la SVF del fetcher, no la correctitud del backend.
- **Disparador de pago:** editar el baseline SQL es barato en greenfield → se pagan en la auditoría retroactiva o al cablear el job en background.
- **Estado:** Abierta.

### DEBT-004 · Nodo Canvas del `sovereign-data-fetcher` no construido
- **Severidad:** 🟡 Baja
- **Origen:** STORY-024 (Opción B, manifestación #3).
- **Descripción:** el nodo Canvas + inspector lateral del fetcher no se construyó porque la infraestructura Canvas aún no existe.
- **Disparador de pago:** al construir la infra Canvas (decisión del usuario: al inicio de la auditoría retroactiva). De ahí en adelante toda feature entrega sus 3 manifestaciones. Reordena el ROADMAP → escalar al Architect (ADR-0117/0136).
- **Estado:** Abierta.

### DEBT-005 · Tanda de UI final del substrato (SVF + galería + harness genérico)
- **Severidad:** 🟡 Baja (autorizada, backend-first)
- **Origen:** decisión del usuario 2026-07-04.
- **Descripción:** los backends de los cimientos #1–#9 se verifican por CLI (Canal #2); su **SVF (Canal #1) + componentes de galería con mocks** se construyen en UNA tanda al final, que incluye: (a) el **harness SVF genérico** (una vez, obligatorio — nadie arma SVF a medida); (b) la SVF retroactiva de #1–#9; (c) el **arreglo de la SVF del `sovereign-data-fetcher`** (hoy dice "descargado" pero no muestra la respuesta del servidor).
- **Disparador de pago:** al cerrar los backends del substrato.
- **Estado:** Abierta.

### DEBT-006 · Auditoría de Inundación de Fundaciones en los 41 moonshots
- **Severidad:** 🟡 Baja
- **Origen:** cierre de la auditoría de features.
- **Descripción:** falta aplicar la misma auditoría de perfiles ADR-0020 a los 41 moonshots.
- **Disparador de pago:** TASK futura, fuera del camino crítico.
- **Estado:** Abierta.

### DEBT-008 · `enriched-domain-events` (#6) sin fan-out al bus (ADR-0085)
- **Severidad:** 🟡 Baja
- **Origen:** STORY-033 (`enriched-domain-events`).
- **Descripción:** el cimiento #6 persiste cada evento en el event-store append-only local (atómico), pero **no lo publica al bus** (ADR-0085). Hoy la Shell escribe a la tabla; el `event_out` fan-out a los consumidores vivos (medición, agregación, reportes) no está cableado.
- **Impacto actual:** nulo — los consumidores del substrato (#4, #7, #9) leen del event-store por puerto, no del bus; el bus solo se necesita para reacción en tiempo real (feedback, telemetría).
- **Disparador de pago:** al cablear el primer consumidor que exija push en vivo (módulo `feedback` / telemetría), o al construir el bus como infraestructura. Distinto del **adaptador de red a la Cabina de Mando**, que vive en el ROADMAP (no aquí).
- **Estado:** Abierta.

### DEBT-009 · Placeholders de tipos del guantelete en #7 (`BacktestResult`/`RobustnessScore`)
- **Severidad:** 🟡 Baja
- **Origen:** STORY-034 (`institutional-report-engine`).
- **Descripción:** `institutional-report-engine` (#7) consume `BacktestResult`/`RobustnessScore` que hoy son **placeholders** (`pub struct X;` en `types/mod.rs`); el reporte se arma con un input mínimo (`metrics: BTreeMap<String,i64>`). La firma es reproducible y correcta, pero el mapeo desde los tipos **reales** del guantelete de validación/ejecución no existe todavía porque esos tipos aún no están construidos.
- **Impacto actual:** ninguno de correctitud — el puerto y la firma son estables; es un mapeo pendiente, no un bug.
- **Disparador de pago:** cuando el guantelete produzca los tipos reales (EPIC de validación/ejecución), mapear `result_in` → `metrics` sin tocar la firma.
- **Estado:** Abierta.

### DEBT-010 · Render Tera→PDF/HTML no cableado en #7
- **Severidad:** 🟡 Baja
- **Origen:** STORY-034 (`institutional-report-engine`).
- **Descripción:** #7 produce hoy la **estructura firmada** del reporte (JSON canónico + `signature_hash`); el render a PDF/HTML con plantilla Tera (ADR-0101) y white-label **no** se añadió (Tera no está como dependencia). El catálogo de productos (stress/validación/forense/certificación) es un moonshot aparte (`institutional-report-products`), NO esta deuda.
- **Impacto actual:** ninguno — el dato firmado y trazable ya existe; falta la capa de presentación.
- **Disparador de pago:** al primer cliente que pida el documento renderado, o al construir el moonshot de productos de reporte.
- **Estado:** Abierta.

### DEBT-011 · Huecos de cobertura en `third-party-api-gateway` (#8)
- **Severidad:** 🟡 Baja (test-coverage, no correctitud)
- **Origen:** gate de QA con mutación en STORY-035 (10 mutantes sobrevivientes).
- **Descripción:** dos rutas del cimiento #8 no las ejercita ninguna prueba: (a) la **ruta de reintento** del append atómico (`is_transient_write_conflict` + contador `attempt` + `WriteContention`) — como `pool.rs` usa `journal_mode=WAL` + `busy_timeout=5s`, los escritores concurrentes esperan el lock en vez de recibir "database is locked", así que el reintento es código vivo pero sin prueba que fuerce contención real; (b) en `revoke`, ningún test compara el `audit_hash` **recalculado** post-revocación contra su valor esperado (solo se verifica `audit_chain_hash`), así que si `revoke` dejara de recalcular el hash, no se cazaría.
- **Impacto actual:** nulo de correctitud — la propiedad crítica (ningún evento se pierde bajo concurrencia real, con transacción atómica) SÍ está probada y la prueba discrimina la implementación correcta (quitar `BEGIN IMMEDIATE` la tumba 3/3). Es cobertura faltante, no un bug.
- **Disparador de pago:** test de contención forzada (mock de `SQLITE_BUSY`) que ejercite reintento exitoso y agotamiento (`WriteContention`), + aserción de `audit_hash` cambiado en `revoke`. Barato; se pliega a la tanda de endurecimiento de tests o a la auditoría retroactiva.
- **Estado:** Abierta.

### DEBT-012 · Huecos de cobertura en `data-aggregation` (#9)
- **Severidad:** 🟡 Baja (test-coverage / diseño placeholder, no correctitud)
- **Origen:** gate de QA con mutación en STORY-036 (18 mutantes sobrevivientes).
- **Descripción:** tres huecos: (a) **fórmula del ruido Box-Muller no fijada** — 10 mutantes sobre la matemática de `apply_differential_privacy` (`*`→`+`/`/`, `+`→`-`) sobreviven porque los tests aseguran determinismo + que difiere del crudo + que la semilla participa, pero **no** el valor exacto con un test de valor-dorado; (b) **ruta de reintento no ejercitada** — `is_transient_write_conflict` + los límites del loop de `record_index` no se prueban (WAL+`busy_timeout` hace esperar, no fallar) — **mismo problema sistémico que DEBT-011**; (c) **hash de topología calculado y descartado** — `run_aggregation` hace `let _ = hash_strategy_topology(raw)` pero no persiste ninguna columna de topología (es un placeholder de la disciplina ADR-0102 hasta que la topología sea una dimensión real de agregación).
- **Impacto actual:** nulo de correctitud — la propiedad crítica (ningún índice se pierde, append atómico) SÍ está probada (la mutación manual de `BEGIN IMMEDIATE` tumba la prueba de 16 escritores); k-anonimato, consentimiento, canales y guardarraíl de datos crudos SÍ están cazados. Es cobertura/diseño placeholder.
- **Disparador de pago:** (a) test de valor-dorado del ruido con semilla fija; (b) test de contención forzada (compartido con DEBT-011, `SQLITE_BUSY` simulado); (c) al cablear la topología como dimensión real (persistir `topology_hash`) o eliminar el cálculo muerto.
- **Estado:** Abierta.

### DEBT-013 · Huecos de cobertura en `verified-account-registry` (#10)
- **Severidad:** 🟡 Baja (test-coverage, no correctitud)
- **Origen:** gate de QA con mutación en STORY-037 (16 mutantes sobrevivientes de 118).
- **Descripción:** tres grupos: (a) **ruta de reintento** del append atómico (`is_transient_write_conflict` + límites del loop de `record_track_record`) no ejercitada — **mismo problema sistémico que DEBT-011/012** (WAL+`busy_timeout`); (b) **campos del struct de retorno** de `update_publication_and_scopes` (`updated_at_ns`, `attestation_scopes`) sin aserción tras el update (misma naturaleza que el hueco de `revoke` en DEBT-011); (c) **bordes de `compute_track_record`**: el filtro cross-cuenta del `AccountSnapshot` (`p.account_id == account_id`) no se prueba con snapshots de otra cuenta; `>`→`>=` en drawdown/win-rate sobrevive (no hay trade de PnL cero que distinga "empate" de "ganado"); el signo del capital-base de respaldo (`total_deposits - total_withdrawals`) no se prueba con un retiro presente en la rama sin snapshot.
- **Impacto actual:** nulo de correctitud — el diferenciador (gain% excluye flujo de capital), la inviolabilidad de ámbitos, el gate de consentimiento y la atomicidad SÍ están cazados (84/118 + la mutación manual de `BEGIN IMMEDIATE` tumba la prueba de 16 escritores). Son bordes y la ruta de reintento sistémica.
- **Disparador de pago:** (a) compartido con DEBT-011/012 (test de contención forzada con `SQLITE_BUSY` simulado); (b) aserción de los campos del struct tras `update`; (c) tests de borde: snapshot de otra cuenta ignorado, trade de PnL cero (no cuenta como win), capital-base con retiro en la rama sin snapshot. Todos baratos.
- **Estado:** Abierta.

### DEBT-014 · Retrabajo de #10 — falta el Eje B (realidad de capital) en STORY-037
- **Severidad:** 🟠 Media (modelado incompleto; una vez exista la superficie de publicación, mostrar un track sin su realidad de capital induce a error — ADR-0145 corregido lo declara "tan grave como confundir el Eje A").
- **Origen:** corrección de ADR-0145 por el Architect (2026-07-06): la atestación pasó de "dos ámbitos" a **dos ejes ortogonales** — Eje A (autoría: `SOVEREIGN`/`BROKER_READONLY`) × Eje B (realidad de capital: `LIVE`/`PAPER`/`DEMO`/`CHALLENGE`). El código de STORY-037 solo modeló el Eje A.
- **Descripción:** una cuenta en `PAPER`/`CHALLENGE` corre en el **mismo entorno determinista** que `LIVE` (NO es backtesting) y **sí es atestiguable** — solo con capital virtual, y nunca se presenta sin esa etiqueta. Falta: (1) añadir el campo del Eje B a `verified_accounts` y `attested_track_records` (migración `0016` es greenfield → editable in situ, o migración nueva); (2) el rótulo de publicación siempre muestra **ambos ejes juntos**; (3) test discriminante nuevo ("una cuenta `PAPER` se atestigua con firma, pero jamás se muestra sin la etiqueta de capital virtual"); actualizar la firma reproducible y el `audit_hash` para incluir el Eje B; parámetro `CAPITAL_MODES` (CONFIG).
- **Impacto actual:** nulo hoy (greenfield, sin superficie de publicación viva — DEBT-005); se vuelve real en cuanto se publique/renderice un track.
- **Disparador de pago:** Story de retrabajo dedicada (toca código sellado de #10 → exige su propio QA), **antes** de construir la superficie de publicación. Anclado por el Architect en el ROADMAP y en la Feature `verified-account-registry.md` (banner 🔶).
- **Estado:** ✅ **Pagada** — [STORY-038](./execution/STORY-038-verified-account-capital-reality.md) (2026-07-06): enum `CapitalReality` (Eje B) ortogonal al Eje A, en `verified_accounts` y `attested_track_records` (migración `0016` in situ + CHECK); firma reproducible y ambos `audit_hash` encadenan el Eje B; proyección de puerto expone `capital_reality` + `is_real_capital` (derivado SOLO del Eje B) siempre junto al Eje A; discriminante `SOVEREIGN`+`PAPER` (atestado pero capital virtual) en 3 capas. Auditoría TL aprobada + QA APTO por mutación (80/92 cazados; Eje B 100% cazado; 6 sobrevivientes = bordes de DEBT-013, no nuevos).

### DEBT-015 · #11 `instance-continuity` — `canonical_delta_bytes` sin test de valor-dorado
- **Severidad:** 🟡 Media (completitud de datos, no seguridad; impacto nulo hoy por greenfield + adaptador de subida diferido, pero un respaldo vacío silencioso sería pérdida de datos al restaurar).
- **Origen:** QA por mutación de STORY-039 (2026-07-06): 3 sobrevivientes, todos `canonical_delta_bytes -> Vec<u8>` reemplazado por `vec![]`/`vec![0]`/`vec![1]` — los tests no fijan la salida EXACTA de la serialización canónica del delta.
- **Descripción:** `compute_backup_delta` (el filtro que EXCLUYE secretos) sí está cazado; lo que falta es un test de valor-dorado sobre `canonical_delta_bytes` que ancle los bytes exactos producidos, de modo que un defecto que devolviera bytes triviales/vacíos (respaldo sin contenido) se detecte. Análogo a DEBT-012 (Box-Muller sin valor-dorado).
- **Impacto actual:** nulo (fase greenfield; el adaptador de subida S3/R2 está diferido, no hay respaldo real aún — STORY-039 §8).
- **Disparador de pago:** añadir el test de valor-dorado **antes** de construir el adaptador de almacén de objetos (antes de que exista un respaldo real que pudiera salir vacío).
- **Estado:** Abierta.

### DEBT-016 · #10 `verified-account-registry` — columna `capital_reality` duplica `institutional_tag` (violación de Inundación de Fundaciones)
- **Severidad:** 🔴 Alta (violación de invariante FIJO — "reutilización antes que creación", ADR-0144/ADR-0020; dos columnas con el mismo dominio de valores en la misma fila).
- **Origen:** auditoría de Inundación de Fundaciones del Architect (2026-07-07), ratificada en ADR-0145 (banner 🔶) y SAD-22 §22.6. STORY-038 implementó el Eje B como columna nueva `capital_reality` (`LIVE`/`PAPER`/`DEMO`/`CHALLENGE`) coexistiendo con `institutional_tag` (Grupo II, ya presente en ambas tablas de `0016` por Perfil D, poblado con el placeholder `"DRASUS_LOCAL"`).
- **Descripción:** el Eje B NO es un campo nuevo — es `institutional_tag` (Grupo II — "Environment") con su vocabulario extendido a `LIVE`/`PAPER`/`DEMO`/`CHALLENGE`. Retrabajo: eliminar `capital_reality` de `verified_accounts` y `attested_track_records` (migración `0016` in situ, greenfield), añadir `CHECK` a `institutional_tag`, y ajustar `domain/verified_account_registry.rs` (`CapitalReality`/`is_real_capital` interpretan `institutional_tag`).
- **Impacto actual:** nulo de comportamiento (el Eje B funciona; QA APTO en STORY-038) — pero es una violación arquitectónica que ensuciaría el esquema y el vocabulario canónico si se congela a BROWNFIELD. Debe corregirse en greenfield.
- **Disparador de pago:** ahora, antes de commitear #10 (para que git no registre la violación). Toca código sellado → Story dedicada + QA.
- **Estado:** ✅ **Pagada** — [STORY-041](./execution/STORY-041-verified-account-eje-b-consolidation.md) (2026-07-07): columna `capital_reality` eliminada de ambas tablas (migración `0016` in-situ); `institutional_tag` gana el `CHECK IN ('LIVE','PAPER','DEMO','CHALLENGE')`; `CapitalReality` conservado como intérprete del campo canónico; validación fail-typed `UnknownInstitutionalTag` + dos tests-guardarraíl anti-regresión. Auditoría TL aprobada + QA APTO por mutación (80/92; 6 sobrevivientes = bordes de DEBT-013, ninguno nuevo — cobertura preservada exactamente).

### DEBT-017 · #3 `plan-tier-quota` — falta la cuota `MAX_CHILD_ACCOUNTS`
- **Severidad:** 🔶 Media (extensión de catálogo pendiente; bloquea a #12/#14 para gatear la creación de cuentas hijas).
- **Origen:** ADR-0149 (cimiento #14 `operator-roles`, 2026-07-07), banner 🔶 en `plan-tier-quota.md`. El código de STORY-029 no incluye este límite.
- **Descripción:** añadir `MAX_CHILD_ACCOUNTS` al catálogo de planes — cuántas cuentas maestras hijas puede crear un fondo bajo `master-account-hierarchy` (#12), fijado **solo por el propietario de Drasus** según el tier, nunca por el fondo. Mismo mecanismo que `NOTIONAL_LIMIT`/activaciones máximas — un campo más del plan, no infraestructura nueva.
- **Impacto actual:** nulo hoy (la creación de cuentas hijas — #12/#14 — aún no gatea contra cuota); necesario antes de #14.
- **Disparador de pago:** ahora, junto con el retrabajo de #10, antes de commitear el substrato. Toca código sellado de #3 → Story dedicada + QA.
- **Estado:** ✅ **Pagada** — [STORY-042](./execution/STORY-042-plan-tier-quota-max-child-accounts.md) (2026-07-07): `max_child_accounts` añadido al catálogo (migración `0009` in-situ + `CHECK >= 0`), `PlanCandidate`/`PlanSnapshot`/`PlanLimits` + sellado en `compute_plan_audit_hash`, sembrado por tier (FREE=`0`/PAID=`5`), expuesto en CLI. `0` válido, `< 0` rechazado. Subagente se estancó a la mitad; el TL completó el patrón. Auditoría TL + QA APTO por mutación (36/39; **0 sobrevivientes**).

### DEBT-007 · `OPTOUT_CHANGE` como primera acción sin guarda explícita
- **Severidad:** 🟡 Baja (falla-seguro)
- **Origen:** observación de QA en STORY-031 (`consent-registry`).
- **Descripción:** `apply_consent_action`/`try_record_action_once` no impiden explícitamente que una `OPTOUT_CHANGE` sea el **primer** evento de un `owner_id` (sin `ACCEPT` previo). Si ocurriera, `accepted_version` queda `""`.
- **Impacto actual:** inofensivo — `needs_reacceptance("", vigente)` es siempre `true` → el veredicto cae a `StaleVersion` (niega), nunca a `Covered`. Falla-seguro, no viola GDPR.
- **Disparador de pago:** añadir una guarda explícita con error tipado (en vez de depender del efecto colateral) → plegado al alcance de **STORY-032**.
- **Estado:** ✅ **Pagada** — [STORY-032](./execution/STORY-032-ledger-atomicity-hardening.md) (2026-07-05): guarda tipada `ConsentRepositoryError::OptoutBeforeAccept` que rechaza `OPTOUT_CHANGE` como primer evento antes de fusionar/persistir.

### DEBT-018 · Cobertura de mutación del patrón de ledger append-only en cimientos previos a #13
- **Severidad:** 🟠 Media (sin corrupción de datos; hueco de resiliencia/reporte de error bajo concurrencia real).
- **Origen:** medición de `cargo-mutants` durante el cierre de #13 (`data-portability`, STORY-043, 2026-07-08). Al matar los 11 sobrevivientes de #13 se confirmó empíricamente que **7 de ellos sobreviven idénticos en #10** (`verified-account-registry`, ya cerrado/commiteado).
- **Descripción:** el patrón "ledger append-only atómico" (`is_transient_write_conflict`, el bucle de reintento `record_*`, y la proyección de la fila devuelta en tablas mutables con `row_version`) está **calcado** en varios cimientos. Los tests actuales de esos cimientos **no matan** tres clases de mutante:
  1. **Clasificador de contención** (`is_transient_write_conflict` → `true`/`false`; `||`→`&&`; `&&`→`||`): sin test unitario directo que le pase un error de "database is locked" real y una violación UNIQUE PERMANENTE (PK, no `event_sequence_id`).
  2. **Bucle de reintento** (`attempt += 1`→`*=`; `attempt < MAX`→`==`/`>`/`<=`): sin test de **contención sostenida** que agote `MAX_RECORD_ATTEMPTS` y afirme `WriteContention { attempts: MAX }`.
  3. **Fidelidad de la fila devuelta** (borrado de campo en la proyección de `reclassify`/`update_*` de tablas mutables): sin assertions sobre la fila que la función DEVUELVE (solo se verifica lo persistido).
- **Alcance a auditar (EPIC-0):** confirmado en **#10**; por herencia del patrón calcado, previsiblemente en **#5** (`consent-registry`), **#6** (`enriched-domain-events`), **#9** (`data-aggregation`), **#11** (`instance-continuity`), **#12** (`master-account-hierarchy`), y los ledgers endurecidos por STORY-032 (`audit_log`, `usage_records`). Cada uno se mide y se cierra a 0 survivors.
- **Impacto actual:** nulo hoy (greenfield, sin carga concurrente real; el núcleo de integridad — `BEGIN IMMEDIATE`, `UNIQUE`, hashes encadenados persistidos — SÍ está cubierto). Muerde bajo concurrencia de producción, que es cuando el modo de falla (rendirse sin reintento, enmascarar un error permanente, o devolver metadato rancio) es más sutil.
- **Patrón de pago (ya validado en #13, STORY-043):** por cada ledger, tres tests deterministas — (1) contención sostenida con segundo escritor reteniendo `BEGIN IMMEDIATE` (`busy_timeout=0`) hasta agotar reintentos; (2) `is_transient_*` directo con violación UNIQUE de PK (no de secuencia); (3) assertions sobre la fila devuelta por la actualización mutable.
- **Disparador de pago:** auditoría retroactiva **EPIC-0** (contraste cimiento por cimiento). Toca código sellado → cada cimiento re-corre su QA por mutación a 0 survivors.
- **Estado:** Abierta.

---

## Deudas pagadas

- **DEBT-001** (ledgers append-only sin transacción atómica) → saldada por [STORY-032](./execution/STORY-032-ledger-atomicity-hardening.md), 2026-07-05.
- **DEBT-007** (`OPTOUT_CHANGE`-primera sin guarda) → saldada por [STORY-032](./execution/STORY-032-ledger-atomicity-hardening.md), 2026-07-05.
- **DEBT-014** (Eje B ausente en #10) → saldada por [STORY-038](./execution/STORY-038-verified-account-capital-reality.md), 2026-07-06.
- **DEBT-016** (columna `capital_reality` duplica `institutional_tag` en #10) → saldada por [STORY-041](./execution/STORY-041-verified-account-eje-b-consolidation.md), 2026-07-07.
- **DEBT-017** (falta cuota `MAX_CHILD_ACCOUNTS` en #3) → saldada por [STORY-042](./execution/STORY-042-plan-tier-quota-max-child-accounts.md), 2026-07-07.

> Nota: DEBT-001, DEBT-007, DEBT-014, DEBT-016 y DEBT-017 se conservan arriba con su ficha completa y Estado ✅ Pagada (para preservar su historia); este índice apunta a la Story que las saldó.
