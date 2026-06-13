# TASK-004 · Auditoría de la Inundación de Fundaciones (ADR-0020 V2)

| Campo | Valor |
|---|---|
| **ID** | TASK-004 |
| **Tipo** | Task (auditoría documental, sin código) |
| **Estado** | 🟡 En curso — Fase 1 (diagnóstico) COMPLETA; Fase 2 (decisiones de diseño) PENDIENTE |
| **Responsable** | Tech-Lead + Usuario (línea de defensa final). Diagnóstico: 13 agentes Sonnet en paralelo |
| **Creada** | 2026-06-13 |

## Objetivo
Verificar que el Filtro de Relevancia por Perfil (ADR-0020 V2) se aplicó bien en las 137 features y 8 módulos: cada tabla = Grupo I universal + solo los campos del Perfil Técnico (A/B/C/D) que le corresponden, sin calcar 25 ni meter campos de grupos ajenos. Origen: el usuario sospechó (correctamente) que durante los meses de redacción documental el filtro se aplicó de forma irregular.

## Fuente de verdad (verificada coherente — NO tocar)
- Tabla canónica de 4 perfiles: `docs/ADR.md` línea ~407 (ADR-0020 V2, "Resto por Filtro de Relevancia por Perfil").
- Resumen en `architect/SKILL.md` (dice "el ADR gana si difieren"), `TEMPLATES.md` (referencia, no copia), `SAD.md` §1285-1287 (ejemplo de las dos capas). Todos coherentes.
- Perfiles: **A. Datos/Ingest** = I + III + IV · **B. IA/R&D** = I + II + III subset + IV · **C. Ops/Hot-Path** = I + II + IV + V subset latencia · **D. Ops/Auditoría** = I + II + IV.
- Grupo I universal (SIEMPRE): `id, created_at, updated_at, audit_hash, audit_chain_hash, event_sequence_id`.

## Método (referencia de eficiencia de tokens)
137 features / 12 lotes (~11-12 c/u) + 1 lote de 8 módulos = 13 agentes Sonnet en paralelo. Tabla canónica EMBEBIDA en cada prompt (no leen el ADR completo). Diagnóstico + corrección de casos inequívocos de campos; reasignación de perfil = REVISAR (decisión humana). Ver regla en `tech-lead/SKILL.md` ("Análisis de Eficiencia de Tokens ANTES de invocar agentes").

---

## RESULTADO FASE 1 (diagnóstico)

### Resumen
- ✅ **Bien hechas (~17):** adaptive-logic-er, adaptive-volume-indicators, anomaly-detector, ast-compiler, audit-log, clock, data-normalization-layer, data-bus-pubsub, data-import-wizard, data-sanitizer-pipeline, data-validator, cpcv-analyzer, databank-manager, duckdb-sql-engine, feature-router, infrastructure-setup, monte-carlo-simulator.
- 🔧 **Corregidas en Fase 1 (~36, casos claros de campos — YA en disco):** alpha-decoupling, alpha-harvesting-gateway, auto-auditoria-portafolios-vivos, autoencoder-outlier-detector, backtest-engine, bayesian-optimizer, complexity-penalization, component-isolation, databank-manager (typo), design-manifest, duckdb-resampler, executable-container, factor-decomposition, flutter-packaging-manager, fragility-gradient-auditor, hierarchical-parameter-optimization, hive-partition-manager, hybrid-data-transformer, multi-ticket-manager, multiplatform-execution-bridge, nsga2-optimizer, order-priority-queue, pdf-charts-rendering, perfect-profit-benchmark, portfolio-backtest, precision-sizing-models, regime-guard, robustness-score-aggregator, robustness-verdict-engine, rule-ablation, signal-correlation-analyzer, slippage-models, strategy-versioning (typo), time-warp-debugger, toxicity-purifier, visual-dag-editor. Tipo de corrección: completar Grupo I + quitar campos de grupo ajeno (ej. `execution_latency_ms` de Grupo V en perfiles B). Verificadas en muestra por el Tech-Lead: quirúrgicas y correctas.

### Patrones detectados
- **P1 — Grupo I incompleto (SISTÉMICO, casi todas).** Falta `updated_at` y `event_sequence_id` (a veces `audit_chain_hash`/`audit_hash`). Causa raíz: plantilla / ejemplo de referencia. OBJETIVO, no diseño. ⚠️ Se corrigió SOLO en algunos lotes → **corpus desigual**, hay que uniformar TODO + arreglar `TEMPLATES.md`.
- **P2 — Perfil mal asignado / etiquetas inexistentes (~15).** Ver tabla de decisiones abajo.
- **P3 — Grupo III (linaje) colado en features Hot-Path (Perfil C).** order-fsm, order-flow-microstructure, incubation-manager, portfolio-optimizer, portfolio-rules, volume-profile-router, hmm-regime-detection. Copy-paste del perfil B.
- **P4 — Campos fuera del catálogo de 25 (ver §Campos nuevos).**
- **P5 — Features sin contrato de persistencia (solo prosa o nada, ~13):** statistical-inference-ebta, strategy-ast-copilot, operational-safety-monitor, persistent-daemons, federated-portfolio, event-driven-pipeline-triggers, secure-updater, volatility-stabilization, efficiency-incubation-dashboard, fractional-differencer, institutional-friction-modeling, institutional-metrics, kinetic-micro-management.
- **Módulos:** mayormente OK. Fuga en `ingest` (Perfil A) que cita `institutional_tag`/`owner_id` (Grupo II, ajeno) en TTR-008/009 → REVISAR. generate/validate/manage/withdraw OK. incubate ambiguo (perfil). execute OK (C). feedback pendiente de tabla final del agente.

---

## DECISIONES DE DISEÑO PENDIENTES (Fase 2 — tú y yo)

### A. Reasignación de perfil (P2/P3) — mi propuesta
| Feature | Perfil declarado | Mi propuesta | Razón |
|---|---|---|---|
| dsr-tracking-engine | A (Datos) | **B** | Minería genética / varianza de Sharpe = R&D |
| order-fsm | B (IA/R&D) | **C** | Transiciones FSM de órdenes, latencia crítica |
| order-flow-microstructure | (sin) | **C** | Snapshot DOM/tick hot-path |
| trade-analysis-bi-suite | A (Datos) | **D o sin persistencia** | BI analítico sobre trades, no ingesta de mercado |
| quantops-daemon | Ops/Hot-Path | **D** | Daemon CI/CD, no hot-path |
| secure-updater | Ops/Hot-Path | **D** | Actualizador firmado, forense |
| databank-lake | Ops/Hot-Path | **B** | Data lake R&D efímero |
| incubation-manager | C | **revisar C vs B** | Lleva Grupo III (linaje), ajeno a C |
| portfolio-optimizer | Ops/Hot-Path | **revisar C vs B** | Lleva Grupo III |
| volume-profile-router | (sin) | **C** | Veto de liquidez <0.5ms; quitar Grupo III |
| hmm-regime-detection | (sin) | **C** | Latencia ≤20ms; revisar campos III |
| robust-reporting | "C. Auditoría" (etiqueta mixta) | **D** | Reportes/exportación forense |
| strategy-self-explanation | "Documentación" (no existe) | **B** | Deriva de generación |
| cross-market-validation | "Ops/Validación" (no existe) | **B** | Motor estadístico comparativo |
| equity-curve-tracker | (sin) | **C o D** | Tracking en vivo, no R&D |
| zui-navigation | D | **sin persistencia** | UI/navegación frontend |
| worker-isolation-orchestrator | (sin) | **D o sin persistencia** | Orquestación de procesos |
| audit-event-store | "AI/R&D" (B) | **D** | Es el registro forense en sí |
| adversarial-noise-agent | D | **B** | Red-team AI / perturbación |
| contextual-fitness-scorer | D | **B** | Tiene logic_hash (Grupo III, propio de B) |

### B. Campos fuera del catálogo (P4) + ¿campos nuevos? (pregunta del usuario)
Respuesta: **SÍ, hay candidatos transversales que probablemente deban entrar al catálogo** vía el Mecanismo de Mantenimiento del ADR-0020 (requiere justificar que ≥3 features lo piden; lo valida el Architect, no se inventa a la ligera):
| Campo detectado | Aparece en | Propuesta |
|---|---|---|
| `compliance_status_id` | toxicity-purifier, copy-trading-engine, prop-firm-grader, multiplatform-execution-bridge | **Candidato a AÑADIR** (Grupo V Forense o II) — transversal a cumplimiento |
| `portfolio_container_id` | fit-to-portfolio-search, cross-market-validation, federated-portfolio | **Candidato a AÑADIR** (Grupo II o IV) — agrupador de portafolio |
| `parent_id` / `parent_strategy_id` / `parent_test_id` | strategy-versioning, incremental-test-engine, databank-lake | **Candidato a AÑADIR** un campo de linaje jerárquico (Grupo III) |
| `signature_hash` | notification, secure-updater | Evaluar: ¿mapear a `audit_hash` o nuevo (integridad cripto)? |
| `source_id` | quality-heatmap-generator, background-download-manager | Probable mapeo a `data_snapshot_id` (III) — NO nuevo |
| `recovery_latency_ms`, `heartbeat_latency_ms` | crash-recovery, system-watchdog | Probables variantes de `execution_latency_ms` (V) — decidir si se unifican |
| `transformation_id` | quality-heatmap-generator | YA existe en catálogo (ADR-0020) — solo verificar uso |
| `risk_audit_id` | varias | YA existe (Grupo V) |

### C. Features sin contrato (P5) → diseñar tabla tras confirmar perfil (Fase 3).

---

## PLAN DE EJECUCIÓN (retomar en próxima sesión)
**Decisión del usuario (2026-06-13): primero decidir diseño, luego corregir TODO junto.**

1. **Fase 2 — Sesión de decisiones (Tech-Lead + Usuario):** revisar tabla A (reasignación de perfil) y tabla B (campos nuevos). El usuario aprueba/ajusta cada fila. Los campos nuevos aprobados → escalar al Architect (Opus) para registrarlos en ADR-0020 V2 vía Mecanismo de Mantenimiento.
2. **Fase 3 — Corrección masiva uniforme (1 sola pasada, agentes Sonnet por lotes):** con los perfiles ya decididos: (a) completar Grupo I universal en TODO el corpus (P1, uniforme); (b) reasignar perfiles aprobados y limpiar campos de grupo ajeno (P2/P3); (c) diseñar contrato de las features sin tabla (P5); (d) aplicar campos nuevos donde corresponda.
3. **Fase 4 — Arreglar la causa raíz:** corregir `TEMPLATES.md` y el ejemplo de referencia para que el Grupo I COMPLETO sea el default (evita que el patrón P1 se repita en features futuras).
4. **Moonshots (41):** auditoría diferida — misma estrategia, sesión aparte (TASK futura).

## Verificación al cerrar
- `grep` de Grupo I completo en todas las features; `cargo`/build no aplica (es documental).
- Cada perfil reasignado coherente con la tabla canónica; cero campos fuera de catálogo sin registrar en ADR.
