# Auto-Hedger — Targeted Drawdown Patching (Evolución SQX Correlation Matrix)

**Carpeta:** `./moonshots/`
**Estado:** Moonshot (Fase 5 - Experimental)
**Última actualización:** 2026-06-06
**Origen:** Propuesta CPO "El Auto-Hedger / Cirugía de Curva de Capital". Generación dirigida de cobertura inversa = R&D → incubación. Complementa (no reemplaza) el hedging cointegrativo de `portfolio-optimizer` y el `shield-netting-translator`.

---

## ¿Qué es?

Motor que pasa de la advertencia pasiva (matriz de correlación en rojo) a la **cirugía activa de la curva de capital**. El analista selecciona con el ratón un "valle" (Drawdown) específico y recurrente en el gráfico (ej. la estrategia colapsa cada septiembre), y el motor genera una **micro-estrategia de cobertura (Parche/Hedge)** diseñada específicamente para ganar dinero solo bajo las condiciones exactas en las que el sistema principal pierde. El humano ensambla así un portafolio simbiótico.

**Por qué es moonshot:** La generación dirigida de un parche que cubra un escenario específico sin introducir sobreajuste es un problema de investigación abierto.

---

## Comportamientos Observables

- [ ] El usuario selecciona un valle de Drawdown en la curva y pide "Generar Micro-Estrategia de Cobertura para este escenario".
- [ ] El motor entrena un algoritmo cuyo objetivo es rendir positivo solo bajo las condiciones del valle seleccionado.
- [ ] El humano evalúa el parche y decide ensamblarlo al portafolio principal.

---

## Tareas (TTRs)

### **TTR-001: Generación Dirigida de Cobertura Inversa**
*   **¿Cuál es el problema?** Una estrategia brillante puede tener un patrón de pérdida recurrente y localizado; borrarla entera desperdicia su alpha en el resto del tiempo.
*   **¿Qué tiene que pasar?** A partir del intervalo/condiciones del valle seleccionado, generar una micro-estrategia cuya ventaja se concentre en ese escenario, validando que no degrade el desempeño global.
*   **¿Cómo sé que está hecho?**
    - [ ] El parche mejora el Drawdown del escenario objetivo sin empeorar significativamente el resto.
*   **¿Qué no puede pasar?** NUNCA aceptar un parche sobreajustado al escenario que no sea robusto fuera de muestra.

---

## Gobernanza y Estándares (ADR-0020 V2)
- Perfil IA / R&D: Identidad + Soberanía + Pesos/Arquitectura + Hardware. Registro del escenario objetivo (intervalo/condiciones) y de la validación out-of-sample del parche.
