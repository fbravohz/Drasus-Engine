# Option Chain Manager — Gestor de Cadenas de Opciones

**Carpeta:** `./moonshots/`
**Estado:** Moonshot (Post-MVP — Diferido por ADR-0140)
**Última actualización:** 2026-06-27
**Decisión Arquitectónica Asociada:** ADR-0140 (Opciones Financieras — Diferimiento al Post-MVP con Puerta Abierta)

---

## ¿Qué es?

Gestor de la estructura completa de contratos de opciones disponibles para un subyacente: organiza la cadena de vencimientos, strikes, tipos (call/put), y metadatos de cada contrato (open interest, volumen, bid/ask, multiplicador). Proporciona la vista unificada que consumen el pricing engine, el strategy builder y el greeks monitor.

**Por qué es moonshot:** La cadena de opciones es una estructura de datos bidimensional (strike × vencimiento) que cambia diariamente con nuevos vencimientos listados y contratos expirados. Mantenerla sincronizada con el mercado en tiempo real y proporcionar acceso eficiente por strike, vencimiento, moneyness (ITM/ATM/OTM) o delta es un problema de ingeniería de datos no trivial.

**Condición de activación (ADR-0140):** los cinco prerrequisitos del ADR-0140 deben cumplirse antes de implementar.

---

## Comportamientos Observables

- [ ] Dado un subyacente (ej. SPY), el gestor devuelve la lista completa de vencimientos disponibles con sus strikes cotizados.
- [ ] El usuario filtra la cadena por moneyness (solo ITM, solo ATM ±5%, etc.) y el sistema devuelve los contratos relevantes.
- [ ] El sistema detecta y notifica cuando un nuevo vencimiento se lista o un vencimiento existente expira.
- [ ] Cada contrato en la cadena muestra: strike, tipo (call/put), bid, ask, last, volume, open interest, IV implícita.
- [ ] El gestor mantiene el multiplicador del contrato (ej. 100 acciones por contrato de opciones sobre acciones US) y lo expone para el cálculo de sizing.

---

## Tareas (TTRs)

### **TTR-001: Estructura de Datos de Cadena de Opciones**
*   **¿Cuál es el problema?** Una cadena de opciones es una matriz bidimensional (strike × vencimiento) con miles de contratos por subyacente. El acceso eficiente por cualquier dimensión (vencimiento, strike, moneyness, delta) requiere una estructura especializada.
*   **¿Qué tiene que pasar?** Implementar la estructura de datos que organiza los contratos por subyacente, vencimiento y strike, con índices para búsqueda rápida por cualquier criterio.
*   **¿Cómo sé que está hecho?**
    - [ ] Una consulta por vencimiento + rango de strikes responde en <1ms para una cadena de 5000+ contratos.

### **TTR-002: Sincronización de Cadena con el Mercado**
*   **¿Cuál es el problema?** La cadena cambia diariamente: nuevos vencimientos se listan, contratos expiran, strikes se añaden o eliminan.
*   **¿Qué tiene que pasar?** Mantener la cadena sincronizada con el proveedor de datos, detectando cambios estructurales (nuevos vencimientos, expiraciones) y actualizando quotes en tiempo real.
*   **¿Cómo sé que está hecho?**
    - [ ] Un nuevo vencimiento listado por el exchange aparece en la cadena dentro del intervalo de actualización configurado.

---

## Gobernanza y Estándares (ADR-0020)
- Perfil A (Datos / Ingest): Identidad + Linaje de Datos + Hardware. Registro del proveedor de datos, timestamp de sincronización y hash de integridad de la cadena.

---

## Dependencias

**Depende de:**
- [`option-data-ingestor`](./option-data-ingestor.md) — para el flujo de datos de opciones desde el proveedor.

**Bloquea:**
- [`option-pricing-engine`](./option-pricing-engine.md) — necesita la estructura de cadena para calcular precios.
- [`option-strategy-builder`](./option-strategy-builder.md) — necesita la cadena para seleccionar contratos.
- [`greeks-monitor`](./greeks-monitor.md) — necesita la cadena para mapear griegas por contrato.
