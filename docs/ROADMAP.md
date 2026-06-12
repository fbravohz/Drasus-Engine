# ROADMAP de Desarrollo — Drasus Engine (ex-QuantForge)

**Versión:** 1.2 | **Fecha:** 2026-06-11 (v1.2: Gates G2–G6 resueltos vía ADR-0112 a 0116; KPI absoluto de throughput eliminado. v1.1: Gate G1 resuelto vía ADR-0107)
**Autor:** Auditoría arquitectónica (Tech Lead)
**Principio rector:** *Alpha-First.* Cada fase debe acortar la distancia entre el código y el dinero generado en mercados reales. Todo lo que no genere Alpha, lo proteja (defensa de capital = Alpha preservado) o lo multiplique (velocidad de descubrimiento), se pospone.

---

## 1. Criterio de Priorización (Alpha vs Vanidad)

| Categoría | Definición | Tratamiento |
|---|---|---|
| **Alpha Directo** | Produce estrategias operables o ejecuta dinero real (backtest, generación, validación, ejecución) | Fases 1–5 |
| **Alpha Defensivo** | Protege capital ya en riesgo (pre-trade checks, SSL, watchdog, prop-firm grader) | Acoplado a la fase que pone dinero en riesgo |
| **Multiplicador** | Acelera el ciclo de descubrimiento (incremental tests, cascada fail-fast, daemons QuantOps) | Fases 4–7 |
| **Vanidad/Confort** | UX avanzada, visualizadores 3D, LLM verdicts, ZUI completa | Fase 8+ |
| **Moonshots** | R&D no validado, monetización de terceros (Colmena, Marketplace, SaaS, Copy-Trading) | Post-Fase 8, según ROI demostrado |

**Regla del Tech Lead:** Una feature entra a una fase solo si su ausencia bloquea el "Entregable Alpha" de esa fase. Lo demás espera, sin excepciones, aunque ya tenga TTR escrito. Los TTRs no se modifican; solo se secuencian.

---

## 2. Gates de Viabilidad Técnica (BLOQUEANTES — resolver en Fase 0)

Estos riesgos invalidaban supuestos centrales de la documentación. **Los 6 gates ya tienen veredicto documentado como ADR (G1: ADR-0107; G2–G6: ADR-0112 a 0116).** Lo que resta en F0 es el trabajo residual de validación de cada veredicto (smoke tests, spikes de medición), no la decisión arquitectónica.

| # | Riesgo | Supuesto en docs | Realidad a verificar | Plan B |
|---|---|---|---|---|
| G1 | **NautilusTrader como crate Rust** — ✅ **RESUELTO (ADR-0107)** | SAD §2.1: "integrado nativamente en el core Rust" | Verificado: el núcleo v2 de NT publica crates Rust puros en crates.io (backtest, modelo, trading, live) que permiten backtesting y ejecución live sin Python. El spike de F0 se reduce a un **smoke test**: compilar los crates vendorizados (versión fijada), correr un backtest mínimo y validar el enlace LGPL reenlazable. | Formalizado en ADR-0107 (en orden): (a) congelar la versión vendorizada estable; (b) fork de mantenimiento mínimo bajo LGPL; (c) motor soberano (`moonshots/sovereign-execution-engine.md`). Descartado: sidecar Python (viola ADR-0104) |
| G2 | **tch-rs (libtorch)** — ✅ **RESUELTO (ADR-0112)** | SAD §2.1: "Laboratorio de IA tch-rs" | tch-rs arrastra libtorch (~2GB C++), rompe la promesa de "binario único sin runtimes" (ADR-0029) y complica el empaquetado en 3 OS. | **Veredicto:** erradicar `tch-rs`. Escalera `ndarray`+Rayon (default) → `candle` si se justifica → `burn` solo en moonshot DRL. Monte Carlo es CPU-first (no es deep learning). |
| G3 | **PySR "en Rust"** — ✅ **RESUELTO (ADR-0113)** | SAD §2.3: "Minería Simbólica (Rust)" + múltiples menciones a PySR | PySR es Python+Julia. No existe puerto Rust. | **Veredicto:** erradicar PySR. La regresión simbólica acotada es un modo del motor NSGA-II nativo sobre el AST; la minería simbólica libre se difiere a moonshot con `egg`. Se rechazan `evalexpr`/`meval` en hot-path. |
| G4 | **Motor de backtest dual** — ✅ **RESUELTO (ADR-0114)** | (KPI absoluto de bars/sec eliminado por humo) | Forzar event-loop tick-a-tick a la minería masiva asfixia la exploración; vectorizar puro impide la gestión de riesgo con estado (ADR-0109). | **Veredicto:** motor dual. Ruta Express híbrida (vectorizado para lo sin-estado + mini-loop secuencial para lo con-estado) + ruta Event-Driven (NT v2) para fidelidad. Modo elegido por el usuario; contrato de consistencia conservadora (FIJO). Criterio relativo: superar a MT5/SQX/QuantConnect. |
| G5 | **Ollama/LLM local** — ✅ **RESUELTO (ADR-0115)** | ADR-0058 exigía LLM local | Ollama es un runtime externo — contradice "cero runtimes" (ADR-0029/0030). | **Veredicto:** Verdict Engine determinista por plantilla, sin LLM por defecto. Ollama derogado como requisito; LLM local soberano (`candle`) opcional. |
| G6 | **flutter_rust_bridge a escala** — ✅ **RESUELTO (ADR-0116)** | ADR-0029/0019 | Verificar streams de alta frecuencia (throttling 100ms) y paso Arrow zero-copy con arrays de 1M+ puntos. | **Veredicto:** downsampling obligatorio en backend (nunca cruzar más resolución que el viewport, ADR-0098). `ZeroCopyBuffer` solo para cargas masivas; throttling en Rust; gRPC fallback (ADR-0033). Spike F0 confirma números. |

**Salida de Fase 0 = los 6 gates con veredicto + ADRs actualizados.** Sin esto, no se escribe código de producción.

---

## 3. Mapa de Fases (Resumen Ejecutivo)

```
F0 Fundación+Spikes → F1 Datos → F2 Motor Backtest → F3 Generación → F4 Guantelete
                                                                          ↓
F8 UI Glass-Box ← F7 Feedback+AutoPipeline ← F6 Manage+Live ← F5 PRIMER DINERO REAL
```

| Fase | Nombre | Módulos | Duración est.* | Entregable Alpha |
|---|---|---|---|---|
| F0 | Fundación y Gates | infra transversal | 4–6 sem | Esqueleto compilable + riesgos resueltos |
| F1 | Soberanía de Datos | `ingest` | 4–6 sem | Data lake limpio y auditado de 2+ fuentes |
| F2 | Motor de Backtest | `validate` (núcleo) | 8–10 sem | Backtest determinista y rápido en quien confiar |
| F3 | Generación | `generate` | 6–8 sem | Miles de candidatas/día en el Databank |
| F4 | Guantelete de Robustez | `validate` (completo) | 6–8 sem | Estrategias con Score ≥75 listas para operar |
| F5 | **Primer Dinero Real** | `incubate` + `execute` (mínimo) | 6–8 sem | **Estrategia viva en cuenta real/fondeo vía bridge MT5** |
| F6 | Portafolio y Ejecución Nativa | `manage` + `execute` (completo) | 8–10 sem | Portafolio multi-estrategia con brokers nativos |
| F7 | Ciclo Cerrado 24/7 | `feedback` + `withdraw` + QuantOps | 4–6 sem | Fábrica autónoma: genera→valida→incuba→opera→aprende |
| F8 | Glass-Box UI | UI completa | 8–12 sem | Editor visual DAG, ZUI, visualizadores |
| F9+ | Moonshots | según ROI | — | Colmena, Marketplace, SaaS, Copy-Trading, etc. |

\* Estimaciones para 1 dev senior + agentes IA, jornada completa. Son relativas: lo importante es el orden y los criterios de salida, no el calendario.

---

## 4. Detalle por Fase

### F0 — Fundación y Gates de Riesgo

**Objetivo:** Esqueleto del monolito modular FCIS compilando, con las fundaciones que evitan retrabajo (ADR-0020 V2), y los 6 gates de viabilidad resueltos.

- Workspace Cargo con los 8 módulos como crates internos + carpeta `shared` (ADR-0003).
- Migraciones SQLx embebidas con el set maestro de 25 campos desde la migración 0001 (ADR-0006, ADR-0020 V2).
- Features transversales P0: [`clock`](./features/clock.md), [`audit-log`](./features/audit-log.md), [`telemetry`](./features/telemetry.md), [`async-job-executor`](./features/async-job-executor.md) (ADR-0011), [`crash-recovery`](./features/crash-recovery.md) (ADR-0027 Event Store), [`worker-isolation-orchestrator`](./features/worker-isolation-orchestrator.md).
- CLI con Clap como primera interfaz (la UI Flutter NO bloquea ninguna fase hasta F8; cada fase expone sus comandos por CLI primero).
- Spike FFI: ventana Flutter mínima + `flutter_rust_bridge` + stream Arrow (Gate G6). Solo "hello world" de infraestructura, cero pantallas de producto.
- Validar los veredictos ya documentados de los Gates G1–G6 (§2). Todos tienen ADR (G1: ADR-0107; G2–G6: ADR-0112 a 0116). En F0 solo queda el trabajo residual: smoke test de compilación/vendoring de los crates NT v2 y empaquetado LGPL (G1); smoke test de cómputo CPU-first `ndarray`/Rayon (G2); spike de medición del motor Express híbrido y el contrato de consistencia (G4); spike FFI con downsampling y throttling (G6).

**Criterio de salida:** `cargo test` verde en esqueleto; migración 0001 aplica los 25 campos; job asíncrono sobrevive a un kill -9 y se recupera (ADR-0011); veredictos G1–G6 documentados y sus validaciones residuales ejecutadas.

---

### F1 — Soberanía de Datos (`ingest`)

**Objetivo:** Datos en los que se puede confiar. Sin esto, todo backtest es ficción.

**P0 (bloquean el entregable):**
- [`data-validator`](./features/data-validator.md) + [`pit-data-validator`](./features/pit-data-validator.md) (anti look-ahead, innegociable).
- [`data-sanitizer-pipeline`](./features/data-sanitizer-pipeline.md) (ADR-0037; pipeline de 6 capas de limpieza).
- [`hive-partition-manager`](./features/hive-partition-manager.md) (ADR-0035) + [`duckdb-sql-engine`](./features/duckdb-sql-engine.md) + [`duckdb-resampler`](./features/duckdb-resampler.md) (ADR-0036).
- Descarga híbrida Bulk+Delta (ADR-0034) para **2 fuentes**: una cripto (Binance Vision) y una Forex/CFD (la del broker/prop firm objetivo). No más fuentes en esta fase.
- [`hybrid-data-transformer`](./features/hybrid-data-transformer.md) (Polars, ADR-0105-datos).

**P1 (si sobra tiempo de fase):** [`background-download-manager`](./features/background-download-manager.md), [`data-import-wizard`](./features/data-import-wizard.md) (CSV manual), [`quality-heatmap-generator`](./features/quality-heatmap-generator.md) (versión CLI/reporte, no UI).

**Se pospone explícitamente:** [`hmm-regime-detection`](./features/hmm-regime-detection.md) (solo se inunda la columna `regime_label` con valor "desconocido" — el SAD §11 ya contempla régimen desconocido como válido), [`algorithmic-bars`](./features/algorithmic-bars.md), [`order-flow-microstructure`](./features/order-flow-microstructure.md), [`manual-regime-tagger`](./features/manual-regime-tagger.md), [`fractional-differencer`](./features/fractional-differencer.md).

**Criterio de salida:** Comando CLI descarga, sanitiza y particiona 5+ años de 2 símbolos; el PIT validator rechaza un dataset con leakage inyectado a propósito (test adversarial); consulta DuckDB de remuestreo 7m responde <200ms.

---

### F2 — Motor de Backtest (`validate` núcleo + features de simulación)

**Objetivo:** El corazón del sistema y el generador de Alpha #1. Un backtest determinista, con fricción institucional, en el que se confía ciegamente. **Aquí se gana o se pierde el proyecto.**

**P0:**
- [`backtest-engine`](./features/backtest-engine.md): arquitectura dual (Gate G4) — ruta vectorizada para minería masiva (Open Prices / 1m OHLC) y ruta event-driven para fidelidad (4-ticks; Real Ticks después). Modos de ADR-0017.
- Fricción institucional mandatoria: [`slippage-models`](./features/slippage-models.md), triple swap, penetración Pardo, Bar-Open Alignment (ADR-0017).
- [`institutional-metrics`](./features/institutional-metrics.md) con implementación dual hot/cold (ADR-0047).
- [`equity-curve-tracker`](./features/equity-curve-tracker.md), [`precision-sizing-models`](./features/precision-sizing-models.md) (ADR-0044 — paridad sizing desde el día uno).
- [`executable-container`](./features/executable-container.md) (ADR-0009) y contrato AST + compilador Serde (sustituye al residuo "Pydantic AST Compiler"; el TTR se conserva, la tecnología es Serde/Rust).
- [`strategy-versioning`](./features/strategy-versioning.md) (ADR-0005 — hash chain; barato ahora, carísimo de retrofitear).

**P1:** [`perfect-profit-benchmark`](./features/perfect-profit-benchmark.md), [`universal-basket-backtester`](./features/universal-basket-backtester.md).

**Se pospone:** [`nautilus-integration`](./features/nautilus-integration.md) completa (el mecanismo ya está resuelto — ADR-0107: crates Rust v2 vendorizados — pero la paridad sim/live se exige recién en F5–F6; en F2 solo se consume la ruta de backtest event-driven), [`institutional-friction-modeling`](./features/institutional-friction-modeling.md) (adverse selection — refinamiento de F4).

**Criterio de salida:** (1) Reproducibilidad bit-a-bit verificada: 2 corridas, mismo hash de resultados. (2) La ruta Express híbrida es medible y demostrablemente más rápida que MT5/SQX/QuantConnect sobre el mismo dataset y hardware (benchmark `criterion` en CI; sin KPI absoluto — ADR-0114). (3) Paridad validada contra una plataforma de referencia (misma estrategia simple en MT5/SQX: diferencias explicables y documentadas).

---

### F3 — Generación (`generate`)

**Objetivo:** La fábrica de candidatas. Con F2 confiable, el volumen de exploración ES el Alpha.

**P0:**
- [`design-manifest`](./features/design-manifest.md) (ADR-0053 — el contrato SMART filtra basura desde el origen).
- [`nsga2-optimizer`](./features/nsga2-optimizer.md) nativo Rust (multi-objetivo Sharpe/DD/WR) con decimación y renovación sanguínea (SAD §2.3).
- AST + WildCards (ADR-0043) — el humano fija el esqueleto, el motor resuelve comodines: este es el modo de generación con mejor ratio esfuerzo/alpha.
- [`databank-lake`](./features/databank-lake.md) + [`databank-manager`](./features/databank-manager.md) (ADR-0055 — semillas Parquet, no AST masivos).
- [`dsr-tracking-engine`](./features/dsr-tracking-engine.md): registrar $N$ intentos desde la PRIMERA corrida (ADR-0067). Si no se cuenta N desde el inicio, el DSR de F4 nace inválido. Costo trivial, valor estadístico enorme.
- [`parameter-optimization`](./features/parameter-optimization.md) + [`zero-crossing-filter`](./features/zero-crossing-filter.md).

**P1:** [`bayesian-optimizer`](./features/bayesian-optimizer.md), [`fit-to-portfolio-search`](./features/fit-to-portfolio-search.md) (cobra valor real cuando ya hay portafolio vivo — puede deslizarse a F6), [`hmm-regime-detection`](./features/hmm-regime-detection.md) (entra aquí como filtro de generación, no en F1).

**Se pospone:** DRL/Deep Learning (Hybrid Genesis), minería simbólica libre (moonshot con `egg`, ADR-0113; la regresión simbólica acotada como modo del NSGA-II sí está disponible), [`strategy-ensemble`](./features/strategy-ensemble.md), [`glass-box-ai-translator`](./features/glass-box-ai-translator.md), [`strategy-ast-copilot`](./features/strategy-ast-copilot.md), [`adaptive-volume-indicators`](./features/adaptive-volume-indicators.md) (P1 de F4 si una familia de estrategias los pide), La Colmena, node-preview (UI).

**Criterio de salida:** Una corrida nocturna produce ≥10K candidatas evaluadas con métricas en el Databank; consulta DuckDB filtra el top 1% en <1s; N global registrado e inmutable.

---

### F4 — Guantelete de Robustez (`validate` completo)

**Objetivo:** Separar suerte de Alpha. Solo sobrevive lo operable. Este filtro es lo que SQX hace a medias y donde Drasus gana.

**P0 (en orden de cascada ADR-0066):**
- Orquestación en cascada LIGHT/MEDIUM/HEAVY con fail-fast (TTR-999 de validate).
- LIGHT: [`complexity-penalization`](./features/complexity-penalization.md) (Ockham), filtros de métricas mínimas.
- MEDIUM: [`rule-ablation`](./features/rule-ablation.md) (ADR-0065), análisis de sensibilidad/meseta básico, [`vector-time-pruning`](./features/vector-time-pruning.md) (ADR-0046).
- HEAVY: [`walk-forward-analyzer`](./features/walk-forward-analyzer.md) (WFA Matrix), [`monte-carlo-simulator`](./features/monte-carlo-simulator.md) (decagonal + Embudo Tóxico, CPU/SIMD primero, GPU según Gate G2), [`cpcv-analyzer`](./features/cpcv-analyzer.md) (PBO, ADR-0063), [`cross-market-validation`](./features/cross-market-validation.md) (ADR-0049).
- [`prop-firm-grader`](./features/prop-firm-grader.md) (ADR-0045) — **prioridad absoluta dentro de HEAVY**: el objetivo de monetización inmediata son cuentas de fondeo.
- [`robustness-score-aggregator`](./features/robustness-score-aggregator.md) (ADR-0058) + [`statistical-inference-ebta`](./features/statistical-inference-ebta.md) (DSR con el N de F3).
- [`incremental-test-engine`](./features/incremental-test-engine.md) (ADR-0060) — multiplicador: ahorra 80% de revalidación.

**P1:** [`alpha-decoupling`](./features/alpha-decoupling.md), [`factor-decomposition`](./features/factor-decomposition.md), [`pca-toxicity-analyzer`](./features/pca-toxicity-analyzer.md), [`autoencoder-outlier-detector`](./features/autoencoder-outlier-detector.md) (según Gate G2), [`dtw-adaptive-window`](./features/dtw-adaptive-window.md), [`contextual-fitness-scorer`](./features/contextual-fitness-scorer.md).

**Se pospone:** [`robustness-verdict-engine`](./features/robustness-verdict-engine.md) (LLM — confort, no Alpha; Gate G5), visualizadores (parallel-coordinates, cross-filtering, UMAP, interactive-stress-lab, plateau-copilot → F8), [`adversarial-noise-agent`](./features/adversarial-noise-agent.md), simulador adversarial y demás moonshots de validación.

**Criterio de salida:** Pipeline F3→F4 corre desatendido y emite estrategias con Score ≥75 + certificación prop-firm; el guantelete completo de 1 estrategia termina en tiempo acotado y configurable; los resultados se heredan entre versiones (test incremental verificado).

---

### F5 — PRIMER DINERO REAL (`incubate` + `execute` mínimo viable)

**Objetivo:** La fase que justifica el proyecto. Decisión arquitectónica clave: **se llega al mercado vía [`multiplatform-execution-bridge`](./features/multiplatform-execution-bridge.md) (MT5) ANTES que con el motor nativo de brokers**, porque las cuentas de fondeo viven en MT5 y el bridge exige una superficie de integración mucho menor que adaptadores nativos + LiveNode completo (ADR-0078). El motor nativo llega en F6.

**P0 — Incubación:**
- [`paper-trader`](./features/paper-trader.md) + [`incubation-manager`](./features/incubation-manager.md): cuarentena de 7 días con Eutanasia Predictiva y Cono de Silencio (ADR-0088). El feed en vivo reutiliza el conector de datos de F1.
- [`pardo-comparison`](./features/pardo-comparison.md) (eficiencia forward vs histórico).

**P0 — Ejecución defensiva (Alpha Defensivo, innegociable antes del primer trade real):**
- [`pre-trade-validator`](./features/pre-trade-validator.md): los 10 checks (ADR-0025) + veto por robustez (ADR-0095).
- [`order-fsm`](./features/order-fsm.md) (ADR-0004), [`portfolio-rules`](./features/portfolio-rules.md) en modo Challenge/Rules Wrapper (ADR-0079) con asignación de capital **manual** (el HRP llega en F6).
- [`operational-safety-monitor`](./features/operational-safety-monitor.md) (SSL + Pardo Profile, ADR-0070), Shadow Watchdog + Kill Switch (ADR-0026/0087), [`sovereign-security`](./features/sovereign-security.md) (ADR-0093 — cifrado de llaves antes de tocar una API real).
- [`multi-ticket-manager`](./features/multi-ticket-manager.md), [`order-priority-queue`](./features/order-priority-queue.md) (ADR-0080), [`notification`](./features/notification.md), [`autopilot-metrics-provider`](./features/autopilot-metrics-provider.md) (consumo por CLI/panel mínimo).
- [`persistent-daemons`](./features/persistent-daemons.md) (ADR-0084) + [`data-bus-pubsub`](./features/data-bus-pubsub.md) (ADR-0085).

**P1:** [`advanced-trade-management`](./features/advanced-trade-management.md) (trailing stop primero; grid/hedging después), [`trade-reconciler`](./features/trade-reconciler.md) (versión mínima: real vs esperado diario).

**Se pospone:** [`broker-connector`](./features/broker-connector.md) nativo completo (F6), [`copy-trading-engine`](./features/copy-trading-engine.md), [`kinetic-micro-management.md`](./features/kinetic-micro-management.md), federación, RPAP.

**Criterio de salida:** Una estrategia generada por Drasus pasa cuarentena de 7 días, opera una cuenta de fondeo demo→real vía bridge, el SSL y el kill switch se prueban con simulacro de fallo (test de guerra obligatorio), y la reconciliación diaria cuadra. **KPI de negocio: primera semana verde con dinero real gestionado 100% por el sistema.**

---

### F6 — Portafolio y Ejecución Nativa (`manage` + `execute` completo)

**Objetivo:** Pasar de 1–3 estrategias a una flota con gestión de capital seria, y del bridge a brokers nativos.

**P0:**
- [`portfolio-data-preparation`](./features/portfolio-data-preparation.md) (ADR-0056), [`portfolio-optimizer`](./features/portfolio-optimizer.md) (HRP primero; Markowitz/Black-Litterman después) (ADR-0075/0089), [`portfolio-backtest`](./features/portfolio-backtest.md) (ADR-0091 — margen compartido real).
- [`signal-correlation-analyzer`](./features/signal-correlation-analyzer.md), Auto-Rebalancing Daemon con circuit breaker (ADR-0089).
- [`broker-connector`](./features/broker-connector.md) nativo (1 broker: el de mayor capital propio) + integración del motor de ejecución mediante el LiveNode de los crates NT v2 vendorizados (ADR-0107); brokers sin adaptador estable en v2 se cubren con adaptadores propios (TTR-004 de [`nautilus-integration`](./features/nautilus-integration.md)).
- [`volatility-stabilization`](./features/volatility-stabilization.md) (ADR-0068), [`equity-curve-tracker`](./features/equity-curve-tracker.md) a nivel portafolio.

**P1:** [`federated-portfolio`](./features/federated-portfolio.md) (ADR-0090), hedging cointegrativo y Router Viviente (ADR-0089 — solo si hay ≥5 estrategias vivas), [`fit-to-portfolio-search`](./features/fit-to-portfolio-search.md) (ahora sí, con flota real).

**Criterio de salida:** Portafolio de ≥3 estrategias descorrelacionadas operando con pesos HRP, rebalanceo automático auditado, y al menos 1 broker nativo con paridad sim/live medida.

---

### F7 — Ciclo Cerrado 24/7 (`feedback` + `withdraw` + QuantOps)

**Objetivo:** Convertir la herramienta en fábrica autónoma. El multiplicador final: el sistema se mejora solo mientras duermes.

**P0:**
- `feedback`: [`trade-reconciler`](./features/trade-reconciler.md) completo, [`pardo-comparison`](./features/pardo-comparison.md) como veredicto de continuidad, [`anomaly-detector`](./features/anomaly-detector.md), Learning Constraints hacia `generate` (ADR-0015).
- `withdraw`: [`performance-monitor`](./features/performance-monitor.md), retiro con pausa reversible (SAD §11), archivo institucional.
- [`quantops-daemon`](./features/quantops-daemon.md) (ADR-0052) + [`event-driven-pipeline-triggers`](./features/event-driven-pipeline-triggers.md): pipelines generate→validate→incubate encadenados sin humano.
- [`auto-auditoria-portafolios-vivos`](./features/auto-auditoria-portafolios-vivos.md) (costes reales vs modelados — realimenta los modelos de fricción de F2).
- Matriz Microrodante Nocturna (ADR-0059) — re-optimización diaria 23:59h.

**P1:** [`robust-reporting`](./features/robust-reporting.md), [`monthly-performance-heatmap`](./features/monthly-performance-heatmap.md) (reporte estático).

**Criterio de salida:** 30 días de operación desatendida: el sistema generó, validó, incubó, promovió y retiró estrategias solo, con audit trail completo y cero intervenciones de emergencia manuales.

---

### F8 — Glass-Box UI (Flutter completo)

**Objetivo:** Hasta aquí el sistema operó por CLI + paneles mínimos. Ahora se construye la experiencia que SQX no puede ofrecer.

- ZUI 3 niveles (ADR-0028): Fleet Command → Orchestrator (DAG editor con `petgraph` + CustomPainter) → Strategy Inspector.
- [`visual-dag-editor`](./features/visual-dag-editor.md), [`node-preview`](./features/node-preview.md) (ADR-0096), [`zui-navigation`](./features/zui-navigation.md).
- Visualizadores: [`umap-scatter-visualizer`](./features/umap-scatter-visualizer.md), [`parallel-coordinates-visualizer`](./features/parallel-coordinates-visualizer.md), [`cross-filtering-visualizer`](./features/cross-filtering-visualizer.md), [`interactive-stress-lab`](./features/interactive-stress-lab.md), [`plateau-copilot`](./features/plateau-copilot.md), [`toxicity-purifier`](./features/toxicity-purifier.md) (ADR-0098), [`time-warp-debugger`](./features/time-warp-debugger.md), [`monthly-performance-heatmap`](./features/monthly-performance-heatmap.md) interactivo.
- [`robustness-verdict-engine`](./features/robustness-verdict-engine.md) (LLM local, Gate G5), [`glass-box-ai-translator`](./features/glass-box-ai-translator.md), AST Copilot.
- Empaquetado comercial: [`flutter-packaging-manager`](./features/flutter-packaging-manager.md), [`licensing-system`](./features/licensing-system.md), instaladores 3 OS (ADR-0029).

**Pista transversal de UI (F1–F7):** cada fase entrega como máximo UNA pantalla utilitaria (telemetría de descarga F1, curva de equidad de backtest F2, tabla Databank F3, semáforo de guantelete F4, panel Autopilot+Kill Switch F5, panel de pesos F6, dashboard de salud F7). Pantallas funcionales, sin pulido. El pulido es F8.

---

### F9+ — Moonshots (estrictamente post-rentabilidad, ADR-0103 Filosofía Dual)

Orden sugerido por ROI esperado, revisable con datos reales:
1. [`universal-strategy-transpiler`](./moonshots/universal-strategy-transpiler.md) (MQL5 export — monetizable, ADR-0101)
2. [`copy-trading-engine`](./features/copy-trading-engine.md) productizado + [`saas-gateway`](./moonshots/saas-gateway.md)/[`monetization-stripe`](./moonshots/monetization-stripe.md)
3. [`deep-learning-suite`](./moonshots/deep-learning-suite.md) + DRL (Hybrid Genesis completo)
4. [`la-colmena`](./moonshots/la-colmena.md) (ADR-0086), [`marketplace-cajas-negras`](./moonshots/marketplace-cajas-negras.md) (ADR-0099)
5. [`saas-cloud-engine`](./moonshots/saas-cloud-engine.md) (ADR-0033 Trimodal modo 3) + HybridComputeCooperative (ADR-0094)
6. Resto del catálogo de moonshots según evidencia.

---

## 5. Dependencias Técnicas Duras (no negociables)

1. **F2 depende de F1:** sin PIT validator, el backtest miente.
2. **F3 depende de F2:** generar sin motor confiable es fabricar overfitting a escala.
3. **DSR (F4) depende de contar N desde F3:** orden inverso = estadística inválida.
4. **F5 depende del Score de F4:** el sizing en vivo consume el robustness score (ADR-0058) y el veto pre-trade consume el veredicto MC (ADR-0095).
5. **Versionado (ADR-0005) y los 25 campos (ADR-0020 V2) nacen en F0/F2:** retrofitearlos cuesta 10x.
6. **G1 (Nautilus) — resuelto por ADR-0107 (crates Rust v2 vendorizados):** el diseño del backtest event-driven de F2 y la ejecución nativa de F6 ya tienen mecanismo definido. En F0 solo queda su smoke test (compilación, backtest mínimo, empaquetado LGPL); si ese smoke test fallara, se activa el Plan B escalonado del ADR-0107 antes de diseñar F2.

## 6. KPIs por Fase (jerarquía de latencias unificada)

| Ruta | SLA | Fase donde se mide |
|---|---|---|
| Pre-trade validation (Guardián) | <1ms | F5 |
| Wrapper de reglas de portafolio | <10ms | F5/F6 |
| Orden end-to-end (señal→broker) | ≤100ms | F5/F6 |
| Kill switch / watchdog | ≤5s | F5 |
| Backtest Express híbrido | Más rápido que MT5/SQX/QuantConnect en igual hardware (ADR-0114; sin KPI absoluto) | F2 |
| Carga UI Time-Warp | <200ms | F8 |
| Recuperación post-crash (Event Store) | <10s | F0/F5 |

(Esta tabla resuelve las contradicciones de KPIs detectadas entre SAD §1.2, §9 y el pipeline §6 — ver CORRECTION-PLAN ítem C1.)
