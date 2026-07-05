# Fit-to-Portfolio Search

**Carpeta:** `./features/fit-to-portfolio-search/`
**Estado:** En Diseño
**Última actualización:** 2026-06-11
**Decisiones Arquitectónicas Asociadas:** ADR-0050, ADR-0108, ADR-0111

## 1. ¿Qué es esta feature?
Inyección del estado del portafolio vivo como una presión restrictiva en el motor evolutivo (NSGA-II).
Resuelve el problema de generar cientos de algoritmos rentables que terminan haciendo exactamente lo mismo. Castiga la redundancia y premia la ortogonalidad (descorrelación).

**Gen de Condición de Estado del Genoma de Portafolio y Correlación (ADR-0108/ADR-0111):** la correlación de Spearman entre la curva de rendimientos del genoma evaluado y la curva consolidada del portafolio vivo es la instancia FIJA del Gen de Condición "Correlación de Curva de Equidad Rodante" del Dominio de Portafolio y Correlación. Cuando ese genoma está activo (co-evolución de cartera, ADR-0111), `PORTFOLIO_CORRELATION_CAP` y `CORRELATION_PENALTY_WEIGHT` dejan de ser constantes globales y se convierten en nodos `wildcard_group` que el motor evolutivo direcciona por miembro de la cartera, en lugar de aplicar el mismo umbral a todos los candidatos.

## 2. Comportamientos Observables
- [ ] El sistema lee la curva de capital consolidada del portafolio actual.
- [ ] Durante el ciclo genético, cualquier estrategia con alta correlación respecto al portafolio activo es masacrada.
- [ ] Las estrategias resultantes incrementan la diversificación y estabilidad global.
- [ ] Cuando el Genoma de Portafolio y Correlación (ADR-0111) está activo, `PORTFOLIO_CORRELATION_CAP` y `CORRELATION_PENALTY_WEIGHT` pueden resolverse a valores distintos por miembro de la cartera en lugar de un único umbral global.

## 3. Restricciones
- La función de aptitud (fitness) asimila la correlación como un factor penalizante, pero nunca como el único (debe seguir siendo rentable).

## 4. Parámetros Configurables
| Parámetro | Default | Rango | Qué hace | FIJO/CONFIG |
|---|---|---|---|---|
| PORTFOLIO_CORRELATION_CAP | 0.3 | -1.0 a 1.0 | Límite máximo de correlación Spearman permitida | CONFIG |
| CORRELATION_PENALTY_WEIGHT | 2.0 | 0.1 a 5.0 | Fuerza de castigo evolutivo para genomas clones | CONFIG |

## 5. Estructura Interna (FCIS)
- **Core (Lógica Pura):** Función matemática de penalización incrustada en la métrica de fitness.
- **Shell (Infraestructura):** Carga dinámica en memoria de la serie de tiempo consolidada de la flota.
- **Frontera Pública:** Modificador interno del motor NSGA-II.

## 6. Ciclo de Vida de la Feature
### Entrada
- Matriz de rendimientos del Portafolio Vivo.
- Serie de rendimientos temporales del Genoma evaluado.
### Proceso
- Computa coeficiente de correlación de Spearman de rendimientos por vela.
- Si supera el límite, devalúa el puntaje final del Genoma para evitar su reproducción.
### Salida
- Score de Fitness ajustado ortogonalmente.
### Contextos de Uso
- **Contexto 1: Descubrimiento Dirigido (generate)**
  - Optimiza la fuerza de cómputo hacia lo que realmente se necesita: diversificación.

## 7. Tareas (TTRs)
### **TTR-001: Penalización por Correlación Dinámica**
* **¿Cuál es el problema?** El motor gasta CPU creando copias degeneradas de lo que ya tenemos.
* **¿Qué tiene que pasar?** El fitness evaluator comprueba la huella de PnL y devalúa los scores de clones direccionales.
* **¿Cómo sé que está hecho?**
  - [ ] Enciendo un portafolio de tendencia y el motor genético empieza a sobrevivir genomas de reversión a la media.
  - [ ] La correlación promedio resultante de la criba nunca supera 0.3.
* **¿Qué no puede pasar?**
  - Detener el algoritmo si el portafolio actual está vacío (debe ignorar la métrica en fase inicial).

### **TTR-002: Gen de Condición — Correlación de Curva de Equidad Rodante del Genoma de Portafolio y Correlación (ADR-0108/ADR-0111)**
* **¿Cuál es el problema?** El Dominio de Portafolio y Correlación necesita que la correlación de Spearman entre miembros de una cartera candidata sea legible como Gen de Condición de Estado por su motor evolutivo de co-evolución (TTR-007 de [`nsga2-optimizer`](./nsga2-optimizer.md)), y que sus umbrales sean direccionables por miembro.
* **¿Qué tiene que pasar?** Cuando `ACTIVE_GENOME_DOMAINS` incluye Portafolio y Correlación, `PORTFOLIO_CORRELATION_CAP` y `CORRELATION_PENALTY_WEIGHT` se exponen como nodos `wildcard_group` del dominio, resolubles de forma independiente para cada miembro de `PORTFOLIO_COEVOLUTION_SIZE`.
* **¿Cómo sé que está hecho?**
    - [ ] Una cartera candidata con 5 miembros puede resolver `PORTFOLIO_CORRELATION_CAP` a 5 valores distintos, uno por miembro.
    - [ ] Sin el Genoma de Portafolio y Correlación activo, ambos parámetros operan como constantes globales (comportamiento actual, sin regresión).
* **¿Qué no puede pasar?** La penalización por correlación nunca se omite por completo; cuando el genoma está activo, se redistribuye por miembro pero no desaparece.

## 8. Gobernanza y Estándares (Fijos)
- **Local-First (ADR-0016):** 100% Local.
- **Inundación de Fundaciones (ADR-0020):**
  - **Perfil IA / R&D:** Identidad + Linaje Genético + Hardware.
  - **Contrato de Persistencia:** Grupo I completo (`id`, `created_at`, `updated_at`, `audit_hash`, `audit_chain_hash`, `event_sequence_id`) + `parent_id`, `version_node_id` (III) + `portfolio_container_id` (V Gobernanza, híbrido) + `process_id` (IV).
  - **Rastro de Evidencia:** El grado de correlación penalizado se registra en la bitácora de generación.
- **Genomas Modulares por Dominio (ADR-0108/ADR-0111):** Esta feature es el origen FIJO del Gen de Condición de Correlación de Curva de Equidad Rodante del Dominio de Portafolio y Correlación. Ver Registro de Dominios Genómicos en [`SAD.md`](../SAD.md) §2.3.
