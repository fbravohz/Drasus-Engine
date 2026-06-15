## 8. Arquitectura de Datos

### Flujo de Datos con Features Reutilizables (Happy Path Completo - 8 Módulos)

```
FLUJO DE EJECUCIÓN DURANTE EL DÍA:

1. ingest: Ingesta de barras de mercado
   ├─► Features consumidas: [data-validator], [pit-data-validator], [hmm-regime-detection], [audit-log]
   ├─► Lógica pura: Normalizar precios
   ├─► Validación: PIT-real (sin look-ahead bias)
   ├─► Acceso datos: Guardar barra en base de datos
   └─► Detección régimen: Identificar estado del mercado (TRENDING, CHOPPY, etc.)

2. generate: Generar candidatos [Proceso batch offline]
   ├─► Features consumidas: [nsga2-optimizer], [hmm-regime-detection], [zero-crossing-filter], [strategy-ensemble], [audit-log]
   ├─► Lógica pura: Evolución multi-objetivo NSGA-II
   ├─► Lógica pura: Descubrimiento de ecuaciones por regresión simbólica nativa (modo NSGA-II, ADR-0113)
   ├─► Lógica pura: Filtrado ortogonal de señales (independent de factores)
   ├─► Síntesis: Ensemble (NSGA + simbólica nativa + HMM) → Estrategias híbridas
   └─► Acceso datos: Guardar candidatos → estado = Pendiente

3. validate: Validar estrategia [Proceso offline, reproducible]
   ├─► Features consumidas: [pit-data-validator], [backtest-engine], [walk-forward-analyzer], [factor-decomposition], [alpha-purity-analyzer], [zero-crossing-filter], [signal-correlation-analyzer], [equity-curve-tracker], [slippage-models], [institutional-metrics], [audit-log], [pca-toxicity-analyzer], [autoencoder-outlier-detector]

   ├─► Validación PIT: Asegurar datos sin look-ahead
   ├─► Backtesting: Ejecución histórica con slippage realista
   ├─► Lógica pura: Análisis walk-forward (robustez en ventanas)
   ├─► Análisis alpha: Descomposición FF5 (habilidad vs factor luck)
   ├─► Análisis señales: Correlaciones, ortogonalidad, diversificación
   ├─► Tracking: Equity curve para Sharpe, Max DD, ratios
   └─► Acceso datos: Guardar análisis → resultado = APROBADO/REVISAR/RECHAZADO

4. incubate: Ejecución paper trading [Quarantine 7 Días, Extended 21 Días o Legacy 3-6 meses — ADR-0088]
   ├─► Features consumidas: [paper-trader], [incubation-manager], [backtest-engine], [slippage-models], [equity-curve-tracker], [institutional-metrics], [trade-reconciler], [order-fsm], [audit-log]
   ├─► Simulación forward: Trading simulado con spreads reales (Paper Trading tradicional o Cuarentena Acelerada de 7 Días con Eutanasia Predictiva por MAE flotante).
   ├─► Cono de Silencio: Proyección de bandas de confianza Monte Carlo (1, 2 y 3 sigmas) para auditar la equidad en caliente.
   ├─► Broken Strategy Flag: Kill Switch automático (pausa, liquidación y reasignación) si la equidad cruza el límite inferior de -1 sigma.
   ├─► Tracking & Eficiencias: Medición de Return Efficiency y Drawdown Efficiency vs backtest.
   └─► Acceso datos: Decidir promoción → promovido a vivo = sí


5. manage: Optimizar y Rebalancear Portafolio [Asignación Adaptativa y Daemon de Rebalanceo]
   ├─► Features consumidas: [portfolio-optimizer], [portfolio-rules], [federated-portfolio], [portfolio-backtest], [signal-correlation-analyzer], [factor-decomposition], [equity-curve-tracker], [institutional-metrics], [hmm-regime-detection], [audit-log]
   ├─► Arquitectura de Contenedores Federados: Aislamiento lógico de reglas y gobernanza autónoma individual de múltiples subportafolios dentro del ecosistema unificado.
   ├─► Simulación de Portafolio Real (Real Portfolio Backtest): Simulación concurrente de múltiples estrategias compartiendo capital de margen, compounding configurable y sincronización de sesiones de mercado.
   ├─► Motores de Pesaje: Asignación clásica (Markowitz, HRP, Equal Weighting, Minimum Variance) y Ensamblador D-Score.
   ├─► Volatility Targeting Engine: Ajuste dinámico de exposición de forma inversa al ATR para mantener constante el riesgo en dólares ($R) de forma parametrizable.
   ├─► Risk-Parity Normalizado: Desacoplamiento de ATR macro para mitigar drawdowns durante pánicos.
   ├─► Mapas de Cointegración, Cópulas y Correlation Neutralizer: Modelado de dependencias de colas pesadas, coberturas extremas fijas, y neutralización/capado de lotaje activo ante cointegraciones > 0.8 en vivo.
   ├─► Router Viviente: Rotación de capital desde lateralidades estancadas (>72h) hacia vectores de liquidez eficientes.
   ├─► Auto-Rebalancing Daemon: Disparador automático por triggers (HMM régimen, Calendario semanal/mensual, Threshold de desviación o alertas VaR/CVaR).
   ├─► Búsqueda Genética de Portafolios: Motor genético offline que busca qué estrategias del Databank combinar óptimamente usando la Weighted Fitness Formula.
   ├─► Análisis de Solapamiento Temporal Real: Consultas vectoriales DuckDB de colisiones de tickets abiertos barra a barra con cálculo de riesgo acumulado simultáneo máximo.
   ├─► Portfolio Weights Rescaler & Ledger: Conversión de pesos en lotes exactos y simulación continua por hora de balance y margen integrado.
   ├─► Mitigación de Riesgo: Restricción a máximo 1 rebalanceo por día (Circuit Breaker) y suspensión si la varianza del portafolio es mayor a 2σ.
   └─► Acceso datos: Historial inmutable en SQLite `portfolio_rebalancing_history` (pesos, régimen, slippage).


6. execute: Colocar orden [EJECUCIÓN VIVA - validación <1ms; orden end-to-end ≤100ms]
   ├─► Features consumidas: [broker-connector], [order-fsm], [slippage-models], [equity-curve-tracker], [institutional-metrics], [hmm-regime-detection], [audit-log]
   ├─► [10 validaciones (ADR-0025): liquidez/spread, slippage, tamaño de posición, exposición de portafolio, correlación, drawdown, pérdida diaria, frecuencia de órdenes, margen, aprobación final]
   ├─► Lógica pura: Transición de estado orden (máquina 64-bit, atómica)
   ├─► Orquestación: Enviar a broker
   ├─► Acceso datos: Guardar ejecución en transacción
   └─► Supervisión: Latido de vida + botón de emergencia + detección anomalías

7. feedback: Veredicto de salud y cierre de bucle [Continuo + cierre batch] — ANALISTA
   ├─► Features consumidas: [pardo-comparison], [trade-reconciler], [anomaly-detector], [factor-decomposition], [alpha-purity-analyzer], [equity-curve-tracker], [audit-log]
   ├─► Lógica pura: Reconciliar real vs esperado (spreads, paper vs vivo)
   ├─► Lógica pura: Diagnosticar causa de degradación (¿murió el Alpha o solo el Beta/régimen?)
   ├─► Detectar anomalías: Cambios ejecución, brechas datos, correlaciones rotas
   ├─► Orquestación: Emitir veredicto de continuidad/retiro (señal AUTO_WITHDRAW si procede)
   └─► Acceso datos: Sugerir constraints a generate (cierre de bucle causal, ADR-0015)

8. withdraw: Salida controlada [Actuador del veredicto de feedback]
   ├─► Lógica pura: Recibir veredicto de retiro (no monitorea; el monitoreo es de feedback)
   ├─► Orquestación: Transición FSM Ejecutando → En Pausa (ventana de veto) → Retirado/Archivo
   └─► Acceso datos: Archivar métricas terminales y notificar a manage → rebalanceo
```

### Persistencia
* **SQLite:** Local, modo **WAL** (Write-Ahead Logging) forzado para lectura/escritura concurrente.
* **Auto-Recovery:** Al arrancar, el orquestador consulta la tabla de `jobs` y reanuda automáticamente tareas interrumpidas (`RUNNING`).
* **Invariante de Trazabilidad:** Todo cambio de estado atómico genera un registro en el `audit-log`.
* **Zero-Copy Performance:** Uso de Polars/Arrow para mover grandes volúmenes de datos OHLCV sin serialización costosa.

### Condiciones de Transición entre Módulos

| Transición | Condición Conceptual | Detalles |
|---|---|---|
| ingest → generate | Datos validados + régimen clasificado | Barras listas para exploración |
| generate → validate | Candidatas generadas | Suite de validación (WFA, Monte Carlo, coherencia) |
| validate → incubate | Validación aprobada + robustez mínima configurable | Forward testing en vivo |
| incubate → manage | Pardo test pasado + drift aceptable (configurable) | Promoción a portafolio candidato |
| execute → feedback | Orden ejecutada / anomalía detectada | Delta real vs esperado disponible |
| feedback → withdraw | Veredicto de retiro (Drift > umbral) | Estrategia enviada a Retiro Emérito (Archivo Institucional) |
| withdraw → generate | Archivo completado + Insights | Nuevo ciclo con restricciones del fallo |
por usuario (ver ADR-0008: Configurabilidad Universal)*

---

