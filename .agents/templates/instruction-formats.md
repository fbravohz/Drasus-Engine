# Formatos de Instrucción — Catálogo Estándar para Knowledge/Skills

Todo documento consumido por agents o skills combina formatos de bloque — hoy Lecturas Obligatorias, Regla, Filtro de Verificación, Ejemplos — catalogados aquí para ser encontrables por ID y aplicables sin ambigüedad. Ninguno es la excepción del otro: cada uno resuelve un tipo de contenido distinto, y un documento normal usa varios. El catálogo es abierto — ver "Cómo Agregar un Formato Nuevo" para extenderlo.

Todo formato con ID usa el mismo identificador legible y único en todo el proyecto: `<DOC>-N` o `<DOC>-N.N` (ej. `EJEMPLO-2.2`). `<DOC>` es un slug en mayúsculas **con guion medio, calcado del nombre de archivo** (nunca guion bajo — un archivo `memory-policy.md` usa slug `MEMORY-POLICY`, no `MEMORY_POLICY`, para que archivo y slug se lean igual) y no cambia después.

---

## Catálogo de Formatos

Referencia rápida — elige por lo que el contenido necesita, no por costumbre.

| Formato | Cuándo usarlo | Posición típica | Lleva | No lleva |
|---|---|---|---|---|
| Lecturas Obligatorias | El documento debe declarar qué otros documentos leer, y cuándo. | `<DOC>-0`, al inicio. | Tabla de 4 columnas. | `Aplica a`, `Precedencia`, `Filtro de verificación`. |
| Regla — Completo | La instrucción compite con otra, tiene doc satélite, plantilla de salida, o 2+ definiciones citables por sub-ID. | `<DOC>-N`, en el cuerpo. | Descripción, `Aplica a`, definiciones con sub-ID; campos de cierre solo si aplican. | Campos de cierre sin contenido real (se omiten, nunca `N/A`). |
| Regla — Ligero | Una sola idea, sin conflicto, sin doc satélite, sin plantilla. | `<DOC>-N`, en el cuerpo. | Una línea: etiqueta + cuerpo. | Descripción rápida, `Aplica a` propio, campos de cierre. |
| Filtro de Verificación | El documento necesita un chequeo de pre-salida que consolide sus reglas. | `<DOC>-FV-N`, namespace propio — segunda sección, justo después de Lecturas Obligatorias (o primera si no hay tabla). | Lista numerada de preguntas, cada una citando la regla que verifica. | `Precedencia`, `Documento extendido`, `Plantilla de referencia`. |
| Ejemplos | Mostrar 2+ reglas del documento actuando juntas sobre un texto real. | `<DOC>-EJ-N`, namespace propio — cierre del documento, después de la última Regla. | Intro + casos con `### <DOC>-EJ-K`, antes/después, medición real si existe. | `Aplica a`, `Precedencia`, `Filtro de verificación`. |

### Cómo Agregar un Formato Nuevo

Esta tabla no es una lista cerrada — crece cuando aparece un tipo de contenido recurrente que ningún formato existente cubre bien.

* **Cuándo:** un mismo "tipo de bloque" se repite en 2+ documentos con una forma propia (campos, posición, lo que lleva/no lleva) que forzarlo dentro de Regla, Lecturas Obligatorias, Filtro de Verificación o Ejemplos lo distorsiona.
* **Cómo:** agrega una fila a "Catálogo de Formatos" (Formato / Cuándo usarlo / Posición típica / Lleva / No lleva) y su propia sección `## Formato: <Nombre>` con la misma estructura que las existentes — ID y posición, campos que lleva, campos que no lleva, formato del cuerpo, un ejemplo.
* **Qué no cambia:** el formato nuevo sigue las reglas transversales de este documento — "Niveles de ID", "IDs para Sub-Ítems" y "Espaciado y Saltos de Línea" — no inventa su propia convención de ID o espaciado.

---

## Niveles de ID

**Tope duro: 2 segmentos, nunca 3.** `<DOC>-N` para un bloque de primer nivel (una Regla, un Filtro de Verificación, etc.), `<DOC>-N.M` para un ítem citable dentro de ese bloque (una definición, una pregunta). `<DOC>-N.M.K` está prohibido — si un ítem `.M` necesita a su vez su propia lista de sub-ítems citables, ese ítem se promueve a su propio bloque de primer nivel (`<DOC>-N` nuevo) y su lista pasa a ser `<DOC>-N.1`, `<DOC>-N.2`... Un ID con punto implica un padre — nunca lo escribas si ese padre no existe como bloque real en el documento.

* **`<DOC>-N`:** la unidad de primer nivel — un bloque `## <DOC>-N — <Título>`. Todo formato del catálogo (Regla, Filtro de Verificación, Ejemplos) vive en este nivel.
* **`<DOC>-N.M`:** un ítem citable dentro del bloque `<DOC>-N` — una definición de una Regla, una pregunta de un Filtro de Verificación. Nunca más profundo.
* **Agrupación visual no es nivel de ID:** si un documento grande organiza varios bloques bajo un título mayor (ej. "Identidad y Rol" en un SKILL.md extenso), ese título es lectura, no numeración — los bloques debajo siguen siendo `<DOC>-N` planos, en la misma secuencia.
* **Posiciones fijas:** Lecturas Obligatorias siempre es `<DOC>-0`. Filtro de Verificación usa namespace propio `<DOC>-FV-N` (ver su formato), no compite por número con las Reglas. Ejemplos usa namespace propio `<DOC>-EJ-N` (ver su formato), tampoco compite por número con las Reglas — por eso puede vivir físicamente al cierre del documento sin importar cuántas Reglas se agreguen después. El resto de las Reglas toma el siguiente número libre de su propia secuencia — ver "Estabilidad de IDs" abajo.

---

## Estabilidad de IDs (Append-Only)

**Un ID publicado es permanente — nunca se renumera, nunca se reutiliza**, igual que ya funcionan `ADR-XXXX` y `DEBT-XXX` en `docs/`. Insertar una regla nueva nunca desplaza a las que ya existen.

* **Regla nueva = siguiente número libre de su propia secuencia**, sin importar dónde "lógicamente" iría en la lectura. Si un bloque se divide en dos, la mitad que conserva el tema original se queda con el número; la mitad nueva toma el siguiente número libre del documento — nunca uno ya usado por otro bloque.
* **Retirar una regla no libera su número.** Si un bloque deja de aplicar, se borra y su número queda vacante para siempre — no se reasigna.
* **Excepción única: reescritura completa de un documento, explícitamente autorizada por el usuario en el turno actual** (no una edición incremental). Ahí sí se renumera todo de una sola vez, porque las referencias externas se corrigen en el mismo movimiento.
* **Nada de lenguaje de enmienda dentro de una regla.** Si una regla cambia, se edita directo — sin anotar "(antes decía X, ahora Y)" ni "enmienda del \<fecha\>" en su cuerpo. El historial vive en git, no en la prosa.

---

## IDs para Listas Dentro de un Bloque

Cualquier campo cuyo nombre termine en `_1..N` (definiciones de una Regla, preguntas de un Filtro de Verificación, pasos de una plantilla enumerada) puede tener más de un ítem, y cada ítem individual lleva **su propio ID** — nunca una referencia por posición. "Ítem 3" o "pregunta 2" no son texto único: no aparecen búscables en el documento, y si alguien inserta un ítem nuevo en el medio, la referencia queda apuntando a otro sin que nadie lo note.

**Formato:** `<DOC>-N.M — <Mnemónico corto>`, donde `N` es el bloque contenedor y `M` la posición del ítem — el mismo nivel `<DOC>-N.M` de "Niveles de ID", nunca un tercer segmento.

```
1. **<DOC>-N.1 — <Mnemónico corto 1>:** <texto del ítem 1>.
2. **<DOC>-N.2 — <Mnemónico corto 2>:** <texto del ítem 2>.
```

**Si uno de estos ítems necesita, a su vez, su propia lista de sub-ítems citables, no anides un tercer segmento — promuévelo.** El ítem completo pasa a ser su propio bloque `<DOC>-N` (siguiente número libre) y esa lista se convierte en sus `<DOC>-N.1`, `<DOC>-N.2`... Caso real: `GOVERNANCE-4.1` ("Auto-Validación Crítica") tenía 7 hijos `GOVERNANCE-4.1.1`–`4.1.7`; se promovió a su propia sección `GOVERNANCE-4`, con hijos planos `GOVERNANCE-4.1`–`4.7`.

Cualquier `Filtro de verificación` que apunte a uno de estos ítems cita el ID exacto, nunca la posición.

---

## Espaciado y Saltos de Línea

Reglas de formato exactas, no estéticas — un agente que las rompe genera un documento inconsistente con el resto del proyecto.

* **Etiqueta y cuerpo en líneas separadas (formato completo):** cada definición es `* **<ID> — <Título>:**` en su propia línea, una línea en blanco, y el cuerpo indentado 2 espacios en la línea siguiente.
* **Una línea en blanco entre ítems.**
* **Un `---` entre bloques:** cada formato nuevo dentro del mismo documento se separa del anterior con una línea `---` sola, con línea en blanco antes y después.
* **Campos de cierre, cada uno en su propio párrafo**, separados por línea en blanco entre sí.
* **`Plantilla de referencia` sin línea en blanco antes del bloque de código:** la etiqueta y el ` ``` ` que abre el bloque van en líneas consecutivas.

---

## Formato: Lecturas Obligatorias

Tabla de qué otros documentos leer antes de operar bajo este, y cuándo — un índice de disparadores local al documento, no solo el índice central (`governance.md`).

* **ID y posición:** `## <DOC>-0 — Lecturas Obligatorias`, siempre antes del primer bloque numerado, justo después del Slug ID del documento.
* **Campos que no lleva:** `Aplica a`, `Precedencia`, `Filtro de verificación` — es una tabla de datos, no una instrucción de comportamiento.
* **Formato del cuerpo:**
  ```
  | Documento | De qué va | Cuándo leerlo (disparador) | Alcance |
  |---|---|---|---|
  | [[<ruta/al/doc.md>]] | <1 línea: qué resuelve> | `SIEMPRE` o el disparador puntual | <a quién aplica> |
  ```
* **`SIEMPRE` es valor reservado:** solo para el propio documento madre (`governance.md` u otro con supremacía declarada). Cualquier otra fila lleva un disparador concreto.
* **No duplica `Documento extendido`:** ese campo de una Regla apunta al detalle de UNA regla puntual; esta tabla es el índice de lectura de TODO el documento.

Ejemplo:

```
## CONTEXT-ENGINEER-0 — Lecturas Obligatorias

| Documento | De qué va | Cuándo leerlo (disparador) | Alcance |
|---|---|---|---|
| [[.agents/knowledge/governance.md]] | Gobernanza meta, supremacía absoluta. | `SIEMPRE` al iniciar cualquier skill. | Todos los agentes |
| [[.agents/knowledge/writing-efficiency.md]] | Economía de redacción, formato de bloques. | `SIEMPRE` — es la autoridad principal de este skill. | Context Engineer |
| [[.agents/templates/instruction-formats.md]] | Catálogo de formatos de bloque. | Al canonizar o reescribir un bloque de knowledge/skill. | Context Engineer |
```

---

## Formato: Regla

La instrucción de comportamiento — qué debe o no debe hacer un agente. Existe en dos variantes; elige por lo que el contenido necesita, no por costumbre. Aplicar el formato completo a una regla que no lo necesita infla el documento sin agregar información (medido: convertir un doc de 94 a 339 líneas, 3.6x, con 27 campos en `N/A` sobre 40 totales).

* **Completo:** la regla tiene precedencia real frente a otra, documento satélite real, plantilla de salida copiable, o 2+ definiciones que otra regla va a necesitar citar por sub-ID.
* **Ligero:** la regla es una sola idea, sin conflicto con otra regla, sin documento satélite, sin plantilla de salida.

Un documento puede mezclar ambos: ligero para lo simple, completo para lo que de verdad compite o tiene plantilla.

### Regla — Completo

| Campo | Obligatorio | Qué va ahí |
|---|---|---|
| `ID` + `ENCABEZADO_REGLA` | Siempre | `## <DOC>-N — <Título>`. Sin paréntesis, sin dos puntos, sin subtítulo pegado. |
| `DESCRIPCION_RAPIDA` | Siempre | 1-2 líneas de contexto — qué problema resuelve — **antes** de listar las definiciones. |
| `APLICA_A` | Siempre | A qué rol, modo o agente aplica. Si no hay restricción, dice explícitamente "Todos los roles/skills". |
| `DEFINICIONES_ESPECIFICAS_1..N` | Siempre | Las reglas concretas, con sub-ID propio (`<DOC>-N.1`, `<DOC>-N.2`...) — ver "IDs para Listas Dentro de un Bloque". Si una definición necesita su propia lista, se promueve a bloque nuevo — nunca un tercer segmento. |
| `EJEMPLO_MAL_1..N` / `EJEMPLO_BIEN_1..N` | Solo si el error es real y no obvio | Se omite si la definición ya es autoexplicativa o el ejemplo saldría forzado — no es relleno obligatorio. |
| `PRECEDENCIA` | Solo si compite con otra regla | **Si no compite con ninguna, el campo entero se omite** — la ausencia en cualquier regla del proyecto significa "no compite" (convención declarada aquí, ver Regla de Oro). |
| `FILTRO_DE_VERIFICACION` | Solo si hay un chequeo dedicado | Cita el **ID exacto del ítem** en la sección Filtro de Verificación del documento. Se omite si no hay uno dedicado. |
| `DOCUMENTO_EXTENDIDO` | Solo si existe un archivo satélite real | Se omite si no existe — no se escribe `N/A`. |
| `LECTURA_BAJO_DEMANDA` | Solo si esta regla remite a otra regla, en otro documento, que aplica solo si un caso puntual lo dispara | No es más detalle de esta misma regla (eso es `Documento extendido`) — es otro tema, aparte, condicional. Puede repetirse si hay más de un caso. Se omite si no aplica. |
| `PLANTILLA_EJEMPLO_COMPLETA` | Solo si hay una plantilla de salida copiable | Se omite si la regla no define una salida estructurada. Etiqueta y bloque de código en líneas consecutivas. |

Cada campo de cierre (`Precedencia`, `Filtro de verificación`, `Documento extendido`, `Lectura bajo demanda`, `Plantilla de referencia`) aparece **solo si tiene contenido real** — se omite entero, no se escribe `N/A`.

**`Documento extendido` vs. `Lectura bajo demanda` — no son lo mismo, y confundirlos es el error que origina esta distinción:** `Documento extendido` es la versión larga de ESTA MISMA regla — el archivo satélite elabora lo que aquí se resume, hablan del mismo tema. `Lectura bajo demanda` apunta a OTRA regla, que vive en otro documento, y que solo aplica si un caso concreto la dispara — típicamente el resultado de partir un documento grande en varios más chicos por economía de tokens. Ejemplo real: una regla de "rigor de archivos" que antes cubría también dot-directories y lectura progresiva, y ahora esos dos casos viven aparte en `tool-usage.md` — eso es `Lectura bajo demanda`, no `Documento extendido`, porque `tool-usage.md` no elabora la regla de rigor, cubre un caso distinto.

**Toda referencia a otro archivo o a una regla ajena dentro de una Regla vive en un campo declarado o citada inline con su ID exacto — nunca una mención vaga sin ID ni ruta.** Los lugares válidos son: el ❌/✅ (si el archivo es parte del ejemplo de uso correcto/incorrecto), `Documento extendido`, `Lectura bajo demanda`, `Plantilla de referencia`, o directo en el cuerpo de la definición citando el `<OTRO-DOC>-N`/`<OTRO-DOC>-N.M` exacto entre backticks. Si necesitas mencionar un archivo o regla fuera de esos usos, es señal de que falta declarar el campo correcto o el ID exacto — no de que la prosa vaga lo resuelve.

**Referencias entre documentos pueden citar el ID interno de una regla ajena, no solo el archivo.** Los IDs son append-only y estables (ver "Estabilidad de IDs") — citar `<OTRO-DOC>-N`/`<OTRO-DOC>-N.M` directo ancla la referencia al ítem exacto en vez de obligar a releer el documento completo, y es preferible cuando la regla que citas es puntual. `Lectura bajo demanda` y `Documento extendido` siguen usando siempre una ruta (`<ruta/al/archivo.md>`) porque apuntan a un documento entero, no a un ítem — la cita de ID puntual va suelta en el cuerpo de la definición, entre backticks. Citar tu propio `<DOC>-N`/`<DOC>-N.M` dentro del mismo documento (Precedencia, Filtro de verificación, IDs para listas) sigue siendo normal y obligatorio.

**Toda referencia directa a otro documento se escribe como wikilink de Obsidian, nunca como ruta suelta en texto plano.** Formato: `[[ruta/relativa/al/archivo.md]]` — la misma ruta que ya usa el proyecto, con extensión (para que siga siendo grepeable fuera de Obsidian). Si la referencia apunta a una regla puntual, ancla el enlace al encabezado `## <DOC>-N` que la contiene: `[[ruta/al/archivo.md#<DOC>-N — <Título exacto del encabezado>]]`, copiando el título tal cual aparece — Obsidian resuelve el salto por coincidencia exacta de texto, no por el ID solo. Un sub-ID `<DOC>-N.M` no tiene encabezado propio (vive dentro del bloque `<DOC>-N`, ver "Niveles de ID"): enlaza al encabezado contenedor y deja el sub-ID exacto entre backticks junto al enlace, para ubicarlo dentro de esa sección al llegar. Si editas el texto de un `<Título>`, corrige todo wikilink `[[...#...]]` que citaba ese texto — exactamente el tipo de contradicción silenciosa que `CONTEXT-ENGINEER-6.1` existe para atrapar.

`````
## <DOC>-N — <Título de la regla>

<Descripción: 1-2 líneas de contexto/problema, en prosa directa>

**Aplica a:** <rol/modo/agente, o "Todos los roles/skills">

* **<DOC>-N.1 — <Nombre de la definición 1>:** 

  <texto de la regla, indentado 2 espacios bajo la etiqueta>
  * ❌ *Incorrecto:* "<ejemplo malo, solo si el error no es obvio>"
  * ✅ *Correcto:* "<ejemplo bueno>"

* **<DOC>-N.2 — <Nombre de la definición 2>:** 

  <texto de la regla, indentado 2 espacios bajo la etiqueta>

**Precedencia:** <solo si choca con otra regla — se omite el campo si no compite con ninguna>

**Filtro de verificación:** <solo si hay un chequeo dedicado — se omite si no lo hay; cita `<DOC>-FV-M`>

**Documento extendido:** <solo si existe un archivo satélite real que elabora ESTA MISMA regla — se omite si no existe>

**Lectura bajo demanda:** [[<ruta/al/archivo.md>]] — <disparador: qué caso puntual hace que esa OTRA regla aplique en vez de, o además de, esta. Se omite si no aplica; se repite la línea si hay más de un caso>

**Plantilla de referencia:**
```
<bloque de código copiable tal cual — la sección entera se omite si la regla no define una salida estructurada>
```
`````

### Regla — Ligero

Una línea basta — sin descripción rápida separada, sin `Aplica a` propio (hereda el de la sección o documento que la contiene), sin campos de cierre.

```
* **<DOC>-N — <Título>:** <cuerpo en una línea, sin campos de cierre>.
```

**Única excepción: `Lectura bajo demanda`.** Si la regla de una línea, en un caso puntual, remite a otra regla que vive en otro documento, se agrega como segunda línea — sigue siendo Ligero, no obliga a subir a Completo solo por esto.

```
* **<DOC>-N — <Título>:** <cuerpo en una línea, sin campos de cierre>.

  **Lectura bajo demanda:** [[<ruta/al/archivo.md>]] — <disparador puntual>.
```

### Ejemplo Aplicado (Regla, Genérico)

Mismo formato, con contenido sintético en vez de una regla real. Nota que `Precedencia` y `Documento extendido` no aparecen: esta regla no compite con nada y no tiene doc satélite, así que esos campos se omiten en vez de escribir `N/A`.

```
## EJEMPLO-1 — Regla de Muestra

Toda regla nueva sigue este formato para ser encontrable por ID y aplicable sin ambigüedad.

**Aplica a:** todos los roles/skills, al redactar una regla nueva.

* **EJEMPLO-1.1 — Primer Ítem:** 

  texto de la definición, indentado 2 espacios bajo la etiqueta.
  * ❌ *Incorrecto:* "ejemplo que viola la definición."
  * ✅ *Correcto:* "ejemplo que la cumple."

* **EJEMPLO-1.2 — Segundo Ítem:** 

  segunda definición — mismo patrón: etiqueta, línea en blanco, cuerpo indentado. Sin ❌/✅ porque el texto ya es autoexplicativo.

**Filtro de verificación:** `EJEMPLO-FV-1`.
```

La misma idea en formato ligero, si ninguna de las dos definiciones necesitara sub-ID citable por otra regla:

```
## EJEMPLO-1 — Regla de Muestra

* **EJEMPLO-1.1 — Primer Ítem:** texto de la definición en una línea.
* **EJEMPLO-1.2 — Segundo Ítem:** segunda definición en una línea.
```

---

## Formato: Filtro de Verificación

El chequeo de pre-salida que consolida las reglas de un documento en una sola lista — el punto que cada regla individual cita en su propio campo `Filtro de verificación`. Usa un namespace propio, separado de la secuencia numérica de las Reglas, para no competir nunca por posición ni por número con ellas.

* **ID y posición:** `## <DOC>-FV — Filtro de Verificación`, segunda sección del documento — justo después de Lecturas Obligatorias (`<DOC>-0`), o primera si el documento no tiene tabla de lecturas. Se puebla incrementalmente: cada regla nueva que se agregue más abajo suma su propio ítem aquí, sin límite — la sección nunca queda "al fondo estorbando" ni obliga a reordenar nada cuando el documento crece.
* **IDs internos:** `<DOC>-FV-1`, `<DOC>-FV-2`... — secuencia propia, también append-only (ver "Estabilidad de IDs").
* **Campos que no lleva:** `Precedencia`, `Documento extendido`, `Plantilla de referencia` — no compite con nada y no tiene salida estructurada propia. Sí lleva `Aplica a` (normalmente "todo texto/respuesta que el agente produce, como último paso").
* **Formato del cuerpo:** una pregunta por regla relevante, citando entre paréntesis qué regla verifica:
  ```
  * **<DOC>-FV-1:** ¿<pregunta de auto-chequeo>? (`<DOC>-N`)
  * **<DOC>-FV-2:** ¿<pregunta de auto-chequeo>? (`<DOC>-M`)
  ```
* **Cobertura 1 a 1:** toda regla del documento cuyo campo `Filtro de verificación` cite un ítem de esta sección debe tener su pregunta correspondiente aquí — nunca al revés (una pregunta sin regla que la cite es ruido).

* **Declaración explicita de verificación:** Se agrega esta linea siempre al final "De ser satisfactoria cada validacion de la lista, declara `<DOCUMENT-SLUG-ID> - FILTRO DE VERIFICACION APROBADO`"

---

## Formato: Ejemplos

Casos compuestos: el mismo texto real transformado aplicando 2+ reglas del documento a la vez — no reemplazan los ❌/✅ de cada regla (esos son atómicos, uno por definición). Usa namespace propio, separado de la secuencia numérica de las Reglas, para no competir nunca por posición ni por número con ellas — el mismo motivo que el Filtro de Verificación, y lo que le permite vivir siempre al cierre físico del documento aunque se agreguen Reglas nuevas después.

* **ID y posición:** `## <DOC>-EJ — Ejemplos` para la sección, física al cierre del documento — después de la última Regla, sin importar su número. Se puebla incrementalmente: cada caso nuevo suma su propio ítem aquí.
* **IDs internos:** `<DOC>-EJ-1`, `<DOC>-EJ-2`... — secuencia propia, también append-only (ver "Estabilidad de IDs"), con título pegado al sub-ID igual que un ítem de Filtro de Verificación.
* **Campos que no lleva:** `Aplica a`, `Precedencia`, `Filtro de verificación`. No define un bloque nuevo — solo muestra evidencia de reglas que ya existen en el mismo documento.
* **Formato del cuerpo:** un párrafo de intro (qué son los casos, qué NO reemplazan) y, por cada caso:
  ```
  **Caso: <qué texto se transformó> — demuestra `<DOC>-N.N`, `<DOC>-N.N` [...].**
  * **Antes:** "<texto sin aplicar las reglas>"
  * **Después:** "<mismo texto, con las reglas aplicadas>"
  ```
  Si hay medición real (tokens, palabras, tiempo), se agrega una tabla debajo del caso — nunca una cifra estimada.

---

## Regla de Oro

**Un campo de cierre ausente significa "no aplica" — esa es la convención de todo el proyecto, declarada una sola vez aquí, no repetida como `N/A` en cada regla.** Escribe `Precedencia`, `Filtro de verificación`, `Documento extendido` o `Plantilla de referencia` solo cuando tengas contenido real que poner ahí. Lo mismo aplica al par ❌/✅: solo cuando el error es real y no obvio. Esto no es licencia para omitir `Aplica a` o la descripción rápida en una Regla — Completo: esos dos siguen siendo siempre obligatorios.

## Checklist Antes de Guardar un Bloque Nuevo

- [ ] ¿Elegiste el formato correcto del catálogo (Lecturas Obligatorias / Regla Completo / Regla Ligero / Filtro de Verificación / Ejemplos) según lo que el contenido necesita, no por costumbre?
- [ ] ¿Tiene ID único y título sin paréntesis ni dos puntos?
- [ ] En Regla — Completo: ¿la descripción rápida va ANTES de las definiciones, y `Aplica a` tiene valor explícito?
- [ ] ¿Cada ❌/✅ que incluiste aporta claridad real, no relleno para llenar el campo?
- [ ] ¿Omitiste los campos de cierre que no aplican, en vez de escribir `N/A`?
- [ ] Si incluiste `Documento extendido`, ¿el archivo satélite existe de verdad y elabora ESTA MISMA regla (si en realidad es un tema distinto y condicional, va en `Lectura bajo demanda`, no aquí)?
- [ ] ¿Toda referencia a otro archivo vive en un campo declarado (❌/✅, `Documento extendido`, `Lectura bajo demanda`, `Plantilla de referencia`) y ninguna quedó suelta en la descripción o el cuerpo de una definición?
- [ ] ¿Toda referencia a otro documento — la ruta del archivo (`Lectura bajo demanda`/`Documento extendido`) o el ID interno de una regla ajena citado inline — es exacta y verificable, nunca una mención vaga?
- [ ] ¿Toda referencia directa a otro documento es un wikilink `[[ruta/archivo.md]]` (o `[[ruta/archivo.md#<DOC>-N — Título exacto>]]` si apunta a una regla puntual), nunca una ruta suelta en texto plano?
- [ ] ¿El ID tiene máximo 2 segmentos (`<DOC>-N` o `<DOC>-N.M`) — ningún `<DOC>-N.M.K`?
- [ ] Si editaste un documento existente, ¿respetaste que los IDs ya publicados son permanentes — la regla nueva toma el siguiente número libre, nunca uno ya usado, salvo reescritura completa autorizada en el turno actual?
- [ ] Si el documento tiene Filtro de Verificación, ¿vive en `<DOC>-FV-N` (namespace propio), como segunda sección tras Lecturas Obligatorias?
- [ ] Si el documento tiene una sección de Ejemplos, ¿vive en `<DOC>-EJ-N` (namespace propio), física al cierre del documento — después de la última Regla, sin competir por número con ellas?
- [ ] Si incluiste una plantilla de ejemplo, ¿es copiable tal cual, sin placeholders sueltos?
- [ ] ¿Respeté el espaciado exacto (etiqueta y cuerpo en líneas separadas, línea en blanco entre ítems, `---` entre bloques)?
- [ ] Si es Filtro de Verificación: ¿cada pregunta corresponde a una regla real que la cita?
- [ ] Si es Ejemplos: ¿cada caso cita los IDs de regla que demuestra, y la medición (si existe) es real, no estimada?

---

## Documentos Relacionados

* `.agents/templates/knowledge.md` — cómo ensamblar varios bloques de este catálogo en un documento de knowledge completo (Lecturas Obligatorias → Filtro de Verificación → Reglas sin límite, con Ejemplos al cierre si aplican).
