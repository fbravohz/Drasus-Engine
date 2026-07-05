# Rule Ablation (Dismantling the Engine)

**Carpeta:** `./features/rule-ablation/`
**Estado:** En Diseño
**Última actualización:** 2026-04-30
**Decisión Arquitectónica Asociada:** ADR-0065

---

## ¿Qué es?

La Ablación de Reglas es una técnica de simplificación y validación de robustez que consiste en desactivar sistemáticamente componentes lógicos de una estrategia (indicadores, filtros horarios, condiciones específicas) para observar su impacto en el rendimiento global.

**Problema que resuelve:** Muchas estrategias complejas contienen reglas que son puro "ruido estadístico" — condiciones que se ajustaron al pasado pero no aportan ventaja real. Si al quitar una regla el resultado es igual o mejor, esa regla era un parásito que aumentaba la fragilidad del sistema.

---

## Comportamientos Observables

- [ ] El sistema identifica todos los nodos de decisión (reglas) en el AST de la estrategia.
- [ ] Ejecuta backtests iterativos desactivando una regla a la vez (One-Rule-Out).
- [ ] Compara el Sharpe Ratio y el Profit Factor de la versión simplificada vs la versión original.
- [ ] Si la versión simplificada mantiene o mejora el rendimiento (dentro de un umbral de tolerancia), la regla se marca como "Redundante" y se recomienda su eliminación.

---

## Restricciones

- **FIJO:** El proceso de ablación debe ser recursivo hasta que no se puedan eliminar más reglas sin degradar significativamente el rendimiento.
- **FIJO:** Una regla eliminada exitosamente condena a la versión compleja; el sistema siempre debe priorizar la variante más simple (Navaja de Ockham).

---

## Parámetros Configurables

| Parámetro | Default | Rango | Qué hace | FIJO/CONFIG |
|---|---|---|---|---|
| ABLATION_TOLERANCE | 0.05 | 0.0 - 0.20 | Degradación de Sharpe permitida para considerar que una regla es prescindible. | CONFIG |
| MIN_RULES_TO_KEEP | 1 | 1 - 5 | Mínimo de reglas que deben permanecer en la estrategia. | CONFIG |

---

## Ciclo de Vida de la Feature — Rule Ablation

### Entrada
- Árbol de sintaxis abstracta (AST) de la estrategia.
- Parámetros de backtest nominales.

### Proceso
- Genera $N$ variantes de la estrategia, donde cada variante tiene una regla desactivada.
- Ejecuta los backtests en paralelo.
- Evalúa la "Contribución de Alpha" de cada regla.

### Salida
- `ablation_matrix` (Impacto de cada regla en el rendimiento).
- `optimized_ast` (Versión simplificada de la estrategia).
- `redundancy_score` (Porcentaje de la lógica que era ruido).

### Contextos de Uso
**Contexto 1: Simplificación Post-Generación (Validate)**
- Limpia las estrategias "barrocas" generadas por el motor genético antes de que lleguen a la incubación.

---

## Tareas (TTRs)

### **TTR-001: Identificador de Nodos y Generador de Variantes**
*   **¿Cuál es el problema?** Necesitamos desarmar el motor pieza por pieza de forma automática.
*   **¿Qué tiene que pasar?** El sistema parsea el AST, identifica los nodos condicionales y genera copias inmutables de la estrategia con nodos específicos desactivados (`pass-through` o `always-true`).
*   **¿Cómo sé que está hecho?**
    - [ ] Una estrategia con 3 indicadores genera 3 variantes simplificadas para testear.

### **TTR-002: Auditor de Contribución de Alpha**
*   **¿Cuál es el problema?** Identificar qué reglas no están "pagando su alquiler" en términos de Sharpe.
*   **¿Qué tiene que pasar?** Compara los resultados. Si `Sharpe(Simplificada) >= Sharpe(Original) * (1 - ABLATION_TOLERANCE)`, la regla original se descarta.
*   **¿Cómo sé que está hecho?**
    - [ ] El sistema emite un log: "Regla 'Tuesday_Filter' eliminada: Rendimiento mejoró un 2% sin ella".

---

## Gobernanza y Estándares (Fijos)

- **Inundación de Fundaciones (ADR-0020):** 
    - Perfil: R&D / Validación.
    - **I. Identidad & Integridad:** `id`, `created_at`, `updated_at`, `audit_hash`, `audit_chain_hash`, `event_sequence_id`.
    - **II. Soberanía & Propiedad:** `owner_id`, `institutional_tag`, `manifest_id`.
    - **III. Linaje Alpha & Datos:** `version_node_id`, `logic_hash`, `data_snapshot_id`.
    - **IV. Infraestructura & Ops:** `process_id`, `node_id`.
