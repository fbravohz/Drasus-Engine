# Incremental Test Engine (Herencia + Delta)

**Carpeta:** `./features/incremental-test-engine/`
**Estado:** Especificación (Standalone)
**Última actualización:** 2026-04-30
**Decisión Arquitectónica Asociada:** ADR-0060 (Tests Incrementales Versionados)

---

## ¿Qué es?

El **Incremental Test Engine** es un motor transversal de optimización computacional que permite al sistema de validación (The Torture Chamber) evitar recálculos redundantes. Utiliza una combinación de hashing determinista de parámetros y herencia de resultados previos para centrarse exclusivamente en computar el "Delta" de una nueva iteración.

**Problema:** Las pruebas de robustez (WFA, Monte Carlo, Stress Tests) son computacionalmente costosas. En un proceso iterativo de refinamiento de estrategias, el 80% de los parámetros suelen permanecer constantes, pero los sistemas tradicionales recalculan todo desde cero en cada cambio.

**Solución:** Desacoplar la lógica de ejecución del almacenamiento de resultados. Cada ejecución de un test genera un `params_hash`. Antes de ejecutar, el motor consulta el historial de la estrategia; si encuentra un hash idéntico o parcial (mismo segmento temporal), inyecta el resultado previo.

---

## Comportamientos Observables

- [ ] **Hashing Transversal:** Genera un `params_hash` único para cualquier tipo de test (WFA, Monte Carlo, Ockham, etc.) basado en su configuración.
- [ ] **Detección de Herencia (v1 → v2):** Si una nueva versión de la estrategia comparte la misma base de datos de trades pero añade un nuevo análisis, el motor recupera los resultados anteriores.
- [ ] **Cómputo de Delta:** En el caso de WFA, si se añaden nuevas ventanas Out-of-Sample, el motor solo calcula las nuevas ventanas y las concatena a la curva de equidad existente.
- [ ] **Validación de Integridad:** Verifica que el `data_snapshot_id` y el `logic_hash` no hayan cambiado antes de permitir la herencia, garantizando que el ahorro no comprometa la veracidad estadística.

---

## Restricciones

- **Inmutabilidad:** Los resultados heredados no pueden ser modificados.
- **Transparencia:** Todo resultado heredado debe marcarse explícitamente como `HEREDADO` en la UI, incluyendo el ID de la versión origen.
- **Cero Colisiones:** El hashing debe incluir todos los parámetros que afecten al resultado (ventanas, activos, configuraciones de motor, costos).

---

## Tareas (TTRs)

### TTR-001: Generador de Params Hash Universal
*   **Descripción:** Implementa la lógica de hashing SHA-256 para configuraciones de test polimórficas.
*   **Entrada:** Objeto de configuración del test (JSON).
*   **Salida:** `params_hash` determinista.

### TTR-002: Buscador de Evidencia en el DAG
*   **Descripción:** Consulta el Grafo de Versiones (Strategy Versioning) buscando coincidencias de `params_hash`.
*   **Entrada:** `strategy_id`, `params_hash`.
*   **Salida:** `TestResult` (si existe) o `null`.

### TTR-003: Orquestador de Concatenación Delta (WFA/Trades)
*   **Descripción:** Lógica para unir resultados de múltiples ejecuciones en una sola serie temporal continua.
*   **Regla:** Asegura que los timestamps de las piezas encajen perfectamente sin solapamiento ni huecos.

### TTR-004: Auditoría de Ahorro Computacional
*   **Descripción:** Registra el tiempo de ejecución real vs. el tiempo estimado ahorrado para reportar en el Dashboard de QuantOps.

---

## Gobernanza y Estándares (Fijos)

- **Inundación de Fundaciones (ADR-0020 V2):** 
    - **I. Identidad:** `id`, `created_at`, `audit_hash`.
    - **III. Linaje:** `version_node_id`, `parent_test_id`, `logic_hash`.
    - **IV. Hardware:** `execution_time_saved_ms`.

---

## Dependencias
**Depende de:**
- [`strategy-versioning`](./strategy-versioning.md) — para la búsqueda en el linaje del DAG.
- [`audit-log`](./audit-log.md) — para registrar el rastro de herencia.

**Consumido por:**
- [`walk-forward-analyzer`](./walk-forward-analyzer.md) — para optimizar ventanas móviles.
- [`monte-carlo-simulator`](./monte-carlo-simulator.md) — para reutilizar simulaciones de trades.
- [`validate`](../modules/validate.md) — orquestador principal.
