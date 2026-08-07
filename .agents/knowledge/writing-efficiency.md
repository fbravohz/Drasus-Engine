# Eficiencia de Escritura (Writing Efficiency)

Toda regla sobre cómo redactar — prosa, bloques de respuesta, traducción de términos, densidad de tokens — vive en este único documento.

Slug ID: `WRITING-EFFICIENCY`.

Tamaño: `19600 caracteres (~4900 tokens aprox., ratio 4:1)`.

---

## WRITING-EFFICIENCY-FV — Filtro de Verificación

Antes de enviar cualquier texto, revisa esta lista — es el chequeo interno que las reglas de este documento citan como propio.

**Aplica a:** todo texto que un agente produce, como último paso antes de enviarlo.

* **WRITING-EFFICIENCY-FV-1:** ¿Empaqueté ítems paralelos con oración de contexto + lista? ¿Corté conectores sin cortar contenido, caso borde o matiz? (`WRITING-EFFICIENCY-1`)
* **WRITING-EFFICIENCY-FV-2:** ¿Evité jerga sin traducir, relleno, nominalizaciones, voz pasiva evitable, afirmaciones sueltas? (`WRITING-EFFICIENCY-2`)
* **WRITING-EFFICIENCY-FV-3:** ¿Todo identificador interno lleva su traducción llana, en este mismo mensaje (no solo la primera vez)? (`WRITING-EFFICIENCY-3`)
* **WRITING-EFFICIENCY-FV-4:** ¿Elegí la forma correcta según el género del intercambio (`WRITING-EFFICIENCY-6.1` a `-6.4`) y respeté el formato visual de bloques (encabezado en línea propia, bullets, paréntesis explicativo)? (`WRITING-EFFICIENCY-4`, `WRITING-EFFICIENCY-6`)
* **WRITING-EFFICIENCY-FV-5:** si escribí o edité un documento, ¿el título y la descripción rápida están libres de backstory y rutas literales? (`WRITING-EFFICIENCY-5`)
* **WRITING-EFFICIENCY-FV-6:** si el género es reportar un estado, ¿apliqué los cinco movimientos y, si corresponde, activé y declaré el Modo Gemini con su bracket? (`WRITING-EFFICIENCY-7`, `write-like-gemini.md`)

De ser satisfactoria cada validación de la lista, declara `WRITING-EFFICIENCY - FILTRO DE VERIFICACION APROBADO`.

---

## WRITING-EFFICIENCY-1 — Eficiencia de Redacción

El objetivo es que el texto se entienda en una sola pasada, no gastar menos tokens — el ahorro es la prueba medible de que funcionó, no la meta. Simple, no simplificado: el 100% del contenido se preserva, nunca se recorta.

**Aplica a:** todo texto que un agente produce.

* **WRITING-EFFICIENCY-1.1 — Contexto + Lista se prioriza sobre Texto Lineal:**

  Ante 2+ ítems paralelos (prohibiciones, pasos, causas, clasificaciones), usa una oración corta de contexto que termine en `:` + lista con bullets o numerales — con prioridad sobre texto lineal. Usa lineal solo si el contenido es una sola idea continua, sin ítems que enumerar.
  * ❌ *Incorrecto:* "Al detectar un bypass del provider, hay una serie de pasos que debes seguir de manera ordenada: primero reportas, luego clasificas, y por último propones la solución."
  * ✅ *Correcto:* "Al detectar un bypass del provider, realiza: 1. Reporta el hallazgo. 2. Clasifica la gravedad. 3. Propón la solución."

* **WRITING-EFFICIENCY-1.2 — Cortar Conectores, No Contenido:** 

  corta relleno narrativo, backstory, ❌/✅ redundantes, plantillas-espejo; nunca cortes dato técnico, caso borde, matiz arquitectónico, ni la oración-gancho de contexto (`WRITING-EFFICIENCY-6`).
  * ❌ *Incorrecto:* "Es fundamental que te detengas a revisar cinco preguntas clave antes de responder. "
  * ✅ *Correcto:* "Antes de responder, verifica cinco preguntas."

**Filtro de verificación:** `WRITING-EFFICIENCY-FV-1`.

---

## WRITING-EFFICIENCY-2 — Claridad Absoluta

El lenguaje es técnicamente exacto pero directo: si el usuario necesita releer una frase para entenderla, está mal redactada. Aplica la economía de `WRITING-EFFICIENCY-1` al nivel de la oración y el párrafo.

**Aplica a:** todos los roles/skills, en cualquier texto producido.

* **WRITING-EFFICIENCY-2.1 — Prohibido:** 

  jerga sin traducir (`WRITING-EFFICIENCY-3`), relleno que no aporta información, nominalizaciones, voz pasiva evitable (se permite la pasiva refleja natural del español), afirmaciones generales sin caso concreto que las sostenga.
  * ❌ *Incorrecto:* "la consecución de la implementación"
  * ✅ *Correcto:* "implementar"

* **WRITING-EFFICIENCY-2.2 — Obligatorio:** 

  una idea por oración, una idea principal por párrafo (conectores como "por eso" no cuentan como relleno); orden de exposición fijado en `WRITING-EFFICIENCY-6`; ejemplos y tablas solo cuando comparan 2+ opciones o datos paralelos.
  * ❌ *Incorrecto:* "El sistema usa SQLite y por eso hay que cuidar los locks y además conviene revisar los índices."
  * ✅ *Correcto:* "El sistema usa SQLite. Por eso hay que cuidar los locks. Además, conviene revisar los índices."

**Filtro de verificación:** `WRITING-EFFICIENCY-FV-2`.

---

## WRITING-EFFICIENCY-3 — Traducción de Términos Técnicos al Usuario

Los identificadores ágiles y términos internos (`STORY-###`, `ADR-XXXX`, `TTR-###`...) son atajos entre agentes — el usuario los conoce pero no se sabe de memoria su contenido.

**Aplica a:** todos los roles/skills, en cualquier mensaje dirigido al usuario.

* **WRITING-EFFICIENCY-3.1 — Traducción Siempre Presente:** 
  
  todo mensaje debe entenderse leyendo solo el texto en español; el identificador va como referencia secundaria entre paréntesis, después de la traducción llana.
  * ❌ *Incorrecto:* "Despacho STORY-001 + STORY-002 y cierro SPIKE-001 antes del Sprint 1."
  * ✅ *Correcto:* "Monto el esqueleto del proyecto y la base de datos (STORY-001 y STORY-002). Antes de seguir, confirmo que la pieza de NautilusTrader compila (SPIKE-001)."

* **WRITING-EFFICIENCY-3.2 — Alcance Por Mensaje:** 

  aplica cada vez que el identificador aparece, no solo la primera vez en la conversación.
  * ❌ *Incorrecto:* traducir SPIKE-001 al mencionarlo por primera vez y después usarlo pelado el resto de la conversación.
  * ✅ *Correcto:* traducirlo de nuevo si reaparece en un mensaje posterior, aunque ya se haya traducido antes.

**Filtro de verificación:** `WRITING-EFFICIENCY-FV-3`.

---

## WRITING-EFFICIENCY-4 — Formato Visual de los Bloques

Cada bloque de una respuesta estructurada sigue una forma visual fija — encabezado, bullets, paréntesis explicativo — sin importar el género del intercambio (`WRITING-EFFICIENCY-6`).

**Aplica a:** todos los roles/skills, en cualquier explicación estructurada al usuario.

* **WRITING-EFFICIENCY-4.2 — Encabezado En Línea Propia:** 
  
  cada bloque lleva una etiqueta corta en su propia línea, con salto de línea antes del texto — nunca pegada al texto.
  * ❌ *Incorrecto:* "[1. Problema] El riesgo es..."
  * ✅ *Correcto:* "### 1. Problema\n\nEl riesgo es..."

* **WRITING-EFFICIENCY-4.3 — Bullets Para 2+ Ítems:** 
  
  cualquier bloque con dos o más riesgos, causas, pasos o datos va en viñetas, nunca en prosa corrida.
  * ❌ *Incorrecto:* "Los riesgos son A, B y C mezclados en una oración."
  * ✅ *Correcto:* "- riesgo A\n- riesgo B\n- riesgo C"

* **WRITING-EFFICIENCY-4.4 — Sintaxis Con Paréntesis Explicativo:** 
  
  cada función, comando o herramienta lleva entre paréntesis qué hace, sobre todo en el bloque de Detalle Técnico.
  * ❌ *Incorrecto:* "Uso `BEGIN IMMEDIATE` para evitar la corrupción."
  * ✅ *Correcto:* "Uso `BEGIN IMMEDIATE` (toma el lock de escritura al abrir la transacción) para evitar la corrupción."

**Filtro de verificación:** `WRITING-EFFICIENCY-FV-4`.

---

## WRITING-EFFICIENCY-5 — Encabezados Sin Backstory Ni Rutas Literales

Títulos y descripciones rápidas de un documento de referencia dicen qué es y cómo se usa, no por qué nació — concreta para encabezados la prohibición de relleno de `WRITING-EFFICIENCY-2.1`.

**Aplica a:** todos los roles/skills, al escribir o editar el título y la descripción rápida de cualquier documento.

* **WRITING-EFFICIENCY-5.1 — Sin Historia De Origen Ni Rutas Literales:** 

  el encabezado no lleva por qué nació la regla, qué error corrigió, ni rutas de carpeta exactas cuando una descripción conceptual alcanza. Esa historia va en el commit o en memoria, no en el documento.
  * ❌ *Incorrecto:* "Cómo escribir UNA regla dentro de cualquier documento de `.agents/knowledge/` o `.claude/skills/`... Nace de una corrección real: `base.md` tenía 8 puntos y ninguno seguía la misma forma."
  * ✅ *Correcto:* "Cómo escribir una regla dentro de cualquier documento de instrucciones para agentes o skills, para que sea encontrable por ID y aplicable sin ambigüedad de formato."

**Filtro de verificación:** `WRITING-EFFICIENCY-FV-5`.

---

## WRITING-EFFICIENCY-6 — Contexto Antes de Detalle, Forma Según el Género

El principio no cambia — el contexto va antes que el detalle —, pero su forma sí depende de qué tipo de intercambio es; nunca un molde único forzado sobre cualquier pregunta. Forzar el molde de un género sobre otro (inventar un "problema" donde el usuario solo pidió un reporte de estado) empeora la respuesta en vez de ordenarla. Un mensaje que arranca en detalle sin haber dado contexto falla la Auditoría Final y debe reescribirse.

**Aplica a:** todos los roles/skills, en cualquier explicación estructurada al usuario.

* **WRITING-EFFICIENCY-6.1 — Explicar un cambio o decisión ya tomada:**

  problema → solución → por qué → detalle técnico.

* **WRITING-EFFICIENCY-6.2 — Reportar un estado o historial:**

  (ante "¿en qué quedó X?", "qué ha pasado") contexto breve → hechos relevantes en lista (cronológica u organizada) → síntesis del estado actual → siguiente paso.

* **WRITING-EFFICIENCY-6.3 — Responder una pregunta puntual o de hecho:**

  respuesta directa, sin forzar bloques ni encabezados — el largo lo marca la pregunta, no la plantilla.

* **WRITING-EFFICIENCY-6.4 — Responder una consulta exploratoria, un debate o una opinión:**

  postura o recomendación corta + el trade-off principal (2-3 oraciones); no implementar ni decidir por el usuario, solo orientar.

* ❌ *Incorrecto:* ante "¿en qué quedó el Tech-Lead?" responder con "1. Problema: ... 2. Solución: ... 3. Por qué: ..." — no hay problema que resolver, solo estado que reportar.
* ✅ *Correcto:* "El PROGRESS dice que el frente activo es EPIC-1, pero eso quedó escrito antes del trabajo real de los últimos 3 commits. Lo que pasó: [lista]. Estatus real hoy: [síntesis]. Siguiente paso: [acción]."

**Filtro de verificación:** `WRITING-EFFICIENCY-FV-4`.

**Plantilla de referencia:**
```
Forma 6.1 — Explicar un cambio o decisión:

### 1. Problema

- riesgo o causa A
- riesgo o causa B

### 2. Solución

Texto de la solución.

### 3. Por qué esa solución

Texto de la justificación.

### 4. Detalle técnico

- `nombre_de_sintaxis` (qué hace, en paréntesis)

Forma 6.2 — Reportar un estado o historial:

**Contexto:** una oración que resume el desfase o el punto de partida.

**Lo que pasó:**
- hecho 1
- hecho 2

**Estatus real hoy:**
- síntesis 1

**Siguiente paso:** acción concreta pendiente.

Formas 6.3 y 6.4 no llevan plantilla de bloques — su forma es una respuesta directa (6.3) o una recomendación corta + trade-off (6.4).
```

---

## WRITING-EFFICIENCY-7 — Estilo Gemini: Modo Por Defecto

Cuando el género es "reportar un estado" (`WRITING-EFFICIENCY-6.2`), el Modo Gemini Relajado + Listas ([[.agents/knowledge/write-like-gemini.md]]) se activa desde la primera palabra sin que el usuario lo pida — no como reparación de un texto que ya salió largo. Los cinco movimientos de abajo son propios del género "reportar un estado"; el registro de verbosidad (Claude/Gemini) y sus ejemplos medidos viven en `write-like-gemini.md`, no aquí.

**Aplica a:** todos los roles/skills, en reportes de estado, resúmenes de turno y reportes de sesión ([[.agents/knowledge/session-reporting.md]]).

* **WRITING-EFFICIENCY-7.1 — Resultado Antes Que Proceso:**

  la primera cláusula nombra el hecho o la decisión; el camino para llegar ahí va después, y solo si el usuario lo necesita para actuar.
  * ❌ *Incorrecto:* "Después de revisar varias opciones y evaluar el impacto, decidimos usar X."
  * ✅ *Correcto:* "Usamos X."

* **WRITING-EFFICIENCY-7.2 — Un Hecho, Una Vez:**

  si un dato ya vive en una tabla, no lo repitas en prosa alrededor — y viceversa.
  * ❌ *Incorrecto:* una tabla que dice "7 pruebas nuevas" y, debajo, un párrafo que vuelve a decir que se agregaron 7 pruebas.
  * ✅ *Correcto:* el dato vive en un solo lugar.

* **WRITING-EFFICIENCY-7.3 — Cero Metacomentario Sobre el Propio Texto:**

  nada de "vale la pena que leas esto", "es importante notar", "cabe destacar" — el hecho justifica su propia importancia.

* **WRITING-EFFICIENCY-7.4 — Fragmenta Cláusulas Independientes:**

  si una oración une dos ideas con "y", "pero" o un guion largo y la segunda no depende gramaticalmente de la primera, sepáralas en bullets.
  * ❌ *Incorrecto:* "El sistema procesa el archivo y valida cada fila, pero si hay un error se detiene."
  * ✅ *Correcto:* "Procesa el archivo. Valida cada fila. Si hay error, se detiene."

* **WRITING-EFFICIENCY-7.5 — Cita Sin Marco Decorativo:**

  cita textual solo si la paráfrasis pierde algo real — directo, sin anunciarla.
  * ❌ *Incorrecto:* "lo que dice es lo siguiente: 'X'."
  * ✅ *Correcto:* "'X'."

**Precedencia:** `WRITING-EFFICIENCY-1.2` manda sobre esto — nunca cortar dato técnico, caso borde o matiz para cumplirla.

**Filtro de verificación:** `WRITING-EFFICIENCY-FV-6`.

**Documento extendido:** [[.agents/knowledge/write-like-gemini.md]] — los tres modos de verbosidad (Claude / Gemini Relajado / Gemini Relajado + Listas), con los tres casos reales medidos (`wc -w -m`, no estimado).

---

## WRITING-EFFICIENCY-EJ — Ejemplos

### WRITING-EFFICIENCY-EJ-1 — Casos Compuestos
El mismo texto real escrito varias veces, aplicando distintas reglas de este documento a la vez. No reemplazan los ❌/✅ de cada regla (esos son atómicos, uno por definición) — muestran cómo varias reglas actúan juntas sobre un párrafo real.

**Caso: lista de prohibiciones — demuestra `WRITING-EFFICIENCY-1.1`, `-1.2` y `-2.1` a la vez.**

* **Sin estas reglas (271 tokens):** "Existen varios patrones de escritura que debemos evitar de forma sistemática porque afectan negativamente la comprensión del usuario. En primer lugar, está prohibido el uso de jerga técnica densa sin una explicación adecuada que la acompañe, ya que esto obliga al lector a hacer un esfuerzo adicional de traducción mental. En segundo lugar, se debe evitar la prosa que rellena el texto sin aportar información real, como las frases de apertura aduladoras (por ejemplo, "¡Excelente pregunta!") o los cierres que simplemente repiten lo que ya se dijo antes. En tercer lugar, las nominalizaciones deben evitarse — por ejemplo, en vez de escribir "la consecución de la implementación", es preferible escribir directamente "implementar". En cuarto lugar, la voz pasiva debe usarse con moderación, prefiriendo siempre la voz activa cuando exista una forma más directa de decir lo mismo. Por último, las afirmaciones generales sin un caso concreto que las sostenga también están prohibidas, ya que dejan al lector sin manera de verificar si la afirmación es cierta o aplicable a su situación."

* **Aplicando `WRITING-EFFICIENCY-1.1` (contexto + lista) y `-2.1` (prohibido, empacado) — 89 tokens:**
  ```
  Evita:
  - Jerga sin traducir — obliga a traducir mentalmente.
  - Relleno cordial o cierres que repiten lo ya dicho.
  - Nominalizaciones: usa el verbo directo ("implementar", no "la consecución de la implementación").
  - Voz pasiva cuando existe una forma más directa.
  - Afirmaciones generales sin un caso concreto que las sostenga.
  ```

**Medido (las 3 versiones completas, no solo esta):**

| Caso | Sin estas reglas | Con `WRITING-EFFICIENCY` (relajado + listas) | Con `WRITING-EFFICIENCY` (lineal) |
|---|---|---|---|
| Prohibiciones | 171 palabras / ~271 tokens | 53 palabras / ~89 tokens (67% menos) | 61 palabras / ~100 tokens (63% menos) |
| Protocolo de pasos | 154 palabras / ~239 tokens | 58 palabras / ~91 tokens (62% menos) | 58 palabras / ~93 tokens (61% menos) |
| Checklist | 193 palabras / ~285 tokens | 58 palabras / ~100 tokens (65% menos) | 65 palabras / ~109 tokens (62% menos) |

Mismo contenido verificable en los tres casos — la reducción es ruido cortado, no información perdida (`WRITING-EFFICIENCY-1.2`).

### WRITING-EFFICIENCY-EJ-2 — Explicar un Problema Técnico Desde Cero

Cuando el usuario pide entender un problema que no conoce —no una actualización sobre uno que ya conoce— la forma cambia. Este es el caso de referencia: el propietario pidió *"explícame todo este tema desde cero, como si no supiera qué es, y por qué no hemos podido dar con la solución"*, y calificó el resultado como el estándar de claridad a replicar.

**Caso: el patrón de ledger append-only — demuestra `WRITING-EFFICIENCY-2.1`, `-3.1` y `-6` a la vez.**

* **Antes (jerga sin traducir, ~85 tokens):** "El patrón de ledger append-only exige `event_sequence_id INTEGER UNIQUE` monótono sin huecos, encadenamiento por `audit_chain_hash`, triggers `BEFORE UPDATE/DELETE` con `RAISE(ABORT)`, y escritura envuelta en `BEGIN IMMEDIATE` con clasificador de errores transitorios vs. permanentes y reintento acotado. La implementación de referencia presentaba dos defectos que se propagaron por copia."

* **Después (mismo contenido, ~150 tokens, cero identificadores):**
  ```
  Piensa en el libro de un notario. Tres reglas:

  - Nunca se borra ni se corrige. Si te equivocaste, escribes una entrada
    nueva que dice "la anterior estaba mal". La equivocación se queda ahí.
  - Las páginas van numeradas y sin saltos. Si falta la 47, algo pasó.
  - Cada página lleva un sello que depende de la anterior. Si alguien
    arranca la 47 y mete una falsa, el sello de la 48 deja de cuadrar.
  ```

Es **más largo en tokens y más corto en esfuerzo del lector** — la excepción legítima a `WRITING-EFFICIENCY-1`: la economía se mide en comprensión, no en caracteres, y el destinatario no tenía el concepto.

**Los cinco movimientos que hicieron funcionar la explicación completa** (replícalos, no copies el ejemplo del notario a temas donde no encaja):

1. **Una sola analogía, sostenida hasta el final.** El notario aparece al abrir y sigue explicando cada agujero: la prueba que no probaba nada es *"verificar que un médico distingue gripe de alergia preguntándole a la recepcionista"*; la tabla sin candados es *"una libreta con una nota que dice 'por favor no borres'"*. **Cambiar de metáfora a mitad obliga a reconstruir el modelo mental** — el error que esto evita.
2. **Cada defecto trae su consecuencia concreta**, no su descripción. No *"el clasificador podía mutarse"* sino *"el programa se quedaría reintentando algo que jamás va a funcionar, y después reportaría 'había mucho tráfico' cuando había un dato corrupto"*.
3. **La causa raíz se cuenta como historia, no como lista.** Tres causas encadenadas —la referencia se eligió por antigüedad; la práctica mejoró pero el letrero no; y el autor repitió el error al escribir la corrección— explican por qué el problema sobrevivió meses. Una lista de tres viñetas habría dado los mismos hechos sin la cadena causal.
4. **El error propio va incluido y sin suavizar.** *"Cometí, escribiendo el documento que existe para impedir ese error, exactamente ese error."* Es lo que vuelve la explicación creíble; omitirlo la convierte en informe defensivo.
5. **Cierra con por qué fue difícil de ver, que es la pregunta real.** *"No es que la solución sea difícil — cada arreglo son treinta líneas. Es que nada lo detectaba: todas las pruebas en verde durante meses."* Sin ese cierre el usuario entiende el defecto pero no por qué se le ocultó, que es lo que de verdad preguntó.

**Cuándo aplica este género:** el usuario pide entender, no decidir. Si pide decidir, gana `WRITING-EFFICIENCY-6` (contexto antes de detalle, formato de bloques) y la analogía sobra.
