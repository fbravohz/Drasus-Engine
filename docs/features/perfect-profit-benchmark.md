# Perfect Profit Benchmark (Model Efficiency)

**Carpeta:** `./features/perfect-profit-benchmark/`
**Estado:** En Diseño
**Última actualización:** 2026-04-30
**Decisión Arquitectónica Asociada:** ADR-0020

---

## ¿Qué es?

El Perfect Profit Benchmark es un filtro de eficiencia del modelo (ME). Su misión es medir qué porcentaje del beneficio teórico máximo captura la estrategia. Compara lo que la estrategia ganó vs. lo que era matemáticamente posible ganar si se hubieran capturado todos los movimientos perfectos.

**Problema que resuelve:** Una estrategia podría ganar $1,000, pero si en ese mismo período el mercado ofreció oportunidades obvias de ganar $100,000, la estrategia es ineficiente. Este componente desenmascara estrategias que aparentan ser buenas pero que en realidad desperdician el Alpha disponible.

---

## Comportamientos Observables

- [ ] El sistema calcula la suma de todos los movimientos de precio direccionales (beneficio teórico máximo o "Perfect Profit").
- [ ] La ganancia neta de la estrategia se divide por el Perfect Profit para obtener el ratio de Eficiencia del Modelo (ME).
- [ ] Si la Eficiencia del Modelo es menor al umbral (ej. < 5%), la estrategia se rechaza automáticamente.
- [ ] Opcionalmente, se aplica un filtro por Efficiency Ratio (ER) de Kaufman para asegurar que se explota una ineficiencia real en el flujo direccional.

---

## Restricciones

- **FIJO:** El Perfect Profit siempre asume la captura del 100% de la distancia entre pivotes mayores, sin slippage ni comisiones.
- La métrica nunca puede ser > 100% (si ocurre, se marca como un error matemático de look-ahead bias).

---

## Parámetros Configurables

| Parámetro | Default | Rango | Qué hace | FIJO/CONFIG |
|---|---|---|---|---|
| MIN_MODEL_EFFICIENCY | 5% | 1% - 15% | Umbral mínimo de captura de beneficio. | CONFIG |
| MIN_EFFICIENCY_RATIO | 0.3 | 0.1 - 0.8 | Filtro direccional (ER de Kaufman). | CONFIG |

---

## Ciclo de Vida de la Feature — Perfect Profit Benchmark

### Entrada
- Histórico completo de precios (Point-in-Time).
- Registro de transacciones (trades) de la estrategia candidata.

### Proceso
- Detecta y suma todos los pivotes (movimientos de precio) en la serie de tiempo para calcular el techo matemático.
- Compara la ganancia de la estrategia con ese techo.

### Salida
- `model_efficiency_ratio` (ej: 0.08 o 8%).
- `kaufman_er_score`.
- Veredicto de Eficiencia (PASSED / FAILED).

### Contextos de Uso
**Contexto 1: Filtro de Robustez (Validate)**
- Permite descartar estrategias que, a pesar de ser rentables, tienen un rendimiento irrisorio comparado con el movimiento total del mercado.

---

## Tareas (TTRs)

### **TTR-001: Motor de Cálculo del Beneficio Teórico**
*   **¿Cuál es el problema?** Necesitamos saber cuánto dinero había en la mesa antes de juzgar si la estrategia fue buena.
*   **¿Qué tiene que pasar?** El sistema recorre las barras y suma la distancia de todos los swings/pivotes para establecer el `MaxPossibleProfit`.
*   **¿Cómo sé que está hecho?**
    - [ ] Dada una serie de barras en tendencia pura, el Perfect Profit es casi igual al Buy and Hold.
    - [ ] Dada una serie en rango, el Perfect Profit suma cada rebote arriba y abajo.
*   **¿Qué no puede pasar?** No puede calcularse post-slippage (es un techo teórico puro).

### **TTR-002: Auditoría de Eficiencia (Model Efficiency Ratio)**
*   **¿Cuál es el problema?** Hay que penalizar a las estrategias perezosas.
*   **¿Qué tiene que pasar?** Se divide `Strategy_Profit / MaxPossibleProfit`. Si no llega a `MIN_MODEL_EFFICIENCY`, se emite un fallo de validación.
*   **¿Cómo sé que está hecho?**
    - [ ] Una estrategia con ratio de 2% es rechazada, mientras que una de 12% aprueba.

---

## Gobernanza y Estándares (Fijos)

- **Inundación de Fundaciones (ADR-0020):** 
    - Perfil: AI / R&D.
    - **I. Identidad & Integridad:** `id`, `created_at`, `updated_at`, `audit_hash`, `audit_chain_hash`, `event_sequence_id`.
    - **II. Soberanía & Propiedad:** `owner_id`, `institutional_tag`, `manifest_id`.
    - **III. Linaje Alpha & Datos:** `version_node_id`, `logic_hash`, `data_snapshot_id`.
    - **IV. Infraestructura & Ops:** `process_id`, `node_id`.
