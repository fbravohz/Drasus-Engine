# Factor Decomposition — Descomposición de Retornos en Factores

**Carpeta:** `./features/factor-decomposition/`
**Estado:** Lista para implementar
**Última actualización:** 2026-04-08

---

---

## ¿Qué es?

Es el motor analítico encargado de realizar la **"Análisis Forense del Retorno"**. Descompone el rendimiento de cualquier estrategia en sus componentes elementales: Alpha (habilidad/edge real), Beta (exposición pasiva a factores de mercado) y Residuales (ruido). Su misión es evitar que el sistema sea engañado por estrategias que solo ganan por "suerte factor" (ej: estar posicionado en Momentum durante un rally de tech).

---

## Comportamientos Observables

- [ ] Calcula la **Sensibilidad (Beta)** respecto a múltiples factores (Mercado, Size, Value, Momentum).
- [ ] Aísla el **Alpha Anualizado** y su significancia estadística (P-Value).
- [ ] Emite un **Score de Pureza (0-1)**: Qué tanto del retorno es independiente de los factores comunes.

---

## Restricciones
- **FIJO:** Si el $R^2$ de la regresión es $> 0.90$, el Alpha se marca como "Cero Técnico" (indistingible de una réplica pasiva).
- **FIJO:** Debe realizarse una validación de invarianza temporal del Alpha (¿Es el Alpha estable o solo ocurrió en un splash de 1 mes?).

---

## Ciclo de Vida de la Feature — Factor Decomposition

### Entrada
- Serie de retornos temporales de la Estrategia.
- Series de retornos de los Factores de Referencia (Benchmarks).
- Nivel de confianza deseado.

### Proceso
- Alineación temporal de series.
- Ejecución de Regresión Lineal Multivariante (OLS) por periodos.
- Cálculo de la matriz de varianza-covarianza residual.

### Salida
- **Alpha Purity Report:** Alpha intercept, Betas vector, P-Value y Score de Pureza.
- **R-Square:** Porcentaje de varianza explicada por el mercado.

### Contextos de Uso
**Contexto 1: Validación (Módulo Validate)**
- Filtra candidatos para asegurar que tienen un edge real que no se puede comprar con un ETF.
**Contexto 2: Retroalimentación (Módulo Feedback)**
- Analiza si la degradación de una estrategia se debe a que su factor entró en régimen hostil o a que su lógica perdió eficacia.

---

## Tareas (TTRs)

### TTR-001: Implementar Motor de Decomposición Lineal Multivariante
*   **Descripción:** Ejecuta la regresión: $Returns = \alpha + \sum \beta_i \cdot Factor_i + \epsilon$.
*   **Criterio de Éxito:** Reporte con coeficientes $\beta$ y nivel de significancia del $\alpha$.

### TTR-002: Cálculo de Score de Pureza y Robustez Estadística
*   **Descripción:** Traduce el intercept ($\alpha$) y el $R^2$ en una puntuación de pureza (0-1).
*   **Criterio de Éxito:** Strategies con pureza $< 0.5$ son recomendadas para rechazo en `validate`.

---

## Gobernanza y Estándares (Fijos)
- **Local-First (ADR-0016):** 100% Local. Los factores de riesgo y la sensibilidad del modelo son datos sensibles.
## Persistencia (Inundación de Fundamentos — ADR-0020)

Toda descomposición registra el set de relevancia técnica para AI/R&D:

| Categoría | Campo | Descripción |
| :--- | :--- | :--- |
| **I. Identidad** | `id` | Identificador único de la autopsia |
| | `created_at` | Timestamp de cálculo |
| | `updated_at` | Timestamp de última actualización del registro |
| | `audit_hash` | Hash del reporte de pureza |
| | `audit_chain_hash` | Hash del rastro de evidencia |
| | `event_sequence_id` | Orden secuencial de la descomposición |
| **II. Soberanía** | `owner_id` | Responsable del análisis |
| | `manifest_id` | ID del contrato de diseño legal |
| **III. Pesos/Arquitectura** | `logic_hash` | Hash del motor de regresión |
| | `data_snapshot_id` | Bundle de factores de referencia |
| | `indicator_state_hash` | Score de Pureza resultante |
| **IV. Hardware** | `node_id` | ID del hardware físico |
| | `process_id` | PID del worker analítico |


---

## Estructura Interna (FCIS — ADR-0002)
- **Core (Lógica Pura):** Funciones de regresión matricial y tests de significancia en `analytics.rs`.
- **Shell (Infraestructura):** Cargador de series de factores locales (Parquet/SQLite).
- **Frontera Pública:** Contrato `decompose_returns(strategy_returns, factor_bundle)`.

---

## Dependencias
**Consumido por:** `validate`, `manage`, `feedback`.
**Depende de:** `institutional-metrics`.
