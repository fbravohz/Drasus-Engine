# Analizador Walk-Forward (WFA)

**Carpeta:** `./features/walk-forward-analyzer/`
**Estado:** Especificación (Extraída de backtest-engine)
**Última actualización:** 2026-06-11
**Decisiones Arquitectónicas Asociadas:** ADR-0108, ADR-0110

---

## ¿Qué es?

Es el motor de validación dinámica de Drasus Engine. Utiliza una **Matriz WFA** y el método **CPCV (Cross-Validation Combinatorial)** con técnicas de **Purging** y **Embargo**.
Su propósito principal es el **Out-of-Sample (OOS) Blind Validation**: Validación automática en datos nunca vistos por el minero, reservando un segmento temporal que el algoritmo genético jamás tocó para certificar la robustez.

- **Regla Inquebrantable:** La poda temporal NUNCA altera el historial pasado. Su único propósito es emitir "Leyes" para el comportamiento *futuro* (Paper Trading y Live). Si se altera el pasado, incurrimos en trampa de sobreajuste (Curve Fitting).
- **Dependencias:** Utilizado como un puente desde `validate` (donde se detecta) hacia `manage` y `execute` (donde se aplica la cirugía).

**Compuerta de Robustez del Genoma de Régimen y Filtro de Entorno — WFA Segmentado por Régimen (ADR-0108/ADR-0110):** cuando el Genoma de Régimen y Filtro de Entorno está activo, este analizador opera además en un modo de validación segmentado: en lugar de ventanas de calendario continuas, agrupa el historial en ventanas curadas por régimen (capitulación, rango, tendencia de baja volatilidad) y exige que la máscara Permitido/Prohibido resuelta por el genoma supere el veredicto WFE/PBO de forma independiente en cada tipo de ventana, no solo en el agregado.

### Ciclo de Vida de la Feature — WFA

### Entrada
- Estrategia candidata.
- Historial de barras Ohlc.
- Configuración de Ventanas (In-Sample / Out-of-Sample ratio).

### Proceso
1. Divide el historial en N ventanas móviles (Rolling) o ancladas (Anchored).
2. Invoca al [backtest-engine](../features/backtest-engine.md) para el periodo de entrenamiento (In-Sample).
3. Valida los parámetros resultantes en el periodo de prueba (Out-of-Sample) reservado.
4. Desplaza la ventana y repite hasta agotar el historial o alcanzar el tiempo real.
5. **Portfolio WFA:** Itera el proceso sobre la combinación lineal de estrategias para validar la robustez del portafolio.
6. **Optimización Nocturna:** Ejecuta el ciclo micro-WFA cada noche (23:59h) sobre los últimos 7-14 días.

### Salida
- **WFA Matrix:** Mapa de calor de 10x10 combinaciones de IS/OOS (ej: 10/20, 15/30).
- **Consolidated Equity Curve:** La unión de todos los periodos Out-of-Sample.
- **WFE Index:** Retorno anualizado OOS / Retorno anualizado IS (Target > 50%).
- **OSE Stability:** Métrica de robustez del espacio de parámetros en el tiempo.
- **Veredicto WFA Segmentado por Régimen:** `PASS`/`FAIL` por categoría (capitulación, rango, tendencia de baja volatilidad) cuando el Genoma de Régimen y Filtro de Entorno está activo (ADR-0110).

### Contextos de Uso
**Contexto 1: Validate (Módulo Validate)**
- Es el filtro definitivo para mover una estrategia a la fase de incubación.

**Contexto 2: Validación del Genoma de Régimen y Filtro de Entorno (ADR-0110, bloqueante)**
- Cuando ese genoma está activo, el modo Segmentado por Régimen (TTR-008) es obligatorio antes de promover el Manifest a incubación.

---

## Tareas (TTRs)

### TTR-001: Implementar Orquestador de Ventanas Móviles (CPCV-ready)
*   **Descripción:** Lógica de slicing del historial con soporte para purga y embargo entre ventanas.
*   **Regla (Purging):** La ventana de purga debe ser ≥ duración máxima histórica del trade.
*   **Regla (Embargo):** Aplicar brecha temporal tras cada ventana OOS para eliminar la autocorrelación serial.
*   **Criterio de Éxito:** Produce una serie continua de trades "fuera de muestra" sin solapamiento de datos.

### TTR-002: Matriz WFA (Exploración Multirotacional)
*   **Descripción:** Ejecuta la validación iterando sobre un rango de ratios IS/OOS (ej: 50/30, 80/20).
*   **Visualización:** Genera datos para un mapa de calor que identifica las ventanas de tiempo más estables (Dashboard §5.4.3).

### TTR-003: Cálculo de Robustez Estadística (WFE, OSE y PBO)
*   **Descripción:** Cálculo de eficiencia WFE ($WFE = OOS_{ret} / IS_{ret}$) y PBO (Probability of Overfitting).
*   **Criterio de Éxito:** Una estrategia "aprueba" si PBO < 0.50 y WFE > WFE_Threshold (configurable, defecto 0.5).
*   **Postcondición:** Registro del veredicto `PASS/FAIL` en el rastro forense.

### TTR-004: Portfolio Walk-Forward Analysis
*   **Descripción:** Extiende el motor de slicing para validar conjuntos de estrategias (Portfolios) iterando linealmente.
*   **Criterio de Éxito:** Genera una curva de equidad OOS agregada que respeta los límites de riesgo del portfolio.

### TTR-005: Filtro de Cluster Contiguo 3x3 (Evaluación Geométrica)
*   **Descripción:** Analiza la matriz WFA buscando zonas de estabilidad de 3x3 celdas verdes (aprobadas).
*   **Regla:** Rechaza configuraciones donde el éxito sea esporádico o aislado, mitigando el azar.

### TTR-006: Orquestación Microrodante Nocturna (Daemon 23:59h)
*   **Descripción:** Implementa la ejecución recurrente nocturna sobre ventanas de corto plazo (7-14 días).
*   **Transferencia:** Inyecta parámetros optimizados vía FFI/gRPC a los puentes de ejecución (Bridges) antes de la apertura de Londres/NY.

### TTR-007: Orquestación de Segmentación Adaptativa (Matriz Orgánica)
*   **Descripción:** Sustituye el slicing por bloques de calendario fijo por la segmentación adaptativa de [`dtw-adaptive-window`](./dtw-adaptive-window.md) cuando el usuario activa el modo "Matriz Orgánica".
*   **Entrada → Invocación:** El WFA entrega historial y la configuración de pivotes; invoca el puerto de la feature y recibe la lista de ventanas de tamaño variable por régimen.
*   **Criterio de Éxito:** El WFA valida sobre ventanas que respiran (anchas en tendencia, densas en crisis) en lugar de bloques fijos, y registra el Índice de Respiración por tramo en el rastro forense.
*   **Regla:** La segmentación adaptativa NUNCA altera el historial pasado (misma invariante anti-curve-fitting del WFA).

### TTR-008: WFA Segmentado por Régimen — Compuerta de Robustez del Genoma de Régimen y Filtro de Entorno (ADR-0108/ADR-0110)
*   **Descripción:** Extiende la Matriz WFA con un modo de segmentación curada por régimen: agrupa las ventanas In-Sample/Out-of-Sample del historial en al menos tres categorías (capitulación, rango, tendencia de baja volatilidad) según `regime_label` ([`hmm-regime-detection`](./hmm-regime-detection.md)), y calcula WFE/PBO de forma independiente para cada categoría.
*   **Criterio de Éxito:** Una estrategia con un Genoma de Régimen y Filtro de Entorno activo solo recibe veredicto `PASS` si cada categoría de régimen cumple individualmente `PBO < 0.50` y `WFE > WFE_Threshold`; un `PASS` agregado que oculte una categoría `FAIL` se reporta como "RECHAZADA".
*   **Regla:** Esta compuerta es **bloqueante** para cualquier Manifest con el Genoma de Régimen y Filtro de Entorno activo (ADR-0110), análogo a la Réplica de Estado de Riesgo (ADR-0109) y al Monte Carlo de Desfase Temporal (ADR-0111) en [`monte-carlo-simulator.md`](./monte-carlo-simulator.md).

---

## Features Consumidas (Reutilizables)

- **[`incremental-test-engine`](./incremental-test-engine.md)** — Optimización de ejecución acumulativa y herencia de resultados (ADR-0060).
- **[`backtest-engine`](./backtest-engine.md)** — Motor de simulación para cada ventana.
- **[`institutional-metrics`](./institutional-metrics.md)** — Cálculo de KPIs por segmento.
- **[`dtw-adaptive-window`](./dtw-adaptive-window.md)** — Segmentación temporal adaptativa por régimen (Matriz Orgánica): emite ventanas de tamaño variable en lugar de bloques de calendario fijo.

---

## Gobernanza y Estándares (Fijos)

- **Inundación de Fundaciones (ADR-0020 V2):** 
    - Perfil: AI / R&D.
    - **I. Identidad & Integridad:** `id`, `created_at`, `audit_hash`, `event_sequence_id`.
    - **II. Soberanía & Propiedad:** `owner_id`, `institutional_tag`, `manifest_id`.
    - **III. Linaje Alpha & Datos:** `version_node_id`, `logic_hash`, `data_snapshot_id`.
    - **IV. Infraestructura & Ops:** `process_id`, `node_id`.
- **Genomas Modulares por Dominio (ADR-0108/ADR-0110):** este analizador es la Compuerta de Robustez del Dominio de Régimen y Filtro de Entorno (modo Segmentado por Régimen). Ver Registro de Dominios Genómicos en [`SAD.md`](../SAD.md) §2.3.

---

## Estructura Interna (FCIS — ADR-0002)
- **Core (Lógica Pura):** Algoritmo de slicing y lógica de agregación de resultados.
- **Shell (Infraestructura):** Orquestador de procesos paralelos.
- **Frontera Pública:** Contrato `calculate_wfa(strategy, windows_config)`.

---

## Dependencias
**Consumido por:** `validate`.
**Depende de:** `backtest-engine`.
