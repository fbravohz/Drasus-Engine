# DEBT-030 · La reagregación barra→barra hereda la temporalidad equivocada

| Campo | Valor |
|---|---|
| **ID** | DEBT-030 |
| **Severidad** | 🟠 Media |
| **Estado** | Abierta |
| **Abierta** | 2026-07-18 |
| **Origen** | Auditoría del Tech-Lead sobre STORY-057 (el Rust-Engineer lo documentó en el código y lo reportó) |
| **Dominio** | Ingesta — identidad de series |

## Causa raíz

`reaggregate_bars` (en `crates/features/data/hybrid-data-transformer/src/domain.rs`) construye la identidad de la barra resultante copiándola de la primera sub-barra del grupo. Para cinco de los seis campos eso es correcto — instrumento, bróker, proveedor y dataset no cambian al reagregar.

**`timeframe_value` sí cambia, y no se re-deriva.** Al combinar barras de 1 minuto en una de 1 hora, la barra resultante queda etiquetada `1M` cuando es `1H`.

La consecuencia no es cosmética. `timeframe_value` es el componente central de la tripleta de linaje que ADR-0155 canonizó: una serie horaria se presentaría al usuario como `EURUSD-1M-PEPPERSTONE`. Y como los campos de identidad entran ahora en `compute_bars_audit_hash` (STORY-057), la etiqueta falsa queda sellada dentro de la huella de auditoría — el mecanismo con el que probamos que una transformación es reproducible acabaría certificando una identidad incorrecta.

Es precisamente la clase de confusión que la tripleta existe para impedir. La doctrina del propietario es que dos series que difieren en cualquiera de los tres componentes **no son la misma serie**; aquí el sistema afirmaría que sí lo son.

## Impacto actual

**Ninguno.** `reaggregate_bars` no tiene ningún llamador en el código: está cableada para TTR-002 pero ningún camino de ejecución la alcanza. El defecto es puramente latente.

Se registra en vez de dejarse como comentario en el código porque el día que alguien conecte la reagregación —que es una función obviamente útil y que alguien querrá usar— el fallo es silencioso: no rompe nada, no lanza error, solo etiqueta mal cada barra que produce.

## Qué hay que hacer

1. **Derivar `timeframe_value` del intervalo destino**, no copiarlo. Requiere decidir la representación canónica del intervalo como texto (`1H`, `4H`, `1D`) y la función que convierte nanosegundos a esa forma — hoy no existe.
2. Considerar si `bar_type` necesita el mismo tratamiento: reagregar barras de tiempo produce barras de tiempo, pero reagregar barras de volumen o de rango probablemente no tenga una semántica bien definida y deba rechazarse explícitamente en vez de producir algo dudoso.
3. **Prueba discriminante:** reagregar barras de 1 minuto a 1 hora debe producir una tripleta con la temporalidad horaria, y un sello de auditoría distinto del que produciría la misma serie etiquetada como de minutos.

## Disparador de pago

**Antes de que `reaggregate_bars` reciba su primer llamador.** No antes: hoy no hay nada que corregir en producción. No después: en cuanto haya un consumidor, cada barra que produzca sale con la identidad falsificada.
