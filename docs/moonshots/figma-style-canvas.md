# Figma-Style Collaborative Canvas — Colaboración Multijugador

**Carpeta:** `./moonshots/figma-style-canvas/`
**Estado:** Incubación (Fase 4 - Moonshot)
**Última actualización:** 2026-06-06

---

## ¿Qué es esta feature?

El Figma-Style Collaborative Canvas es una característica de incubación para posibilitar el diseño de estrategias en equipo de forma síncrona. Expone cursores y selecciones en tiempo real para que múltiples analistas editen el mismo lienzo de Visual Scripting (Nivel 2/3) de forma concurrente, facilitando la investigación cuantitativa colaborativa.

---

## Comportamientos Observables

- [ ] Los analistas pueden visualizar cursores identificados por nombre y color moviéndose en el lienzo en tiempo real.
- [ ] Si el analista A altera el umbral de un indicador en el panel contextual, el cambio se propaga de forma inmediata y visible al analista B.
- [ ] El sistema gestiona bloqueos locales de edición sobre nodos específicos para evitar conflictos de escritura simultáneos.

---

## Restricciones

- **NUNCA** comprometer el rendimiento de la simulación local de trading por sobrecargas en la capa de sincronización de red.
- **NUNCA** requerir conexión a internet obligatoria para la operativa local monousuario (la colaboración multijugador es un overlay opcional).

---

## Tareas (TTRs)

### **TTR-001: Servidor WebSockets de Sincronización de Cursores**
*   **¿Cuál es el problema?** Transmitir coordenadas de mouse en tiempo real a múltiples clientes sin saturar el core.
*   **¿Qué tiene que pasar?** Crear un micro-servicio ligero de señalización WebSocket que difunda las coordenadas y estados de selección sin procesar en el core de Rust de la estrategia.
