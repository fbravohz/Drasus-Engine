# Universal Basket Backtester — Agregador de Equidad Multi-Activo

**Carpeta:** `./features/universal-basket-backtester/`
**Estado:** En Diseño
**Última actualización:** 2026-04-13
**Decisión Arquitectónica Asociada:** ADR-0005 (Versioning con DAG)

---

## ¿Qué es esta feature?

Es un motor de orquestación de simulaciones diseñado para evaluar una estrategia (o un conjunto de ellas) sobre múltiples activos y temporalidades de forma simultánea. Genera una **Curva de Equidad Global** agregada, permitiendo ver el desempeño real del capital como si fuera un portafolio único.

**Problema que resuelve:** El sesgo de ajuste de curvas (*curve-fitting*). Una estrategia puede funcionar bien en BTC/USDT por suerte, pero si falla en otros 5 activos similares con los mismos parámetros, es "frágil". Esta feature expone esa fragilidad instantáneamente.

## Comportamientos Observables

- [ ] El usuario define una "Canasta" de activos (ej: Top 10 Alts) y presiona "Basket Backtest".
- [ ] El sistema ejecuta el backtest en paralelo para cada activo usando hilos independientes de CPU (Rust SIMD/Rayon).
- [ ] Se genera una única gráfica de rendimiento que suma las ganancias y pérdidas de toda la canasta.
- [ ] El sistema calcula métricas de correlación entre los activos de la canasta para detectar riesgos ocultos.

## Restricciones

- **ALTA FIDELIDAD:** Cada activo en la canasta debe seguir las reglas de fricción institucional (Swaps/Comisiones) individuales de su mercado.
- **DETERMINISMO:** El resultado agregado debe ser reproducible al 100%.
- **Límite Técnico:** El número de activos en la canasta está limitado por la RAM disponible (Out-of-Core DuckDB recomendado para datasets masivos).

---

## Ciclo de Vida de la Feature — Universal Basket Backtester

### Entrada
- Estrategia candidata (DNA/Parámetros).
- Lista de símbolos (Basket).
- Datos históricos OHLCV de todos los símbolos (Parquet/Arrow).

### Proceso
- Despliegue de instancias de simulación en paralelo via NautilusTrader.
- Agregación temporal de resultados (Sincronización de eventos de ejecución).
- Sumatoria de P&L ajustado por pesos.

### Salida
- Curva de Equidad Agregada.
- Reporte de Robustez Multi-activo (Sharpe consolidado).

### Contextos de Uso

**Contexto 1: Stress Test de Generalización (MOD-03)**
- Valida si un Alfa descubierto en un activo es "Universal" o solo un error estadístico local.

---

## Tareas (TTRs)

### **TTR-001: Orquestador de Simulaciones en Paralelo**
*   **¿Cuál es el problema?** Ejecutar backtests uno tras otro es demasiado lento para canastas grandes.
*   **¿Qué tiene que pasar?** El sistema debe repartir la carga de la canasta entre los núcleos de CPU disponibles usando hilos nativos de Rust (Rayon).
*   **¿Cómo sé que está hecho?**
    - [ ] El tiempo de ejecución de una canasta de 10 activos es < 3x el tiempo de uno solo.

### **TTR-002: Agregador Sincronizado de Equidad**
*   **¿Cuál es el problema?** Las órdenes ocurren en diferentes momentos en diferentes activos; sumarlas mal crea resultados falsos.
*   **¿Qué tiene que pasar?** El sistema debe alinear cronológicamente todos los trades de la canasta antes de calcular la equidad global.
*   **¿Cómo sé que está hecho?**
    - [ ] La equidad global refleja correctamente el drawdown simultáneo de varios activos.

---

## Persistencia (Inundación de Fundamentos — ADR-0020 V2)

Perfil **B (IA/R&D)**: Grupo I (universal) + Soberanía (II) + Pesos/Arquitectura, subset de III + Hardware (IV). Solo los campos relevantes para un reporte de backtest multi-activo (Filtro de Relevancia, ADR-0020 V2):

| Categoría | Campo | Descripción |
| :--- | :--- | :--- |
| **I. Identidad** | `id` | Identificador único del reporte de canasta |
| | `created_at` | Timestamp de inicio del Job |
| | `audit_hash` | Hash del veredicto de generalización |
| | `audit_chain_hash` | Hash de la integridad de la canasta (Símbolos) |
| **II. Soberanía** | `owner_id` | Usuario que lanzó el stress test |
| | `institutional_tag` | Etiqueta de cumplimiento/entorno |
| **III. Pesos/Arquitectura** | `logic_hash` | Hash del motor del backtester universal |
| | `data_snapshot_id` | Puntero al dataset consolidado |
| | `indicator_state_hash` | Snapshot del Sharpe consolidado de canasta |
| | `version_node_id` | Versión de la estrategia en el DAG |
| **IV. Hardware** | `node_id` | ID del hardware físico (Cores usados) |
| | `process_id` | PID del orquestador de procesos |
| | `execution_latency_ms` | Tiempo total de cómputo en paralelo |

## Gobernanza y Estándares (Fijos)

## Dependencias y Bloqueantes
**Depende de:** `backtest-engine`.
**Bloquea:** `portfolio-optimizer`.
