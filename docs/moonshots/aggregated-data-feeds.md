# Aggregated Data Feeds (Productos de Datos Alternativos)

**Carpeta:** `./moonshots/`
**Estado:** Moonshot (adaptadores del cimiento — requieren volumen de usuarios)
**Última actualización:** 2026-07-03
**Decisión Arquitectónica Asociada:** ADR-0144 (substrato) · cimiento `data-aggregation` · ADR-0102

---

## ¿Qué es?

Un conjunto de **adaptadores** sobre el cimiento `data-aggregation` (ADR-0144 #9) que empaquetan los índices agregados anonimizados como feeds vendibles a entidades institucionales. Todos consumen el mismo puerto de agregación; cada feed es una lectura distinta del mismo consenso anonimizado.

Feeds incluidos:
- **Regime Intelligence Feed** — consenso de régimen de mercado en tiempo real basado en N estrategias activas. Cliente: multi-strategy funds, OCIOs. Ticket $30K–$300K/año.
- **Broker Friction Index** — índice comparativo de fricción real por bróker (slippage, fill-time, rechazo) como subproducto de la ejecución. Cliente: fondos (best execution), reguladores, prime brokers. Ticket $50K–$500K/año.
- **Correlation Breakdown Warning** — alerta temprana de convergencia de correlaciones a 1 (fin de la diversificación). Cliente: risk parity, multi-asset. Ticket $20K–$300K/año.
- **Liquidity Risk Signals** — señales agregadas de estrés de liquidez por instrumento. Cliente: fondos, market makers. Ticket $20K–$100K.

## Comportamientos Observables

- [ ] Un tercero autenticado consume un feed por la API → recibe el índice agregado, sin exponer a ningún usuario.
- [ ] Ningún agregado se publica por debajo del tamaño mínimo de cohorte (k-anonimato).
- [ ] Un usuario que ejerce opt-out deja de alimentar los feeds.

## Restricciones

- **NUNCA** un feed permite reconstruir la operación de un usuario individual (ADR-0102).
- **PROHIBIDO** cualquier feed que permita operar contra los propios usuarios (front-running — descartado en ADR-0144).
- Solo datos con consentimiento vigente (`consent-registry`) entran a los feeds externos.

## Dependencias

- **Depende de:** `data-aggregation`, `enriched-domain-events`, `consent-registry`, `third-party-api-gateway`.
- **Se activa:** cuando haya volumen de usuarios suficiente para que los agregados tengan valor estadístico (los contratos ya existen desde el día 1, por eso se capturan datos desde el usuario #1).

## Por qué es moonshot

El puerto y la captura se construyen ahora (no se puede agregar mañana lo que no se capturó hoy); el pipeline de agregación y la comercialización de cada feed dependen de masa crítica de usuarios. No bloquea el MVP.
