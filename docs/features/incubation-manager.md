# Incubation Manager — Cone of Silence (Sandbox de Cuarentena e Incubación)

**Carpeta:** `./features/incubation-manager/`
**Estado:** Especificación (Fase 2)
**Última actualización:** 2026-05-31
**Decisión Arquitectónica Asociada:** ADR-0088 (Protocolo de Incubación & Cono de Silencio)

---

## ¿Qué es esta feature?

El Incubation Manager es el componente transversal responsable de orquestar el periodo de prueba final en tiempo real (Paper Trading) antes de autorizar la promoción del capital virtual a ejecución con dinero real. Su principal función es la **Validación de Deriva (Drift)** y la prevención de drawdowns catastróficos en entornos fondeados (Prop-Firms) mediante una supervisión estadística en caliente.

El sistema implementa tres paradigmas de validación (perfiles configurables, ADR-0088):
1. **Modo Legacy (Paper Trading Convencional / Incubación Prolongada, 3-6 meses):** Evaluación asintótica tradicional durante periodos de 3 a 6 meses demo para certificar el factor Pardo en tiempo largo y verificar la robustez global del modelo.
2. **Modo Extended (Sandbox Extendido, 21 días):** Modo intermedio que aplica el mismo motor de Eutanasia Predictiva y Cono de Silencio del Modo Quarantine, pero sobre una ventana de 21 días, para estrategias que requieren mayor confirmación estadística antes de promoción a capital real.
3. **Modo Quarantine (Sandbox 7-Day Live Quarantine con Eutanasia Predictiva):** Cuarentena virtual ultra-abreviada de 7 días. El motor contrasta en tiempo real el desempeño de la estrategia frente a una Matriz de Avance Progresivo (WFM) retroactiva estática. Si el comportamiento de las operaciones se desvía del comportamiento fuera de muestra (OOS) esperado, excediendo subrutinas predictivas de letargología o un incremento flotante de riesgo (Excursión Máxima Adversa extra de +15% o detención asimilada), se ejecuta una **Purga Sintética** que elimina por completo el genoma de la estrategia del catálogo operativo para evitar que cause drawdowns colaterales en la cuenta de capital real master del Commodity Pool Operator (CPO).

---

## Comportamientos Observables

- [ ] **Selección de Perfil del Pipeline:** El operador puede inicializar una estrategia en `Status::Incubating` eligiendo entre Quarantine (7 días), Extended (21 días) o Legacy (3-6 meses).
- [ ] **Ejecución en Sandbox Quarantine:** Durante el Sandbox de 7 días, la estrategia es inyectada en un entorno de ejecución virtual de alta fidelidad, simulando la fricción de órdenes sin arriesgar capital real.
- [ ] **Eutanasia Predictiva Activa:** Si el riesgo flotante acumulado de la estrategia durante las transacciones virtuales supera el límite establecido (+15% de Excursión Máxima Adversa con respecto al óptimo OOS del WFM), el orquestador aborta la sesión inmediatamente, eliminando el genoma de la estrategia.
- [ ] **Proyección del Cono de Silencio:** El sistema dibuja bandas de confianza basadas en percentiles dinámicos (1, 2 y 3 sigmas) proyectadas a partir del backtest original y las simulaciones Monte Carlo previas.
- [ ] **Broken Strategy Flag:** Si la equidad en caliente de la sesión cruza la banda inferior del Cono de Silencio (-1 sigma), la estrategia es catalogada automáticamente como degradada (`drifted`). El sistema la detiene, liquida las posiciones y reasigna los recursos de capital simulado.

---

## Restricciones

- **Inmutabilidad Absoluta:** Queda terminantemente prohibido alterar cualquier parámetro, lógica o ponderación de indicadores durante el Sandbox o periodo de incubación activa.
- **Cero Tolerancia a Derivas:** La violación de la banda de confianza (-1 sigma) activa el Kill Switch de forma atómica con latencia menor a 1ms a nivel del núcleo en Rust.
- **Aislamiento en Sandbox:** Las órdenes del sandbox son virtuales y se inyectan únicamente al libro de órdenes local del paper trader, sin interacción alguna con APIs de ejecución real en vivo.

---

## Parámetros Configurables

| Parámetro | Default | Rango | Qué hace | FIJO/CONFIG |
|---|---|---|---|---|
| INCUBATION_MODE | quarantine | legacy / extended / quarantine | Selecciona el perfil de incubación: paper trading convencional (3-6 meses), sandbox extendido (21 días) o sandbox acelerado (7 días). | CONFIG |
| SANDBOX_DURATION_DAYS | 7 | 3 - 30 días | Duración del periodo de cuarentena acelerado en días. | CONFIG |
| LEGACY_DURATION_MONTHS | 3 | 1 - 12 meses | Duración del paper trading tradicional en meses. | CONFIG |
| MAX_SHARPE_DRIFT | -20% | -50% a 0% | Desviación máxima aceptable del Sharpe en vivo vs backtest. | CONFIG |
| EXTRA_MAE_LIMIT | 15% | 5% - 50% | Porcentaje extra de Excursión Máxima Adversa flotante tolerable antes de eutanasia. | CONFIG |
| CONE_SIGMA_LIMIT | -1.0 | -3.0 a -0.5 | Desviación en sigmas de la banda inferior del cono para activar la Broken Strategy Flag. | CONFIG |

---

## Estructura Interna (FCIS — ADR-0002)

- **Core (Lógica Pura):** Algoritmos de cálculo de bandas de confianza, evaluación del rebasamiento MAE y veredictos de deriva en `drift.rs`.
- **Shell (Infraestructura):** Orquestador de la sesión de sandbox, listener de ticks en caliente y gestor de eventos de eutanasia/suspensión.
- **Frontera Pública:** Interfaz asíncrona de orquestación `start_incubation_session()` y puertos de consulta de estado del cono.

---

## Ciclo de Vida de la Feature — Incubation Manager

### Entrada
- Estrategia aprobada en el módulo de validación con su catálogo de parámetros inmutables.
- Historial de simulaciones Monte Carlo asociadas (semillas y percentiles originales).
- Stream de datos de mercado (ticks/velas en tiempo real).

### Proceso
- Mapea barra a barra el comportamiento del PnL y la excursión flotante de los trades simulados.
- Proyecta bandas estadísticas (1, 2, 3 sigmas) hacia adelante en función de la distribución Monte Carlo.
- Compara diariamente las métricas de `Return Efficiency` y `Drawdown Efficiency` frente a las expectativas del backtest.
- Monitorea constantemente los umbrales de rebasamiento OOS y desvíos de la banda de confianza (-1 sigma).

### Salida
- Reporte inmutable de consistencia diaria.
- Veredicto de sesión: `PROMOTED` (promoción a real), `EUTHANIZED` (purga del genoma por rebasar MAE), o `DRIFTED` (desactivación y Broken Strategy Flag activada).

### Contextos de Uso

**Contexto 1: Cuarentena de Cuentas de Fondeo (Sandbox Quarantine)**
- Filtro ultra-veloz de 7 días antes de exponer cuentas master reales al capital de nuevas estrategias de R&D.

**Contexto 2: Paper Trading Convencional (Legacy Incubating)**
- Seguimiento de consistencia estadística en tiempo largo (3-6 meses) para validar resiliencia estructural.

---

## Tareas (TTRs)

### **TTR-001: Monitor de Cuarentena y Eutanasia Predictiva (MAE Tracker)**
*   **¿Cuál es el problema?**
    Las estrategias sobreajustadas en backtesting a menudo colapsan rápidamente al operar en vivo, pero detectarlo con paper trading tradicional cuesta de 3 a 6 meses de tiempo muerto.
*   **¿Qué tiene que pasar?**
    El sistema realiza un rastreo barra a barra de la MAE flotante de la sesión virtual durante un lapso estricto de 7 días. Si la excursión se desfasa más de un 15% del límite OOS con respecto a la matriz WFM retroactiva, la sesión se aborta instantáneamente, eliminando el genoma de la base de datos de producción para evitar el drawdown del gestor (CPO).
*   **¿Cómo sé que está hecho?**
    - [ ] Puedo forzar una simulación con deslizamientos y spreads ampliados en el Sandbox de 7 días y observar la purga automática por rebasamiento de MAE.
    - [ ] La base de datos del hall de la fama elimina el genoma completo de la estrategia al dispararse la eutanasia.
    - [ ] El log del sistema registra la alerta de purga con la latencia exacta del disparo.
*   **¿Qué no puede pasar?**
    - No se permite la manipulación de los parámetros del modelo durante las fases de cuarentena.
    - La eutanasia no debe dejar posiciones huérfanas en el motor simulado; debe cerrarlas inmediatamente.

### **TTR-002: Proyector del Cono de Silencio (Statistical Confidencer)**
*   **¿Cuál es el problema?**
    El operador cuantitativo no puede determinar visual o matemáticamente si una racha perdedora inicial en papel es una fluctuación normal o una falla estructural (drift) del modelo.
*   **¿Qué tiene que pasar?**
    El sistema proyecta bandas de confianza (1, 2 y 3 sigmas) hacia el futuro basándose en los resultados de las simulaciones Monte Carlo previas del backtest, calculando diariamente la `Return Efficiency` (Real / Esperado) y la `Drawdown Efficiency` (Degradación de Drawdown).
*   **¿Cómo sé que está hecho?**
    - [ ] El gráfico interactivo del dashboard dibuja el cono sombreado en sus tres percentiles de desviación.
    - [ ] El sistema actualiza diariamente los ratios de eficiencia en el reporte de persistencia.
*   **¿Qué no puede pasar?**
    - Las bandas del cono no pueden ser recalculadas estocásticamente a favor del rendimiento en caliente; el baseline permanece inmutable.

### **TTR-003: Broken Strategy Flag (Kill Switch por Deriva)**
*   **¿Cuál es el problema?**
    Cuando una estrategia sufre un drift severo en caliente por cambio drástico del régimen de mercado, el capital real (o virtual) se degrada sin control del operador humano.
*   **¿Qué tiene que pasar?**
    Si la curva de equidad real en caliente sale del Cono de Silencio cruzando el borde inferior (-1 sigma), el sistema activa automáticamente la Broken Strategy Flag, pausa la estrategia de inmediato, cancela órdenes virtuales pendientes, liquida las posiciones y reasigna su porción de capital.
*   **¿Cómo sé que está hecho?**
    - [ ] Al cruzar la línea de -1 sigma en equidad, se dispara el apagado atómico en menos de 1ms.
    - [ ] El estado del DAG de versiones marca la estrategia como pausada (`DRIFTED`).
*   **¿Qué no puede pasar?**
    - No se permite continuar operando si la Broken Strategy Flag está en estado activo.

---

## Persistencia (Inundación de Fundamentos — ADR-0020 V2)

Cada reporte de incubación y sesión activa registra el set de relevancia técnica de perfil Ops / Hot-Path:

| Categoría | Campo | Descripción |
| :--- | :--- | :--- |
| **I. Identidad** | `id` | Identificador único de la sesión de incubación |
| | `created_at` | Timestamp de inicio de la incubación (nanosegundos) |
| | `audit_chain_hash` | Hash criptográfico del historial de la sesión |
| **II. Soberanía** | `owner_id` | Propietario del capital o cuenta simulada |
| | `institutional_tag` | Tag de cumplimiento (AUDIT / OPERATIONAL) |
| **III. Pesos/Arquitectura** | `logic_hash` | Hash de la lógica de trading inmutable |
| | `indicator_state_hash` | Snapshot del drift medido (Return / Drawdown Efficiency) |
| | `version_node_id` | Identificador en el DAG de versiones de la estrategia |
| **IV. Hardware** | `node_id` | ID único del hardware físico ejecutor |
| | `process_id` | PID del worker que gestiona la sesión de sandbox |
| | `execution_latency_ms` | Latencia de cálculo de la telemetría en tiempo real |

---

## Dependencias
- [`paper-trader`](../features/paper-trader.md) — para la ejecución virtual.
- [`monte-carlo-simulator`](../features/monte-carlo-simulator.md) — para los conos de confianza.
- [`equity-curve-tracker`](../features/equity-curve-tracker.md) — para la curva de equidad.
