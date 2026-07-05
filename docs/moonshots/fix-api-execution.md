# FIX API Ultra-Low Latency Execution (Evolución SQX Source Code)

**Carpeta:** `./moonshots/`
**Estado:** Moonshot (Fase 5 - Experimental, Institucional)
**Última actualización:** 2026-06-06
**Origen:** Propuestas CPO "Ejecución Directa FIX API (El Fin de MetaTrader)" y "Smart Order Routing / Market Impact". NO retail-first (co-location institucional) → incubación.

---

## ¿Qué es?

Capa de ejecución institucional que conecta vía **FIX API** directamente con proveedores de liquidez (LMAX, PrimeXM, Currenex), eliminando los puentes minoristas (MetaTrader) propensos a latencia y recotizaciones. Las estrategias residirían en contenedores en Edge Computing (co-location en datacenters de mercado) para latencia de microsegundos. Incluye nodos de **Smart Order Routing** (Iceberg/TWAP/VWAP) y simulación de **impacto de mercado** (cuánto mueve el precio la propia orden al consumir el libro).

**Por qué es moonshot:** Requiere infraestructura de co-location y volumen institucional; el operador retail usa el `multiplatform-execution-bridge` y `nautilus-integration` existentes.

---

## Comportamientos Observables

- [ ] La señal de entrada viaja a un proveedor de liquidez vía FIX en microsegundos, sin pasar por software minorista.
- [ ] El backtest calcula cuánto moverá el precio la propia orden al consumir liquidez del DOM (impacto de mercado).
- [ ] El usuario arrastra un nodo de Smart Order Routing para fragmentar una orden grande (ej. VWAP en 45 min) y ocultar su huella.

---

## Tareas (TTRs)

### **TTR-001: Conector FIX y Simulador de Impacto de Mercado**
*   **¿Cuál es el problema?** Asumir fills instantáneos con slippage fijo es ilusorio para grandes volúmenes; y MetaTrader añade latencia y manipulación.
*   **¿Qué tiene que pasar?** Implementar sesión FIX con proveedores institucionales y un simulador que estime el impacto de la propia orden sobre el libro durante backtest.
*   **¿Cómo sé que está hecho?**
    - [ ] Una orden de gran tamaño en backtest refleja un coste de impacto creciente con el volumen.
*   **¿Qué no puede pasar?** NUNCA asumir absorción instantánea de liquidez para órdenes de gran tamaño.

---

## Gobernanza y Estándares (ADR-0020)
- Perfil Ops / Hot-Path: Identidad + Soberanía + Hardware + Latencia (objetivo microsegundos). Registro de la ruta de ejecución y del proveedor de liquidez por cada orden.
