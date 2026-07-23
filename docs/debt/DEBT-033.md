# DEBT-033 · El techo de magnitud de la escala ×10⁸ excluye una clase entera de activos

| Campo | Valor |
|---|---|
| **ID** | DEBT-033 |
| **Severidad** | 🟠 Media |
| **Estado** | Abierta |
| **Abierta** | 2026-07-18 |
| **Origen** | Quant-Engineer, Etapa 6 de STORY-051 (validación post-código); aritmética verificada de forma independiente por el Tech-Lead |
| **Dominio** | Arquitectura de datos — ADR-0141 |
| **Escalamiento** | **TASK-061** — decisión del Architect sobre ADR-0141 |

## Causa raíz

ADR-0141 fija la escala entera ×10⁸ y justifica esa elección por **precisión decimal**: cubre los 8 decimales de cripto y los 5 de Forex. Es una justificación correcta y suficiente para el problema que resolvía.

Lo que nunca se re-derivó es el **techo de magnitud** que esa elección impone. Un campo `i64` escalado ×10⁸ solo puede representar:

```
i64::MAX / 10⁸ ≈ 92 233 720 369 unidades
```

Poco más de **92 200 millones de unidades** en cualquier magnitud acumulada — señaladamente `volume_traded`, que suma el tamaño de todas las ejecuciones del intervalo.

## Por qué es un problema real y no un extremo teórico

Existen activos negociados hoy, en la misma fuente que STORY-051 usa como muestra real, cuya **oferta circulante es de billones de unidades y cuyo precio unitario es ínfimo**: SHIB (~589 billones de unidades), PEPE, BONK y la clase de las monedas meme.

Verificación aritmética (Tech-Lead, 2026-07-18):

| Escenario | Unidades por barra de 1h | Contra el techo |
|---|---|---|
| SHIB a ~$0.00002, volumen conservador de 50 M USD/día | ~104 200 millones | **1,13× — desborda** |
| SHIB en día activo, 200 M USD/día | ~416 700 millones | **4,52× — desborda** |
| PEPE a ~$0.000012, 30 M USD/día | ~104 200 millones | **1,13× — desborda** |

No hace falta un pico de actividad ni un caso patológico: **el volumen normal de un día cualquiera excede el techo en una sola barra horaria.**

## Qué hace el sistema hoy

Lo correcto dentro de sus posibilidades: `checked_add` detecta el desbordamiento y devuelve `TransformError::Overflow` explícito. **No hay corrupción de datos ni envoltura silenciosa** — el invariante "fallar visible, nunca envolver en silencio" se cumple.

Pero el resultado práctico es que el transformador **rechaza sistemáticamente producir barras para esa clase de activo**, no en el margen sino bajo operación normal. Falla seguro, y falla siempre.

## Por qué no es un defecto de STORY-051

El código implementa fielmente la decisión de ADR-0141 y la implementa de forma segura. La laguna está en el ADR, que eligió la escala por precisión sin comprobar el rango. Corregirlo desde una feature sería parchear una decisión arquitectónica en el sitio equivocado.

## Directiva del propietario (2026-07-19)

> *"Mi intención no es descartar esos activos. Mientras más activos podamos operar, mejor. 100%."*

**La opción de declarar la clase de activo fuera de alcance queda descartada de entrada.** La decisión es *cómo* soportarla, no *si* se soporta. El detalle del encargo, con las tres salidas técnicas y sus costes, vive en **TASK-061**.

## Qué hay que decidir (Architect)

1. **Re-derivar el supuesto de ADR-0141 contra rango, no solo contra precisión.** ¿Qué combinaciones de precio y oferta debe soportar el sistema, declaradas explícitamente?
2. **Elegir la salida.** Las opciones no son equivalentes y tienen costes muy distintos: escala variable por instrumento; acumuladores de volumen en `i128`; una escala menor para volumen que para precio; o declarar la clase de activo fuera de alcance de forma explícita y documentada.
3. **Si se decide excluirla, decirlo donde el usuario lo vea.** La doctrina de la plataforma es que el usuario trae sus propios datos (ADR-0154). Si trae un activo que no podemos procesar, merece un mensaje que lo explique — no un error de desbordamiento genérico.

## Disparador de pago

**Antes de habilitar ingesta real de cripto de oferta extrema.** EPIC-1 incluye cripto explícitamente y la muestra de prueba ya viene de una fuente donde estos activos se negocian a diario. No es urgente hoy porque nadie ingiere en producción todavía; sí es previo a cualquier promesa pública de cobertura de mercados.

## Resolución de arquitectura (Architect, TASK-061, 2026-07-19)

**Decisión sellada, implementación pendiente.** `Bars.volume_traded`/`volume_buy` ensanchan de `i64` a `i128` — la escala ×10⁸ no cambia (sigue única para todo el sistema); solo se ensancha el ancho del acumulador en los dos campos donde el riesgo es real. Detalle completo, costo de infraestructura verificado (SQLite/Polars/DuckDB/`flutter_rust_bridge`) y alcance de la corrección: `docs/adr/ADR-0141.md` §"Ensanche de ancho para volumen acumulado" y `.agents/plans/architect-2026-07-19-escala-barras-ingesta.md`. Esta ficha permanece **Abierta** hasta que la Story de implementación aterrice el cambio en código (`domain.rs`/`schemas.rs`/`persistence.rs`/`Cargo.toml` del crate `hybrid-data-transformer` + `BarDto` del Bridge) — el Tech-Lead cierra/promueve esta ficha en ese momento, no antes.
