# Reporte de Sesión

Cómo redirigir a un archivo el texto que un agente escribiría en el chat, dejando en la terminal solo lo que exige interacción.

Slug ID: `SESSION-REPORTING`.

Tamaño: `13924 caracteres (~3481 tokens aprox., ratio 4:1)`.

---

## SESSION-REPORTING-0 — Lecturas Obligatorias

| Documento | De qué va | Cuándo leerlo (disparador) | Alcance |
|---|---|---|---|
| [[.agents/knowledge/governance.md]] | Gobernanza meta, supremacía absoluta. | `SIEMPRE` al iniciar cualquier skill. | Todos los agentes |
| [[.agents/knowledge/writing-efficiency.md]] | Economía de redacción, traducción de términos, formato visual de bloques. | `SIEMPRE` al escribir una entrada del reporte — el reporte es texto dirigido al usuario. | Todos los agentes |
| [[.agents/knowledge/write-like-gemini.md]] | El Modo Gemini Relajado + Listas, activo por defecto en toda entrada (`WRITING-EFFICIENCY-7`) — el hecho solo, desde la primera palabra. | `SIEMPRE` al redactar una entrada — es el estilo por defecto del reporte, no una pasada de limpieza posterior. | Todos los agentes |
| `.agents/templates/session-report.md` | Plantilla copiable del archivo de sesión y de cada entrada. | Al abrir un archivo de sesión nuevo o añadir una entrada. | Todos los agentes |

---

## SESSION-REPORTING-FV — Filtro de Verificación

Checklist antes de enviar cualquier respuesta bajo modo de reporte.

**Aplica a:** todos los agentes con el modo de reporte activo, como último paso antes de responder.

* **SESSION-REPORTING-FV-1:**

  ¿escribí la entrada completa en el archivo de sesión ANTES de responder en el chat, en vez de escribir en el chat y resumir después (`SESSION-REPORTING-1`)?

* **SESSION-REPORTING-FV-2:**

  ¿el chat quedó reducido al eco de tres líneas más lo que exige interacción, sin repetir el cuerpo del reporte (`SESSION-REPORTING-2`)?

* **SESSION-REPORTING-FV-3:**

  ¿la entrada trae sus secciones en el orden fijo, omitiendo enteras las que no tienen contenido real en vez de escribir `N/A` (`SESSION-REPORTING-3`)?

* **SESSION-REPORTING-FV-4:**

  ¿toda entrada anterior quedó intacta, y cualquier rectificación se escribió como entrada nueva (`SESSION-REPORTING-4`)?

* **SESSION-REPORTING-FV-5:**

  ¿cada archivo que toqué tiene su fila en la tabla, con qué cambió y por qué, de modo que el usuario no necesite abrirlo (`SESSION-REPORTING-5`)?

* **SESSION-REPORTING-FV-6:**

  si despaché subagentes, ¿cada uno escribió su propia sección con su nombre, sin que yo reescribiera su contenido (`SESSION-REPORTING-6`)?

* **SESSION-REPORTING-FV-7:**

  ¿escribí la entrada en estilo Gemini desde la primera palabra, en vez de redactarla larga y comprimirla después (`SESSION-REPORTING-8`)?

De ser satisfactoria cada validacion de la lista, declara `SESSION-REPORTING - FILTRO DE VERIFICACION APROBADO`

---

## SESSION-REPORTING-1 — Qué Es el Reporte y Dónde Vive

El reporte de sesión es la salida del chat redirigida a un archivo: exactamente el texto que el agente escribiría en la terminal, escrito en disco. No es documentación, no es bitácora y no es orden de trabajo.

**Aplica a:** todos los roles/skills/agentes, cuando el modo de reporte está activo.

* **SESSION-REPORTING-1.1 — La Duplicación Es Esperada, No Un Defecto:**

  el reporte repite libremente lo que ya vive en una orden de trabajo, una bitácora o una ficha de deuda. Es una conversación escrita, y una conversación repite lo que hace falta para entenderse. Ningún agente recorta contenido del reporte alegando que "ya está en otro documento".
  * ❌ *Incorrecto:* escribir "el detalle está en la orden de trabajo" en vez del detalle.
  * ✅ *Correcto:* escribir el detalle completo en el reporte y, si aplica, enlazar además el documento donde también vive.

* **SESSION-REPORTING-1.2 — No Sustituye Ningún Registro de Dominio:**

  abrir el reporte no exime de escribir la orden de trabajo, la bitácora, la ficha de deuda ni la memoria. Esos siguen siendo obligatorios y se escriben igual; el reporte narra que se escribieron.

* **SESSION-REPORTING-1.3 — Ubicación y Nombre:**

  un archivo por sesión en `.agents/sessions/`, nombrado `YYYY-MM-DD-<slug>.md`, donde el slug describe el tema de la sesión en kebab-case. Dos sesiones el mismo día llevan slugs distintos — nunca un sufijo numérico.

* **SESSION-REPORTING-1.4 — Escribir Antes de Responder:**

  la entrada se escribe completa en el archivo y solo después se responde en el chat. El orden inverso —redactar en el chat y resumir al archivo— produce dos textos distintos y devuelve el ruido que este modo elimina.

**Filtro de verificación:** `SESSION-REPORTING-FV-1`.

**Plantilla de referencia:**
```
.agents/sessions/2026-08-02-reporte-de-sesion.md
```

---

## SESSION-REPORTING-2 — Qué Queda en el Chat

El chat conserva solo lo que exige que el usuario haga algo, más un eco mínimo que le permita saber si debe abrir el reporte.

**Aplica a:** todos los roles/skills/agentes, cuando el modo de reporte está activo.

* **SESSION-REPORTING-2.1 — Eco de Tres Líneas:**

  toda respuesta deja en el chat: el estado en una palabra, qué pasó en una frase, y la ruta del reporte. Nada más de la narración.
  * ❌ *Incorrecto:* pegar en el chat el resumen del trabajo además de escribirlo en el reporte.
  * ✅ *Correcto:* `🟢 Listo — corregidos los tres defectos de la zona de carga. → .agents/sessions/2026-08-02-<slug>.md`

* **SESSION-REPORTING-2.2 — Lo Que Sigue Siendo del Chat:**

  preguntas al usuario y sus respuestas; menús de selección múltiple y cualquier elemento nativo de la terminal; bloqueos que impiden continuar; y la respuesta directa a una pregunta puntual del usuario, que se contesta en el chat y no se desvía al archivo.

* **SESSION-REPORTING-2.3 — Un Bloqueo Nunca Se Esconde en el Archivo:**

  si el trabajo se detuvo, el eco lo dice con el estado en rojo y una frase que nombre la causa. El usuario no debe abrir un archivo para enterarse de que algo está parado.

**Precedencia:** gana sobre `SESSION-REPORTING-1.4` cuando ambas compiten — una pregunta directa del usuario se responde en el chat de inmediato, sin esperar a que la entrada esté escrita.

**Filtro de verificación:** `SESSION-REPORTING-FV-2`.

---

## SESSION-REPORTING-3 — Estructura de una Entrada

Cada turno del usuario produce exactamente una entrada, con secciones en orden fijo para que el usuario sepa siempre dónde mirar.

**Aplica a:** todos los roles/skills/agentes, al añadir una entrada al reporte.

* **SESSION-REPORTING-3.1 — Orden Fijo de Secciones:**

  qué pediste · qué hice · archivos tocados · comprobado · decisiones y por qué · pendientes y bloqueos · preguntas para ti · otros. El orden no cambia entre entradas ni entre agentes.

* **SESSION-REPORTING-3.2 — Sección Vacía Se Omite Entera:**

  una sección sin contenido real se borra de la entrada; nunca se escribe `N/A`, "ninguno" ni un guion. Es la misma convención de campos de cierre que rige los documentos de knowledge.

* **SESSION-REPORTING-3.3 — Las Preguntas Van Duplicadas a Propósito:**

  toda pregunta escrita en la sección correspondiente se repite en el chat. Es la única sección que existe en los dos sitios, porque una pregunta que solo vive en un archivo no obtiene respuesta.

* **SESSION-REPORTING-3.4 — Numeración y Encabezado:**

  cada entrada abre con `## Turno N — <hora> — <título corto>`, con `N` correlativo dentro del archivo.

**Filtro de verificación:** `SESSION-REPORTING-FV-3`.

**Documento extendido:** `.agents/templates/session-report.md`.

---

## SESSION-REPORTING-4 — Append-Only

El reporte solo crece: una entrada publicada no se edita ni se borra, igual que una conversación no se reescribe hacia atrás.

**Aplica a:** todos los roles/skills/agentes, al volver sobre trabajo ya reportado.

* **SESSION-REPORTING-4.1 — Rectificar Es Añadir:**

  si un dato reportado resultó falso, se escribe una entrada nueva que lo corrige y nombra cuál corrige. La entrada equivocada se queda donde está.
  * ❌ *Incorrecto:* editar el turno 3 para arreglar un conteo que salió mal.
  * ✅ *Correcto:* escribir en el turno 7 "el conteo del turno 3 estaba mal: eran 30, no 22" y seguir.

* **SESSION-REPORTING-4.2 — Excepción Única:**

  completar una entrada en curso antes de responder ese mismo turno. Una vez respondido el turno, la entrada queda cerrada.

**Filtro de verificación:** `SESSION-REPORTING-FV-4`.

---

## SESSION-REPORTING-5 — Tabla de Archivos Tocados

El usuario debe poder saber qué cambió en cada archivo sin abrir ninguno. Esa tabla es el corazón del reporte.

**Aplica a:** todos los roles/skills/agentes, en toda entrada que modifique archivos.

* **SESSION-REPORTING-5.1 — Una Fila Por Archivo:**

  ruta, qué cambió, y por qué — cada campo en una línea legible, sin volcados de código ni diferencias línea a línea.

* **SESSION-REPORTING-5.2 — El "Por Qué" Es Obligatorio:**

  una fila que solo dice qué cambió obliga a abrir el archivo para entenderlo, que es justo lo que la tabla existe para evitar.
  * ❌ *Incorrecto:* `dropzone.dart · ancho fijo`
  * ✅ *Correcto:* `dropzone.dart · ancho fijo y borde de grosor constante · la caja encogía al pasar el cursor y se salía de debajo del puntero`

* **SESSION-REPORTING-5.3 — Archivos Nuevos y Borrados Se Marcan:**

  la fila indica si el archivo es nuevo o se eliminó; sin la marca, se asume modificación de uno existente.

**Filtro de verificación:** `SESSION-REPORTING-FV-5`.

**Plantilla de referencia:**
```
| Archivo | Qué cambió | Por qué |
|---|---|---|
| `ruta/al/archivo.ext` | <una línea> | <una línea> |
| `ruta/nueva.ext` *(nuevo)* | <una línea> | <una línea> |
```

---

## SESSION-REPORTING-6 — Subagentes Escriben Su Propia Sección

Un especialista despachado escribe en el reporte directamente, en vez de entregar un informe que el orquestador reescribe.

**Aplica a:** el agente que despacha subagentes y los subagentes despachados.

* **SESSION-REPORTING-6.1 — Sección Propia Con Nombre:**

  el subagente añade `### <Rol> — <título>` dentro de la entrada del turno, y escribe ahí su parte con la misma estructura de secciones.

* **SESSION-REPORTING-6.2 — El Orquestador No Reescribe:**

  quien despacha añade su lectura del resultado como texto propio aparte; no edita ni resume el texto del especialista sobre sí mismo. El usuario debe poder leer el original.
  * ❌ *Incorrecto:* condensar el hallazgo del especialista en una frase y borrar su sección.
  * ✅ *Correcto:* dejar la sección del especialista intacta y añadir debajo la valoración del orquestador.

* **SESSION-REPORTING-6.3 — Ruta en el Despacho:**

  el prompt de despacho incluye la ruta del archivo de sesión y el número de turno donde el subagente debe escribir.

**Filtro de verificación:** `SESSION-REPORTING-FV-6`.

---

## SESSION-REPORTING-7 — Sección Otros

Cajón declarado para lo que no encaja en ninguna sección fija, y mecanismo por el que el formato crece.

**Aplica a:** todos los roles/skills/agentes, al cerrar una entrada.

* **SESSION-REPORTING-7.1 — Qué Va Ahí:**

  observaciones al margen, hallazgos ajenos al encargo, dudas propias del agente, contexto que el usuario podría querer y ninguna sección reclama.

* **SESSION-REPORTING-7.2 — Es el Disparador de Cambio de Formato:**

  si el mismo tipo de contenido aparece en `Otros` tres veces, deja de ser un caso suelto: se propone al usuario como sección fija nueva. El formato no se cambia sin ese aval.

---

## SESSION-REPORTING-8 — Estilo de Redacción

Una entrada se escribe en estilo Gemini desde la primera palabra: el hecho primero, cada dato una sola vez, sin frases que comentan el propio texto.

**Aplica a:** todos los roles/skills/agentes, al redactar cualquier sección de una entrada.

* **SESSION-REPORTING-8.1 — Estilo Por Defecto, No Reparación:**

  no se redacta largo para comprimir después. Medido en la primera sesión del formato: dos entradas pasaron de 2 671 a 1 356 palabras sin perder un solo hecho, y el 49% recortado era narración.

* **SESSION-REPORTING-8.2 — La Compresión Nunca Toca el Contenido:**

  se corta narración, conector y metacomentario. Nunca un dato técnico, un caso borde, un tope, un pendiente ni una pregunta.
  * ❌ *Incorrecto:* borrar de la tabla de archivos la columna del porqué para acortar la entrada.
  * ✅ *Correcto:* dejar las tres columnas y cortar el párrafo que las repetía en prosa.

**Filtro de verificación:** `SESSION-REPORTING-FV-7`.

**Documento extendido:** [[.agents/knowledge/write-like-gemini.md]] — el Modo Gemini Relajado + Listas y sus 3 casos medidos; este bloque (`SESSION-REPORTING-8`) cubre lo específico del reporte de sesión, no repite el modo en sí.

---

## SESSION-REPORTING-9 — Ejemplos

### SESSION-REPORTING-9.1 — Casos Compuestos

Casos reales de esta sesión. No reemplazan los ❌/✅ atómicos de cada regla — muestran varias reglas actuando sobre el mismo texto.

**Caso: respuesta de trabajo terminado — demuestra `SESSION-REPORTING-2.1`, `-1.1` y `-5.2`.**

* **Sin estas reglas (~900 tokens en el chat):** el agente escribe en la terminal el diagnóstico de los tres defectos, la causa de cada uno, la lista de archivos, los resultados de las pruebas y las deudas abiertas — y además lo registra en la orden de trabajo y en la bitácora. El usuario lee tres veces lo mismo, disperso.

* **Aplicando las reglas — 3 líneas en el chat, todo lo demás en el archivo:**
  ```
  🟢 Listo — tres defectos de la zona de carga corregidos y verificados.
  Uno de ellos habría afectado a cualquier usuario de Linux, no solo a ti.
  → .agents/sessions/2026-08-02-reporte-de-sesion.md (turno 4)
  ```

**Caso: bloqueo — demuestra `SESSION-REPORTING-2.3`.**

* **Incorrecto:** eco en verde que dice "trabajo avanzado, ver reporte", con el bloqueo escondido en la sección de pendientes del archivo.
* **Correcto:**
  ```
  🔴 Detenido — el gate de mutación no corre: falta autorización para el crate nuevo.
  → .agents/sessions/2026-08-02-reporte-de-sesion.md (turno 5)
  ```
