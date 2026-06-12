# Parameter Optimization — Optimización de Parámetros

**Carpeta:** `./features/parameter-optimization/`
**Estado:** Lista para implementar
**Última actualización:** 2026-04-08

---

## ¿Qué es?

Busca los parámetros óptimos de una estrategia usando Grid Search (exhaustivo) o Bayesian Search (inteligente).

---

## Comportamientos Observables

- [ ] Usuario define espacio de parámetros: periodo RSI = [5, 10, 15, 20], threshold = [40, 50, 60]
  → Grid Search prueba todas las 12 combinaciones
  → Devuelve los parámetros con Sharpe máximo

- [ ] Usuario usa Bayesian Search en mismo espacio
  → Prueba primero 5 combinaciones aleatoriamente
  → Usa esos resultados para predecir dónde está el óptimo
  → Prueba nuevas combinaciones cercanas al óptimo predicho
  → Es más rápido que Grid Search exhaustivo

---

## Restricciones

- **NUNCA se prueba fuera del espacio de parámetros definido.**
- **NUNCA se devuelve parámetros sin evaluación.**

---

## Parámetros Configurables

| Parámetro | Default | Rango | Qué hace |
|---|---|---|---|
| METHOD | grid | grid / bayesian | Tipo de búsqueda |
| MAX_ITERATIONS | 100 | 10-1000 | Máximo evaluaciones (si Bayesian) |

---

## Tareas (TTRs)

### **TTR-001: Optimizar parámetros via grid search o búsqueda bayesiana**

**Qué hace:** Busca parámetros óptimos en el espacio definido.

**Entrada:**
- Estrategia parametrizable
- Espacio de parámetros (rangos o grilla)
- Método (grid o bayesian)

**Salida:**
- Parámetros óptimos encontrados

---

## Gobernanza y Estándares (Fijos)

- **Local-First (ADR-0016):** 100% Local. La búsqueda de parámetros es intensiva en cómputo y debe realizarse en el hardware del usuario.
- **Inundación de Fundaciones (ADR-0020 V2):** 

| Categoría | Campo | Descripción |
| :--- | :--- | :--- |
| **I. Identidad** | `id` | Identificador único de la sesión de búsqueda |
| | `created_at` | Timestamp de inicio |
| | `audit_hash` | Hash de los parámetros ganadores |
| | `audit_chain_hash` | Hash de la secuencia de búsqueda |
| **II. Soberanía** | `owner_id` | Usuario que optimiza |
| | `manifest_id` | ID del diseño de la estrategia |
| **III. Pesos/Arquitectura** | `logic_hash` | Hash del motor de optimización |
| | `data_snapshot_id` | Ref al dataset de evaluación |
| | `version_node_id` | Versión resultante en el DAG |
| **IV. Hardware** | `node_id` | ID del hardware físico |
| | `process_id` | PID del proceso de búsqueda |


---

## Dependencias

**Depende de:**
- `backtest-engine` (para evaluar cada combinación)

**Depende de ella:**
- `validate` (ajuste post-Torture)
- `generate` (búsqueda de parámetros iniciales)
