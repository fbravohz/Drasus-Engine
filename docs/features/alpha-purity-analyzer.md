# Alpha Purity Analyzer — Analizador de Pureza de Alpha

**Carpeta:** `./features/alpha-purity-analyzer/`
**Estado:** Especificación / Prioritario
**Última actualización:** 2026-06-14
**Decisión Arquitectónica Asociada:** ADR-0008 (Configurabilidad), ADR-0016 (Local-First)

---

## ¿Qué es?

Es el motor estadístico que mide **cuánto del rendimiento de una estrategia es habilidad real (Alpha) y cuánto es solo exposición pasiva al mercado (Beta)**. Su misión es evitar el engaño de estrategias que ganan únicamente porque el mercado sube — un "Buy & Hold" disfrazado de algoritmo.

Se diferencia de sus features hermanas: `factor-decomposition` reparte el retorno entre varios factores (Fama-French 5), `alpha-decoupling` neutraliza el Beta para aislar la ventaja. El Alpha Purity Analyzer emite el **veredicto de significancia estadística** del Alpha (¿es real o es suerte?) mediante su P-Value y un Score de Pureza normalizado.

---

## Narrativa de Producto

**Capacidad:** `CAP-04` — El guantelete de robustez

**Historia de usuario:** Como investigador de estrategias, quiero saber si la ventaja que mi estrategia reporta es estadísticamente distinguible de cero, y no solo un número positivo que podría ser ruido, para no confiar en una ventaja que la estadística no respalda.

**El problema (técnico):** separar el rendimiento en una parte de habilidad y una parte de exposición pasiva al mercado no basta si nadie certifica que la parte de habilidad es real y no solo una fluctuación estadística alrededor de cero. Una estrategia puede reportar una ventaja positiva pequeña que, sometida a una prueba de significancia, resulta indistinguible del azar.

**La solución (técnica):** calcula la ventaja ajustada por riesgo, mide cuánto de esa ventaja está genuinamente separada del comportamiento del mercado de referencia (si la correlación con el mercado es demasiado alta, anula la ventaja por considerarla un clon disfrazado), y emite un veredicto de significancia basado en su nivel de confianza estadístico. Además, cuando una estrategia en vivo empieza a degradarse, distingue si lo que murió fue la ventaja real de la lógica o si simplemente se apagó el impulso general del mercado que la acompañaba — dos causas de fondo completamente distintas que exigen decisiones distintas.

**Explicado sin tecnicismos:** imagina a alguien que dice tener un don para predecir el clima, y acierta el setenta por ciento de las veces. Suena impresionante, hasta que alguien pregunta: ¿y si solo dijiste "va a hacer sol" todos los días, en una ciudad donde hace sol el setenta por ciento del año? No hay ningún don ahí, solo la inercia del clima local.

Esta pieza hace esa pregunta incómoda sobre tu estrategia: ¿la ventaja que reportas es tu don, o es la inercia del mercado hablando por ti? Y cuando esa ventaja empieza a debilitarse en vivo, te dice cuál de las dos cosas se apagó — si perdiste el don, o si simplemente cambió el clima.

## Comportamientos Observables

- [ ] Calcula el **Alpha Anualizado** ajustado por riesgo tras cada backtest.
- [ ] Determina la sensibilidad (Beta) respecto a múltiples referencias (subyacente, sector, volatilidad).
- [ ] Emite un **Veredicto de Pureza**: marca el rendimiento como significativo solo si el P-Value queda por debajo del umbral configurado.
- [ ] Marca como "DUDOSA" cualquier estrategia cuyo Alpha no sea estadísticamente distinguible de cero.

---

## Restricciones

- **NUNCA** se emite un Alpha positivo si el R-cuadrado de la regresión supera el umbral de réplica (indica que la estrategia es un clon del mercado, no un edge).
- **FIJO:** El benchmark de referencia es el activo subyacente de la estrategia, salvo que se configure un índice externo.

---

## Parámetros Configurables (ADR-0008)

| Parámetro | Default | Rango | Qué hace | FIJO/CONFIG |
|---|---|---|---|---|
| P_VALUE_THRESHOLD | 0.05 | 0.01 – 0.10 | Umbral para considerar un Alpha como real/significativo | CONFIG |
| MIN_ALPHA_PURITY | 0.60 | 0.0 – 1.0 | Puntuación mínima de pureza (0-1) para no ser rechazada | CONFIG |
| MAX_RSQUARED_REPLICA | 0.90 | 0.5 – 0.99 | R² por encima del cual el Alpha se anula (clon del mercado) | CONFIG |
| BENCHMARK_LOCK | true | true / false | Si está activo, usa el subyacente como referencia obligatoria | CONFIG |

---

## Ciclo de Vida de la Feature — Alpha Purity

### Entrada
- Serie de retornos temporales de la estrategia (candidata o en vivo).
- Serie de retornos del benchmark (mercado o índice configurado).
- Nivel de confianza deseado (ej. 95%).

### Proceso
- Alinea temporalmente las series.
- Ejecuta una regresión lineal multivariante sobre los retornos.
- Aísla el intercepto como el Alpha puro y calcula la varianza explicada por el mercado (R²).
- Evalúa la significancia del Alpha (P-Value) y la traduce a un Score de Pureza normalizado.

### Salida
- **Alpha Purity Score:** valor normalizado entre 0 y 1.
- **P-Value:** nivel de significancia del Alpha.
- **Beta Distribution:** exposición a los factores de referencia.
- **Veredicto:** PURO / DUDOSO / RÉPLICA.

### Contextos de Uso

**Contexto 1: Validación (Módulo Validate)**
- Filtra estrategias "Buy & Hold" disfrazadas de algorítmicas antes de aprobarlas.

**Contexto 2: Retroalimentación (Módulo Feedback)**
- Diagnostica la causa de una degradación: distingue si **murió el Alpha** (la lógica perdió eficacia) o si solo **se apagó el Beta** (el mercado dejó de empujar). Esa distinción alimenta el veredicto de retiro.

---

## Tareas (TTRs)

### TTR-001: Implementar Motor de Decomposición Lineal
*   **Descripción:** Motor de regresión que separa Alpha y Beta (modelo CAPM o multifactor).
*   **Criterio de Éxito:** Reporte que descompone el PnL total con coeficientes Beta y la significancia del Alpha (ej. "70% Alpha, 30% Beta").

### TTR-002: Ranking por Pureza y Robustez Estadística
*   **Descripción:** Traduce el intercepto y el R² en un Score de Pureza (0-1) y prioriza estrategias con Alpha significativo.
*   **Criterio de Éxito:** Marca como "DUDOSA" cualquier estrategia cuyo Alpha tenga P-Value por encima del umbral configurado.

---

## Estructura Interna (FCIS — ADR-0002)
- **Core (Lógica Pura):** Funciones de regresión matricial y cálculo de P-Value en `analytics.rs`.
- **Shell (Infraestructura):** Cargador de series de tiempo y benchmarks locales (Parquet/SQLite).
- **Frontera Pública:** Contrato `calculate_purity(returns, benchmark)`.

---

## Gobernanza y Estándares (Fijos)

- **Local-First (ADR-0016):** 100% Local. Los cálculos de regresión se realizan en el backend local, sin recurrir a servicios en la nube.
- **Fidelidad (ADR-0017):** Alta. Los retornos se calculan sobre la serie de precios de cierre para mantener coherencia estadística.

### Persistencia (Inundación de Fundaciones — ADR-0020) — Perfil B (IA / R&D)

Toda firma de pureza registra el Grupo I completo (universal) más los campos de relevancia técnica para IA/R&D:

| Categoría | Campo | Descripción |
| :--- | :--- | :--- |
| **I. Identidad e Integridad** | `id` | UUID del registro de pureza |
| | `created_at` | Timestamp de cálculo |
| | `updated_at` | Timestamp de última actualización |
| | `audit_hash` | Hash del reporte de pureza |
| | `audit_chain_hash` | Hash encadenado al registro anterior |
| | `event_sequence_id` | Secuencia de recuperación post-crash |
| **II. Soberanía** | `owner_id` | Responsable del análisis |
| | `manifest_id` | Contrato de diseño vinculado |
| **III. Linaje Alpha y Datos** | `logic_hash` | Hash del motor de regresión |
| | `data_snapshot_id` | Bundle de series (estrategia + benchmark) usado |
| | `indicator_state_hash` | Score de Pureza y P-Value resultantes |
| **IV. Infraestructura y Ops** | `node_id` | ID del hardware físico |
| | `process_id` | PID del worker analítico |

- **Rastro de Evidencia:** Firma de Pureza (Score + P-Value + Veredicto) adjunta al veredicto del módulo `validate` y al diagnóstico de `feedback`.

---

## Dependencias

**Consumido por:** `validate`, `feedback`.
**Depende de:** `institutional-metrics`.
