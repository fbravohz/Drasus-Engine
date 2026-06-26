# ROADMAP de Implementación — Drasus Engine

**Versión:** 3.1 | **Fecha:** 2026-06-19

> **Qué es este documento:** una **guía de orden de implementación** — qué módulo se construye, en qué orden y por qué. Nada más.
>
> **Qué NO es (ADR-0118):** no es una bitácora ni un registro de estado detallado. El "cómo" se hizo cada cosa y sus resultados **no viven aquí**: viven en las Órdenes de Trabajo de [`docs/execution/`](./execution/) y en los **sellos de implementación** que cada Feature/Módulo lleva en su propio documento. Aquí solo hay un estado simple por entrega: `pendiente` · `en curso` · `terminado`.

---

## 1. Cómo leer este ROADMAP

- **Unidad de entrega = un módulo completo (ADR-0118).** Cada fase libera el 100% del **núcleo** de su módulo, no una selección de piezas. La fuente de verdad de qué TTRs componen un módulo es su tabla "TTRs Etiquetados por Fase" dentro del propio `docs/modules/<módulo>.md`.
- **Una Feature se construye una sola vez**, en el primer módulo que la usa en el pipeline (`ingest → generate → validate → incubate → manage → execute → feedback → withdraw`). Los módulos posteriores solo la **integran** (enchufan su puerto), no la reconstruyen.
- **La vanidad está fuera del núcleo:** la UI unificada va a EPIC-8 (ADR-0117) y el R&D no validado a `moonshots/`/EPIC-9+ (ADR-0103). Por eso "módulo completo" no obliga a construir adornos antes de llegar al dinero.
- **Vocabulario:** los identificadores internos (`EPIC-n`, `STORY-###`, `TTR`, `ADR-XXXX`) están traducidos en `.claude/skills/base/SKILL.md` ("Habla en Cristiano"). Aquí: **EPIC = fase/gran bloque**, **STORY = trabajo con código**, **SPIKE = investigación de un riesgo técnico**, **TASK = trabajo sin código**.

---

## 2. Principio de orden (Alpha-First)

El orden de los módulos busca acortar la distancia entre el código y el dinero real lo antes posible. Alpha-First decide **el orden de los módulos** y justifica los **splits por dependencia dura** — ya **no** se usa para escoger piezas dentro de un módulo (eso era la fragmentación que ADR-0118 elimina).

| Categoría | Qué es | Cuándo se construye |
|---|---|---|
| **Alpha Directo** | Produce u opera estrategias (datos, motor, generación, validación, ejecución) | El núcleo del pipeline, EPIC-1 a EPIC-5 |
| **Alpha Defensivo** | Protege capital en riesgo (pre-trade, watchdog, kill switch, prop-firm) | Acoplado a la fase que pone dinero en riesgo (EPIC-5/6) |
| **Multiplicador** | Acelera el ciclo de descubrimiento (tests incrementales, daemons, ciclo 24/7) | EPIC-4 a EPIC-7 |
| **Vanidad / Confort** | UX avanzada, visualizadores, ZUI completa, veredictos LLM | EPIC-8 |
| **Moonshots** | R&D no validado y monetización de terceros | EPIC-9+, según ROI demostrado |

---

## 3. Mapa de entregas (la guía)

```
EPIC-0 Fundación → EPIC-1 ingest → EPIC-2 validate(núcleo) → EPIC-3 generate → EPIC-4 validate(guantelete)
                                                                          ↓
EPIC-8 ZUI ← EPIC-7 feedback+withdraw ← EPIC-6 manage+execute(nativo) ← EPIC-5 PRIMER DINERO REAL
```

| Orden | Entrega | Módulo(s) | Qué desbloquea | Estado |
|---|---|---|---|---|
| EPIC-0 | Fundación y Spikes | infra transversal | Esqueleto compilable + riesgos resueltos | 🟡 en curso |
| EPIC-1 | Soberanía de Datos | `ingest` | Data lake limpio y auditado | pendiente |
| EPIC-2 | Motor de Backtest | `validate` (núcleo) | Backtest determinista y confiable | pendiente |
| EPIC-3 | Generación | `generate` | Miles de candidatas/día | pendiente |
| EPIC-4 | Guantelete de Robustez | `validate` (guantelete) | Estrategias operables (Score ≥75) | pendiente |
| EPIC-5 | **Primer Dinero Real** | `incubate` + `execute` (bridge + REST/FIX nativo) | Estrategia viva en cuenta real vía bridge (MT5/cTrader) y conectores nativos REST/FIX (forex/CFD/futuros) | pendiente |
| EPIC-6 | Portafolio y Ejecución Nativa Profunda | `manage` + `execute` (LiveNode) | Flota multi-estrategia + stack nativo profundo (LiveNode, todos los brókeres) | pendiente |
| EPIC-7 | Ciclo Cerrado 24/7 | `feedback` + `withdraw` | Fábrica autónoma que se mejora sola | pendiente |
| EPIC-8 | Canvas [Forge/Reactor — TBD] y Pulido | UI | Dashboard + canvas unificado + inspector panels + empaquetado (ADR-0136, ADR-0117) | pendiente |
| EPIC-9+ | Moonshots | según ROI | Colmena, Marketplace, SaaS, Copy-Trading | pendiente |

**Splits por dependencia dura (ADR-0118):** `validate` se entrega en dos fases (núcleo de backtest en EPIC-2, necesario antes de generar; guantelete completo en EPIC-4). `execute` igual: el bridge (MT5/cTrader) y los conectores nativos REST/FIX (forex/CFD/futuros) son **co-prioritarios en EPIC-5** para llegar al dinero rápido; el stack nativo profundo (LiveNode de todos los brókeres, incl. cripto, + ejecución a nivel portafolio) llega en EPIC-6. Son las únicas particiones de módulo permitidas, y cada una está justificada por una dependencia citable.

---

## 4. Detalle por entrega

Cada ficha enlaza a la tabla de TTRs del módulo (fuente de verdad del alcance) y declara el criterio de salida. El estado por TTR vive en las Órdenes de Trabajo, no aquí.

### EPIC-0 — Fundación y Spikes de Riesgo

**Objetivo:** esqueleto del workspace hexagonal (ADR-0137) compilando, con las fundaciones anti-retrabajo (ADR-0020 V2) y los 6 gates de viabilidad resueltos.

**Alcance:** workspace Cargo hexagonal (ADR-0137): `shared` (catálogo de tipos + plomería de infraestructura), `crates/features/<dominio>/` (un crate hexagonal por feature de dominio — vacío al cierre de EPIC-0, se puebla desde EPIC-1), `presets/` (cableado sin lógica), `app` (binario) y `bridge` (FFI). Las 6 features de Fundación viven en `shared` como infraestructura crosscutting bendecida (ADR-0137 enmienda 2026-06-23), no como crates propios. Migración 0001 con la tabla ancla `foundation_master_fields` (catálogo de 25 campos, ADR-0020 V2); features transversales de plomería [`clock`](./features/clock.md), [`audit-log`](./features/audit-log.md), [`telemetry`](./features/telemetry.md), [`async-job-executor`](./features/async-job-executor.md) (ADR-0011), [`worker-isolation-orchestrator`](./features/worker-isolation-orchestrator.md), [`agentic-mcp-gateway`](./features/agentic-mcp-gateway.md) (núcleo del servidor MCP + evaluador de permisos, ADR-0123); CLI con Clap + binario raíz `app` (SAD §4.2); Panel Operativo Fundacional (SPIKE-006/ADR-0117).

> **Nota de secuenciación (ADR-0118):** la recuperación post-crash que EPIC-0 exige (job que sobrevive a un cierre y se recupera <10s) la cubre [`async-job-executor`](./features/async-job-executor.md). La feature [`crash-recovery`](./features/crash-recovery.md) (reconciliación de una sesión de trading en vivo contra el bróker, ADR-0027) **no** es de Fundación: pertenece a `execute`/EPIC-5, porque necesita el conector de bróker y el Event Store que no existen hasta entonces.
>
> **Nota de secuenciación (ADR-0123):** el núcleo de [`agentic-mcp-gateway`](./features/agentic-mcp-gateway.md) (servidor MCP + evaluador de permisos para `ingest`/`generate`/`validate`/`incubate`/`feedback`, abiertos por defecto) se construye en EPIC-0 como el resto de la plomería transversal. El permiso condicionado por `institutional_tag` sobre `manage` y el bloqueo de `execute`/`withdraw` se activan naturalmente cuando esos módulos existen (EPIC-5/6) — no requieren trabajo adicional, son la misma evaluación de permisos aplicada a fronteras que aún no existían. El flujo de aceptación de términos de riesgo en modo SaaS depende de `saas-gateway` y llega con EPIC-9+.

**Criterio de salida:** `cargo test` verde en el esqueleto; la migración 0001 aplica los 25 campos; un job asíncrono sobrevive a un `kill -9` y se recupera; los 6 gates SPIKE tienen veredicto en ADR y, de los que aplican a Fundación, su residual está ejercido y verificable en EPIC-0 — smoke de compilación para SPIKE-001 (NautilusTrader) y SPIKE-006 (`flutter_rust_bridge`), erradicación grep-verificable para SPIKE-002 (`tch`/`libtorch`) y SPIKE-003 (`pysr`/`python`); la medición de desempeño del motor (SPIKE-004) se ejerce en EPIC-2 y la validación del LLM/Verdict Engine (SPIKE-005) en EPIC-8, porque esos motores no existen en Fundación (ver §6); el Panel Operativo Fundacional muestra en vivo clock, cola de trabajos y bitácora de auditoría (primera Cáscara Delgada, ADR-0117).

**Estado de las entregas de EPIC-0:**

| Entrega | Estado | Orden de Trabajo |
|---|---|---|
| STORY-001 — Esqueleto Cargo (8 módulos + `shared`) | terminado | [STORY-001](./execution/STORY-001-skeleton.md) |
| STORY-002 — Migración 0001 (25 campos) | terminado | [STORY-002](./execution/STORY-002-migration.md) |
| STORY-003 — `clock` (timestamps deterministas) | terminado | [STORY-003](./execution/STORY-003-clock.md) |
| STORY-004 — `audit-log` (hash chain) | parcial (TTR-002 → EPIC-2+) | [STORY-004](./execution/STORY-004-audit-log.md) |
| STORY-005 — `async-job-executor` (cola durable + recuperación) | terminado | [STORY-005](./execution/STORY-005-async-job-executor.md) |
| STORY-007 — `telemetry` (buffer + heartbeat) | parcial (TTR-002 → EPIC-7) | [STORY-007](./execution/STORY-007-telemetry.md) |
| STORY-008 — `worker-isolation-orchestrator` | terminado | [STORY-008](./execution/STORY-008-worker-isolation-orchestrator.md) |
| TASK-011 — Enmienda ADR-0003: tabla única por feature + TTRs de integración vs construcción | pendiente | [TASK-011](./execution/TASK-011-persistencia-reutilizacion-feature.md) |
| STORY-009 — CLI Clap + binario raíz `app` | ✅ terminado | [Orden](./execution/STORY-009-cli-app.md) |
| STORY-010 — `agentic-mcp-gateway` (núcleo MCP + evaluador de permisos) | 🟡 parcial (TTR-001 UI + TTR-004 SaaS → futuras épicas) | [Orden](./execution/STORY-010-agentic-mcp-gateway.md) |
| STORY-014 — Smoke test NautilusTrader v2 crates (cierra SPIKE-001) | ✅ terminado | [Orden](./execution/STORY-014-nautilus-smoke-test.md) |
| STORY-015 — Panel Operativo Fundacional (cierra SPIKE-006, primera Cáscara Delgada Flutter) | ✅ terminado | [Orden](./execution/STORY-015-panel-operativo-fundacional.md) |
| STORY-016 — Tema dinámico (énfasis + paleta + panel de ajustes) | ✅ terminado (color de fuente entregado por STORY-020) | [Orden](./execution/STORY-016-drasus-theme.md) |
| STORY-017 — Cáscara del Tablero (Bento + registro de widgets) | 🟡 parcial (widgets reales → por épica) | [Orden](./execution/STORY-017-dashboard-shell.md) |
| STORY-018 — Cáscara del Lienzo (DAG interactivo + lista de features) | 🟡 parcial (nodos reales → por épica) | [Orden](./execution/STORY-018-canvas-shell.md) |
| STORY-019 — Centralización del Design System (ADR-0138) | 🟡 parcial (migración → STORY-021) | [Orden](./execution/STORY-019-design-system.md) |
| STORY-020 — Contrato de tokens extensible (modos N, color de fuente, borde=énfasis, espaciado) | ✅ terminado (QA APTO, contrato congelado) | [Orden](./execution/STORY-020-token-contract.md) |
| STORY-021 — Estandarización total de la biblioteca de componentes (4 lotes + bugs de interacción) | ✅ terminado (QA APTO, build verde, cobertura 100%) | [Orden](./execution/STORY-021-component-standardization.md) |

> **Biblioteca de Componentes UI + Sistema de Tema (registro retroactivo, 2026-06-25):** el tema dinámico, las cáscaras de Tablero/Lienzo, el design system y la galería de ~160 componentes se construyeron de forma ad-hoc entre commits **sin Story ni Orden de Trabajo** (deuda de gobernanza). STORY-016 a 019 los registran retroactivamente con su estado real; STORY-020 extiende el contrato de tokens (tema extensible, ADR-0138 enmienda 2026-06-25) y STORY-021 estandariza toda la biblioteca y corrige bugs de interacción. La galería es biblioteca de producción: piezas reutilizables hoy sin lógica; la lógica se enchufa por épica.

> **Nota de reestructura (ADR-0137, 2026-06-23):** el esqueleto original de STORY-001 eran 8 crates de módulo vacíos (`ingest`, `generate`, …). El giro a arquitectura hexagonal los **demolió**: el módulo dejó de ser dueño runtime y pasó a ser preset de composición. El workspace hoy es `shared` + `crates/features/<dominio>/` (vacío al cierre de EPIC-0) + `presets/` + `app` + `bridge`. STORY-001 sigue "terminada" como hito histórico, pero su artefacto fue sustituido por la nueva estructura; el código de Fundación (que vivía en `shared`, no en los crates de módulo) sobrevivió intacto y verde. Pendiente documental progresivo: añadir `## Puertos de Integración` (ADR-0137) a las 6 features de Fundación.

### EPIC-1 — Soberanía de Datos (`ingest`)

**Objetivo:** datos en los que se puede confiar. Sin esto, todo backtest es ficción.

**Alcance:** el 100% del núcleo de `ingest`. Ver su tabla de TTRs: [`docs/modules/ingest.md`](./modules/ingest.md#ttrs-etiquetados-por-fase). Incluye anti look-ahead (`data-validator` + `pit-data-validator`), sanitización de 6 capas (ADR-0037), persistencia Hive/Parquet + DuckDB (ADR-0035/0036), descarga híbrida Bulk+Delta (ADR-0034) de 2 fuentes (prioridad de clase de activo: una Forex/CFD primero; una cripto como segunda fuente), transformación Polars (ADR-0105), barras algorítmicas, diferenciación fraccional y microestructura histórica (CVD, parte histórica del split de [`order-flow-microstructure`](./features/order-flow-microstructure.md), ADR-0118); almacén PIT de eventos fundamentales ([`fundamental-event-store`](./features/fundamental-event-store.md): ingesta de hecho crudo con linaje de proveedor + versionado vintage/as-of first-print vs revisiones, ADR-0126/0127).

**Criterio de salida:** un comando CLI descarga, sanitiza y particiona 5+ años de 2 símbolos; el PIT validator rechaza un dataset con leakage inyectado a propósito; una consulta DuckDB de remuestreo responde <200ms.

### EPIC-2 — Motor de Backtest (`validate` núcleo)

**Objetivo:** el corazón del sistema y el generador de Alpha #1. Un backtest determinista, con fricción institucional, en el que se confía ciegamente. **Aquí se gana o se pierde el proyecto.**

**Alcance:** la mitad "núcleo" de `validate` (ver su tabla de TTRs, fase EPIC-2 en [`docs/modules/validate.md`](./modules/validate.md#ttrs-etiquetados-por-fase)): motor de backtest dual (ADR-0114), fricción institucional (`slippage-models`, Bar-Open Alignment, ADR-0017), métricas duales (ADR-0047), `equity-curve-tracker`, `precision-sizing-models` (ADR-0044), `executable-container` (ADR-0009), `strategy-versioning` (ADR-0005). El compilador AST y el `design-manifest` se construyen aquí porque el motor los necesita, aunque pertenezcan a `generate` (primer consumidor real es el backtest). El motor soporta **N posiciones concurrentes** independientes por señal fresca con de-duplicación por vela (ADR-0129) y es **agnóstico a la temporalidad**: scalping, intradía, swing, posición y ticks son ciudadanos de primera clase (ADR-0130).

**Criterio de salida:** reproducibilidad bit-a-bit (2 corridas, mismo hash); la ruta Express híbrida es medible y más rápida que MT5/SQX/QuantConnect en igual hardware (ADR-0114); paridad documentada contra una plataforma de referencia.

### EPIC-3 — Generación (`generate`)

**Objetivo:** la fábrica de candidatas. Con un motor confiable, el volumen de exploración ES el Alpha.

**Alcance:** el 100% del núcleo de `generate` (ver [`docs/modules/generate.md`](./modules/generate.md#ttrs-etiquetados-por-fase)): `nsga2-optimizer` nativo, AST + WildCards (ADR-0043), `databank-lake`/`databank-manager` (ADR-0055), `dsr-tracking-engine` desde la primera corrida (ADR-0067), `parameter-optimization`, `zero-crossing-filter`, detección de régimen [`hmm-regime-detection`](./features/hmm-regime-detection.md) (modelo ajustado offline + etiquetado; EPIC-1 dejó la columna `regime_label='desconocido'` como placeholder válido, SAD §11); capa de indicadores fundamentales: [`event-impact-scorer`](./features/event-impact-scorer.md) (Event Study + Surprise, ADR-0125), [`asset-exposure-map`](./features/asset-exposure-map.md) (relevancia evento→activo por vector de exposición, ADR-0128) y [`fundamental-indicator-projector`](./features/fundamental-indicator-projector.md) (indicador estándar normalizado por instrumento, ADR-0128); frecuencia y horizonte de operación como objetivo declarable en NSGA-II con mínimo de trades configurable y agnosticismo de temporalidad (ADR-0130).

**Criterio de salida:** una corrida nocturna produce ≥10K candidatas evaluadas con métricas en el Databank; una consulta DuckDB filtra el top 1% en <1s; el N global queda registrado e inmutable.

### EPIC-4 — Guantelete de Robustez (`validate` guantelete)

**Objetivo:** separar suerte de Alpha. Solo sobrevive lo operable. Este filtro es donde Drasus gana a SQX.

**Alcance:** la mitad "guantelete" de `validate` (fase EPIC-4 en su tabla de TTRs): cascada LIGHT/MEDIUM/HEAVY con fail-fast (ADR-0066), `walk-forward-analyzer`, `monte-carlo-simulator` (CPU/SIMD, ADR-0061), `cpcv-analyzer` (PBO, ADR-0063), `cross-market-validation`, `prop-firm-grader` (prioridad absoluta, ADR-0045), `robustness-score-aggregator` (ADR-0058), `statistical-inference-ebta`, `incremental-test-engine` (ADR-0060).

**Criterio de salida:** el pipeline EPIC-3→EPIC-4 corre desatendido y emite estrategias con Score ≥75 + certificación prop-firm; los resultados se heredan entre versiones (test incremental verificado).

### EPIC-5 — PRIMER DINERO REAL (`incubate` + `execute`: bridge + conectores nativos REST/FIX prioritarios)

**Objetivo:** la fase que justifica el proyecto. Se llega al mercado por **dos vías co-prioritarias**: el [`multiplatform-execution-bridge`](./features/multiplatform-execution-bridge.md) para plataformas que lo exigen (MT5 — cuentas de fondeo — y cTrader, ADR-0078) y los **conectores nativos REST/FIX** ([`broker-connector`](./features/broker-connector.md)) para brókeres de forex/CFD/futuros que los ofrecen. Las cuentas de fondeo (MT5) siguen siendo el camino más corto al primer dólar. El stack nativo profundo (LiveNode de todos los brókeres + ejecución a nivel portafolio) y los conectores de cripto se difieren a EPIC-6.

**Alcance:** `incubate` completo (cuarentena de 7 días con `paper-trader` + `incubation-manager`, Eutanasia Predictiva y Cono de Silencio, ADR-0088; `pardo-comparison`) y la mitad "bridge" de `execute` (fase EPIC-5 en [`docs/modules/execute.md`](./modules/execute.md#ttrs-etiquetados-por-fase)): defensa innegociable antes del primer trade — `pre-trade-validator` (ADR-0025 + veto ADR-0095), `order-fsm` (ADR-0004/ADR-0129: N posiciones concurrentes independientes con de-duplicación de señal), `advanced-trade-management` (ADR-0081/ADR-0129), `operational-safety-monitor` (SSL+Pardo, ADR-0070), Shadow Watchdog + Kill Switch (ADR-0026/0087), `sovereign-security` (ADR-0093), `crash-recovery` (ADR-0027), `persistent-daemons` (ADR-0084) + `data-bus-pubsub` (ADR-0085). **Conectores de mercado co-prioritarios:** el bridge ([`multiplatform-execution-bridge`](./features/multiplatform-execution-bridge.md), MT5 + cTrader) y los conectores nativos REST/FIX para forex/CFD/futuros ([`broker-connector`](./features/broker-connector.md), su TTR-002 ya está en EPIC-5). El wrapper de reglas de portafolio en modo Challenge con asignación manual (ADR-0079) se construye aquí porque las cuentas de fondeo lo exigen. *Dependencia a vigilar (ADR-0107):* el `broker-connector` se apalanca en los adaptadores de NautilusTrader; EPIC-5 entrega el camino REST/FIX para los brókeres prioritarios, el kernel LiveNode completo (multi-bróker + portafolio) es EPIC-6.

**Criterio de salida:** una estrategia generada por Drasus pasa cuarentena de 7 días, opera una cuenta de fondeo demo→real vía bridge (o un bróker de forex/CFD vía conector nativo REST/FIX), el SSL y el kill switch se prueban con simulacro de fallo, y la reconciliación diaria cuadra. **KPI de negocio: primera semana verde con dinero real gestionado 100% por el sistema.**

### EPIC-6 — Portafolio y Ejecución Nativa (`manage` + `execute` nativo)

**Objetivo:** pasar de 1–3 estrategias a una flota con gestión de capital seria, y completar el **stack nativo profundo** (LiveNode) para todos los brókeres (incl. cripto).

**Alcance:** `manage` completo (ver [`docs/modules/manage.md`](./modules/manage.md#ttrs-etiquetados-por-fase): `portfolio-optimizer` con HRP, ADR-0075/0089; `portfolio-backtest` con margen compartido, ADR-0091; `signal-correlation-analyzer`; Auto-Rebalancing con circuit breaker) y la mitad **nativa profunda** de `execute`: integración del **LiveNode de los crates NT v2 (ADR-0107)** para todos los brókeres restantes (incl. cripto) y la ejecución a nivel portafolio — los conectores nativos REST/FIX de forex/CFD/futuros ya se entregaron en EPIC-5; guardia de microestructura en vivo OFI/DOM (parte viva del split de order-flow, ADR-0118); `volatility-stabilization` (ADR-0068).

**Criterio de salida:** portafolio de ≥3 estrategias descorrelacionadas operando con pesos HRP, rebalanceo automático auditado, y ≥1 broker nativo con paridad sim/live medida.

### EPIC-7 — Ciclo Cerrado 24/7 (`feedback` + `withdraw`)

**Objetivo:** convertir la herramienta en fábrica autónoma. El sistema se mejora solo mientras duermes.

**Alcance:** `feedback` completo (ver [`docs/modules/feedback.md`](./modules/feedback.md#ttrs-etiquetados-por-fase): `trade-reconciler`, `pardo-comparison` como veredicto de continuidad, `anomaly-detector`, Learning Constraints hacia `generate`, ADR-0015) y `withdraw` completo (`performance-monitor`, retiro con pausa reversible, archivo institucional). Más los daemons de QuantOps (`quantops-daemon`, ADR-0052; `event-driven-pipeline-triggers`) y la Matriz Microrodante Nocturna (ADR-0059).

**Criterio de salida:** 30 días de operación desatendida — el sistema generó, validó, incubó, promovió y retiró estrategias solo, con audit trail completo y cero intervenciones de emergencia.

### EPIC-8 — Canvas [Forge/Reactor — TBD] y Pulido (ADR-0136, ADR-0117)

**Objetivo:** desde EPIC-0, cada Feature con superficie UI entrega su Cáscara Delgada junto a su backend. EPIC-8 ya **no** construye la interfaz desde cero: **unifica** las Cáscaras Delgadas en el Dashboard + canvas [Forge/Reactor] (ADR-0136), implementa el sistema de card-nodes con puertos tipados (ADR-0137), los inspector panels de features, la vista de módulo (mixer + graph view) y los visualizadores que requieren datos agregados de varias fases (UMAP, parallel-coordinates, time-warp, etc.) o lo diferido por veredicto SPIKE-005 (LLM). Incluye el empaquetado comercial (ADR-0029).

**Criterio de salida:** Dashboard + canvas unificado operativo, inspector panels de todas las features funcionando, card-nodes con tipos validados, instaladores para los 3 OS.

### EPIC-9+ — Moonshots (post-rentabilidad, ADR-0103)

Orden sugerido por ROI esperado, revisable con datos reales: transpilador MQL5 (ADR-0101) → Copy-Trading + SaaS gateway → Deep Learning/DRL → La Colmena (ADR-0086) + Marketplace (ADR-0099) → SaaS Cloud (ADR-0033) → Operación Distribuida Edge/Control ([`distributed-edge-execution`](./moonshots/distributed-edge-execution.md), ADR-0119 — compuerta Client Zero, requiere SaaS Cloud previo) → resto del catálogo según evidencia.

---

## 5. Dependencias duras (no negociables)

1. **EPIC-2 depende de EPIC-1:** sin PIT validator, el backtest miente.
2. **EPIC-3 depende de EPIC-2:** generar sin motor confiable es fabricar overfitting a escala.
3. **El DSR (EPIC-4) depende de contar N desde EPIC-3:** orden inverso = estadística inválida.
4. **EPIC-5 depende del Score de EPIC-4:** el sizing en vivo consume el robustness score (ADR-0058) y el veto pre-trade consume el veredicto Monte Carlo (ADR-0095).
5. **Versionado (ADR-0005) y los 25 campos (ADR-0020 V2) nacen en EPIC-0/EPIC-2:** retrofitearlos cuesta 10x.
6. **El mecanismo NautilusTrader está resuelto (ADR-0107):** crates Rust v2 vendorizados. EPIC-0 solo confirma el smoke test; si falla, se activa el Plan B escalonado del ADR-0107 antes de diseñar EPIC-2.

---

## 6. Estado de los Spikes de Viabilidad

Los 6 gates ya tienen veredicto documentado como ADR. En EPIC-0 solo resta la validación residual (smoke tests, mediciones). El detalle de cada veredicto vive en su ADR — aquí solo el estado.

| Spike | Riesgo | Veredicto (ADR) | Estado |
|---|---|---|---|
| SPIKE-001 | NautilusTrader como crate Rust | ADR-0107 | ✅ cerrado (STORY-014, `nautilus-model =0.58.0` compila, QA APTO 2026-06-21) |
| SPIKE-002 | `tch-rs`/libtorch | ADR-0112 (erradicado; escalera `ndarray`→`candle`→`burn`) | ✅ cerrado para EPIC-0 (erradicación verificada: `tch`/`libtorch` ausentes del workspace, grep 2026-06-23; el build nativo de ML llega en su épica) |
| SPIKE-003 | PySR "en Rust" | ADR-0113 (erradicado; simbólico nativo en NSGA-II) | ✅ cerrado para EPIC-0 (erradicación verificada: `pysr`/`python` ausentes del workspace, grep 2026-06-23; el simbólico nativo se construye en EPIC-3) |
| SPIKE-004 | Motor de backtest dual | ADR-0114 (Express híbrido + Event-Driven) | veredicto en ADR; **validación de desempeño diferida a EPIC-2** (es su criterio de salida — el motor no existe en EPIC-0) |
| SPIKE-005 | Ollama/LLM local | ADR-0115 (Verdict Engine determinista; LLM opcional) | veredicto en ADR; **validación residual diferida a EPIC-8** (el Verdict Engine/LLM no existe en EPIC-0) |
| SPIKE-006 | `flutter_rust_bridge` a escala | ADR-0116 + ADR-0117 (Panel Operativo Fundacional) | ✅ cerrado (STORY-015, `flutter build linux` verde, QA APTO 2026-06-21) |

---

## 7. KPIs por Fase (jerarquía de latencias)

| Ruta | SLA | Fase donde se mide |
|---|---|---|
| Pre-trade validation (Guardián) | <1ms | EPIC-5 |
| Wrapper de reglas de portafolio | <10ms | EPIC-5/6 |
| Orden end-to-end (señal→broker) | ≤100ms | EPIC-5/6 |
| Kill switch / watchdog | ≤5s | EPIC-5 |
| Backtest Express híbrido | Más rápido que MT5/SQX/QuantConnect en igual hardware (ADR-0114) | EPIC-2 |
| Recuperación post-crash | <10s | EPIC-0 (cola de trabajos) / EPIC-5 (sesión live) |
| Carga UI Time-Warp | <200ms | EPIC-8 |
