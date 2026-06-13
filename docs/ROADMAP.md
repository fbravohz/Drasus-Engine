# ROADMAP de Desarrollo — Drasus Engine (ex-QuantForge)

**Versión:** 2.0 | **Fecha:** 2026-06-12 (v2.0: Unificación con EXECUTION-PLAN.md en un solo mapa de implementación. v1.2: SPIKE-002–SPIKE-006 resueltos vía ADR-0112 a 0116. v1.1: SPIKE-001 resuelto vía ADR-0107)
**Autor:** Auditoría arquitectónica (Tech Lead)
**Principio rector:** *Alpha-First.* Cada fase debe acortar la distancia entre el código y el dinero generado en mercados reales. Todo lo que no genere Alpha, lo proteja (defensa de capital = Alpha preservado) o lo multiplique (velocidad de descubrimiento), se pospone.

---

## 1. Criterio de Priorización (Alpha vs Vanidad)

| Categoría | Definición | Tratamiento |
|---|---|---|
| **Alpha Directo** | Produce estrategias operables o ejecuta dinero real (backtest, generación, validación, ejecución) | Épicas 1–5 |
| **Alpha Defensivo** | Protege capital ya en riesgo (pre-trade checks, SSL, watchdog, prop-firm grader) | Acoplado a la fase que pone dinero en riesgo |
| **Multiplicador** | Acelera el ciclo de descubrimiento (incremental tests, cascada fail-fast, daemons QuantOps) | Épicas 4–7 |
| **Vanidad/Confort** | UX avanzada, visualizadores 3D, LLM verdicts, ZUI completa | Épica 8+ |
| **Moonshots** | R&D no validado, monetización de terceros (Colmena, Marketplace, SaaS, Copy-Trading) | Post-Épica 8, según ROI demostrado |

**Regla del Tech Lead:** Una feature entra a una fase solo si su ausencia bloquea el "Entregable Alpha" de esa fase. Lo demás espera, sin excepciones, aunque ya tenga TTR escrito. Los TTRs no se modifican; solo se secuencian.

---

## 2. Spikes de Viabilidad Técnica (BLOQUEANTES — resolver en Épica 0)

Estos riesgos invalidaban supuestos centrales de la documentación. **Los 6 gates ya tienen veredicto documentado como ADR (SPIKE-001: ADR-0107; SPIKE-002–SPIKE-006: ADR-0112 a 0116).** Lo que resta en EPIC-0 es el trabajo residual de validación de cada veredicto (smoke tests, spikes de medición), no la decisión arquitectónica.

| # | Riesgo | Supuesto en docs | Realidad a verificar | Plan B |
|---|---|---|---|---|
| SPIKE-001 | **NautilusTrader como crate Rust** — ✅ **RESUELTO (ADR-0107)** | SAD §2.1: "integrado nativamente en el core Rust" | Verificado: el núcleo v2 de NT publica crates Rust puros en crates.io (backtest, modelo, trading, live) que permiten backtesting y ejecución live sin Python. El spike de EPIC-0 se reduce a un **smoke test**: compilar los crates vendorizados (versión fijada), correr un backtest mínimo y validar el enlace LGPL reenlazable. | Formalizado en ADR-0107 (en orden): (a) congelar la versión vendorizada estable; (b) fork de mantenimiento mínimo bajo LGPL; (c) motor soberano (`moonshots/sovereign-execution-engine.md`). Descartado: sidecar Python (viola ADR-0104) |
| SPIKE-002 | **tch-rs (libtorch)** — ✅ **RESUELTO (ADR-0112)** | SAD §2.1: "Laboratorio de IA tch-rs" | tch-rs arrastra libtorch (~2GB C++), rompe la promesa de "binario único sin runtimes" (ADR-0029) y complica el empaquetado en 3 OS. | **Veredicto:** erradicar `tch-rs`. Escalera `ndarray`+Rayon (default) → `candle` si se justifica → `burn` solo en moonshot DRL. Monte Carlo es CPU-first (no es deep learning). |
| SPIKE-003 | **PySR "en Rust"** — ✅ **RESUELTO (ADR-0113)** | SAD §2.3: "Minería Simbólica (Rust)" + múltiples menciones a PySR | PySR es Python+Julia. No existe puerto Rust. | **Veredicto:** erradicar PySR. La regresión simbólica acotada es un modo del motor NSGA-II nativo sobre el AST; la minería simbólica libre se difiere a moonshot con `egg`. Se rechazan `evalexpr`/`meval` en hot-path. |
| SPIKE-004 | **Motor de backtest dual** — ✅ **RESUELTO (ADR-0114)** | (KPI absoluto de bars/sec eliminado por humo) | Forzar event-loop tick-a-tick a la minería masiva asfixia la exploración; vectorizar puro impide la gestión de riesgo con estado (ADR-0109). | **Veredicto:** motor dual. Ruta Express híbrida (vectorizado para lo sin-estado + mini-loop secuencial para lo con-estado) + ruta Event-Driven (NT v2) para fidelidad. Modo elegido por el usuario; contrato de consistencia conservadora (FIJO). Criterio relativo: superar a MT5/SQX/QuantConnect. |
| SPIKE-005 | **Ollama/LLM local** — ✅ **RESUELTO (ADR-0115)** | ADR-0058 exigía LLM local | Ollama es un runtime externo — contradice "cero runtimes" (ADR-0029/0030). | **Veredicto:** Verdict Engine determinista por plantilla, sin LLM por defecto. Ollama derogado como requisito; LLM local soberano (`candle`) opcional. |
| SPIKE-006 | **flutter_rust_bridge a escala** — ✅ **RESUELTO (ADR-0116)** | ADR-0029/0019 | Verificar streams de alta frecuencia (throttling 100ms) y paso Arrow zero-copy con arrays de 1M+ puntos. | **Veredicto:** downsampling obligatorio en backend (nunca cruzar más resolución que el viewport, ADR-0098). `ZeroCopyBuffer` solo para cargas masivas; throttling en Rust; gRPC fallback (ADR-0033). Spike EPIC-0 confirma números. |

**Salida de Épica 0 = los 6 gates con veredicto + ADRs actualizados.** Sin esto, no se escribe código de producción.

---

## 3. Mapa de Fases (Resumen Ejecutivo)

```
EPIC-0 Fundación+Spikes → EPIC-1 Datos → EPIC-2 Motor Backtest → EPIC-3 Generación → EPIC-4 Guantelete
                                                                          ↓
EPIC-8 UI Glass-Box ← EPIC-7 Feedback+AutoPipeline ← EPIC-6 Manage+Live ← EPIC-5 PRIMER DINERO REAL
```

| Fase | Nombre | Módulos | Duración est.* | Entregable Alpha |
|---|---|---|---|---|
| EPIC-0 | Fundación y Spikes | infra transversal | 4–6 sem | Esqueleto compilable + riesgos resueltos |
| EPIC-1 | Soberanía de Datos | `ingest` | 4–6 sem | Data lake limpio y auditado de 2+ fuentes |
| EPIC-2 | Motor de Backtest | `validate` (núcleo) | 8–10 sem | Backtest determinista y rápido en quien confiar |
| EPIC-3 | Generación | `generate` | 6–8 sem | Miles de candidatas/día en el Databank |
| EPIC-4 | Guantelete de Robustez | `validate` (completo) | 6–8 sem | Estrategias con Score ≥75 listas para operar |
| EPIC-5 | **Primer Dinero Real** | `incubate` + `execute` (mínimo) | 6–8 sem | **Estrategia viva en cuenta real/fondeo vía bridge MT5** |
| EPIC-6 | Portafolio y Ejecución Nativa | `manage` + `execute` (completo) | 8–10 sem | Portafolio multi-estrategia con brokers nativos |
| EPIC-7 | Ciclo Cerrado 24/7 | `feedback` + `withdraw` + QuantOps | 4–6 sem | Fábrica autónoma: genera→valida→incuba→opera→aprende |
| EPIC-8 | Glass-Box UI | UI completa | 8–12 sem | Editor visual DAG, ZUI, visualizadores |
| EPIC-9+ | Moonshots | según ROI | — | Colmena, Marketplace, SaaS, Copy-Trading, etc. |

\* Estimaciones para 1 dev senior + agentes IA, jornada completa. Son relativas: lo importante es el orden y los criterios de salida, no el calendario.

---

## 4. Detalle por Fase

### EPIC-0 — Fundación y Spikes de Riesgo

**Objetivo:** Esqueleto del monolito modular FCIS compilando, con las fundaciones que evitan retrabajo (ADR-0020 V2), y los 6 gates de viabilidad resueltos.

- Workspace Cargo con los 8 módulos como crates internos + carpeta `shared` (ADR-0003).
- Migraciones SQLx embebidas: la migración 0001 crea la tabla ancla `foundation_master_fields` con el catálogo de 25 campos (referencia única); las tablas por feature aplican Grupo I universal + su Perfil Técnico (ADR-0006, ADR-0020 V2).
- Features transversales P0: [`clock`](./features/clock.md), [`audit-log`](./features/audit-log.md), [`telemetry`](./features/telemetry.md), [`async-job-executor`](./features/async-job-executor.md) (ADR-0011), [`crash-recovery`](./features/crash-recovery.md) (ADR-0027 Event Store), [`worker-isolation-orchestrator`](./features/worker-isolation-orchestrator.md).
- CLI con Clap como primera interfaz (la UI Flutter NO bloquea ninguna fase hasta EPIC-8; cada fase expone sus comandos por CLI primero).
- Spike FFI: ventana Flutter mínima + `flutter_rust_bridge` + stream Arrow (SPIKE-006). Solo "hello world" de infraestructura, cero pantallas de producto.
- Validar los veredictos ya documentados de los SPIKE-001–SPIKE-006 (§2). Todos tienen ADR (SPIKE-001: ADR-0107; SPIKE-002–SPIKE-006: ADR-0112 a 0116). En EPIC-0 solo queda el trabajo residual: smoke test de compilación/vendoring de los crates NT v2 y empaquetado LGPL (SPIKE-001); smoke test de cómputo CPU-first `ndarray`/Rayon (SPIKE-002); spike de medición del motor Express híbrido y el contrato de consistencia (SPIKE-004); spike FFI con downsampling y throttling (SPIKE-006).

**Criterio de salida:** `cargo test` verde en esqueleto; migración 0001 aplica los 25 campos; job asíncrono sobrevive a un kill -9 y se recupera (ADR-0011); veredictos SPIKE-001–SPIKE-006 documentados y sus validaciones residuales ejecutadas.

#### Plan de Ejecución de EPIC-0 — Paso a Paso

**Dueño:** Tech-Lead (orquestación y auditoría). Este plan NO define diseño — el diseño vive en SAD/ADR/Features.

**Regla vigente:** Ningún TTR de EPIC-1+ avanza a implementación mientras los SPIKE-001–SPIKE-006 no tengan su validación residual ejecutada. Los 6 gates ya tienen veredicto en ADR; solo resta confirmar con smoke tests y spikes.

##### Seguimiento de ejecución (Spec-Driven) y vocabulario

Cada trabajo se ejecuta desde una **Orden de Trabajo** en [`docs/execution/`](./execution/) (plantilla: [`_TEMPLATE.md`](./execution/_TEMPLATE.md)): un archivo por trabajo con la instrucción exacta dada al agente, los comandos para validar y el registro de ejecución. El ROADMAP enlaza el estado de cada trabajo a su Orden; los documentos fuente (feature/módulo/TTR) se sellan como implementados al cerrarse.

**Identificadores** (estilo Jira: palabra completa + número, estables):

| ID | Tipo | Qué es |
|---|---|---|
| `EPIC-0`…`EPIC-9` | Épica | una fase del producto (EPIC-0 = Fundación) |
| `SPRINT-n` | Sprint | tanda de trabajos despachados juntos |
| `STORY-###` | Story | trabajo con código |
| `SPIKE-###` | Spike | investigación de un riesgo técnico bloqueante |
| `TASK-###` | Task | trabajo sin código (investigación, admin) |
| `BUG-###` | Bug | corrección de un defecto |
| `TTR` · `ADR` · Feature · Módulo | — | conservan su nombre (unidades de especificación) |

##### Tablero de Spikes — Spikes de Validación (máxima prioridad)

Cada spike entrega veredicto binario + Plan B si aplica. Los 6 spikes corren en paralelo entre sí y en paralelo con las tareas de cimentación, porque no dependen del esqueleto.

| Spike | Qué hay que confirmar | Quién lo hace | Qué se espera | Estado |
|---|---|---|---|---|
| SPIKE-001 | Compilar los crates de NautilusTrader v2 vendorizados, correr un backtest mínimo, verificar empaquetado LGPL | Rust-Engineer | Pasa → SPIKE-001 cerrado. Falla → activar Plan B del ADR-0107 | Pendiente |
| SPIKE-002 | Smoke test de cómputo CPU-first con `ndarray`/Rayon y tamaño de binario sin libtorch | Quant-Engineer + Rust-Engineer | Confirmar que funciona sin GPU y sin romper el binario único | Veredicto documentado — resta validación |
| SPIKE-003 | Prototipo del modo simbólico nativo (regresión simbólica como modo del NSGA-II sobre el AST) | Quant-Engineer | Confirmar que funciona sin PySR | Veredicto documentado — resta validación |
| SPIKE-004 | Spike de medición del motor Express híbrido vs MT5/SQX/QuantConnect (criterio relativo, sin KPI absoluto) | Rust-Engineer + Quant-Engineer | Confirmar ventaja competitiva y contrato de consistencia | Veredicto documentado — resta validación |
| SPIKE-005 | Implementar la plantilla determinista del Verdict Engine (sin LLM) | Rust-Engineer | Confirmar veredicto reproducible sin Ollama | Veredicto documentado — resta validación |
| SPIKE-006 | Spike FFI: ventana Flutter mínima + `flutter_rust_bridge` + stream Arrow con downsampling y throttling | Bridge-Engineer | Confirmar latencia de stream con throttle 100ms | Veredicto documentado — resta validación |

**Nota:** El único spike con UI permitida en EPIC-0 es SPIKE-006, y es exclusivamente "hello world" de infraestructura — cero pantallas de producto.

##### Backlog EPIC-0 — Sprint de Trabajo por Sprints

**Clasificación:** Ninguna feature de EPIC-0 es matemática/estrategia (son infraestructura transversal). Flujo estándar: Implementación Rust → QA continuo + gate final.

**Sprint 0 — Sin precondición (despacho inmediato, paralelo con spikes):**

| ID | Qué se construye | Quién | Cómo se sabe que está listo | Estado |
|---|---|---|---|---|
| STORY-001 | Workspace Cargo: 8 módulos como crates internos + carpeta `shared`. Esqueleto compilable con interfaces públicas vacías por módulo | Rust-Engineer | `cargo build` y `cargo test` verdes; estructura FCIS auditada (cero lógica en orquestadores) | ✅ **Completado** ([Orden STORY-001](./execution/STORY-001-skeleton.md), 2026-06-12, auditado: build/test 9/9 verdes, 0 warnings, FCIS verificado) |
| STORY-002 | Migraciones SQLx embebidas: migración 0001 con los 25 campos maestros | Rust-Engineer | La migración aplica los 25 campos en SQLite WAL; verificación de idempotencia | ✅ **Completado** ([Orden STORY-002](./execution/STORY-002-migration.md), 2026-06-12, auditado. 25 campos exactos, WAL, idempotente. Veredicto Architect: contrato lógico + filtro por perfil — ADR-0020 V2 actualizado) |

**Sprint 1 — Precondición: STORY-001+STORY-002 completados:**

| ID | Qué se construye | Quién | Cómo se sabe que está listo | Estado |
|---|---|---|---|---|
| STORY-003 | [`clock`](./features/clock.md) — Timestamps deterministas en nanosegundos | Rust-Engineer | Mismo seed/datos → misma secuencia temporal bit-a-bit | ✅ **Completado** ([Orden STORY-003](./execution/STORY-003-clock.md), 2026-06-12, auditado. Fase 1: determinismo bit-a-bit + FCIS. Fase 2: rastro de auditoría del reloj (`clock_audit`, 3 eventos a la bitácora vía `details_json`, Perfil D), tras escalamiento al Architect. 28 tests verdes, clippy `-D warnings` limpio, granularidad del hot-path verificada) |
| STORY-004 | [`audit-log`](./features/audit-log.md) — Registro inmutable con hash chain | Rust-Engineer | Intento de mutación de evento histórico es rechazado y detectado | 🟡 **Parcial** ([Orden STORY-004](./execution/STORY-004-audit-log.md), 2026-06-12, auditado: TTR-001 hecho — append-only por triggers + cadena de hash detecta mutación; 22 tests verdes. TTR-002 diferido a EPIC-2+) |
| STORY-005 | [`async-job-executor`](./features/async-job-executor.md) — Cola de trabajos con Tokio + SQLite | Rust-Engineer | **Test de guerra:** job sobrevive `kill -9` y se recupera al arranque | ✅ **Completado** ([Orden STORY-005](./execution/STORY-005-async-job-executor.md), 2026-06-12, auditado. Gate demostrado: `jobs_survive_simulated_crash_and_are_recovered_on_restart` sobre DB en archivo. 62 tests verdes, clippy `-D warnings` limpio, cobertura 90.80%. TTR-007 secuenciado a EPIC-2+; `kill -9` real diferido a STORY-009) |
| STORY-006 | [`crash-recovery`](./features/crash-recovery.md) — Recuperación post-crash con Event Store | Rust-Engineer | Recuperación post-crash en menos de 10 segundos | En espera |
| STORY-007 | [`telemetry`](./features/telemetry.md) — Métricas de hardware sin bloquear el hot-path | Rust-Engineer | Telemetría local emitiendo sin bloquear la ruta crítica | En espera |
| STORY-008 | [`worker-isolation-orchestrator`](./features/worker-isolation-orchestrator.md) — Aislamiento de workers | Rust-Engineer | Caída de un worker no contamina al orquestador; shutdown limpio | En espera |
| STORY-009 | CLI con Clap como primera interfaz: comandos para STORY-003–STORY-008. **Incluye crear el crate binario raíz `app` (archivo principal de orquestación, SAD §4.2)** que STORY-001 dejó pendiente a propósito — es el punto de arranque del programa y su hogar natural es la CLI | Rust-Engineer | Comandos básicos operativos: estado de jobs, telemetría, auditoría; el binario raíz compila y arranca | En espera |

**Sprint 2 — Cierre de fase (precondición: Sprints 0–1 completadas + veredictos SPIKE-001–SPIKE-006 recibidos):**

| ID | Qué se construye | Quién | Estado |
|---|---|---|---|
| TASK-001 | Gate final QA de EPIC-0: suite completa contra el criterio de salida | QA-Engineer | En espera |
| TASK-002 | Confirmar que los 6 veredictos SPIKE-001–SPIKE-006 están registrados como ADR | Tech-Lead → Architect | Veredictos registrados |
| TASK-003 | Seleccionar el primer TTR de EPIC-1 (data-validator + pit-data-validator) | Tech-Lead | En espera |

##### Reglas Operativas de EPIC-0

1. **Paralelismo controlado:** Los spikes de gates y la Sprint 0 corren simultáneos. La Sprint 1 NO arranca hasta que STORY-001+STORY-002 estén completados — los 25 campos y el esqueleto son fundación anti-retrabajo (retrofitear cuesta 10x).
2. **QA continuo:** Cada entregable pasa por QA apenas se produce (tests unitarios, determinismo); el gate final (TASK-001) audita el conjunto. Defecto de implementación → regresa al ingeniero; defecto de diseño → escalar al Architect.
3. **Bloqueo EPIC-1+:** Prohibido despachar TTRs de EPIC-1 hasta que los 6 gates estén cerrados.
4. **SLA aplicable:** En EPIC-0 solo se exige recuperación post-crash menor a 10 segundos. Prohibido exigir SLAs de fases futuras.
5. **Sin UI de producto:** La pista de UI inicia en EPIC-1 (una pantalla utilitaria por fase). En EPIC-0 solo existe el spike SPIKE-006.
6. **Cero invención:** Si algún TTR resulta ambiguo durante el despacho, se bloquea y se escala al Architect con evidencia — los ingenieros no rellenan vacíos de spec.

##### Registro de Estado EPIC-0

| Ítem | Estado | Última actualización |
|---|---|---|
| SPIKE-001 | Pendiente (smoke test); veredicto ADR-0107 | 2026-06-10 |
| SPIKE-002–SPIKE-006 | Veredictos documentados (ADR-0112 a 0116); resta validación residual | 2026-06-11 |
| STORY-001 | ✅ Completado y auditado (esqueleto FCIS, 9 crates, build/test verdes) | 2026-06-12 |
| STORY-002 | ✅ Completado y auditado (migración 0001, 25 campos, WAL, idempotente) | 2026-06-12 |
| STORY-003 | ✅ Completado y auditado (reloj determinista + FCIS + rastro de auditoría `clock_audit`, 3 eventos a la bitácora). Cerrado tras escalamiento al Architect | 2026-06-12 |
| STORY-004 | 🟡 Parcial y auditado (audit-log TTR-001: append-only + hash chain). TTR-002 diferido a EPIC-2+ | 2026-06-12 |
| STORY-005 | ✅ Completado y auditado (`async-job-executor`: cola durable + recuperación tras crash, gate EPIC-0 demostrado sobre DB en archivo; cobertura 90.80%). TTR-007 → EPIC-2+ | 2026-06-12 |
| STORY-006–STORY-008 | En espera. Siguiente: STORY-006 (`crash-recovery`, recuperación post-crash <10s) | 2026-06-12 |
| STORY-009 | En espera. Recordatorio: incluye el crate binario raíz `app` (SAD §4.2) | 2026-06-12 |
| TASK-001–TASK-003 | En espera (cierre de fase) | 2026-06-10 |

**Descubrimientos y decisiones de EPIC-0 (bitácora):**
- **2026-06-12 — Crate binario raíz `app`:** STORY-001 creó solo los 8 crates de módulo + `shared` (criterio literal). El SAD §4.2 prevé además un "archivo principal de orquestación" (binario raíz). Decisión Tech-Lead: ese binario se crea en STORY-009 junto a la CLI, su hogar natural. No es deuda, es secuenciación.
- **2026-06-12 — Contrato de 25 campos (ADR-0020 V2):** escalado a Architect. Veredicto: los 25 campos son un **contrato lógico/vocabulario obligatorio**, no 25 columnas calcadas en cada tabla. Grupo I (Identidad) universal; grupos II–V por **Filtro de Relevancia por Perfil** (ya en `architect/SKILL.md` y `TEMPLATES.md`). La tabla ancla `foundation_master_fields` de EPIC-0 es correcta. ADR-0020 V2 y SAD §17.9/§20 actualizados para reflejarlo. **Implicación para Sprint 1:** las tablas de STORY-003–STORY-008 NO copian 25 columnas; aplican el filtro por perfil.
- **2026-06-12 — `transformation_id`:** es un identificador (TEXT/UUID) del paso de transformación, no un flag booleano. Glosa corregida en ADR-0020 V2 y los 8 módulos.
- **2026-06-12 — STORY-003 `clock` completado.** Reloj en `crates/shared` (núcleo determinista + cáscara `SystemClock`). Determinismo bit-a-bit verificado. **Pendiente diferido a STORY-004:** las postcondiciones de `clock.md` (TTR-001/002) piden registrar en auditoría el `ntp_sync_offset`, el `virtual_process_id` y la delta real/virtual. No se implementó porque (a) requiere que exista `audit-log` (STORY-004), y (b) el Architect debe definir el perfil de persistencia/auditoría de la entidad `clock` (campos propios de `clock` + subconjunto del contrato por perfil, ADR-0020 V2). **Acción al llegar a STORY-004:** escalar a Architect para definir ese perfil y entonces implementar el rastro de auditoría del reloj.
- **2026-06-12 — Perfil de auditoría del reloj resuelto (Architect, escalamiento §3).** Veredicto: los tres "campos" citados por `clock.md` (`ntp_sync_offset`, proceso virtual de simulación, delta real/virtual) NO existen en el catálogo ADR-0020 V2 y son **payload de evento** (`details_json` opaco de `AuditEventContent`), no columnas; el `virtual_process_id` huérfano se sustituye por `session_id` del catálogo (Grupo IV). El reloj NO tiene persistencia propia: emite a la bitácora existente vía `AuditEventContent`, **Perfil D (Ops/Auditoría)**. Granularidad acotada a 3 eventos (`CLOCK_NTP_SYNC` al arranque, `CLOCK_MODE_TRANSITION` en REAL↔SIMULATION, `CLOCK_SESSION_CLOSE` con la delta acumulada) — PROHIBIDO auditar el hot-path. **ADR-0020 V2 sin cambios** (campos no son transversales a 3+ features). `clock.md` corregido. **Implicación para STORY-004:** el rastro del reloj ya es implementable sin inventar campos.

---

### EPIC-1 — Soberanía de Datos (`ingest`)

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

### EPIC-2 — Motor de Backtest (`validate` núcleo + features de simulación)

**Objetivo:** El corazón del sistema y el generador de Alpha #1. Un backtest determinista, con fricción institucional, en el que se confía ciegamente. **Aquí se gana o se pierde el proyecto.**

**P0:**
- [`backtest-engine`](./features/backtest-engine.md): arquitectura dual (SPIKE-004) — ruta vectorizada para minería masiva (Open Prices / 1m OHLC) y ruta event-driven para fidelidad (4-ticks; Real Ticks después). Modos de ADR-0017.
- Fricción institucional mandatoria: [`slippage-models`](./features/slippage-models.md), triple swap, penetración Pardo, Bar-Open Alignment (ADR-0017).
- [`institutional-metrics`](./features/institutional-metrics.md) con implementación dual hot/cold (ADR-0047).
- [`equity-curve-tracker`](./features/equity-curve-tracker.md), [`precision-sizing-models`](./features/precision-sizing-models.md) (ADR-0044 — paridad sizing desde el día uno).
- [`executable-container`](./features/executable-container.md) (ADR-0009) y contrato AST + compilador Serde (sustituye al residuo "Pydantic AST Compiler"; el TTR se conserva, la tecnología es Serde/Rust).
- [`strategy-versioning`](./features/strategy-versioning.md) (ADR-0005 — hash chain; barato ahora, carísimo de retrofitear).

**P1:** [`perfect-profit-benchmark`](./features/perfect-profit-benchmark.md), [`universal-basket-backtester`](./features/universal-basket-backtester.md).

**Se pospone:** [`nautilus-integration`](./features/nautilus-integration.md) completa (el mecanismo ya está resuelto — ADR-0107: crates Rust v2 vendorizados — pero la paridad sim/live se exige recién en EPIC-5–EPIC-6; en EPIC-2 solo se consume la ruta de backtest event-driven), [`institutional-friction-modeling`](./features/institutional-friction-modeling.md) (adverse selection — refinamiento de EPIC-4).

**Criterio de salida:** (1) Reproducibilidad bit-a-bit verificada: 2 corridas, mismo hash de resultados. (2) La ruta Express híbrida es medible y demostrablemente más rápida que MT5/SQX/QuantConnect sobre el mismo dataset y hardware (benchmark `criterion` en CI; sin KPI absoluto — ADR-0114). (3) Paridad validada contra una plataforma de referencia (misma estrategia simple en MT5/SQX: diferencias explicables y documentadas).

---

### EPIC-3 — Generación (`generate`)

**Objetivo:** La fábrica de candidatas. Con EPIC-2 confiable, el volumen de exploración ES el Alpha.

**P0:**
- [`design-manifest`](./features/design-manifest.md) (ADR-0053 — el contrato SMART filtra basura desde el origen).
- [`nsga2-optimizer`](./features/nsga2-optimizer.md) nativo Rust (multi-objetivo Sharpe/DD/WR) con decimación y renovación sanguínea (SAD §2.3).
- AST + WildCards (ADR-0043) — el humano fija el esqueleto, el motor resuelve comodines: este es el modo de generación con mejor ratio esfuerzo/alpha.
- [`databank-lake`](./features/databank-lake.md) + [`databank-manager`](./features/databank-manager.md) (ADR-0055 — semillas Parquet, no AST masivos).
- [`dsr-tracking-engine`](./features/dsr-tracking-engine.md): registrar $N$ intentos desde la PRIMERA corrida (ADR-0067). Si no se cuenta N desde el inicio, el DSR de EPIC-4 nace inválido. Costo trivial, valor estadístico enorme.
- [`parameter-optimization`](./features/parameter-optimization.md) + [`zero-crossing-filter`](./features/zero-crossing-filter.md).

**P1:** [`bayesian-optimizer`](./features/bayesian-optimizer.md), [`fit-to-portfolio-search`](./features/fit-to-portfolio-search.md) (cobra valor real cuando ya hay portafolio vivo — puede deslizarse a EPIC-6), [`hmm-regime-detection`](./features/hmm-regime-detection.md) (entra aquí como filtro de generación, no en EPIC-1).

**Se pospone:** DRL/Deep Learning (Hybrid Genesis), minería simbólica libre (moonshot con `egg`, ADR-0113; la regresión simbólica acotada como modo del NSGA-II sí está disponible), [`strategy-ensemble`](./features/strategy-ensemble.md), [`glass-box-ai-translator`](./features/glass-box-ai-translator.md), [`strategy-ast-copilot`](./features/strategy-ast-copilot.md), [`adaptive-volume-indicators`](./features/adaptive-volume-indicators.md) (P1 de EPIC-4 si una familia de estrategias los pide), La Colmena, node-preview (UI).

**Criterio de salida:** Una corrida nocturna produce ≥10K candidatas evaluadas con métricas en el Databank; consulta DuckDB filtra el top 1% en <1s; N global registrado e inmutable.

---

### EPIC-4 — Guantelete de Robustez (`validate` completo)

**Objetivo:** Separar suerte de Alpha. Solo sobrevive lo operable. Este filtro es lo que SQX hace a medias y donde Drasus gana.

**P0 (en orden de cascada ADR-0066):**
- Orquestación en cascada LIGHT/MEDIUM/HEAVY con fail-fast (TTR-999 de validate).
- LIGHT: [`complexity-penalization`](./features/complexity-penalization.md) (Ockham), filtros de métricas mínimas.
- MEDIUM: [`rule-ablation`](./features/rule-ablation.md) (ADR-0065), análisis de sensibilidad/meseta básico, [`vector-time-pruning`](./features/vector-time-pruning.md) (ADR-0046).
- HEAVY: [`walk-forward-analyzer`](./features/walk-forward-analyzer.md) (WFA Matrix), [`monte-carlo-simulator`](./features/monte-carlo-simulator.md) (decagonal + Embudo Tóxico, CPU/SIMD primero, GPU según SPIKE-002), [`cpcv-analyzer`](./features/cpcv-analyzer.md) (PBO, ADR-0063), [`cross-market-validation`](./features/cross-market-validation.md) (ADR-0049).
- [`prop-firm-grader`](./features/prop-firm-grader.md) (ADR-0045) — **prioridad absoluta dentro de HEAVY**: el objetivo de monetización inmediata son cuentas de fondeo.
- [`robustness-score-aggregator`](./features/robustness-score-aggregator.md) (ADR-0058) + [`statistical-inference-ebta`](./features/statistical-inference-ebta.md) (DSR con el N de EPIC-3).
- [`incremental-test-engine`](./features/incremental-test-engine.md) (ADR-0060) — multiplicador: ahorra 80% de revalidación.

**P1:** [`alpha-decoupling`](./features/alpha-decoupling.md), [`factor-decomposition`](./features/factor-decomposition.md), [`pca-toxicity-analyzer`](./features/pca-toxicity-analyzer.md), [`autoencoder-outlier-detector`](./features/autoencoder-outlier-detector.md) (según SPIKE-002), [`dtw-adaptive-window`](./features/dtw-adaptive-window.md), [`contextual-fitness-scorer`](./features/contextual-fitness-scorer.md).

**Se pospone:** [`robustness-verdict-engine`](./features/robustness-verdict-engine.md) (LLM — confort, no Alpha; SPIKE-005), visualizadores (parallel-coordinates, cross-filtering, UMAP, interactive-stress-lab, plateau-copilot → EPIC-8), [`adversarial-noise-agent`](./features/adversarial-noise-agent.md), simulador adversarial y demás moonshots de validación.

**Criterio de salida:** Pipeline EPIC-3→EPIC-4 corre desatendido y emite estrategias con Score ≥75 + certificación prop-firm; el guantelete completo de 1 estrategia termina en tiempo acotado y configurable; los resultados se heredan entre versiones (test incremental verificado).

---

### EPIC-5 — PRIMER DINERO REAL (`incubate` + `execute` mínimo viable)

**Objetivo:** La fase que justifica el proyecto. Decisión arquitectónica clave: **se llega al mercado vía [`multiplatform-execution-bridge`](./features/multiplatform-execution-bridge.md) (MT5) ANTES que con el motor nativo de brokers**, porque las cuentas de fondeo viven en MT5 y el bridge exige una superficie de integración mucho menor que adaptadores nativos + LiveNode completo (ADR-0078). El motor nativo llega en EPIC-6.

**P0 — Incubación:**
- [`paper-trader`](./features/paper-trader.md) + [`incubation-manager`](./features/incubation-manager.md): cuarentena de 7 días con Eutanasia Predictiva y Cono de Silencio (ADR-0088). El feed en vivo reutiliza el conector de datos de EPIC-1.
- [`pardo-comparison`](./features/pardo-comparison.md) (eficiencia forward vs histórico).

**P0 — Ejecución defensiva (Alpha Defensivo, innegociable antes del primer trade real):**
- [`pre-trade-validator`](./features/pre-trade-validator.md): los 10 checks (ADR-0025) + veto por robustez (ADR-0095).
- [`order-fsm`](./features/order-fsm.md) (ADR-0004), [`portfolio-rules`](./features/portfolio-rules.md) en modo Challenge/Rules Wrapper (ADR-0079) con asignación de capital **manual** (el HRP llega en EPIC-6).
- [`operational-safety-monitor`](./features/operational-safety-monitor.md) (SSL + Pardo Profile, ADR-0070), Shadow Watchdog + Kill Switch (ADR-0026/0087), [`sovereign-security`](./features/sovereign-security.md) (ADR-0093 — cifrado de llaves antes de tocar una API real).
- [`multi-ticket-manager`](./features/multi-ticket-manager.md), [`order-priority-queue`](./features/order-priority-queue.md) (ADR-0080), [`notification`](./features/notification.md), [`autopilot-metrics-provider`](./features/autopilot-metrics-provider.md) (consumo por CLI/panel mínimo).
- [`persistent-daemons`](./features/persistent-daemons.md) (ADR-0084) + [`data-bus-pubsub`](./features/data-bus-pubsub.md) (ADR-0085).

**P1:** [`advanced-trade-management`](./features/advanced-trade-management.md) (trailing stop primero; grid/hedging después), [`trade-reconciler`](./features/trade-reconciler.md) (versión mínima: real vs esperado diario).

**Se pospone:** [`broker-connector`](./features/broker-connector.md) nativo completo (EPIC-6), [`copy-trading-engine`](./features/copy-trading-engine.md), [`kinetic-micro-management.md`](./features/kinetic-micro-management.md), federación, RPAP.

**Criterio de salida:** Una estrategia generada por Drasus pasa cuarentena de 7 días, opera una cuenta de fondeo demo→real vía bridge, el SSL y el kill switch se prueban con simulacro de fallo (test de guerra obligatorio), y la reconciliación diaria cuadra. **KPI de negocio: primera semana verde con dinero real gestionado 100% por el sistema.**

---

### EPIC-6 — Portafolio y Ejecución Nativa (`manage` + `execute` completo)

**Objetivo:** Pasar de 1–3 estrategias a una flota con gestión de capital seria, y del bridge a brokers nativos.

**P0:**
- [`portfolio-data-preparation`](./features/portfolio-data-preparation.md) (ADR-0056), [`portfolio-optimizer`](./features/portfolio-optimizer.md) (HRP primero; Markowitz/Black-Litterman después) (ADR-0075/0089), [`portfolio-backtest`](./features/portfolio-backtest.md) (ADR-0091 — margen compartido real).
- [`signal-correlation-analyzer`](./features/signal-correlation-analyzer.md), Auto-Rebalancing Daemon con circuit breaker (ADR-0089).
- [`broker-connector`](./features/broker-connector.md) nativo (1 broker: el de mayor capital propio) + integración del motor de ejecución mediante el LiveNode de los crates NT v2 vendorizados (ADR-0107); brokers sin adaptador estable en v2 se cubren con adaptadores propios (TTR-004 de [`nautilus-integration`](./features/nautilus-integration.md)).
- [`volatility-stabilization`](./features/volatility-stabilization.md) (ADR-0068), [`equity-curve-tracker`](./features/equity-curve-tracker.md) a nivel portafolio.

**P1:** [`federated-portfolio`](./features/federated-portfolio.md) (ADR-0090), hedging cointegrativo y Router Viviente (ADR-0089 — solo si hay ≥5 estrategias vivas), [`fit-to-portfolio-search`](./features/fit-to-portfolio-search.md) (ahora sí, con flota real).

**Criterio de salida:** Portafolio de ≥3 estrategias descorrelacionadas operando con pesos HRP, rebalanceo automático auditado, y al menos 1 broker nativo con paridad sim/live medida.

---

### EPIC-7 — Ciclo Cerrado 24/7 (`feedback` + `withdraw` + QuantOps)

**Objetivo:** Convertir la herramienta en fábrica autónoma. El multiplicador final: el sistema se mejora solo mientras duermes.

**P0:**
- `feedback`: [`trade-reconciler`](./features/trade-reconciler.md) completo, [`pardo-comparison`](./features/pardo-comparison.md) como veredicto de continuidad, [`anomaly-detector`](./features/anomaly-detector.md), Learning Constraints hacia `generate` (ADR-0015).
- `withdraw`: [`performance-monitor`](./features/performance-monitor.md), retiro con pausa reversible (SAD §11), archivo institucional.
- [`quantops-daemon`](./features/quantops-daemon.md) (ADR-0052) + [`event-driven-pipeline-triggers`](./features/event-driven-pipeline-triggers.md): pipelines generate→validate→incubate encadenados sin humano.
- [`auto-auditoria-portafolios-vivos`](./features/auto-auditoria-portafolios-vivos.md) (costes reales vs modelados — realimenta los modelos de fricción de EPIC-2).
- Matriz Microrodante Nocturna (ADR-0059) — re-optimización diaria 23:59h.

**P1:** [`robust-reporting`](./features/robust-reporting.md), [`monthly-performance-heatmap`](./features/monthly-performance-heatmap.md) (reporte estático).

**Criterio de salida:** 30 días de operación desatendida: el sistema generó, validó, incubó, promovió y retiró estrategias solo, con audit trail completo y cero intervenciones de emergencia manuales.

---

### EPIC-8 — Glass-Box UI (Flutter completo)

**Objetivo:** Hasta aquí el sistema operó por CLI + paneles mínimos. Ahora se construye la experiencia que SQX no puede ofrecer.

- ZUI 3 niveles (ADR-0028): Fleet Command → Orchestrator (DAG editor con `petgraph` + CustomPainter) → Strategy Inspector.
- [`visual-dag-editor`](./features/visual-dag-editor.md), [`node-preview`](./features/node-preview.md) (ADR-0096), [`zui-navigation`](./features/zui-navigation.md).
- Visualizadores: [`umap-scatter-visualizer`](./features/umap-scatter-visualizer.md), [`parallel-coordinates-visualizer`](./features/parallel-coordinates-visualizer.md), [`cross-filtering-visualizer`](./features/cross-filtering-visualizer.md), [`interactive-stress-lab`](./features/interactive-stress-lab.md), [`plateau-copilot`](./features/plateau-copilot.md), [`toxicity-purifier`](./features/toxicity-purifier.md) (ADR-0098), [`time-warp-debugger`](./features/time-warp-debugger.md), [`monthly-performance-heatmap`](./features/monthly-performance-heatmap.md) interactivo.
- [`robustness-verdict-engine`](./features/robustness-verdict-engine.md) (LLM local, SPIKE-005), [`glass-box-ai-translator`](./features/glass-box-ai-translator.md), AST Copilot.
- Empaquetado comercial: [`flutter-packaging-manager`](./features/flutter-packaging-manager.md), [`licensing-system`](./features/licensing-system.md), instaladores 3 OS (ADR-0029).

**Pista transversal de UI (EPIC-1–EPIC-7):** cada fase entrega como máximo UNA pantalla utilitaria (telemetría de descarga EPIC-1, curva de equidad de backtest EPIC-2, tabla Databank EPIC-3, semáforo de guantelete EPIC-4, panel Autopilot+Kill Switch EPIC-5, panel de pesos EPIC-6, dashboard de salud EPIC-7). Pantallas funcionales, sin pulido. El pulido es EPIC-8.

---

### EPIC-9+ — Moonshots (estrictamente post-rentabilidad, ADR-0103 Filosofía Dual)

Orden sugerido por ROI esperado, revisable con datos reales:
1. [`universal-strategy-transpiler`](./moonshots/universal-strategy-transpiler.md) (MQL5 export — monetizable, ADR-0101)
2. [`copy-trading-engine`](./features/copy-trading-engine.md) productizado + [`saas-gateway`](./moonshots/saas-gateway.md)/[`monetization-stripe`](./moonshots/monetization-stripe.md)
3. [`deep-learning-suite`](./moonshots/deep-learning-suite.md) + DRL (Hybrid Genesis completo)
4. [`la-colmena`](./moonshots/la-colmena.md) (ADR-0086), [`marketplace-cajas-negras`](./moonshots/marketplace-cajas-negras.md) (ADR-0099)
5. [`saas-cloud-engine`](./moonshots/saas-cloud-engine.md) (ADR-0033 Trimodal modo 3) + HybridComputeCooperative (ADR-0094)
6. Resto del catálogo de moonshots según evidencia.

---

## 5. Dependencias Técnicas Duras (no negociables)

1. **EPIC-2 depende de EPIC-1:** sin PIT validator, el backtest miente.
2. **EPIC-3 depende de EPIC-2:** generar sin motor confiable es fabricar overfitting a escala.
3. **DSR (EPIC-4) depende de contar N desde EPIC-3:** orden inverso = estadística inválida.
4. **EPIC-5 depende del Score de EPIC-4:** el sizing en vivo consume el robustness score (ADR-0058) y el veto pre-trade consume el veredicto MC (ADR-0095).
5. **Versionado (ADR-0005) y los 25 campos (ADR-0020 V2) nacen en EPIC-0/EPIC-2:** retrofitearlos cuesta 10x.
6. **SPIKE-001 (Nautilus) — resuelto por ADR-0107 (crates Rust v2 vendorizados):** el diseño del backtest event-driven de EPIC-2 y la ejecución nativa de EPIC-6 ya tienen mecanismo definido. En EPIC-0 solo queda su smoke test (compilación, backtest mínimo, empaquetado LGPL); si ese smoke test fallara, se activa el Plan B escalonado del ADR-0107 antes de diseñar EPIC-2.

## 6. KPIs por Fase (jerarquía de latencias unificada)

| Ruta | SLA | Fase donde se mide |
|---|---|---|
| Pre-trade validation (Guardián) | <1ms | EPIC-5 |
| Wrapper de reglas de portafolio | <10ms | EPIC-5/EPIC-6 |
| Orden end-to-end (señal→broker) | ≤100ms | EPIC-5/EPIC-6 |
| Kill switch / watchdog | ≤5s | EPIC-5 |
| Backtest Express híbrido | Más rápido que MT5/SQX/QuantConnect en igual hardware (ADR-0114; sin KPI absoluto) | EPIC-2 |
| Carga UI Time-Warp | <200ms | EPIC-8 |
| Recuperación post-crash (Event Store) | <10s | EPIC-0/EPIC-5 |

(Esta tabla resuelve las contradicciones de KPIs detectadas entre SAD §1.2, §9 y el pipeline §6 — ver CORRECTION-PLAN ítem C1.)
