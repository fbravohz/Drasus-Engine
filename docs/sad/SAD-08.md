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

---

### Tipos Canónicos de Columna por Uso (ADR-0141)

| Uso | Tipo SQLite | Rust | Polars | Parquet |
|---|---|---|---|---|
| UUID, hash, texto, JSON, snapshot_id | `TEXT NOT NULL` | `String` | `Utf8` / `Categorical` | `BYTE_ARRAY` / `STRING` |
| Timestamp nanosegundos UTC | `INTEGER NOT NULL` | `i64` | `Int64` | `INT64` |
| Precio / volumen escalado × 10⁸ | `INTEGER NOT NULL` | `i64` | `Int64` | `INT64` |
| Contador / secuencia | `INTEGER NOT NULL` | `i64` / `u64` | `Int64` / `UInt64` | `INT64` |
| Booleano | `INTEGER NOT NULL (0/1)` | `bool` | `Boolean` | `BOOLEAN` |
| JSON payload | `TEXT NOT NULL` | `serde_json::Value` | `Utf8` | `BYTE_ARRAY` |
| Latencia en ms | `INTEGER NOT NULL` | `i64` | `Int64` | `INT64` |

**Reglas de precio:** escala fija × 10⁸ en toda la pila. `REAL` prohibido en precio/volumen. La conversión float↔entero ocurre solo en la Shell.

**Reglas de timestamp:** nanosegundos Unix UTC en `INTEGER`. `created_at` = tiempo de persistencia; `event_timestamp_ns` / `bar_timestamp_ns` = tiempo del evento de mercado. UTC estricto; zona local solo en Flutter.

**STRICT mode:** toda tabla declarada con `CREATE TABLE nombre (...) STRICT;`. Aplica a las 6 tablas del baseline y a todas las nuevas.

---

### Claves Primarias y Generadores de Secuencia (ADR-0141)

**Claves primarias:** `TEXT NOT NULL PRIMARY KEY`, valor UUIDv7 generado con `Uuid::now_v7()` del crate `uuid` (feature `v7`). Uniformidad total — sin UUID v4, sin ULID.

**Dos semánticas de secuencia (distintas, no intercambiables):**
- `event_sequence_id`: posición monótona global en tablas append-only (event-stores). Siempre con `UNIQUE`. Generación dentro de `BEGIN IMMEDIATE`.
- `row_version`: contador de versión por fila en tablas mutables. Empieza en 1, incrementa con cada UPDATE. Sin `UNIQUE` global.

---

### Política de Indexado (ADR-0141)

Convención de nombres: `idx_<tabla>_<columna1>[_<columna2>]`, minúsculas, snake_case.

**Índices obligatorios:** toda columna FK hijo; `event_sequence_id` en tablas append-only; columna de `state`/`status` si es eje de recovery al arranque.

**Índices opcionales (guiados por el query path):** compuesto `(categoría, timestamp)` en series temporales; `created_at` aislado si hay poda temporal; `(entity_type, entity_id)` para lookup de historial por entidad.

**PROHIBIDO indexar** columnas de baja cardinalidad (booleanos) o tablas con < 10 000 filas esperadas.

---

### Integridad Cruzada SQLite↔Parquet (ADR-0141)

SQLite no puede verificar mediante FK si una partición Parquet existe. La integridad se garantiza en la capa de aplicación:

1. Todo campo que referencie una partición Parquet incluye en el comentario SQL el formato canónico del valor y un `CHECK` de formato. Formato canónico de `data_snapshot_id`: `<exchange>_<symbol>_<timeframe>_<year><month>` (ejemplo: `binance_BTCUSDT_1m_202601`).
2. **Reconciler de startup:** el módulo Ingest, al inicializar la app, compara todos los `data_snapshot_id` registrados en SQLite contra los paths Parquet reales en disco. Los huérfanos se reportan en el audit-log con `action_type = 'PARQUET_ORPHAN_DETECTED'`.
3. PROHIBIDO confiar en que un `data_snapshot_id NOT NULL` implica que la partición existe.

---

### Evolución de Esquema Parquet (ADR-0141)

- Solo se añaden columnas; nunca se eliminan ni renombran.
- Toda columna nueva tiene un valor default documentado para particiones antiguas (NULL si no hay valor natural).
- Cada partición incluye el metadata field `schema_version` (string semver, ejemplo `"1.0"`, `"1.1"`).
- PROHIBIDO reescribir particiones históricas como mecanismo de migración.

---

### DuckDB vs Puerto de Feature (ADR-0141)

- **DuckDB directo (sin puerto):** lectura analítica sobre Parquets que la feature actual owna; remuestreo dinámico sobre datos propios; R&D ad-hoc.
- **Puerto de feature obligatorio:** si el Parquet pertenece a otra feature; si el resultado va a otra feature del pipeline; si la consulta tiene implicaciones de PIT.
- Regla mnemónica: *"DuckDB dentro de tu hexágono; puerto para hablar con otro hexágono."*

---

### Tablas M:N — Regla de Ownership (ADR-0141)

- Si la relación tiene atributos propios: pertenece a la feature que gestiona esa relación.
- Si no tiene atributos propios: pertenece a la feature "dueña" del lado más dependiente en el pipeline.
- PROHIBIDO tabla puente compartida entre dos features; la otra feature accede por el `OutputPort` de la dueña.

---

### Patrón Transaccional Estado+Auditoría (ADR-0141)

Todo cambio de estado de una entidad de dominio se realiza en una única transacción `BEGIN IMMEDIATE` que incluye el UPDATE de la entidad Y el INSERT en `audit_events`. Si cualquiera falla, todo hace rollback. PROHIBIDO insertar en `audit_events` fuera de esa transacción. No se implementa como trigger (la lógica de `audit_hash` requiere Rust).

---

### Configuración del Pool SQLite (ADR-0141)

Los siguientes parámetros son obligatorios en `pool.rs` (implementación en la auditoría retroactiva del Tech Lead):

| Parámetro | Valor |
|---|---|
| WAL | ya activo |
| `foreign_keys` | `ON` |
| `busy_timeout` | 5 000 ms |
| `synchronous` | `NORMAL` |
| `journal_size_limit` | 67 108 864 (64 MB) |
| `wal_autocheckpoint` | 1 000 páginas |

---

### Política de Retención y VACUUM (ADR-0141)

| Tipo | Ejemplos | Retención |
|---|---|---|
| Event-store inmutable | `audit_events`, `job_results`, `permission_decisions` | Forever. Sin poda. |
| Telemetría | `telemetry_samples` | Configurable (default: 30 días). Poda por `created_at < ?`. |
| Estado operativo | `jobs`, `sovereign_download_records` | Forever. Jobs `CANCELLED` > 90 días: poda opcional. |
| Cache/staging (futuras) | tablas temporales | TTL configurable declarado en la migración. |

`VACUUM` PROHIBIDO en runtime. Es operación de mantenimiento manual del módulo `withdraw`.

---

### Condiciones de Transición entre Módulos

| Transición | Condición Conceptual | Detalles |
|---|---|---|
| ingest → generate | Datos validados + régimen clasificado | Barras listas para exploración |
| generate → validate | Candidatas generadas | Suite de validación (WFA, Monte Carlo, coherencia) |
| validate → incubate | Validación aprobada + robustez mínima configurable | Forward testing en vivo |
| incubate → manage | Pardo test pasado + drift aceptable (configurable) | Promoción a portafolio candidato |
| manage → execute | Portafolio optimizado + reglas de riesgo listas | Activación de ejecución viva (configurable por usuario — ver ADR-0008: Configurabilidad Universal) |
| execute → feedback | Orden ejecutada / anomalía detectada | Delta real vs esperado disponible |
| feedback → withdraw | Veredicto de retiro (Drift > umbral) | Estrategia enviada a Retiro Emérito (Archivo Institucional) |
| withdraw → generate | Archivo completado + Insights | Nuevo ciclo con restricciones del fallo |

---

