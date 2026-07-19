# TASK-059 · Fijar como contrato el "momento de conocibilidad" de una barra

| Campo | Valor |
|---|---|
| **ID** | TASK-059 |
| **Tipo** | Task (arquitectura — requisito de diseño para una feature futura) |
| **Responsable** | Architect |
| **Estado** | 🟡 Abierta — pendiente de despacho |
| **Creada** | 2026-07-18 |
| **Origen** | Quant-Engineer, Etapa 6 de STORY-051 |
| **Disparador** | Antes de diseñar el validador PIT (EPIC-2/EPIC-3) |

> ⚠️ **CORRECCIÓN (2026-07-19, tras verificación del propietario).** La versión original de esta Task afirmaba que *"nada en el sistema fija todavía"* esta regla. **Era falso, y el propietario lo detectó de memoria.** La regla existe, es explícita y está declarada **FIJA** desde ADR-0114:
>
> > *"**Bar-Open Alignment obligatorio** — toda señal calculada al cierre de la barra N se ejecuta al precio de apertura de la barra N+1, sin entradas a precios intermedios."*
>
> Forma parte del **Contrato de Consistencia Conservadora**, junto con la Regla Intrabar Pesimista (si el stop y el objetivo se tocan en la misma vela, se asume siempre que saltó el stop). Está además reflejada en `docs/features/backtest-engine.md`.
>
> **Qué queda realmente pendiente, que es mucho menos de lo que esta Task decía:** ADR-0114 declara la regla en lenguaje de negocio ("al cierre de la barra N", "apertura de N+1") pero **no la ata al campo concreto** que el transformador ya produce (`bar_close_timestamp_ns`). El alcance de esta Task se reduce a cerrar esa costura.
>
> **Lección de método, más valiosa que la Task:** el Quant recomendó canonizar algo ya canonizado, y yo lo registré sin verificar. Es exactamente el fallo que la nueva **Etapa 0.4 (Barrido de Coherencia Pre-Orden)** existe para impedir — trabajo propuesto sobre decisiones ya tomadas, por no buscar antes de proponer.

## El problema, en una frase

Una barra lleva dos marcas de tiempo, y usar la equivocada produce un sesgo de anticipación tan sutil que casi nadie lo detecta revisando código.

## Detalle

El transformador expone honestamente los dos instantes de cada barra:

- `bar_timestamp_ns` — **inicio** del intervalo. Es la etiqueta con la que la barra se identifica y se ordena.
- `bar_close_timestamp_ns` — **cierre** exclusivo. Es el instante a partir del cual la barra está completa y, por tanto, es conocible.

La etiqueta es correcta y el intervalo es semiabierto, verificado por prueba explícita. **Pero nada en el sistema fija todavía, como contrato obligatorio, que el motor de backtest y el validador anti look-ahead deben usar el instante de CIERRE como "momento en que esta barra puede consultarse".**

Un consumidor que use la etiqueta de inicio como momento de disponibilidad está permitiendo que una decisión tomada al principio de la hora vea el precio de cierre de esa misma hora. Es el clásico sesgo de anticipación, y es especialmente traicionero porque:

- No produce ningún error ni ninguna advertencia.
- Los resultados salen **mejores**, no peores — que es exactamente la dirección en la que nadie sospecha.
- Confundir "etiqueta de la barra" con "momento de disponibilidad de la barra" es un error que un generalista no asocia; hay que conocer el dominio para verlo.

## Qué se pide (alcance corregido)

1. **Atar la regla de ADR-0114 al campo concreto**, mediante enmienda a ADR-0114 o a ADR-0155: "el cierre de la barra N" es exactamente `bar_close_timestamp_ns`, y "la apertura de N+1" es el `open` de la barra cuyo `bar_timestamp_ns` es igual a ese valor. Sin esa equivalencia escrita, un implementador puede cumplir el ADR al pie de la letra y aun así leer el campo equivocado.
2. **Nombrar dónde se hace cumplir**: qué es responsabilidad del validador PIT, qué del motor de backtest, y qué de la capa de features.
3. **Dejar planteado cómo se comprueba**, no solo cómo se declara. Un contrato de este tipo que solo vive en prosa se viola en la primera feature que tenga prisa.

## Por qué sigue mereciendo una Task, aunque la regla exista

Porque el punto de fallo se ha desplazado, no desaparecido. La regla de negocio está sellada y es FIJA; lo que falta es la **traducción a campos concretos**, y esa costura es donde se cuela el error: alguien lee "al cierre de la barra N", ve un campo llamado `bar_timestamp_ns` —que suena a "el timestamp de la barra"— y lo usa. El backtest sale **mejor**, no peor, que es la dirección en la que nadie sospecha.
