# Meta-Learning Hub — Aprendizaje de Estrategias sobre Estrategias

**Carpeta:** `./moonshots/`
**Estado:** Moonshot (Fase 5 - Experimental)
**Última actualización:** 2026-04-13

---

## ¿Qué es?

El Meta-Learning Hub implementa el concepto de "Aprender a Aprender". En lugar de optimizar una estrategia aislada, el sistema analiza el éxito y fracaso de toda la población del Databank para transferir conocimientos (Transfer Learning) y acelerar la convergencia de nuevos Alphas.

---

## Comportamientos Observables

- [ ] **Cross-Strategy Feature Importance:** Identifica qué tipos de indicadores funcionan mejor colectivamente en ciertos regímenes.
- [ ] **Transfer Learning:** Inicializa el genoma de nuevas estrategias usando pesos de estrategias exitosas previas.
- [ ] **Self-Correcting Architecture:** El sistema ajusta sus propios hiperparámetros de generación basándose en el meta-análisis del P&L real.

---

## Tareas (TTRs)

### **TTR-001: Extracción de Meta-Features del Databank**
*   **Descripción:** Proceso de ETL sobre los metadatos de 100K+ estrategias para identificar patrones de éxito global.

### **TTR-002: Inicialización de Población por Transferencia (Seed Transfer)**
*   **Descripción:** Inyectar individuos en NSGA-II que heredan propiedades de la "élite" histórica.

---

## Gobernanza y Estándares (ADR-0020)
- Registro del **Grupo I (universal) + Perfil IA/R&D** (ADR-0020).
