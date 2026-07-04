# PFOF / Venta de Flujo de Órdenes

**Carpeta:** `./moonshots/`
**Estado:** Moonshot — ⚠️ **ZIZAÑA** (imposibilidad en el modelo local; condicionado a un pivote de arquitectura)
**Última actualización:** 2026-07-03
**Decisión Arquitectónica Asociada:** ADR-0144 (clasificación de modelos)

---

## ¿Qué es?

La idea (del documento-semilla): cobrar a brókers/market makers por el flujo de órdenes de los usuarios (Payment for Order Flow), estilo Robinhood.

## ⚠️ Por qué es zizaña (no va al núcleo)

- **Imposibilidad en el modelo local:** en Drasus el usuario conecta **su propio** bróker desde **su** máquina; el proveedor **no custodia ni rutea** la orden. Sin ser el ruteador/bróker, no hay flujo que vender. Capturar PFOF exigiría convertirse en intermediario de ejecución (broker-dealer regulado) — un pivote de arquitectura y de licencia, no una feature.
- **Riesgo regulatorio:** el PFOF está bajo escrutinio de la SEC; funciona mejor en futuros/forex que en equities y arrastra conflicto de interés.
- **Frontera con el veneno reputacional:** rozar la venta de flujo identificable deriva hacia operar contra los propios usuarios — explícitamente descartado (ADR-0144).

## Condición para reconsiderarlo

Solo tendría sentido si algún día se ofrece **ejecución hosteada por el proveedor** (el proveedor rutea), lo cual contradice hoy el cómputo Local-First (ADR-0143). Queda archivado como posibilidad remota, no como cimiento.

## Dependencias

- Requeriría: rol de ruteador/broker-dealer, licencias regulatorias, y derogar el cómputo Local-First para la ruta de orden. Ninguna existe ni se planea.
