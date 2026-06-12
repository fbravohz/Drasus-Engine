# PLAN DE EJECUCIÓN — Arranque de Desarrollo desde Cero (Fase F0)

**Versión:** 1.0 | **Fecha:** 2026-06-10
**Dueño:** Tech-Lead (orquestación y auditoría; este documento NO define diseño — el diseño vive en SAD/ADR/Features)
**Fuentes de verdad:** [`ROADMAP.md`](./ROADMAP.md) v1.1 (§2 Gates, §4 F0, §5 dependencias duras), [`ADR.md`](./ADR.md) (ADR-0003, ADR-0006, ADR-0011, ADR-0020 V2, ADR-0027, ADR-0029, ADR-0107), features transversales P0.
**Regla vigente:** ningún TTR de F1+ avanza a implementación (Etapa 2) mientras los Gates G1–G6 no tengan veredicto documentado como ADR (ROADMAP §2). **Los 6 gates ya tienen veredicto en ADR** (G1: ADR-0107; G2–G6: ADR-0112 a 0116); resta únicamente la validación residual de cada uno (smoke tests y spikes de medición).

---

## 1. Fase Activa y Entregable Alpha

| Campo | Valor |
| :--- | :--- |
| **Fase activa** | F0 — Fundación y Gates de Riesgo |
| **Entregable Alpha** | Esqueleto del monolito modular FCIS compilando + 6 gates de viabilidad con veredicto + fundaciones anti-retrabajo (25 campos desde la migración 0001) |
| **Duración estimada (ROADMAP)** | 4–6 semanas |
| **Criterio de salida (verbatim ROADMAP §F0)** | `cargo test` verde en esqueleto; migración 0001 aplica los 25 campos; job asíncrono sobrevive a un `kill -9` y se recupera (ADR-0011); veredictos G1–G6 escritos |

---

## 2. Tablero de Gates de Viabilidad (Spikes — máxima prioridad)

Cada spike entrega **veredicto binario + Plan B si aplica**. Yo recibo el veredicto y lo escalo al Architect para su registro como ADR (yo no redacto ADRs). Los 6 spikes corren **en paralelo** entre sí y en paralelo con la Onda 0 del backlog (§3), porque no dependen del esqueleto.

| Gate | Spike | Ingeniero asignado | Salida esperada | Estado |
| :--- | :--- | :--- | :--- | :--- |
| **G1** | Smoke test NT v2: vendorizar crates con versión fijada, compilar, correr un backtest mínimo, verificar empaquetado LGPL reenlazable | Rust-Engineer | PASA → G1 cerrado (el veredicto de fondo ya es ADR-0107). FALLA → escalo a Architect para activar Plan B escalonado del ADR-0107 | Pendiente — despacho Onda 0 |
| **G2** | ✅ Veredicto en **ADR-0112** (erradicar `tch-rs`; escalera `ndarray`/Rayon → `candle` → `burn`). Resta: smoke test CPU-first y tamaño de binario | Quant-Engineer (diseño del benchmark) + Rust-Engineer (ejecución) | Validación: cómputo CPU-first confirmado sin romper ADR-0029 | Veredicto documentado — resta validación residual |
| **G3** | ✅ Veredicto en **ADR-0113** (erradicar PySR; regresión simbólica como modo del NSGA-II nativo; minería libre → moonshot con `egg`). Resta: prototipo del modo simbólico | Quant-Engineer | Validación: GP-sobre-AST funcional como modo del motor existente | Veredicto documentado — resta validación residual |
| **G4** | ✅ Veredicto en **ADR-0114** (motor dual; ruta Express híbrida vectorizado+secuencial; modo elegido por el usuario; contrato de consistencia conservadora). Resta: spike de medición (criterio relativo vs MT5/SQX/QuantConnect, sin KPI absoluto) | Rust-Engineer (prototipo) + Quant-Engineer (validez metodológica) | Validación: ventaja competitiva medida y contrato de consistencia verificado | Veredicto documentado — resta validación residual |
| **G5** | ✅ Veredicto en **ADR-0115** (Verdict Engine determinista por plantilla; Ollama derogado como requisito; LLM `candle` opcional). Resta: implementar la plantilla determinista | Rust-Engineer | Validación: veredicto reproducible sin LLM presente | Veredicto documentado — resta validación residual |
| **G6** | ✅ Veredicto en **ADR-0116** (downsampling obligatorio en backend; `ZeroCopyBuffer` solo para cargas masivas; throttling en Rust; gRPC fallback). Resta: spike FFI que confirme números | Bridge-Engineer | Validación: latencia de stream con throttle 100ms y transferencia masiva medidas | Veredicto documentado — resta validación residual |

**Nota de gobernanza:** el único spike con UI permitida en F0 es G6, y es exclusivamente "hello world" de infraestructura (ROADMAP §F0) — cero pantallas de producto.

---

## 3. Backlog F0 — Ondas de Despacho

Clasificación Etapa 0: **ninguna feature de F0 es matemática/estrategia** (son infraestructura transversal), por lo tanto la Etapa 1 (Quant pre-código) **no aplica** en esta fase salvo para los benchmarks de gates G2/G3/G4 ya asignados arriba. Ninguna feature de F0 declara superficie UI (Etapas 3–4 no aplican, excepto el spike G6). Flujo estándar: Etapa 2 (Rust) → Etapa 5 (QA continuo + gate final).

### Onda 0 — Sin precondición (despacho inmediato, paralelo con §2)

| ID | Trabajo | TTRs / Fuente | Ingeniero | Criterio de cierre (gate QA) | Estado |
| :--- | :--- | :--- | :--- | :--- | :--- |
| W1 | Workspace Cargo: 8 módulos como crates internos + `shared` (ADR-0003, FCIS); esqueleto compilable con `public_interface` vacías por módulo | ROADMAP §F0 | Rust-Engineer | `cargo build` y `cargo test` verdes en esqueleto; estructura FCIS auditada (Thin Shell: cero lógica en orquestadores) | Pendiente |
| W2 | Migraciones SQLx embebidas: migración 0001 con el set maestro de 25 campos (ADR-0006, ADR-0020 V2) | ROADMAP §F0 | Rust-Engineer | La migración 0001 aplica los 25 campos en SQLite WAL; verificación de idempotencia | Pendiente |

### Onda 1 — Precondición: W1+W2 `Completado`

| ID | Trabajo | TTRs / Fuente | Ingeniero | Criterio de cierre (gate QA) | Estado |
| :--- | :--- | :--- | :--- | :--- | :--- |
| W3 | [`clock`](./features/clock.md) | TTR-001 (timestamp ns), TTR-002 (reloj determinista backtest-ready) | Rust-Engineer | Mismo seed/datos → misma secuencia temporal bit-a-bit | Secuenciado / En Espera |
| W4 | [`audit-log`](./features/audit-log.md) | TTR-001 (append-only + hash chain). TTR-002 (reconciliación rastro Nautilus) queda **En Espera hasta F2+** — no bloquea el Entregable Alpha de F0 | Rust-Engineer | Intento de mutación de un evento histórico es rechazado y detectado por verificación de cadena | Secuenciado / En Espera |
| W5 | [`async-job-executor`](./features/async-job-executor.md) | TTR-ASYNC-EXECUTOR-001 → 006 en cadena (queue, worker pool, persistencia, recuperación, progreso, cancelación). TTR-007 (integración con módulos costosos) es **progresivo**: se completa por fase a medida que cada módulo cobra vida | Rust-Engineer | **Test de guerra obligatorio:** job sobrevive `kill -9` y se recupera al arranque (criterio de salida F0) | Secuenciado / En Espera |
| W6 | [`crash-recovery`](./features/crash-recovery.md) | TTR-001 (persistencia transaccional hot-path), TTR-002 (modo recovery al arranque). TTR-003/004 (sincronización broker e hidratación de indicadores) **En Espera hasta F5** | Rust-Engineer | Recuperación post-crash <10s (KPI ROADMAP §6, ruta F0) | Secuenciado / En Espera |
| W7 | [`telemetry`](./features/telemetry.md) | TTR-001 (buffer de alta velocidad). TTR-002 (vistas de correlación = UI) **En Espera hasta F8** | Rust-Engineer | Telemetría local de hardware emitiendo sin bloquear el hot-path | Secuenciado / En Espera |
| W8 | [`worker-isolation-orchestrator`](./features/worker-isolation-orchestrator.md) | TTR-001 (bridge memoria compartida), TTR-002 (watchdog de procesos y graceful shutdown) | Rust-Engineer | Caída inducida de un worker no contamina al orquestador; shutdown limpio auditado | Secuenciado / En Espera |
| W9 | CLI con Clap como primera interfaz: cada comando expone las capacidades de W3–W8 (la UI Flutter NO bloquea ninguna fase hasta F8) | ROADMAP §F0 | Rust-Engineer | Comandos básicos operativos: estado de jobs, telemetría, verificación de auditoría | Secuenciado / En Espera |

### Onda 2 — Cierre de fase (precondición: Ondas 0–1 `Completado` + veredictos G1–G6 recibidos)

| ID | Trabajo | Responsable | Estado |
| :--- | :--- | :--- | :--- |
| W10 | Gate final QA de F0: suite completa contra el criterio de salida del ROADMAP (compilación, migración, kill -9, FCIS, Zero-Docker, soberanía de datos) | QA-Engineer | Secuenciado / En Espera |
| W11 | ✅ Veredictos G2–G6 registrados como ADR-0112 a 0116 por el Architect. Resta solo escalar a Architect si alguna validación residual (smoke test/spike) contradice un veredicto | Tech-Lead → Architect | Veredictos registrados — vigilancia de validación |
| W12 | Relectura de `/documentation/` post-ADRs y selección Etapa 0 del primer TTR de F1 (`ingest`: data-validator + pit-data-validator como P0 innegociables) | Tech-Lead | Secuenciado / En Espera |

---

## 4. Reglas Operativas de este Ciclo

1. **Paralelismo controlado:** §2 (spikes) y Onda 0 corren simultáneos. La Onda 1 NO arranca hasta que W1+W2 estén `Completado` — los 25 campos y el esqueleto FCIS son fundación anti-retrabajo (retrofitear cuesta 10x, ROADMAP §5.5).
2. **QA continuo:** cada entregable de Onda 0–1 pasa por QA-Engineer apenas se produce (tests unitarios, determinismo); el gate final (W10) audita el conjunto. Defecto de implementación → regresa al ingeniero dueño; defecto de diseño/spec → escalo al Architect.
3. **Bloqueo F1+:** prohibido despachar TTRs de F1 a Etapa 2 hasta W11 cerrado. Excepción ya gobernada: nada — G1 resuelto no abre F1; se requieren los 6 veredictos.
4. **SLA aplicable:** en F0 solo se exige el KPI de su ruta (recuperación post-crash <10s). Prohibido exigir SLAs de fases futuras (ej. <1ms pre-trade) a entregables de F0.
5. **Sin UI de producto:** la pista transversal de UI inicia en F1 (una pantalla utilitaria por fase). En F0 solo existe el spike G6.
6. **Cero invención:** si algún TTR de las features transversales resulta ambiguo o huérfano contra TEMPLATES.md durante el despacho, se bloquea ese ítem y se escala al Architect con evidencia — los ingenieros no rellenan vacíos de spec.

## 5. Registro de Estado (se actualiza al cierre de cada ítem)

| Ítem | Estado | Última transición |
| :--- | :--- | :--- |
| G1 | Pendiente (smoke test); veredicto ADR-0107 | 2026-06-10 — plan emitido |
| G2–G6 | Veredictos documentados (ADR-0112 a 0116); resta validación residual | 2026-06-11 — gates resueltos por el Architect |
| W1–W2 | Pendiente (despacho autorizado) | 2026-06-10 — plan emitido |
| W3–W9 | Secuenciado / En Espera (precondición W1+W2) | 2026-06-10 — plan emitido |
| W10–W12 | Secuenciado / En Espera (cierre de fase) | 2026-06-10 — plan emitido |
