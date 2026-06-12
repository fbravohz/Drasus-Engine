# Statistical Inference Layer (EBTA)

**Carpeta:** `./features/statistical-inference-ebta/`
**Estado:** En Diseño
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
- **Inundación de Fundaciones (ADR-0020 V2):** 
    - **Perfil IA / R&D:** Foco en Linaje de Generación ($N$) + Hardware de Cómputo (VRAM).
- **Contrato de Persistencia:** Campos de auditoría (DSR Score, p-value, Detrended Sharpe).
- **Rastro de Evidencia:** Veredicto de significancia para el módulo de `feedback`.
