# Factor Decomposition — Descomposición de Retornos en Factores

**Carpeta:** `./features/factor-decomposition/`
**Estado:** Lista para implementar
**Última actualización:** 2026-04-08

---

---

## ¿Qué es?

Es el motor analítico encargado de realizar la **"Análisis de Atribución del Retorno"**. Descompone el rendimiento de cualquier estrategia en sus componentes elementales: Alpha (habilidad/edge real), Beta (exposición pasiva a factores de mercado) y Residuales (ruido). Su misión es evitar que el sistema sea engañado por estrategias que solo ganan por "suerte factor" (ej: estar posicionado en Momentum durante un rally de tech).

---

## Narrativa de Producto

**Capacidad:** `CAP-04` — El guantelete de robustez

**Historia de usuario:** Como investigador de estrategias, quiero saber si mi ventaja depende de que un estilo de mercado completo —no solo el mercado en general— estuvo de moda durante el periodo que probé, para no confundir estar en la corriente correcta con haber encontrado algo propio.

**El problema (técnico):** una estrategia puede no depender de la dirección general del mercado y aun así ganar solo porque, por casualidad, estuvo expuesta a un estilo de inversión particular —empresas pequeñas, acciones de "impulso", activos "de valor"— que tuvo un buen periodo. Ese tipo de suerte es más difícil de detectar que la suerte de mercado simple, porque la estrategia sí parece descorrelacionada del índice general mientras monta una ola distinta pero igual de ajena a su propia lógica.

**La solución (técnica):** descompone el retorno contra varios factores de referencia a la vez —no solo uno— midiendo la sensibilidad de la estrategia a cada estilo de mercado conocido, y aísla lo que sobra después de descontar todos ellos como la ventaja genuina. Si la varianza explicada por esos factores es demasiado alta, la ventaja se marca como técnicamente nula. Y verifica que esa ventaja restante sea estable en el tiempo, no un estallido de un solo mes que se diluye en el promedio anual.

**Explicado sin tecnicismos:** imagina a un inversionista que presume haberle ganado al mercado general comprando puras acciones pequeñas y de moda, justo en el año en que ese tipo de acciones estuvo de moda en todo el mundo. No le ganó al mercado por habilidad — se subió a una ola particular que levantó a todos los que estaban parados en el mismo lugar, no solo a él.

Esta pieza revisa varias olas a la vez —no solo la marea general del mercado, sino las corrientes particulares de cada estilo de inversión— y te dice qué te queda después de descontar todas las que solo te cargaron sin que hicieras nada. Y comprueba que lo que queda no fue un buen mes aislado, sino algo que se sostiene con el tiempo.

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
| **I. Identidad** | `id` | Identificador único del análisis de causa raíz |
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
