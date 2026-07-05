# Option Data Ingestor — Ingesta de Datos de Opciones

**Carpeta:** `./moonshots/`
**Estado:** Moonshot (Post-MVP — Diferido por ADR-0140)
**Última actualización:** 2026-06-27
**Decisión Arquitectónica Asociada:** ADR-0140 (Opciones Financieras — Diferimiento al Post-MVP con Puerta Abierta)

---

## ¿Qué es?

Gateway de ingesta especializado en datos de opciones financieras: cadenas de vencimientos, quotes bid/ask por strike, volatilidad implícita (IV surface), open interest, volumen de contratos y datos de ejercicio/asignación. Transforma los datos crudos del proveedor en el formato estándar interno de Drasus Engine (Parquet/Arrow + DuckDB) con validación Point-In-Time (PIT).

**Por qué es moonshot:** Los datos históricos de opciones son significativamente más caros, fragmentados y difíciles de obtener que los datos de instrumentos lineales. Proveedores como CBOE, OPRA (US options) o exchanges de crypto options (Deribit) tienen formatos propietarios, latencias distintas y coberturas de strikes/vencimientos variables. La IV surface histórica es particularmente problemática: reconstruirla requiere quotes snapshot de cada strike × vencimiento en cada momento, no solo OHLCV.

**Condición de activación (ADR-0140):** los cinco prerrequisitos del ADR-0140 deben cumplirse antes de implementar. El prerrequisito #4 (fuente de datos históricos de opciones viable) es el más crítico para esta feature.

---

## Comportamientos Observables

- [ ] El sistema descarga la cadena completa de opciones de un subyacente (todos los vencimientos × strikes cotizados) desde el proveedor configurado.
- [ ] Los quotes bid/ask se almacenan con timestamp PIT (Point-In-Time) para evitar look-ahead bias en backtesting.
- [ ] La IV surface se reconstruye a partir de los quotes almacenados para cualquier instante histórico.
- [ ] El open interest y volumen se almacenan por contrato y se agregan por vencimiento.
- [ ] El sistema valida la integridad de los datos: detecta gaps de quotes, strikes sin cotización y vencimientos expirados.

---

## Tareas (TTRs)

### **TTR-001: Adaptador de Proveedor de Datos de Opciones**
*   **¿Cuál es el problema?** Cada proveedor de datos de opciones (CBOE, OPRA, Deribit, IBKR) tiene su propio formato de cadena, quotes y metadata.
*   **¿Qué tiene que pasar?** Implementar un adaptador por proveedor que normalice los datos al formato interno estándar (Parquet/Arrow), siguiendo el patrón de `sovereign-data-fetcher` y `data-normalization-layer`.
*   **¿Cómo sé que está hecho?**
    - [ ] Al menos un proveedor de datos de opciones está soportado y produce datos normalizados consumibles por el `option-chain-manager`.

### **TTR-002: Almacenamiento PIT de Quotes de Opciones**
*   **¿Cuál es el problema?** Los quotes de opciones (bid/ask por strike × vencimiento) cambian constantemente. Para backtesting se necesita el quote exacto que existía en cada instante (PIT), no el último disponible.
*   **¿Qué tiene que pasar?** Almacenar los quotes con versionado Point-In-Time (ADR-0127), siguiendo el patrón de `fundamental-event-store` para eventos con vintage/as-of.
*   **¿Cómo sé que está hecho?**
    - [ ] Una consulta PIT devuelve el quote exacto que existía en un timestamp histórico, sin look-ahead.

### **TTR-003: Reconstrucción de IV Surface Histórica**
*   **¿Cuál es el problema?** La IV surface es una construcción derivada de los quotes. Para backtesting de estrategias de opciones se necesita la surface que existía en cada momento, no la actual.
*   **¿Qué tiene que pasar?** Reconstruir la IV surface a partir de los quotes PIT almacenados, con interpolación para strikes y vencimientos no cotizados directamente.
*   **¿Cómo sé que está hecho?**
    - [ ] La IV surface reconstruida para un timestamp histórico produce volatilidades coherentes con los quotes de ese momento.

---

## Gobernanza y Estándares (ADR-0020)
- Perfil A (Datos / Ingest): Identidad + Linaje de Datos + Hardware. Registro del proveedor, timestamp de ingesta, hash de integridad y versionado PIT.

---

## Dependencias

**Depende de:**
- [`sovereign-data-fetcher`](../features/sovereign-data-fetcher.md) — patrón de descarga masiva.
- [`data-normalization-layer`](../features/data-normalization-layer.md) — patrón de normalización de formatos de proveedores.
- [`pit-data-validator`](../features/pit-data-validator.md) — validación Point-In-Time.

**Bloquea:**
- [`option-chain-manager`](./option-chain-manager.md) — consume los datos normalizados.
- [`option-pricing-engine`](./option-pricing-engine.md) — consume la IV surface.
