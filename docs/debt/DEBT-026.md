# DEBT-026 · Marca de agregabilidad por origen en el linaje de datos

| Campo | Valor |
|---|---|
| **ID** | DEBT-026 |
| **Severidad** | 🟠 Media |
| **Estado** | Abierta |
| **Abierta** | 2026-07-18 |
| **Origen** | Veredicto del General Counsel (`.agents/plans/veredicto-legal-datos-mercado.md`), Frente 1 |
| **Dominio** | Ingesta / agregación / cumplimiento |

## Causa raíz

El motor de agregación (cimiento #9) es agnóstico al origen del dato: no sabe si el índice que construye deriva de precios de un tercero o del comportamiento del propio motor. El veredicto legal establece que esa distinción **es la frontera de lo que puede monetizarse**:

- Índices derivados de **datos de mercado de terceros** (régimen, correlación) quedan bloqueados para venta externa sin permiso del proveedor.
- Índices derivados del **comportamiento del motor de Drasus** (fricción medida en ejecuciones reales) tienen postura defendible.

Corrección de premisa que fundamenta esta deuda: anonimizar al usuario **no** rompe la restricción contractual de la fuente. Son dos problemas legales distintos. Nadie puede ceder un derecho que nunca tuvo.

Además, el veredicto distingue el **dato histórico** del **dato en vivo**: la fuente publica activamente el archivo histórico, mientras que el canal de tiempo casi real vive dentro de términos más restrictivos. Esa diferencia tampoco está representada en ningún sitio.

## Impacto actual

Ninguno hoy: no se vende ningún índice ni existe adaptador de red hacia el servidor central. La deuda muerde el día que el interruptor de venta externa se active, porque entonces no habrá forma de saber qué índice puede salir y cuál no — y la respuesta no se puede reconstruir a posteriori si el origen no quedó marcado en el momento de la ingesta.

## Qué hay que construir

1. Marca de **agregabilidad** en el linaje de cada partición: qué fuente la originó y qué permite la licencia de esa fuente (campo previsto en la tabla de capacidades de ADR-0154).
2. Distinción **histórico frente a vivo** en esa misma marca.
3. Guardarraíl en el motor de agregación: un índice cuyo insumo tenga una partición no agregable no se emite al canal externo.

## Disparador de pago

El primero que ocurra: **activación del interruptor de venta externa**, o **construcción del adaptador de red hacia la Cabina de Mando**. Debe pagarse ANTES de que cualquiera de los dos entre en servicio, nunca después.
