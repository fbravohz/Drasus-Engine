# Plantilla de Regla — Formato Estándar para Knowledge/Skills

Cómo escribir UNA regla dentro de cualquier documento consumido por agents o skills, para que sea encontrable por ID y aplicable sin ambigüedad de formato.

Esta plantilla reemplaza el símbolo `§` (jerga tipográfica densa) por un identificador legible y único en todo el proyecto: `<DOC>-N.N` (ej. `EJEMPLO-2.2`). `<DOC>` es un slug en mayúsculas **que es el nombre exacto del documento** (`BASE` para `base.md`, `MEMORY_POLICY` para `memory-policy.md`, etc.) y no cambia después. `N.N` es el número de sección tal como ya aparece en el documento.

---

## Niveles de ID: Cuándo Usar `<DOC>-N` vs `<DOC>-N.N`

Un ID con punto implica un padre — nunca lo escribas si ese padre no existe como encabezado real en el documento.

* **`<DOC>-N` (plano):** úsalo cuando el documento NO tiene secciones H1 numeradas propias — la mayoría de los knowledge docs son pequeños. Cada regla es de primer nivel; no hay tema común del que colgarla.
* **`<DOC>-N.N` (anidado):** úsalo SOLO si existe una sección H1 numerada real (`# <DOC> N. <Título de Sección>`) que agrupa varias reglas en documentos mayores, como los de skills. `N` es esa sección, `.N` es la posición de la regla dentro de ella.
* **Sub-ítems:** siempre un nivel más que el ID de la regla misma — `<DOC>-N.k` si la regla es plana, `<DOC>-N.N.k` si la regla ya está anidada en una sección.

---

## Campos de la Plantilla

| Campo | Obligatorio | Qué va ahí |
|---|---|---|
| `ID` + `ENCABEZADO_REGLA` | Siempre | `## <DOC>-N.N — <Título>`. El título es lo que es: sin paréntesis, sin dos puntos, sin subtítulo pegado. |
| `DESCRIPCION_RAPIDA` | Siempre | 1-2 líneas de contexto — qué problema resuelve esta regla — **antes** de listar las definiciones. Prosa directa, sin etiqueta `**Descripción:**` pegada al frente. |
| `APLICA_A` | Siempre | A qué rol, modo o agente aplica. Si no hay restricción, dice explícitamente "Todos los roles/skills" — nunca se omite en silencio. |
| `DEFINICIONES_ESPECIFICAS_1..N` | Siempre | Las reglas concretas, en viñetas, con el nombre de cada una en negrita. **Cada definición lleva su propio sub-ID** (`<DOC>-N.N.1`, `<DOC>-N.N.2`...) — ver "IDs para Sub-Ítems" abajo. |
| `EJEMPLO_MAL_1..N` / `EJEMPLO_BIEN_1..N` | Siempre, un par por cada definición específica o N/A si no aplica | Ejemplo ❌ incorrecto y ✅ correcto de esa definición puntual. No lleva ID propio: va pegado 1-a-1 al sub-ID de su definición. |
| `PRECEDENCIA` | Siempre (valor explícito) | Qué regla gana si esta choca con otra. Si no compite con ninguna, el valor es literalmente `N/A — no compite con otra regla` (nunca se omite el campo). |
| `FILTRO_DE_VERIFICACION` | Siempre (valor explícito) | En qué chequeo de pre-salida se verifica esto antes de responder — cita el **ID exacto del sub-ítem**, nunca su posición ("guarda 1"). Si no hay uno dedicado, se dice explícitamente que no lo hay. |
| `DOCUMENTO_EXTENDIDO` | Siempre (valor explícito, `N/A.` si no aplica) | Link al archivo satélite donde vive el detalle completo. Mismo principio que `PRECEDENCIA`/`FILTRO_DE_VERIFICACION`: la ausencia se declara, nunca se omite el campo en silencio. |
| `PLANTILLA_EJEMPLO_COMPLETA` | Siempre (valor explícito, `N/A.` si no aplica) | Bloque de código con la plantilla literal, copiable tal cual. Si aplica, la etiqueta y el bloque de código van en líneas consecutivas, sin línea en blanco entre ellos. |

---

## IDs para Sub-Ítems (Listas Dentro de una Regla)

**Regla general, no solo para chequeos:** cualquier campo cuyo nombre termine en `_1..N` (`DEFINICIONES_ESPECIFICAS_1..N`, los pasos de un `PLANTILLA_EJEMPLO_COMPLETA` si son enumerados, cualquier lista de chequeos dentro de una regla) puede tener más de un ítem, y cada ítem individual lleva **su propio ID** — nunca una referencia por posición. "Ítem 3" o "definición 2" no son texto único: no aparecen búscables en el documento, y si alguien inserta un ítem nuevo en el medio, la referencia queda apuntando a otro ítem sin que nadie lo note.

**Formato:** `<DOC>-N.N.k — <Mnemónico corto>`, escrito literalmente en el propio ítem de la lista, para que el buscador de VS Code o `grep "<DOC>-N.N.k"` lo encuentre directo, sin tener que abrir el archivo y contar.

Ejemplo genérico de una lista de 3 ítems dentro de una regla `<DOC>-N.N`:

```
1. **<DOC>-N.N.1 — <Mnemónico corto 1>:** <texto del ítem 1>.
2. **<DOC>-N.N.2 — <Mnemónico corto 2>:** <texto del ítem 2>.
3. **<DOC>-N.N.3 — <Mnemónico corto 3>:** <texto del ítem 3>.
```

Cualquier `FILTRO_DE_VERIFICACION` que apunte a uno de estos ítems cita el ID exacto (`<DOC>-N.N.k`), nunca la posición ("ítem 1"). Lo mismo aplica dentro de `DEFINICIONES_ESPECIFICAS_1..N`: cada definición es `<DOC>-N.N.1`, `<DOC>-N.N.2`, etc. — ver el ejemplo aplicado más abajo.

---

## Plantilla en Bruto

`````
## <DOC>-N.N — <Título de la regla>

<Descripción: 1-2 líneas de contexto/problema, en prosa directa>

**Aplica a:** <rol/modo/agente, o "Todos los roles/skills">

* **<DOC>-N.N.1 — <Nombre de la definición 1>:** 

  <texto de la regla, indentado 2 espacios bajo la etiqueta>
  * ❌ *Incorrecto:* "<ejemplo malo>"
  * ✅ *Correcto:* "<ejemplo bueno>"

* **<DOC>-N.N.2 — <Nombre de la definición 2>:** 

  <texto de la regla, indentado 2 espacios bajo la etiqueta>
  * ❌ *Incorrecto:* "<ejemplo malo>"
  * ✅ *Correcto:* "<ejemplo bueno>"

**Precedencia:** <qué gana si choca con otra regla, o "N/A — no compite con otra regla">

**Filtro de verificación:** <ID exacto del chequeo, o "N/A">

**Documento extendido:** <link al doc satélite, o "N/A.">

**Plantilla de referencia:**
```
<bloque de código copiable tal cual>
```
`````

Si la regla no define una plantilla de salida, la última línea es `**Plantilla de referencia:** N/A.` — un solo renglón, sin bloque de código.

---

## Espaciado y Saltos de Línea

Reglas de formato exactas, no estéticas — un agente que las rompe genera un documento inconsistente con el resto del proyecto.

* **Etiqueta y cuerpo en líneas separadas:** cada definición es `* **<ID> — <Título>:**` en su propia línea, una línea en blanco, y el cuerpo indentado 2 espacios en la línea siguiente — nunca etiqueta y cuerpo pegados en la misma línea.
* **Una línea en blanco entre ítems:** cada definición nueva lleva una línea en blanco antes, nunca pegada a la anterior.
* **Un `---` entre reglas:** cada regla nueva dentro del mismo documento se separa de la anterior con una línea `---` sola, con una línea en blanco antes y después.
* **Campos de cierre, cada uno en su propio párrafo:** `**Precedencia:**`, `**Filtro de verificación:**` y `**Documento extendido:**` van cada uno en su propia línea, separados por una línea en blanco entre sí — nunca todos pegados en un solo bloque.
* **`Plantilla de referencia` sin línea en blanco antes del bloque de código:** la etiqueta y el ` ``` ` que abre el bloque van en líneas consecutivas.

---

## Ejemplo Aplicado (Genérico)

Mismo formato, con contenido sintético en vez de una regla real — para que el patrón de espaciado se copie sin arrastrar contenido de ningún documento concreto:

```
## EJEMPLO-1 — Regla de Muestra

Toda regla nueva sigue este formato para ser encontrable por ID y aplicable sin ambigüedad.

**Aplica a:** todos los roles/skills, al redactar una regla nueva.

* **EJEMPLO-1.1 — Primer Ítem:** 

  texto de la definición, indentado 2 espacios bajo la etiqueta.
  * ❌ *Incorrecto:* "ejemplo que viola la definición."
  * ✅ *Correcto:* "ejemplo que la cumple."

* **EJEMPLO-1.2 — Segundo Ítem:** 

  segunda definición — mismo patrón: etiqueta, línea en blanco, cuerpo indentado.
  * ❌ *Incorrecto:* "ejemplo que viola la definición."
  * ✅ *Correcto:* "ejemplo que la cumple."

**Precedencia:** N/A — no compite con otra regla.

**Filtro de verificación:** `EJEMPLO-6.1`.

**Documento extendido:** N/A.

**Plantilla de referencia:** N/A.
```

---

## Excepción: Sección de Ejemplos, No Regla

Un documento de knowledge puede cerrar con una sección de Ejemplos en vez de una regla (ver "Documentos Relacionados" al final). No sigue el resto de esta plantilla.

* **ID igual, contenido distinto:** mismo estilo de ID que una regla (`## <DOC>-N — Ejemplos` para la sección, `### <DOC>-N.K` por cada sub-bloque de casos), y también lleva título pegado al sub-ID, igual que una regla normal.
* **Campos que se omiten:** no lleva `Aplica a`, `Precedencia` ni `Filtro de verificación`. No hacen falta porque esta sección no define una regla nueva — solo muestra evidencia de reglas que ya existen en el mismo documento.
* **Formato del cuerpo:** un párrafo de intro (qué son los casos compuestos, qué NO reemplazan) y luego, por cada caso:
  ```
  **Caso: <qué texto se transformó> — demuestra `<DOC>-N.N`, `<DOC>-N.N` [...].**
  * **Antes:** "<texto sin aplicar las reglas>"
  * **Después:** "<mismo texto, con las reglas aplicadas>"
  ```
  Si hay medición real (tokens, palabras, tiempo), se agrega una tabla debajo del caso — nunca una cifra estimada.

---

## Regla de Oro

**Un campo sin valor explícito es indistinguible de un olvido.** Si una regla no compite con nada, no tiene gate dedicado, o no tiene doc extendido, se dice explícitamente  — nunca se deja el campo vacío sin comentario.

## Checklist Antes de Guardar una Regla Nueva

- [ ] ¿Tiene ID único `<DOC>-N.N` y título sin paréntesis ni dos puntos?
- [ ] ¿La descripción rápida va ANTES de las definiciones (Contexto Antes de Detalle)?
- [ ] ¿`APLICA_A` tiene valor explícito, aunque sea "todos"?
- [ ] ¿Cada definición específica tiene su par ❌/✅?
- [ ] ¿`PRECEDENCIA`, `FILTRO_DE_VERIFICACION` y `DOCUMENTO_EXTENDIDO` tienen valor explícito, nunca ausente?
- [ ] Si `DOCUMENTO_EXTENDIDO` no es `N/A.`, ¿el archivo satélite existe de verdad?
- [ ] ¿La plantilla de ejemplo completo (si aplica) es copiable tal cual, sin placeholders sueltos?
- [ ] ¿Respeté el espaciado exacto (etiqueta y cuerpo en líneas separadas, línea en blanco entre ítems, `---` entre reglas)?

---

## Documentos Relacionados

* `.agents/templates/KNOWLEDGE.md` — cómo ensamblar varias reglas (con este formato) en un documento de knowledge completo, y cómo cerrarlo con una sección de Ejemplos.
