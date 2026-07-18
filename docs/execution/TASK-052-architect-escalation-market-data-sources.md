# TASK-052 · Paquete de escalamiento al Architect (fuentes de datos de mercado)

> Task SIN código. El Tech-Lead reúne aquí la evidencia de un hueco de diseño que **NO es suyo de resolver**: qué proveedores de datos históricos adopta Drasus. El Architect decide y documenta (ADR + features); el Tech-Lead no redacta ADRs.

> 🔄 **REPLANTEADA (2026-07-18) — la pregunta de esta Task ya no es la correcta.**
>
> El propietario fijó la doctrina que faltaba: **Drasus no provee datos de mercado; cada usuario trae los suyos desde sus propios brókers.** Canonizada en **ADR-0154**. La pregunta original —"qué proveedor adopta Drasus"— presuponía justamente lo que la doctrina niega.
>
> **Qué sobrevive de esta Task:** la evidencia verificada de qué puede entregar cada bróker (H3), que ahora es el insumo de la tabla de capacidades; el hueco de la clase de activo Forex/CFD, que pasa de "falta un proveedor" a "faltan conectores"; y la vía de exportación desde terminal, promovida a vía canónica de ingesta.
>
> **Qué muere:** la comparación de proveedores para adopción por Drasus, la ruta de adopción en fases, y el análisis de gratuito frente a pago desde nuestro presupuesto — el costo del dato es del usuario.
>
> **Qué la reemplaza:** una Task nueva de secuenciación de conectores, que responderá "qué conectores primero, con qué tabla de capacidades y en qué orden", tomando ADR-0154 como marco. Entregas relacionadas: `.agents/plans/esquema-canonico-datos-mercado.md` (Quant, campos de la operación y la vela) y `.agents/plans/veredicto-legal-datos-mercado.md` (General Counsel, tres veredictos).

| Campo | Valor |
|---|---|
| **ID** | TASK-052 |
| **Tipo** | Task (escalamiento) |
| **Épica** | EPIC-1 — Soberanía de Datos (`ingest`) |
| **Estado** | 🔄 **Replanteada (2026-07-18)** — la pregunta original quedó obsoleta; ver banner |
| **Creada** | 2026-07-17 |
| **Origen** | Pregunta del propietario durante el Gate de Coherencia de STORY-051 (transformador de datos): "¿qué datos van a usar para los tests?" |

---

## Entregable exigido por el propietario (condición de forma, no negociable)

El Architect **NO decide la fuente en esta iteración**. Produce primero un **plan de decisión** en `.agents/plans/<nombre>.md` que el propietario revisa y aprueba **antes** de que se selle cualquier proveedor en un ADR.

El plan debe permitir al propietario comparar opciones y elegir, no presentarle una decisión ya tomada.

---

## El hueco (evidencia)

### H1 — La fuente Forex/CFD del plan de EPIC-1 nunca se construyó

- **Lo que el ROADMAP exige** (`docs/ROADMAP.md` §EPIC-1): descarga híbrida de **2 fuentes**, con "prioridad de clase de activo: **una Forex/CFD primero**; una cripto como segunda fuente".
- **Lo que existe en el código:** solo cripto. `crates/features/data/sovereign-data-fetcher/src/schemas.rs:29` fija `https://data.binance.vision` como volcado masivo y `public_interface.rs:338` fija `https://api.binance.com` para el delta. No hay adaptador de ninguna fuente Forex/CFD.
- **Naturaleza:** el orden de construcción se invirtió respecto al plan. STORY-024 cerró con QA APTO sobre el motor de descarga, que es agnóstico a la fuente (`trait BulkSource` / `trait DeltaSource`) — no es un defecto de implementación, es una fuente que nunca se eligió.
- **Impacto:** el criterio de salida de EPIC-1 ("descarga, sanitiza y particiona 5+ años de 2 símbolos") no puede cumplirse con una sola clase de activo.

### H2 — ADR-0034 nombra proveedores de cripto, no de Forex/CFD ni de futuros

`docs/adr/ADR-0034.md` fija el patrón Bulk + Delta y cita "AWS S3, Binance Vision" para el volcado masivo, y "adaptadores de NautilusTrader (Binance, IBKR, Oanda v20)" para el delta. La ingesta masiva de Forex/CFD y de futuros **no está cubierta por ninguna decisión vigente**.

### H3 — Los brókers que el propietario usará no son proveedores de histórico masivo

El propietario nombró IC Markets, Darwinex, Pepperstone y Swissquote como brókers de uso inicial. **Contraste que el Architect debe resolver explícitamente, no asumir:** son brókers de **ejecución**; su histórico se obtiene por terminal MT4/MT5 o por su API de trading, orientada a lo reciente y a lo vivo. Salvo excepciones (Darwinex publica tick data), no publican volcados descargables de años.

Consecuencia de dominio que el plan debe abordar de frente:

- El precio de Forex/CFD es **propio de cada bróker** (cada uno cotiza su spread). Un backtest fiel a IC Markets exige, en última instancia, el dato de IC Markets.
- Ese dato llega por el **conector de bróker** (`broker-connector`, EPIC-5), no por la ingesta masiva de EPIC-1.
- Por tanto: ¿la fuente masiva de EPIC-1 es un **proveedor de referencia** para construir el lago y validar el motor, con la fidelidad al bróker concreto calibrada después por fricción institucional (ADR-0017/ADR-0069)? ¿O el propietario exige desde EPIC-1 el dato del bróker de ejecución? La respuesta cambia toda la estrategia de ingesta.

---

## Requisitos explícitos del propietario para el plan

1. **Dos granularidades, no una.** Se necesitan **datos de 1 minuto** y **datos de tick**. Son cosas distintas, con proveedores, formatos, volúmenes y costos distintos. El plan las trata por separado, nunca como un solo renglón.
2. **Futuros entran en el alcance del análisis.** Además de Forex/CFD y cripto, el plan evalúa proveedor(es) de datos de **futuros**.
3. **Prioridad a datos gratuitos.** Se agota lo gratuito primero. Si un proveedor de pago es necesario, se plantea con su costo y su justificación, y el propietario lo valida — no se descarta el pago por principio, pero tampoco se adopta por comodidad.
4. **Futuros: profundidad Level 1 y Level 2 (FYI del propietario, no exigencia de esta iteración).** El uso previsto incluye order flow y herramientas de microestructura, que requieren libro de órdenes (Level 2), no solo el mejor precio de compra/venta (Level 1). Es contexto para no elegir un proveedor que cierre esa puerta más adelante; no es entregable de esta Task.
5. **Licencia de uso.** Cada proveedor candidato se evalúa por sus condiciones de uso y redistribución. Drasus persiste datos en el equipo del usuario y agrega índices anonimizados (cimiento #9, `data-aggregation`) — hay términos de servicio que prohíben ambas cosas. **Sub-gate del General Counsel** (Etapa 0.4 legal) antes de sellar cualquier proveedor.

---

## Preguntas de diseño que el Architect debe resolver en el plan

| # | Pregunta | Por qué importa |
|---|---|---|
| Q1 | ¿La fuente masiva de EPIC-1 es un proveedor de referencia, o debe ser el bróker de ejecución? | Determina si la ingesta masiva y el conector de bróker son dos caminos o uno. Ver H3. |
| Q2 | ¿Un proveedor por clase de activo, o un proveedor multiactivo? | Afecta el número de adaptadores `BulkSource`/`DeltaSource` y la complejidad de normalización (TTR-008). |
| Q3 | ¿El tick de Forex debe traer precio de compra y de venta, o basta el último precio? | Sin bid/ask no hay spread real y la fricción del backtest se vuelve una suposición. Es el corazón de ADR-0017. |
| Q4 | ¿Qué formato de volcado soporta cada candidato? | El motor actual (`ReqwestBulkSource`) asume descarga HTTP de `.zip`/`.csv.gz` (ADR-0034). Un formato binario propietario o un canal FTP exige un adaptador nuevo — trabajo de Story, no de configuración. |
| Q5 | ¿El 1-minuto se descarga, o se deriva del tick con el transformador (STORY-051)? | Si se deriva, hay una sola fuente de verdad y menos superficie de ingesta; si se descarga, hay que reconciliar dos series que pueden discrepar. Decisión de arquitectura de datos, no de gusto. |
| Q6 | ¿Qué proveedor de futuros preserva la opción de Level 2 sin rehacer la ingesta? | Requisito 4. Elegir hoy un proveedor sin libro de órdenes puede costar una migración después. |
| Q7 | ¿Cómo se declara la procedencia del dato? | El linaje (`data_snapshot_id`, ADR-0141) debe distinguir de qué proveedor viene cada partición: dos fuentes del mismo símbolo no son intercambiables. |

---

## Lo que NO bloquea

STORY-051 (transformador de datos) **avanza sin esperar esta decisión**: su lógica es agnóstica a la fuente y sus pruebas son deterministas sobre datos fabricados más una muestra real congelada. La muestra se entrega en dos tandas por decisión del propietario: cripto ahora, Forex/CFD cuando esta Task se resuelva → registrado como **DEBT-025** con disparador explícito.

---

## Documentos que el Architect probablemente toque (no exhaustivo, lo decide él)

- `.agents/plans/<nombre>.md` — el plan de decisión (entregable de esta iteración).
- `docs/adr/ADR-0034.md` — enmienda o ADR nuevo con los proveedores adoptados.
- `docs/features/sovereign-data-fetcher.md` — fuentes soportadas.
- `docs/ROADMAP.md` §EPIC-1 — si el orden de entregas cambia.
