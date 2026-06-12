# Sovereign Execution Engine (Motor de Ejecución Propio Multi-Activo)

**Carpeta:** `./moonshots/`
**Estado:** Moonshot (Contingencia — Inactivo por Decisión Arquitectónica)
**Última actualización:** 2026-06-10
**Decisión Arquitectónica Asociada:** ADR-0107 (opción evaluada y rechazada como ruta primaria)

---

## ¿Qué es?

Motor de backtesting y ejecución event-driven escrito desde cero, 100% propio, que reemplazaría a NautilusTrader como músculo de simulación y operativa real. Replicaría los atributos que hacen institucional a NT: paridad investigación-producción (misma semántica de ejecución en backtest y live), loop de eventos determinista con bus de mensajes en memoria de cero copias, matching engine con modelos de fill y latencia configurables, y resolución temporal de nanosegundos.

**Por qué es moonshot:** El ADR-0107 resolvió integrar los crates Rust nativos del núcleo v2 de NautilusTrader. Construir un motor propio consumiría años-persona sin generar Alpha diferencial (el Alpha vive en Generate/Validate, no en el matching engine). Este documento existe únicamente como **plan de contingencia de salida**: se activa solo si el upstream de NT se abandona o vira contra los intereses del proyecto, y después de agotar las opciones de congelamiento de versión vendorizada y fork de mantenimiento mínimo descritas en el ADR-0107.

---

## Mandato de Cobertura Multi-Activo (Requisito de Producto)

Si este motor llegara a construirse, DEBE soportar cualquier clase de activo financiero con la siguiente priorización:

1. **Primera clase desde el día uno:** acciones, forex, futuros, ETFs y CFDs.
2. **Última fase del roadmap:** opciones financieras — diferidas por la complejidad intrínseca del instrumento (griegas, cadenas de vencimientos, ejercicio/asignación, liquidación) y por el acceso problemático a datos históricos de calidad.

---

## Comportamientos Observables

- [ ] Una misma estrategia produce resultados bit-a-bit idénticos entre dos corridas de backtest con las mismas semillas y datos.
- [ ] La estrategia promovida a producción opera bajo la misma semántica de ejecución que mostró en backtest, sin reescritura ni adaptación.
- [ ] El operador simula y opera instrumentos de las cinco clases de activo de primera clase con el mismo contrato de la capa anticorrupción (`nautilus-integration`), sin que la lógica de negocio detecte el cambio de motor.

---

## Tareas (TTRs)

### **TTR-001: Núcleo de Eventos Determinista y Matching Engine Multi-Activo**
*   **¿Cuál es el problema?** Si el upstream de NautilusTrader muere, el proyecto necesita un motor propio que preserve la paridad sim/live sin heredar la deuda de un fork de cientos de miles de líneas.
*   **¿Qué tiene que pasar?** Implementar en Rust el loop de eventos determinista, el bus de mensajes en memoria y el matching engine con modelos de fill, latencia y fricción institucional (ADR-0017), exponiendo el mismo contrato que hoy consume la capa anticorrupción.
*   **¿Cómo sé que está hecho?**
    - [ ] La suite de paridad bit-a-bit del puente pasa contra el motor propio sin modificar ninguna feature de negocio.
*   **¿Qué no puede pasar?** NUNCA acoplar la lógica de negocio a tipos internos del nuevo motor: la capa anticorrupción sigue siendo el único punto de contacto.

---

## Gobernanza y Estándares (ADR-0020 V2)
- Perfil Ops / Hot-Path: Identidad + Soberanía + Hardware + Latencia (Máximo 1ms en validación pre-trade; orden end-to-end ≤100ms).
