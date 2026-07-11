# Expedition Ledger — Instancias de Ejecución Inmutables y Linaje de Procedencia

**Carpeta:** `./features/expedition-ledger/`
**Estado:** En Diseño
**Última actualización:** 2026-07-11
**Decisión Arquitectónica Asociada:** ADR-0150 (Expedition — Ledger de Ejecución + Linaje + Pipeline Versionado) · **ADR-0151 (modelo de aplicación del N por punto de decisión — esta feature registra el INSUMO, no aplica el N)** · ADR-0067 (DSR — insumo N+σ²+sketch) · ADR-0005 (versionado del artefacto, complementario) · ADR-0141 (conformidad de esquema)

---

## ¿Qué es esta feature?

El Expedition Ledger es la **columna vertebral de procedencia** del sistema: registra cada **corrida** de un Pipeline como una entidad inmutable (una *Expedition*) y ata cada corrida a los artefactos que tocó, con la **naturaleza del toque**. Responde preguntas que hoy no tienen dueño: ¿cuántas corridas lleva este pipeline?, ¿en qué corrida nació o cambió esta estrategia?, ¿qué corrida trajo los mejores resultados?

**Problema que resuelve:** hoy se persisten los artefactos producidos (estrategias, portafolios, clusters) pero **no la corrida que los produjo**. Sin la corrida como entidad, no hay histórico, no hay conteo, y — crítico — el **N total de pruebas** que el Deflated Sharpe Ratio (ADR-0067) necesita para descontar el sesgo de selección queda desconocido, inflando el DSR. **La Expedition es el sistema de registro del presupuesto de pruebas.**

**Qué NO es:** no versiona el artefacto (`strategy-versioning`, ADR-0005) ni la ruta (`pipeline-registry`); no almacena los resultados masivos (`databank-*`, ADR-0055) — los referencia por id. Registra la *corrida* y su *linaje*.

---

## Comportamientos Observables

- [ ] Al lanzar una corrida se crea una Expedition: UUIDv7, snapshot de configuración inmutable, `pipeline_version_hash` de la ruta, ventana de datos/universo/semilla, estado `PENDING`.
- [ ] La Expedition transita `PENDING → RUNNING → DONE/FAILED/CANCELLED`; cada transición es atómica (estado + auditoría en la misma transacción, ADR-0141).
- [ ] **Corren N Expeditions en simultáneo** sobre N pipelines sin interferencia.
- [ ] Cada vez que la corrida toca un artefacto se escribe una fila de linaje append-only con la naturaleza: `CREATED` / `PARAMS_MUTATED` / `PROMOTED` / `DEGRADED` / `RE_VALIDATED` / `DISCARDED`.
- [ ] Una misma estrategia acumula toques de varias Expeditions en el tiempo (nace en A, muta en B, se promueve en C) — muchos-a-muchos temporal.
- [ ] Al finalizar una Expedition de minería, su `trials_count`, `sharpe_variance` (σ²) y **sketch acotado del vector de Sharpe** quedan inmutables como **insumo** — la Expedition declara su **familia/espacio-de-búsqueda** a priori en el snapshot de config. El N *aplicable* a cada selección de `validate` (EPIC-4) lo computa la política de ADR-0151 por punto de decisión desde el linaje, NO es el `trials_count` crudo.
- [ ] El usuario consulta "las mejores corridas de este pipeline" o "el linaje de esta estrategia" con una query sobre el ledger.

## Restricciones

- **NUNCA** una Expedition se borra ni se reescribe; corregir es una corrida nueva o una fila de linaje que enmienda.
- **NUNCA** el snapshot de configuración de una Expedition cambia tras crearse (reproducibilidad bit-a-bit, ADR-0002).
- **NUNCA** el `trials_count` de por vida se usa como "el N" del DSR (guardia de límite degenerado, ADR-0151: N→∞ ⇒ DSR→0 ⇒ condena universal). El N se computa por punto de decisión desde el linaje; el ledger solo registra el **insumo**.
- **NUNCA** N se resetea por añadir barras de datos frescas (portillo de data mining); solo por alcance de inferencia coherente (Expedition + decisión + familia declarada a priori, ADR-0151).
- **NUNCA** existe un contador de N paralelo que pueda divergir (deroga el contador de sesión de `dsr-tracking-engine`).
- **NUNCA** una feature infiere procedencia por su cuenta: el linaje es la única fuente de verdad.

## Parámetros Configurables (ADR-0008)

| Parámetro | Default | Rango | Qué hace | FIJO/CONFIG |
|---|---|---|---|---|
| MAX_CONCURRENT_EXPEDITIONS | 2 | 1 - 8 | Expeditions en ejecución simultánea (alineado con `MAX_PARALLEL_PIPELINES` de los triggers) | CONFIG |
| EXPEDITION_TRIALS_BATCH | 512 | 32 - 8192 | Tamaño de lote para el incremento atómico de `trials_count` desde los workers (baja contención) | CONFIG |

## Estructura Interna (FCIS — ADR-0002)

- **Core (Lógica Pura):** agregación incremental de estadísticas de corrida (media y varianza de Sharpe, algoritmo de Welford — heredado de `dsr-tracking-engine`); validación de transiciones de estado; resolución de qué naturaleza de toque aplica.
- **Shell (Infraestructura):** repositorio SQLite del ledger de Expeditions (mutable con `row_version` + atomicidad estado+auditoría) y del junction de linaje (append-only puro); contador atómico de `trials_count` por lotes.
- **Frontera Pública:** puerto para crear/transicionar una Expedition, para registrar un toque de linaje, y para consultar el histórico y el N de una corrida (consumido por `validate`/DSR).

## Tareas (TTRs)

### TTR-001: Ledger de Expeditions (creación + ciclo de vida atómico)
Crear la Expedition con snapshot inmutable y gestionar sus transiciones de estado con `BEGIN IMMEDIATE` + auditoría atómica (ADR-0141).

### TTR-002: Junction de linaje append-only con naturaleza del toque
Registrar filas Expedition↔Artefacto con `artifact_kind` y `touch_nature` (enums con `CHECK`), muchos-a-muchos temporal, `event_sequence_id UNIQUE`.

### TTR-003: Contador atómico de trials + sketch de Sharpe (insumo del DSR) por lotes
Incrementar `trials_count` y `sharpe_variance` (Welford) por lotes confirmados sin degradar el throughput de backtesting (resiliencia ante fallo de worker), y mantener un **sketch acotado del vector de Sharpe** (reservoir sample / t-digest — pondera volumen: pueden ser millones de trials). Generaliza el `dsr-tracking-engine` (ADR-0067). Es el **insumo** de la política de N de ADR-0151, no el N aplicado.

## Puertos de Integración (ADR-0137)

| Puerto | ID de tipo | Dirección | Cardinalidad | Descripción |
|---|---|---|---|---|
| `pipeline_definition_in` | `PipelineDefinition` (de `pipeline-registry`) | Input | `1` | Versión de ruta que la corrida ejecuta. |
| `backtest_result_in` | `BacktestResult` | Input | `0..N` | Resultados producidos por la corrida, atados por linaje. |
| `metrics_in` | `MetricsDict` | Input | `0..N` | Métricas resumen de la corrida (Sharpe para el N/varianza del DSR). |
| `version_node_in` | `StrategyVersionNode` | Input | `0..N` | Versión de artefacto tocada (estrategia/portafolio), para el linaje. |
| `expedition_out` | `Expedition` (tipo de dominio nuevo — se cataloga en ADR-0137 con color de procedencia al construir el nodo Canvas, patrón progresivo) | Output | `1..N` | Instancia de ejecución inmutable con su estado, N y métricas resumen. |
| `expedition_lineage_out` | `ExpeditionLineageLink` (tipo de dominio nuevo — cableado de Canvas diferido, ídem) | Output | `1..N` | Toque Expedition↔Artefacto con su naturaleza. Consumido por `feedback`, `strategy-versioning`, la UI de linaje. |

> Los nombres canónicos de `struct`/tipo Rust los fija el ingeniero (anti-alucinación, ADR-0144). Los tipos de entrada ya están catalogados (ADR-0137); los de salida son tipos de dominio nuevos cuyo cableado en Canvas se difiere a EPIC-8 (ADR-0136 §Enmienda 2026-06-28). El subsistema (ledger) no depende del Canvas para existir en EPIC-2.

## Cáscara Visual (Thin Shell)

> Pendiente de la Etapa 0.5 (UI-Designer). Superficie prevista: histórico de corridas de un pipeline (tabla ordenable por métrica), vista de linaje de un artefacto (qué Expeditions lo tocaron y cómo), nodo de Canvas de la Expedition. El Architect NO rellena esta sección.

## Gobernanza y Estándares (Fijos)

- **Local-First (ADR-0016):** 100% local (SQLite WAL) para la coordinación atómica del contador y el ledger.
- **Inundación de Fundaciones (ADR-0020):** **Perfil B (IA/R&D)** — la Expedition es el rastro de linaje del descubrimiento. Grupo I completo (6 campos, incl. `event_sequence_id`) + Soberanía (`owner_id`, `manifest_id`) + subset Pesos/Arquitectura (`logic_hash`, `data_snapshot_id`, `version_node_id`, `parent_id`) + Hardware (`node_id`, `process_id`). B ⊇ D aporta el rastro de auditoría.

## Persistencia (Inundación de Fundamentos — ADR-0020)

Dos grupos de tablas: (1) **Expeditions** — identidad y snapshot de config inmutables; estado del ciclo de vida mutable bajo `row_version` con atomicidad estado+auditoría; append-only en el sentido de no-borrado; `event_sequence_id UNIQUE`; `pipeline_version_hash` (FK), `owner_id` (FK a `accounts(id)`), ventana de datos (ns UTC), universo, semilla, `trials_count`, `sharpe_variance`, **campo de familia/espacio-de-búsqueda** (gobierna el reset de N, ADR-0151; mapea a `ACTIVE_GENOME_DOMAINS`), **sketch acotado del vector de Sharpe** (`TEXT`/`BLOB` con `CHECK` de estructura, ADR-0151 §Esquema), métricas resumen monetarias como `INTEGER` ×10⁸. (2) **Linaje** — junction append-only puro (`event_sequence_id UNIQUE`), `expedition_id` (FK `ON DELETE RESTRICT`), referencia al artefacto, `artifact_kind` y `touch_nature` con `CHECK`. Es la fuente para reconstruir el pool comparado (N) de cada punto de decisión, incluida la unión multi-etapa. `STRICT`, PK UUIDv7. Detalle canónico de esquema en ADR-0150 §restricciones, ADR-0151 §Esquema y ADR-0141.

**Rastro de Evidencia:** emite hacia `feedback` la corrida completa (config, N, métricas, linaje) que produjo cada artefacto — la causalidad corrida→resultado que hoy falta.

## Dependencias y Bloqueantes

**Depende de:** [`clock`](../features/clock.md), [`audit-log`](../features/audit-log.md), [`pipeline-registry`](../features/pipeline-registry.md) (versión de ruta), [`central-identity`](../features/central-identity.md) (`owner_id` → `accounts`).
**Consumido por:** módulo [`validate`](../modules/validate.md) (crea la Expedition del backtest y reconstruye el pool/N por punto de decisión desde el linaje para el DSR/PBO, ADR-0151, EPIC-2/EPIC-4), [`generate`](../modules/generate.md) (Expedition de minería, registra insumo N+σ²+sketch, EPIC-3), [`manage`](../modules/manage.md) (toques de portafolio/cluster, EPIC-6), [`feedback`](../modules/feedback.md) (causalidad corrida→resultado).
**Reconciliación:** generaliza el `SessionID`/`mining_sessions` de [`dsr-tracking-engine`](../features/dsr-tracking-engine.md) (ADR-0067); complementario a [`strategy-versioning`](../features/strategy-versioning.md) (ADR-0005 — versiona el artefacto, no la corrida).
