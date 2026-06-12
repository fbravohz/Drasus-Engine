# Knowledge Graphs Vectoriales + Explorador de Galaxias (SQX Mod 25)

**Carpeta:** `./moonshots/`
**Estado:** Moonshot (Fase 5 - Experimental)
**Última actualización:** 2026-06-06

---

## ¿Qué es?

Base de datos orientada a grafos (Neo4j/embebida) que modela las relaciones semánticas, de parentesco y descendencia genética de las estrategias dentro del repositorio masivo. Se acompaña de una visualización 3D interactiva en Flutter en forma de galaxia de partículas, donde la distancia espacial representa el grado de correlación y parentesco de las lógicas operativas.

---

## Comportamientos Observables

- [ ] **Grafo de Parentesco Semántico:** Almacenamiento de dependencias en Neo4j indicando relaciones (ej. "Estrategia A mutó de Estrategia B al cambiar el parámetro X").
- [ ] **Galaxia Algorítmica 3D:** Renderizado tridimensional navegable de constelaciones de estrategias, donde los sistemas de trading se agrupan en cúmulos según afinidad operativa.
- [ ] **Detección de Superposición de Linaje:** Alerta al usuario si intenta incluir en una cartera estrategias con ancestros comunes muy recientes para evitar sobreajuste cooperativo.

---

## Tareas (TTRs)

### **TTR-001: Mapeador de Linaje en Base de Datos de Grafos**
*   **¿Cuál es el problema?** El almacenamiento en tablas planas dificulta rastrear la evolución genética y el impacto de cambios de parámetros a través de miles de generaciones de estrategias.
*   **¿Qué tiene que pasar?** Diseñar un esquema de base de datos de grafos que registre los nodos (estrategias, datasets) y sus relaciones evolutivas para permitir consultas complejas de ancestros y linaje.
*   **¿Cómo sé que está hecho?**
    - [ ] El sistema es capaz de retornar el árbol genealógico completo de cualquier estrategia en menos de 50ms.
*   **¿Qué no puede pasar?**
    - No deben generarse ciclos infinitos en el grafo de linaje.

---

## Gobernanza y Estándares (ADR-0020 V2)
- Registro de **25 campos mandatorios** para la trazabilidad de nodos del grafo en cada mutación registrada.
