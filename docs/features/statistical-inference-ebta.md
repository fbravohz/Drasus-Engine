# Statistical Inference Layer (EBTA)

**Carpeta:** `./features/statistical-inference-ebta/`
**Estado:** En Diseño

> **Aplicación del DSR (ADR-0151):** el DSR se aplica **por punto de decisión**, con N acotado a los ensayos que compitieron por esa selección (reconstruido desde `expedition_lineage`) y correlación por `V[{SR_n}]` — **nunca** con el `trials_count` de por vida (guardia de límite degenerado: N→∞ ⇒ DSR→0). Impacto progresivo (ADR-0137).
**Última actualización:** 2026-04-30
**Decisión Arquitectónica Asociada:** ADR-0067 (Capa de Inferencia Estadística (EBTA))

## ¿Qué es esta feature?

La capa EBTA (Evidence-Based Technical Analysis) es el filtro de rigor estadístico final del módulo `validate`. Su objetivo es cuantificar la probabilidad de que el rendimiento de una estrategia sea producto del azar o del sobreajuste derivado de probar múltiples variables (Data-Mining Bias).

**Problema que resuelve:** Los motores de minería genética pueden generar miles de estrategias por "fuerza bruta" que parecen rentables en el backtest pero carecen de Alpha real. EBTA deflacta los resultados y ajusta la significancia estadística para proteger el capital institucional.

## Comportamientos Observables

- [ ] El sistema calcula el **Deflated Sharpe Ratio (DSR)** extrayendo el número de intentos ($N$) del metadato de la estrategia.
- [ ] Se ejecuta el test de **Romano-Wolf** mediante bootstrap para emitir un p-value ajustado por múltiples comparaciones.
- [ ] El **Market Detrender** genera una serie de tiempo sintética sin tendencia base; la estrategia debe mantener rentabilidad en este escenario.
- [ ] La **Logic Inversion** valida que el comportamiento de la regla opuesta sea simétricamente robusto (o justifique su asimetría).

## Restricciones

- NUNCA aprobar una estrategia con un p-value de Romano-Wolf > 0.05 (configurable).
- NUNCA ejecutar DSR si el dato de $N$ (intentos totales) es desconocido o nulo.
- EL cálculo de Romano-Wolf DEBE realizarse en GPU o Rust SIMD/Rayon Parallel para no bloquear el Event-Loop.

## Parámetros Configurables

| Parámetro | Default | Rango | Qué hace | FIJO/CONFIG |
|---|---|---|---|---|
| P_VALUE_THRESHOLD | 0.05 | 0.01 - 0.10 | Umbral de significancia para Romano-Wolf | CONFIG |
| BOOTSTRAP_ITERATIONS | 1000 | 500 - 10000 | Número de permutaciones para tests estadísticos | CONFIG |
| DETRENDER_BENCHMARK | SPY | Lista de activos | Activo base para eliminar tendencia | CONFIG |
| MIN_TRADES_PROM | 30 | 10 - 100 | Mínimo de trades para el cálculo de Pessimistic Return | CONFIG |

## Estructura Interna (FCIS)

- **Core (Lógica Pura):** Algoritmos de deflación de Sharpe, funciones de bootstrap estadístico, lógica de detrending vectorial.
- **Shell (Infraestructura):** Orquestador que inyecta los Ledgers de NautilusTrader hacia los arrays de `ndarray`/Rust SIMD-Rayon (CPU-first, ADR-0112).
- **Frontera Pública:** Interfaz que recibe un `ExecutableContainer` y devuelve un `RobustnessReport` enriquecido con métricas EBTA.

## Ciclo de Vida de la Feature — Statistical Inference Layer

### Entrada
- Ledger de operaciones (Trades) de la estrategia.
- Serie de tiempo del activo base y del benchmark.
- Metadatos de generación (Intentos $N$, Varianza de Sharpe de la sesión).

### Proceso
1.  **DSR Engine:** Calcula la probabilidad de que el Sharpe sea inflado basándose en $N$.
2.  **Bootstrap Suite:** Ejecuta Romano-Wolf para corregir el error por familia de hipótesis.
3.  **Trend Removal:** Calcula los retornos residuales restando la componente tendencial del benchmark.
4.  **Symmetry Check:** Evalúa la robustez de la lógica inversa.

### Salida
- Score de EBTA (0-100) integrado en el Robustness Score Aggregator.
- Veredicto de Significancia (APROBADA/FALLIDA).

## Tareas (TTRs)

### **TTR-001: Implementación del Motor DSR**
Desarrollar la lógica de deflación del Sharpe Ratio utilizando la distribución de máximos de Sharpe observados en la sesión de minería.

### **TTR-002: Orquestador de Romano-Wolf con Bootstrap**
Crear el pipeline de remuestreo masivo acelerado por hardware para el cálculo del p-value ajustado.

### **TTR-003: Filtro Market Detrender Vectorial**
Implementar la eliminación de tendencia base sobre las series de tiempo de retornos acumulados.

### **TTR-004: Test de Simetría (Logic Inversion)**
Implementar la validación automática mediante la inversión de señales de entrada. El sistema debe ejecutar la lógica opuesta y verificar que la degradación sea simétrica, confirmando que el indicador es el motor del Alpha y no el ruido del activo.

## Gobernanza y Estándares

- **Local-First (ADR-0016):** 100% Local.
- **Inundación de Fundaciones (ADR-0020): Perfil B (IA / R&D)** — inferencia estadística sobre el linaje de generación ($N$ pruebas).

  | Categoría | Campo | Descripción |
  | :--- | :--- | :--- |
  | **I. Identidad** | `id` | Identificador único de la inferencia |
  | | `created_at` | Timestamp del cálculo |
  | | `updated_at` | Timestamp de última modificación del registro |
  | | `audit_hash` | Hash de integridad del veredicto estadístico |
  | | `audit_chain_hash` | Hash encadenado del historial |
  | | `event_sequence_id` | Secuencia de recuperación |
  | **II. Soberanía** | `owner_id` | Dueño del experimento |
  | | `manifest_id` | Estrategia/genoma evaluado |
  | **III. Pesos/Arquitectura** | `logic_hash` | Hash del motor de inferencia (EBTA) |
  | | `data_snapshot_id` | Snapshot de los retornos analizados |
  | | `version_node_id` | Versión del motor estadístico |
  | **IV. Hardware** | `node_id` | ID del hardware de cómputo (VRAM) |
  | | `process_id` | PID del proceso de inferencia |
- **Contrato de Persistencia:** Grupo I completo + Perfil B arriba, más los campos propios: `dsr_score`, `p_value`, `detrended_sharpe`.
- **Rastro de Evidencia:** Veredicto de significancia para el módulo de `feedback`.
