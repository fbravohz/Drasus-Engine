# Análisis de Datos Topológicos para Aislamiento de Cisnes Negros (SQX Mod 13)

**Carpeta:** `./moonshots/`
**Estado:** Moonshot (Fase 5 - Experimental)
**Última actualización:** 2026-06-06

---

## ¿Qué es?

Mapeo del rendimiento de un portafolio en un **Espacio de Fase** n-dimensional, sobre el que se aplica **Análisis de Datos Topológicos (TDA)** para detectar estructuras y bucles ocultos que la estadística clásica de correlación lineal ignora.

**Problema que resuelve:** El filtro de correlación tradicional (descartar estrategias con correlación lineal alta) es peligroso: dos estrategias pueden estar descorrelacionadas 364 días al año y colapsar exactamente el mismo día durante un Flash Crash. La correlación de Pearson no ve esa co-caída de cola. El TDA sí: encuentra el "agujero" topológico donde varias estrategias caen juntas bajo un shock extremo.

Diferenciador frente a StrategyQuant X: SQX solo ofrece un filtro de correlación lineal con umbral fijo. Esto aísla riesgo de cola estructural, no lineal.

---

## Comportamientos Observables

- [ ] **Mapeo de Espacio de Fase:** El portafolio se representa como una nube de puntos n-dimensional donde cada estrategia es una coordenada de rendimiento.
- [ ] **Detección de Co-Colapso de Cola:** El motor identifica grupos de estrategias que, bajo un shock extremo simulado (ej. caída de liquidez del 90%), caen juntas aunque su correlación diaria sea baja.
- [ ] **Constelación de Riesgo (UI espacial):** Visualización 3D donde cada estrategia es una estrella; las que comparten riesgo de cola se atraen gravitacionalmente hacia un "centro de colapso". El usuario arrastra una estrella fuera del clúster y el sistema re-pesa el portafolio para desincronizarla del grupo de la muerte.

---

## Tareas (TTRs)

### **TTR-001: Construcción del Espacio de Fase y Homología Persistente**
*   **¿Cuál es el problema?** La correlación lineal no captura la estructura de co-caída de cola que provoca el colapso simultáneo de un portafolio en crisis.
*   **¿Qué tiene que pasar?** Mapear los retornos del portafolio en un espacio n-dimensional y aplicar homología persistente para aislar los grupos que comparten riesgo de cola estructural.
*   **¿Cómo sé que está hecho?**
    - [ ] El motor marca grupos de co-colapso que el filtro de correlación lineal clasificaba como independientes.
    - [ ] El análisis sobre un portafolio dado es determinista y reproducible.
*   **¿Qué no puede pasar?**
    - El cálculo topológico no debe bloquear el pipeline de validación local (corre como tarea de fondo, no en hot-path).

---

## Gobernanza y Estándares (ADR-0020)
- Registro del **Grupo I (universal) + Perfil IA/R&D** (ADR-0020) por cada análisis topológico de portafolio.
- **Relegación:** R&D no prioritario para el operador retail individual (Foco Retail-First). Permanece en `/moonshots/` hasta validar valor operativo. La detección de co-caída para retail se cubre hoy de forma suficiente con la matriz de correlación y CVaR existentes.
