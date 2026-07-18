# DEBT-028 · El precio medio ponderado no pertenece a la barra: es un indicador

| Campo | Valor |
|---|---|
| **ID** | DEBT-028 |
| **Severidad** | 🟡 Baja |
| **Estado** | ✅ Pagada (2026-07-18) |
| **Abierta** | 2026-07-18 |
| **Pagada** | 2026-07-18 — [STORY-051](../execution/STORY-051-hybrid-data-transformer.md) §7/§8 punto 1 |
| **Origen** | El Rust-Engineer paró y escaló la duda en STORY-051; resuelta por doctrina del propietario |
| **Dominio** | Ingesta / frontera con el motor de indicadores |

## Causa raíz

El tipo `Bar` del transformador incluye un campo de precio medio ponderado por volumen. La doctrina del propietario (2026-07-18) resuelve que **no corresponde ahí**:

> *"Los datos del mercado —máximo, mínimo, apertura, cierre, compra, venta, volumen— son materia prima. Cualquier cosa que use esos datos primitivos debe considerarse un indicador. El precio promedio ponderado definitivamente es un indicador, y se usa como el ATR, el MACD, el RSI, el perfil de volumen o una media móvil."*

Es decir: la **ingesta persiste primitivos**; los **indicadores se calculan en el motor** (EPIC-2/EPIC-3). El campo se coló en la fase equivocada del pipeline. No lo pedía ningún TTR.

Nota de precisión, para que nadie lo reclasifique mal en el futuro: este campo **no** viola el principio "guarda el hecho, calcula la interpretación" — a diferencia del precio ajustado por split, no cambia retroactivamente: queda fijo en cuanto la barra cierra. El motivo de sacarlo es de **frontera de fase**, no de mutabilidad.

## Impacto actual

Ninguno. La barra no se escribe a disco todavía y el campo no tiene consumidor. El coste de sacarlo hoy es trivial; dejarlo abriría la puerta a que la ingesta acumule indicadores uno a uno.

## Qué hay que hacer

1. Retirar el campo del tipo `Bar` y de la salida del transformador, junto con las pruebas que solo lo cubren a él. **Conservar** la aritmética de acumulador ancho y redondeo al par: esa lógica es correcta y la necesitará el motor de indicadores.
2. Registrar el indicador en el catálogo del motor, para EPIC-2/EPIC-3.
3. Considerar allí el patrón de **precio aplicable** que el propietario señaló de MT5 (`ENUM_APPLIED_PRICE`): un indicador se calcula eligiendo sobre qué precio se aplica — apertura, máximo, mínimo, cierre, mediana, típico o ponderado.

## Disparador de pago

**El cierre de STORY-051**, antes de despachar el puente hacia el frontend — para no congelar el campo en el contrato con la interfaz.

## Resolución (2026-07-18)

Punto 1 pagado en `crates/features/data/hybrid-data-transformer/`: `vwap` retirado de `domain::Bar`, `schemas::BarOutput` y del esquema expuesto por `verify`; la aritmética de acumulador ancho (`checked_scaled_product`, `round_half_to_even_div_i128`) se conservó intacta, con sus pruebas reorientadas para ejercitarse directamente en vez de a través de `Bar.vwap`. `compute_bars_audit_hash` se corrigió de paso (hallazgo de la auditoría del Tech-Lead): ahora incluye `bar_close_timestamp_ns`, `spread_reported` y los nueve campos de Futuros/Opciones, que quedaban fuera de la representación canónica pese a que su doc-comment afirmaba protegerlos. Detalle en [STORY-051 §7/§8 punto 1](../execution/STORY-051-hybrid-data-transformer.md) y en la lección `docs/lessons/rust/STORY-051-hybrid-data-transformer.md`.

Puntos 2 y 3 (registrar el indicador en el catálogo del motor y considerar el patrón de precio aplicable de MT5) **NO se pagan aquí** — dependen de que exista el motor de indicadores, que no se construye hasta EPIC-2/EPIC-3. Quedan fuera del alcance de esta corrección; no requieren una nueva entrada de deuda porque ya están descritos en este documento como trabajo futuro, sin disparador propio adicional al de EPIC-2/EPIC-3 mismo.
