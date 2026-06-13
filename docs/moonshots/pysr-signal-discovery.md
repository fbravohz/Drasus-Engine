# Symbolic Signal Discovery — Descubrimiento de Ecuaciones de Señal (Moonshot)

**Carpeta:** `./moonshots/pysr-signal-discovery/`
**Estado:** Moonshot (diferido — post-rentabilidad, ADR-0103)
**Última actualización:** 2026-06-11 (reescrito por ADR-0113: erradicación de PySR; motor simbólico nativo + `egg`)

> **Nota de gobernanza (ADR-0113):** este moonshot es la **minería simbólica de forma libre** (descubrir ecuaciones matemáticas arbitrarias sin esqueleto humano). NO usa PySR (Python+Julia, viola ADR-0104). Se implementa con un motor de **regresión simbólica nativa en Rust** —programación genética sobre árboles de expresión con frente de Pareto precisión/complejidad, reutilizando el `nsga2-optimizer` nativo—, y se designa **`egg` (e-graphs, Rust puro)** como tecnología recomendada para la saturación de equivalencias algebraicas y el control de *bloat*. La regresión simbólica **acotada** (sobre el catálogo cerrado del AST) ya vive en el MVP como modo del motor genético; este moonshot es la variante de búsqueda libre.

---

## ¿Qué es esta feature?

Es el laboratorio de descubrimiento de señales basado en **Regresión Simbólica nativa**. A diferencia de las redes neuronales tradicionales que son "cajas negras", busca activamente la ecuación matemática más simple que explique la ventaja competitiva (Alpha) en los datos históricos.

**Problema que resuelve:** Las estrategias basadas en IA compleja suelen ser imposibles de explicar y fallan por sobreajuste. El motor simbólico nativo genera fórmulas matemáticas legibles que el usuario puede auditar y entender antes de operar, sin runtimes externos.

---

## Comportamientos Observables

- [ ] El sistema toma un dataset de indicadores y precios y devuelve una lista de ecuaciones (ej: `Signal = sin(RSI) * volatility / momentum`).
- [ ] El usuario puede exportar la ecuación directamente como código **Rust nativo** listo para el motor de ejecución, sin dependencias externas.
- [ ] El sistema muestra un reporte de "Parquedad" (balance entre complejidad de la fórmula y su capacidad predictiva — frente de Pareto).
- [ ] Evaluación de Estabilidad: corre la ecuación en ventanas rodantes.
  → Si la varianza del R² entre ventanas es > 0.2 → Candidato RECHAZADO por inestabilidad.

---

## Restricciones

- **NUNCA ecuación más compleja que MAX_COMPLEXITY.** Limita overfitting; el control de *bloat* se apoya en `egg` (saturación de equivalencias algebraicas).
- **NUNCA R² negativo o NaN.** Validación de resultado.
- **Cero Python/PySR (ADR-0104/0113).** Motor 100% Rust nativo.

---

## Parámetros Configurables

| Parámetro | Default | Rango | Qué hace |
|---|---|---|---|
| TARGET | "forward_returns" | cadena | Qué predecir (por defecto retornos futuros) |
| MAX_COMPLEXITY | 10 | 3-20 | Máximo operaciones matemáticas en ecuación |
| POPULATION_SIZE | 100 | 50-500 | Candidatos por generación |

---

## Tareas (TTRs)

### **TTR-003: Optimización de Hiperparámetros (nativa)**
*   **Descripción:** Ajusta automáticamente `population_size`, `n_iterations` y `mutation_weights` mediante el optimizador bayesiano nativo del sistema ([`bayesian-optimizer.md`](../features/bayesian-optimizer.md)). Cero `scikit-optimize`.
*   **Objetivo:** Maximizar el área bajo el frente de Pareto (AUC).

### **TTR-004: Auditoría de Estabilidad (Rolling Window Stability)**
*   **Descripción:** Divide el historial en K ventanas (ej: 4 trimestres) y mide la persistencia de la señal.
*   **Regla:** El signo del coeficiente de la ecuación debe ser constante en al menos el 75% de las ventanas.

### **TTR-005: Traductor Bidireccional Ecuación ↔ AST (Glass-Box Evolution)**
*   **Descripción:** Convierte la fórmula matemática descubierta en un Grafo Dirigido Acíclico (DAG) compatible con el motor y viceversa, usando `egg` para canonicalizar/simplificar la expresión antes del mapeo.
*   **Objetivo:** Permitir que el humano modifique visualmente la tesis de IA para anclar su intuición antes de generar el código final.

---

## Gobernanza y Estándares (ADR-0020 V2)
- Toda sesión de descubrimiento simbólico registra el **Grupo I (universal) + Perfil IA/R&D** (ADR-0020 V2).
- Metadatos: `logic_hash` (config del motor simbólico nativo), `audit_chain_hash`, `model_lineage_id`.

## Dependencias

**Depende de:**
- `backtest-engine` (para evaluar ecuaciones)
- `nsga2-optimizer` (motor genético base de la regresión simbólica)

**Depende de ella:**
- `generate` (incluye ecuaciones simbólicas nativas en estrategias)
