# DEBT-039 · El rango declarado nunca se coteja contra lo que el archivo contiene

| Campo | Valor |
|---|---|
| **ID** | DEBT-039 |
| **Severidad** | 🔴 Alta |
| **Estado** | 🟠 **Parcialmente saldada (2026-07-31, STORY-073 Etapa 2)** — ver "Qué quedó cerrado y qué no" al final |
| **Abierta** | 2026-07-27 |
| **Origen** | Gate de QA de STORY-070 (foco 3), confirmado por el Tech-Lead contra el código |
| **Disparador de pago** | **La entrega del hito de la pantalla de importación** — antes de que un usuario real pueda sellar una declaración. No después. |

---

## El hecho

STORY-070 hizo que cada declaración de importación traiga su rango explícito (`declared_range_start_ns`/`declared_range_end_ns`). El sistema verifica que ese rango sea **internamente coherente** (fin posterior a inicio, vía `CHECK` de base de datos) y que **no anteceda la fecha de operación declarada del bróker**, cuando esa fecha existe.

**Lo que no verifica es que el rango se corresponda con el contenido real del archivo.** Ni siquiera de la forma más simple: comparar contra el mínimo y el máximo de las fechas efectivamente parseadas.

Un llamador puede declarar un tramo de un siglo sobre un archivo de un solo día, y el sistema lo acepta y lo sella.

## Por qué es Alta y no Media

Porque lo que se sella **es la evidencia legal del usuario**. La protección de responsabilidad de LDR-0008 descansa en que el usuario confirmó un mapeo sobre una vista previa de su propio dato ya transformado: lo declarado es suyo, y responde él. Una declaración cuyo rango no describe el archivo **no es una declaración imprecisa: es una declaración falsa firmada de buena fe**, y contamina exactamente el registro que existe para dirimir responsabilidades.

Añadido: la clave de idempotencia de reimportación es `(hash del archivo, inicio, fin)`. Un rango incorrecto no solo miente sobre el pasado — **rompe la deduplicación futura**, porque el mismo tramo declarado dos veces con rangos distintos produce dos filas donde debería producir un no-op.

## Por qué no es un defecto de STORY-070

No es un vacío silencioso. Está diferido de forma explícita y en tres lugares independientes: la propia Story (§4.1, "lo que NO entra"), el comentario de `migrations/0023` (que enuncia literalmente que dos declaraciones sobre el mismo archivo pueden traer rangos que no coinciden con su contenido), y la especificación de `pit-data-validator` TTR-005, marcada "canonizado, no implementado".

El alcance de STORY-070 fue el esquema y el camino de importación. Cotejar contenido contra declaración es trabajo del validador, que aún no existe en código.

## El riesgo concreto de calendario

La secuencia de construcción de ingesta pone el validador (`pit-data-validator`) en el paso 4, **después** de la pantalla de importación. Es decir: tal como está planificado hoy, **el usuario puede sellar declaraciones antes de que exista el mecanismo que las contrasta**.

Esa es la razón de que esta deuda tenga disparador propio en vez de esperar su turno en la secuencia. Las opciones al llegar al hito son dos, y hay que elegir una conscientemente:

1. **Un cotejo mínimo dentro del propio asistente** — mínimo y máximo de las fechas parseadas contra el rango declarado, avisando o rechazando. Barato; no sustituye al validador, pero cierra el caso grosero.
2. **Adelantar TTR-005** antes de que la pantalla sea usable.

Lo que no es opción es entregar la pantalla y dejar el hueco abierto sin decidir.

## Qué quedó cerrado y qué no (2026-07-31, STORY-073 Etapa 2)

**Cerrado:** `check_declared_range_against_parsed_content` (dominio puro) rechaza cuando **el contenido cae fuera del rango declarado** — mínimo de apertura anterior al inicio declarado, o máximo de cierre posterior al fin. Cableado en **ambas vías** (plantilla y manual). Es un chequeo estructural, sin umbral inventado.

**NO cerrado, y el ingeniero lo reportó en vez de dar la deuda por saldada:** el caso que motiva esta ficha —declarar 2007 cuando el tramo empieza en 2010— **no lo caza la contención**, porque el contenido está trivialmente contenido en un rango más ancho. Lo mismo con "un siglo sobre un archivo de un día".

**Por qué no se cerró con un umbral, y por qué eso fue correcto:** medir "cuánto más ancho es demasiado" exige un criterio de dominio con auditoría de especialista (`.agents/knowledge/critical-domain-reasoning.md`), no una elección de implementación. Inventar ese número en una etapa de código habría sido exactamente el fallo que ese documento existe para impedir.

**La salida, y no necesita umbral alguno.** Un tramo declarado es una afirmación de **procedencia**, no de presencia de dato: declarar "de 2010 a 2017 esto viene de IC Markets" es legítimo aunque el archivo tenga huecos dentro. Por eso ninguna regla automática puede decidir si un rango ancho es un error o una declaración deliberada — **pero el usuario sí puede, si se lo enseñas**. La primera y la última fecha que el archivo realmente contiene, mostradas **junto a** lo que el usuario acaba de declarar, en el paso de confirmación. Ahí la discrepancia de 2007 salta a la vista y la decide quien responde por ella.

Eso encaja con LDR-0008 sin forzar nada: lo ves antes de firmarlo. **Asignado a la Etapa 4 (interfaz) de STORY-073.** Lo que quede después de eso —detección estadística de rangos absurdos sin intervención humana— pertenece al Gate de Homogeneidad de `pit-data-validator` (TTR-005) y sigue siendo especificación.

## Relación con otras deudas

- [`DEBT-035`](./DEBT-035.md) y [`DEBT-036`](./DEBT-036.md) comparten disparador (el hito de la pantalla) — conviene evaluarlas juntas.
- Especificación que la salda: TTR-005 en [`pit-data-validator`](../features/pit-data-validator.md).
