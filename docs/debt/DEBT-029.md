# DEBT-029 · Los contratos de entrada aceptan campos desconocidos en silencio

| Campo | Valor |
|---|---|
| **ID** | DEBT-029 |
| **Severidad** | 🟠 Media |
| **Estado** | Abierta |
| **Abierta** | 2026-07-18 |
| **Origen** | Auditoría del Tech-Lead sobre el retrofit ADR-0155 de STORY-051 |
| **Dominio** | Transversal — todo crate que deserialice datos de mercado desde el exterior |

## Causa raíz

Ningún `struct` de deserialización del repositorio lleva `#[serde(deny_unknown_fields)]` — verificado por barrido: la anotación no aparece ni una vez en `crates/`. Por defecto, `serde` **ignora en silencio** todo campo del JSON de entrada que el struct no declara.

Para `TradeTickInput` (STORY-051) esto deja la garantía de ADR-0155 restricción 2 a medio camino:

- **Contra una cotización pura la garantía es sólida y está probada.** Un JSON con `bid`/`ask` y sin `price_scaled`/`trade_volume_scaled` falla al deserializar por campos obligatorios ausentes — prueba `quote_shaped_json_never_deserializes_into_a_trade_tick_and_cannot_reach_aggregation`, en el borde real del CLI.
- **Contra un payload híbrido la garantía no existe.** Un JSON que traiga `bid`/`ask` *además* de `price_scaled`/`trade_volume_scaled` deserializa sin protestar, y los campos de cotización se descartan sin dejar rastro. Nadie se entera de que la fuente mezclaba dos tipos físicos distintos.

El caso híbrido no es hipotético: un exportador de terminal mal configurado, o un CSV que un usuario concatenó a mano, producen exactamente esa forma. Y la doctrina de la plataforma es que **el usuario trae sus propios datos** (ADR-0154) — la variedad de formas de entrada no la controlamos nosotros.

## Por qué no se arregló en STORY-051

Añadir la anotación solo a `TradeTickInput` sería un parche cosmético: el hueco es del repositorio entero, y la decisión de si el rechazo estricto es la política por defecto —y qué hacer con los campos extra legítimos que una fuente rica puede traer— es de arquitectura, no de una feature.

Hay una tensión real que hay que resolver antes de aplicarlo a ciegas: rechazar todo campo desconocido puede romper la ingesta de una fuente que simplemente entrega más columnas de las que hoy modelamos. La respuesta probablemente no sea "rechazar siempre" sino "rechazar en los tipos donde la confusión entre tipos físicos es el riesgo, y registrar lo descartado en el resto".

## Qué hay que hacer

1. **Decisión del Architect** (ADR o enmienda a ADR-0155): política de campos desconocidos en los contratos de entrada de datos de mercado. Como mínimo debe cubrir los tipos donde ADR-0155 separa tipos físicos (`TradeTick` / `QuoteTick` / profundidad / eventos de orden), que es donde el silencio hace daño.
2. **Aplicación** a los structs afectados, con una prueba por struct que demuestre el rechazo del payload híbrido.
3. **Barrido** del resto de contratos de deserialización del repositorio bajo la política que se fije.

## Disparador de pago

**Antes de que exista el segundo conector de ingesta.** Con una sola fuente el riesgo es teórico; en cuanto haya dos formatos de entrada distintos conviviendo, un payload híbrido deja de ser una hipótesis. Se compone con la Task de secuenciación de conectores.

## Resolución parcial de arquitectura (Architect, TASK-063, 2026-07-19)

**El punto 1 ("Decisión del Architect: política de campos desconocidos") queda resuelto por doctrina D3 del propietario, canonizada en `docs/adr/ADR-0156.md` restricción 5:** un payload que mezcla columnas de cotización y de ejecución sin que ningún mecanismo de plataforma conocido (ej. el bitmask de MT5, que sigue permitido vía plantilla) lo explique **se rechaza con error de formato explícito** — nunca se consume parcialmente en silencio. Esto cubre exactamente el caso que esta ficha describe (`TradeTickInput` deserializando un JSON con `bid`/`ask` de sobra). **Lo que sigue abierto en esta ficha, sin cambio:** el punto 2 (aplicar `#[serde(deny_unknown_fields)]` o equivalente a los structs afectados, con su prueba discriminante por struct) y el punto 3 (barrido del resto de contratos de deserialización del repositorio). Son trabajo de implementación, no de decisión — quedan para la Story que el Tech-Lead despache. Detalle: `.agents/plans/architect-2026-07-19-escala-barras-ingesta.md`.
