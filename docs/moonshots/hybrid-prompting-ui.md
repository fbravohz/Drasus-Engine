# Hybrid Prompting UI

**Carpeta:** `./moonshots/hybrid-prompting-ui/`
**Estado:** Incubación
**Última actualización:** 2026-06-06
**Decisión Arquitectónica Asociada:** ADR-0106 (Paradigma de Interfaz de Usuario y Dashboards Visuales de Alta Precisión)

---

## ¿Qué es esta feature?

El `Hybrid Prompting UI` es un asistente copiloto de autocompletado y sugerencia predictiva integrado en el Visual DAG Editor. A medida que el usuario ensambla y arrastra nodos lógicos en los Niveles 2 y 3 de la interfaz, el copiloto analiza la topología activa y recomienda nodos compatibles o sugiere parámetros optimizados basados en patrones de éxito históricos almacenados en el databank.

---

## Comportamientos Observables

- [ ] Mientras el usuario edita el lienzo visual (Nivel 3), al hacer clic derecho o arrastrar un pin de conexión, aparece un menú contextual predictivo que destaca "Sugerencias del Copiloto" (nodos de indicadores altamente correlacionados o filtros complementarios).
- [ ] Muestra un feed lateral con sugerencias de ajuste rápido de parámetros lógicos cuando detecta desestabilización en el performance de la curva del backtest en vivo.

---

## Restricciones

- **NUNCA** bloquear las interacciones físicas de arrastre o conexión de nodos en el lienzo Flutter con cálculos de predicción del copiloto.
- **NUNCA** forzar conexiones predictivas automáticas que violen las reglas estrictas de tipos o aciclicidad del DAG (petgraph).

---

## Parámetros Configurables

| Parámetro | Default | Rango | Qué hace | FIJO/CONFIG |
|---|---|---|---|---|
| ENABLE_AUTO_SUGGEST | true | true/false | Habilitar o deshabilitar sugerencias de autocompletado predictivo | CONFIG |
| SUGGESTION_THRESHOLD | 0.75 | 0.50 - 0.99 | Grado mínimo de certidumbre estadística para mostrar una recomendación | CONFIG |

---

## Estructura Interna (FCIS — ADR-0002)

- **Core (Lógica Pura):** Motor estadístico de afinidad y cálculo de topologías de grafos de éxito.
- **Shell (Infraestructura):** Cachés en memoria y FFI Commands hacia el visualizador de Flutter.
- **Frontera Pública:** Puerto de autocompletado que recibe la topología parcial y retorna recomendaciones.

---

## Tareas (TTRs)

### **TTR-001: Motor de Afinidad Topológica (Rust)**
*   **¿Cuál es el problema?** Recomendar nodos complementarios requiere cruzar la topología actual del DAG con patrones históricos rápidamente.
*   **¿Qué tiene que pasar?** Rust procesa la estructura parcial y calcula correlaciones utilizando la base de datos de estrategias ganadoras.
*   **¿Cómo sé que está hecho?**
    - [ ] El motor retorna sugerencias consistentes en menos de 5ms.

### **TTR-002: Menú Contextual Predictivo (Flutter)**
*   **¿Cuál es el problema?** El operador necesita sugerencias de diseño rápidas sin interrumpir el flujo visual de conexión.
*   **¿Qué tiene que pasar?** Incorporar en el lienzo CustomPainter los nodos flotantes predictivos recomendados por el motor Rust.
*   **¿Cómo sé que está hecho?**
    - [ ] Al arrastrar un cable en el lienzo, el menú de conexiones muestra las recomendaciones del copiloto.

---

## Gobernanza y Estándares (Fijos)
- **Local-First (ADR-0016):** 100% Local.
- **Inundación de Fundaciones (ADR-0020):** Perfil IA / R&D. Registra `version_node_id`, `node_id`, `audit_chain_hash`.
- **Rastro de Evidencia:** Emite métricas de uso del copiloto al módulo de `feedback`.
