# Formatos de Instrucción — Catálogo Estándar para Knowledge/Skills

Todo documento consumido por agents o skills combina formatos de bloque — hoy Lecturas Obligatorias, Regla, Filtro de Verificación, Ejemplos — catalogados aquí para ser encontrables por ID y aplicables sin ambigüedad. Ninguno es la excepción del otro: cada uno resuelve un tipo de contenido distinto, y un documento normal usa varios. El catálogo es abierto — ver "Cómo Agregar un Formato Nuevo" para extenderlo.

Todo formato con ID usa el mismo identificador legible y único en todo el proyecto: `<DOC>-N` o `<DOC>-N.N` (ej. `EJEMPLO-2.2`). `<DOC>` es un slug en mayúsculas **que es el nombre exacto del documento que lo contiene** (`BASE` para `base.md`, `MEMORY_POLICY` para `memory-policy.md`, etc.) y no cambia después.

---

## Catálogo de Formatos

Referencia rápida — elige por lo que el contenido necesita, no por costumbre.

| Formato | Cuándo usarlo | Posición típica | Lleva | No lleva |
|---|---|---|---|---|
| Lecturas Obligatorias | El documento debe declarar qué otros documentos leer, y cuándo. | `<DOC>-0`, al inicio. | Tabla de 4 columnas. | `Aplica a`, `Precedencia`, `Filtro de verificación`. |
| Regla — Completo | La instrucción compite con otra, tiene doc satélite, plantilla de salida, o 2+ definiciones citables por sub-ID. | `<DOC>-N` o `<DOC>-N.N`, en el cuerpo. | Descripción, `Aplica a`, definiciones con sub-ID; campos de cierre solo si aplican. | Campos de cierre sin contenido real (se omiten, nunca `N/A`). |
| Regla — Ligero | Una sola idea, sin conflicto, sin doc satélite, sin plantilla. | `<DOC>-N`, en el cuerpo. | Una línea: etiqueta + cuerpo. | Descripción rápida, `Aplica a` propio, campos de cierre. |
| Filtro de Verificación | El documento necesita un chequeo de pre-salida que consolide sus reglas. | `<DOC>-N`, penúltima sección (antes de Ejemplos si los hay). | Lista numerada de preguntas, cada una citando la regla que verifica. | `Precedencia`, `Documento extendido`, `Plantilla de referencia`. |
| Ejemplos | Mostrar 2+ reglas del documento actuando juntas sobre un texto real. | `<DOC>-N`, cierre del documento. | Intro + casos con `### <DOC>-N.K`, antes/después, medición real si existe. | `Aplica a`, `Precedencia`, `Filtro de verificación`. |

### Cómo Agregar un Formato Nuevo

Esta tabla no es una lista cerrada — crece cuando aparece un tipo de contenido recurrente que ningún formato existente cubre bien.

* **Cuándo:** un mismo "tipo de bloque" se repite en 2+ documentos con una forma propia (campos, posición, lo que lleva/no lleva) que forzarlo dentro de Regla, Lecturas Obligatorias, Filtro de Verificación o Ejemplos lo distorsiona.
* **Cómo:** agrega una fila a "Catálogo de Formatos" (Formato / Cuándo usarlo / Posición típica / Lleva / No lleva) y su propia sección `## Formato: <Nombre>` con la misma estructura que las existentes — ID y posición, campos que lleva, campos que no lleva, formato del cuerpo, un ejemplo.
* **Qué no cambia:** el formato nuevo sigue las reglas transversales de este documento — "Niveles de ID", "IDs para Sub-Ítems" y "Espaciado y Saltos de Línea" — no inventa su propia convención de ID o espaciado.

---

## Niveles de ID

Un ID con punto implica un padre — nunca lo escribas si ese padre no existe como encabezado real en el documento.

* **`<DOC>-N` (plano):** documento sin secciones H1 numeradas propias — la mayoría de los knowledge docs son pequeños. Cada bloque es de primer nivel.
* **`<DOC>-N.N` (anidado):** solo si existe una sección H1 numerada real (`# <DOC> N. <Título de Sección>`) que agrupa varias reglas, como en documentos de skill mayores. `N` es esa sección, `.N` la posición del bloque dentro de ella.
* **Posiciones fijas:** Lecturas Obligatorias siempre es `<DOC>-0`. El resto de los formatos toma el siguiente número disponible según su posición real en el documento.

---

## IDs para Sub-Ítems (Listas Dentro de un Bloque)

Cualquier campo cuyo nombre termine en `_1..N` (definiciones de una Regla, preguntas de un Filtro de Verificación, pasos de una plantilla enumerada) puede tener más de un ítem, y cada ítem individual lleva **su propio ID** — nunca una referencia por posición. "Ítem 3" o "pregunta 2" no son texto único: no aparecen búscables en el documento, y si alguien inserta un ítem nuevo en el medio, la referencia queda apuntando a otro sin que nadie lo note.

**Formato:** `<DOC>-N.N.k — <Mnemónico corto>`, escrito literalmente en el propio ítem de la lista, para que `grep "<DOC>-N.N.k"` lo encuentre directo.

```
1. **<DOC>-N.N.1 — <Mnemónico corto 1>:** <texto del ítem 1>.
2. **<DOC>-N.N.2 — <Mnemónico corto 2>:** <texto del ítem 2>.
```

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

Tabla de qué otros documentos leer antes de operar bajo este, y cuándo — un índice de disparadores local al documento, no solo el índice central (`base.md`).

* **ID y posición:** `## <DOC>-0 — Lecturas Obligatorias`, siempre antes del primer bloque numerado, justo después del Slug ID del documento.
* **Campos que no lleva:** `Aplica a`, `Precedencia`, `Filtro de verificación` — es una tabla de datos, no una instrucción de comportamiento.
* **Formato del cuerpo:**
  ```
  | Documento | De qué va | Cuándo leerlo (disparador) | Alcance |
  |---|---|---|---|
  | `<ruta/al/doc.md>` | <1 línea: qué resuelve> | `SIEMPRE` o el disparador puntual | <a quién aplica> |
  ```
* **`SIEMPRE` es valor reservado:** solo para el propio documento madre (`base.md` u otro con supremacía declarada). Cualquier otra fila lleva un disparador concreto.
* **No duplica `Documento extendido`:** ese campo de una Regla apunta al detalle de UNA regla puntual; esta tabla es el índice de lectura de TODO el documento.

Ejemplo:

```
## PROMPT_ENGINEER-0 — Lecturas Obligatorias

| Documento | De qué va | Cuándo leerlo (disparador) | Alcance |
|---|---|---|---|
| `.agents/knowledge/base.md` | Gobernanza meta, supremacía absoluta. | `SIEMPRE` al iniciar cualquier skill. | Todos los agentes |
| `.agents/knowledge/WRITING-EFFICIENCY.MD` | Economía de redacción, formato de bloques. | `SIEMPRE` — es la autoridad principal de este skill. | Prompt Engineer |
| `.agents/templates/INSTRUCTION-FORMATS.md` | Catálogo de formatos de bloque. | Al canonizar o reescribir un bloque de knowledge/skill. | Prompt Engineer |
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
| `ID` + `ENCABEZADO_REGLA` | Siempre | `## <DOC>-N.N — <Título>`. Sin paréntesis, sin dos puntos, sin subtítulo pegado. |
| `DESCRIPCION_RAPIDA` | Siempre | 1-2 líneas de contexto — qué problema resuelve — **antes** de listar las definiciones. |
| `APLICA_A` | Siempre | A qué rol, modo o agente aplica. Si no hay restricción, dice explícitamente "Todos los roles/skills". |
| `DEFINICIONES_ESPECIFICAS_1..N` | Siempre | Las reglas concretas, con sub-ID propio (`<DOC>-N.N.1`, `<DOC>-N.N.2`...) — ver "IDs para Sub-Ítems". |
| `EJEMPLO_MAL_1..N` / `EJEMPLO_BIEN_1..N` | Solo si el error es real y no obvio | Se omite si la definición ya es autoexplicativa o el ejemplo saldría forzado — no es relleno obligatorio. |
| `PRECEDENCIA` | Solo si compite con otra regla | **Si no compite con ninguna, el campo entero se omite** — la ausencia en cualquier regla del proyecto significa "no compite" (convención declarada aquí, ver Regla de Oro). |
| `FILTRO_DE_VERIFICACION` | Solo si hay un chequeo dedicado | Cita el **ID exacto del ítem** en la sección Filtro de Verificación del documento. Se omite si no hay uno dedicado. |
| `DOCUMENTO_EXTENDIDO` | Solo si existe un archivo satélite real | Se omite si no existe — no se escribe `N/A`. |
| `PLANTILLA_EJEMPLO_COMPLETA` | Solo si hay una plantilla de salida copiable | Se omite si la regla no define una salida estructurada. Etiqueta y bloque de código en líneas consecutivas. |

Cada campo de cierre (`Precedencia`, `Filtro de verificación`, `Documento extendido`, `Plantilla de referencia`) aparece **solo si tiene contenido real** — se omite entero, no se escribe `N/A`.

`````
## <DOC>-N.N — <Título de la regla>

<Descripción: 1-2 líneas de contexto/problema, en prosa directa>

**Aplica a:** <rol/modo/agente, o "Todos los roles/skills">

* **<DOC>-N.N.1 — <Nombre de la definición 1>:** 

  <texto de la regla, indentado 2 espacios bajo la etiqueta>
  * ❌ *Incorrecto:* "<ejemplo malo, solo si el error no es obvio>"
  * ✅ *Correcto:* "<ejemplo bueno>"

* **<DOC>-N.N.2 — <Nombre de la definición 2>:** 

  <texto de la regla, indentado 2 espacios bajo la etiqueta>

**Precedencia:** <solo si choca con otra regla — se omite el campo si no compite con ninguna>

**Filtro de verificación:** <solo si hay un chequeo dedicado — se omite si no lo hay>

**Documento extendido:** <solo si existe un archivo satélite real — se omite si no existe>

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

**Filtro de verificación:** `EJEMPLO-6.1`.
```

La misma idea en formato ligero, si ninguna de las dos definiciones necesitara sub-ID citable por otra regla:

```
## EJEMPLO-1 — Regla de Muestra

* **EJEMPLO-1.1 — Primer Ítem:** texto de la definición en una línea.
* **EJEMPLO-1.2 — Segundo Ítem:** segunda definición en una línea.
```

---

## Formato: Filtro de Verificación

El chequeo de pre-salida que consolida las reglas de un documento en una sola lista — el punto que cada regla individual cita en su propio campo `Filtro de verificación`.

* **ID y posición:** `## <DOC>-N — Filtro de Verificación`, casi siempre la penúltima sección del documento (antes de Ejemplos, si los hay).
* **Campos que no lleva:** `Precedencia`, `Documento extendido`, `Plantilla de referencia` — no compite con nada y no tiene salida estructurada propia. Sí lleva `Aplica a` (normalmente "todo texto/respuesta que el agente produce, como último paso").
* **Formato del cuerpo:** una pregunta por regla relevante, numerada con el ID del ítem, citando entre paréntesis qué regla verifica:
  ```
  * **<DOC>-N.1:** ¿<pregunta de auto-chequeo>? (`<DOC>-M`)
  * **<DOC>-N.2:** ¿<pregunta de auto-chequeo>? (`<DOC>-M`)
  ```
* **Cobertura 1 a 1:** toda regla del documento cuyo campo `Filtro de verificación` cite un ítem de esta sección debe tener su pregunta correspondiente aquí — nunca al revés (una pregunta sin regla que la cite es ruido).

---

## Formato: Ejemplos

Casos compuestos: el mismo texto real transformado aplicando 2+ reglas del documento a la vez — no reemplazan los ❌/✅ de cada regla (esos son atómicos, uno por definición).

* **ID y contenido:** `## <DOC>-N — Ejemplos` para la sección, `### <DOC>-N.K` por cada sub-bloque de casos, con título pegado al sub-ID igual que una regla normal.
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
- [ ] Si incluiste `Documento extendido`, ¿el archivo satélite existe de verdad?
- [ ] Si incluiste una plantilla de ejemplo, ¿es copiable tal cual, sin placeholders sueltos?
- [ ] ¿Respeté el espaciado exacto (etiqueta y cuerpo en líneas separadas, línea en blanco entre ítems, `---` entre bloques)?
- [ ] Si es Filtro de Verificación: ¿cada pregunta corresponde a una regla real que la cita?
- [ ] Si es Ejemplos: ¿cada caso cita los IDs de regla que demuestra, y la medición (si existe) es real, no estimada?

---

## Documentos Relacionados

* `.agents/templates/KNOWLEDGE.md` — cómo ensamblar varios bloques de este catálogo en un documento de knowledge completo (Lecturas Obligatorias al inicio, Reglas en el cuerpo, Filtro de Verificación y Ejemplos al cierre).
