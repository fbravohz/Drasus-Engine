# Shared Capital Pool (Dynamic Margin Clustering)

**Carpeta:** `./moonshots/shared-capital-pool/`
**Estado:** Incubación / R&D (Fase 3 Moonshot)
**Última actualización:** 2026-05-31

---

## ¿Qué es este Moonshot?

El **Shared Capital Pool** es un módulo de investigación y desarrollo diseñado para permitir que múltiples portafolios federados (ej. Portafolio A y Portafolio B) coexistan operando sobre un único pool de capital de margen global consolidado, distribuyendo de forma dinámica la capacidad de compra y el margen requerido según las necesidades operativas de cada contenedor en tiempo real.

*   **Problema de R&D:** En entornos de trading institucional o multifirma, asignar capital estático ("Isolated Capital") a subcuentas genera ineficiencias críticas de liquidez: si el Portafolio A tiene 500k inactivos y el Portafolio B necesita 100k extra de margen para capturar una ineficiencia fugaz, B no puede tomarlos prestados, reduciendo el rendimiento global del capital.
*   **La Solución de Cómputo:** Un motor de asignación dinámica de margen ("Dynamic Margin Allocator") que monitoriza el riesgo intra-segundo, calcula la correlación cruzada global y cede capacidad de margen de forma transitoria de contenedores inactivos a contenedores activos, controlando estrictamente que el riesgo agregado sistémico no detone un Margin Call catastrófico.

---

## Retos Técnicos y Matemáticos

1.  **Cálculo de Margen Cruzado Tick-by-Tick:**
    *   Determinar los requisitos de margen en tiempo real de múltiples activos operando bajo diferentes regulaciones (ej. apalancamiento de futuros en CME vs apalancamiento lineal en Binance) sobre una única cuenta física de compensación.
2.  **Correlación Cruzada Dinámica Limitante:**
    *   Para evitar liquidaciones simultáneas en cascada (donde una caída en el Portafolio A líquida el capital que sostiene al Portafolio B), el motor debe exigir que la correlación inter-portafolio cruzada real se mantenga por debajo de un umbral dinámico (ej. < 0.6) en todo momento.
3.  **Algoritmo de Priorización de Margen:**
    *   Definir qué subportafolio tiene "derecho" a consumir el exceso de margen libre basado en métricas de expectativa matemática en caliente (`Live Expectancy` / `D-Score`) y penalizar a aquellos con alta volatilidad o drawdowns recientes.

---

## Comportamientos Esperados (Fase Experimental)

*   **Préstamo de Margen Dinámico:** El Portafolio A y el Portafolio B comparten un pool de 800,000. Si el Portafolio A requiere 600,000 de margen para sostener una serie de operaciones de alta probabilidad, el sistema reduce la capacidad máxima de exposición del Portafolio B a 200,000 en tiempo real. Al cerrarse las posiciones de A, la capacidad se libera y vuelve a estar a disposición de ambos.
*   **Veto por Riesgo Sistémico:** Si la volatilidad realizada del clúster se duplica y la correlación inter-portafolio escala de 0.3 a 0.75, el sistema suspende inmediatamente las solicitudes de nuevo margen de todos los contenedores y activa el modo de desapalancamiento coordinado.

---

## Línea de Investigación y Trabajo Futuro

1.  **Fase 1: Modelado de Margen Sintético (Simulación):**
    *   Crear una suite de simulación en Rust que consuma históricos de trades de múltiples estrategias concurrentes y compute el balance de margen flotante bajo reglas de cartera de margen cruzado (Portfolio Margin).
2.  **Fase 2: Filtro de Correlación Predictiva:**
    *   Integrar un modelo autoregresivo ligero que prediga picos de correlación inter-portafolio antes de que ocurran para congelar asignaciones de capital riesgoso.
3.  **Fase 3: Integración con Adaptadores Multibroker:**
    *   Validar la simulación contra APIs de compensación reales (ej. Interactive Brokers Portfolio Margin) para contrastar los cálculos locales vs las fórmulas del bróker.
