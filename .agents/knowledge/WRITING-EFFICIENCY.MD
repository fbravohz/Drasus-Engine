# Eficiencia de Escritura (Writing Efficiency)

Toda regla sobre cómo redactar — prosa, bloques de respuesta, traducción de términos, densidad de tokens — vive en este único documento.

Slug ID: `WRITING_EFFICIENCY` para búsqueda rápida de secciones.

---

## WRITING_EFFICIENCY-1 — Eficiencia de Redacción

El objetivo no es gastar menos tokens: es que el texto se entienda en una sola pasada. Simple, no simplificado — el 100% del contenido se preserva, nunca se recorta. El ahorro de tokens es la prueba medible de que funcionó, no la meta.

**Por qué esto y no otra cosa:** En `WRITING-EFFICIENCY-7.1` se escribió un ejemplo con el mismo contenido dos veces — una vez con conectores narrativos, otra sin ellos — y se le pidió a alguien que lo leyera. La versión sin conectores no solo midió menos tokens: se entendió más rápido, porque lo que sobraba en la primera no era información, era ruido que el lector tenía que atravesar para llegar a lo que importaba.

**Aplica a:** todo texto que un agente produce.

* **WRITING_EFFICIENCY-1.1 — Contexto + Lista se prioriza sobre Texto Lineal:**

  Ante 2+ ítems paralelos (prohibiciones, pasos, causas, clasificaciones), usa una oración corta de contexto que termine en `:` + lista con bullets o numerales — con prioridad sobre texto lineal. Usa lineal solo si el contenido es una sola idea continua, sin ítems que enumerar.
  * ❌ *Incorrecto:* "Al detectar un bypass del provider, hay una serie de pasos que debes seguir de manera ordenada: primero reportas, luego clasificas, y por último propones la solución."
  * ✅ *Correcto:* "Al detectar un bypass del provider, realiza: 1. Reporta el hallazgo. 2. Clasifica la gravedad. 3. Propón la solución."

* **WRITING_EFFICIENCY-1.2 — Cortar Conectores, No Contenido:** 

  corta relleno narrativo, backstory, ❌/✅ redundantes, plantillas-espejo; nunca cortes dato técnico, caso borde, matiz arquitectónico, ni la oración-gancho de contexto (`WRITING_EFFICIENCY-4.1`).
  * ❌ *Incorrecto:* "Es fundamental que te detengas a revisar cinco preguntas clave antes de responder. "
  * ✅ *Correcto:* "Antes de responder, verifica cinco preguntas."

**Precedencia:** N/A.

**Filtro de verificación:** `WRITING_EFFICIENCY-6.1`.

**Documento extendido:** `WRITING_EFFICIENCY-7.1`

**Plantilla de referencia:** N/A.

---

## WRITING_EFFICIENCY-2 — Claridad Absoluta

El lenguaje es técnicamente exacto pero directo: si el usuario necesita releer una frase para entenderla, está mal redactada.

**Aplica a:** todos los roles/skills, en cualquier texto producido.

* **WRITING_EFFICIENCY-2.1 — Prohibido:** 

  jerga sin traducir (`WRITING_EFFICIENCY-3`), relleno que no aporta información, nominalizaciones, voz pasiva evitable (se permite la pasiva refleja natural del español), afirmaciones generales sin caso concreto que las sostenga.
  * ❌ *Incorrecto:* "la consecución de la implementación"
  * ✅ *Correcto:* "implementar"

* **WRITING_EFFICIENCY-2.2 — Obligatorio:** 

  una idea por oración, una idea principal por párrafo (conectores como "por eso" no cuentan como relleno); orden de exposición fijado en `WRITING_EFFICIENCY-4.1`; ejemplos y tablas solo cuando comparan 2+ opciones o datos paralelos.
  * ❌ *Incorrecto:* "El sistema usa SQLite y por eso hay que cuidar los locks y además conviene revisar los índices."
  * ✅ *Correcto:* "El sistema usa SQLite. Por eso hay que cuidar los locks. Además, conviene revisar los índices."

**Precedencia:** corolario de `WRITING_EFFICIENCY-1` — aplica su economía al nivel de la oración y el párrafo; no compite.

**Filtro de verificación:** `WRITING_EFFICIENCY-6.2`.

**Documento extendido:** N/A.

**Plantilla de referencia:** N/A.

---

## WRITING_EFFICIENCY-3 — Traducción de Términos Técnicos al Usuario

Los identificadores ágiles y términos internos (`STORY-###`, `ADR-XXXX`, `TTR-###`...) son atajos entre agentes — el usuario los conoce pero no se sabe de memoria su contenido.

**Aplica a:** todos los roles/skills, en cualquier mensaje dirigido al usuario.

* **WRITING_EFFICIENCY-3.1 — Traducción Siempre Presente:** 
  
  todo mensaje debe entenderse leyendo solo el texto en español; el identificador va como referencia secundaria entre paréntesis, después de la traducción llana.
  * ❌ *Incorrecto:* "Despacho STORY-001 + STORY-002 y cierro SPIKE-001 antes del Sprint 1."
  * ✅ *Correcto:* "Monto el esqueleto del proyecto y la base de datos (STORY-001 y STORY-002). Antes de seguir, confirmo que la pieza de NautilusTrader compila (SPIKE-001)."

* **WRITING_EFFICIENCY-3.2 — Alcance Por Mensaje:** 

  aplica cada vez que el identificador aparece, no solo la primera vez en la conversación.
  * ❌ *Incorrecto:* traducir SPIKE-001 al mencionarlo por primera vez y después usarlo pelado el resto de la conversación.
  * ✅ *Correcto:* traducirlo de nuevo si reaparece en un mensaje posterior, aunque ya se haya traducido antes.

**Precedencia:** N/A — no compite con otra regla.

**Filtro de verificación:** `WRITING_EFFICIENCY-6.3`.

**Documento extendido:** N/A.

**Plantilla de referencia:** N/A.

---

## WRITING_EFFICIENCY-4 — Contexto Antes de Detalle y Formato Visual de los Bloques

Toda respuesta pone el contexto antes que el detalle, pero la forma exacta depende del género del intercambio (`WRITING_EFFICIENCY-4.1.1` a `-4.1.4`) — nunca un molde único forzado sobre cualquier pregunta. Un mensaje que arranca en detalle sin haber dado contexto falla la Auditoría Final y debe reescribirse.

**Aplica a:** todos los roles/skills, en cualquier explicación estructurada al usuario.

* **WRITING_EFFICIENCY-4.1 — Contexto Antes de Detalle, Forma Según el Género:** 

  el principio no cambia — el contexto va antes que el detalle —, pero su forma sí depende de qué tipo de intercambio es. Forzar el molde de un género sobre otro (inventar un "problema" donde el usuario solo pidió un reporte de estado) empeora la respuesta en vez de ordenarla.
  1. **WRITING_EFFICIENCY-4.1.1 — Explicar un cambio o decisión ya tomada:**

     problema → solución → por qué → detalle técnico.
  2. **WRITING_EFFICIENCY-4.1.2 — Reportar un estado o historial:**

     (ante "¿en qué quedó X?", "qué ha pasado") contexto breve → hechos relevantes en lista (cronológica u organizada) → síntesis del estado actual → siguiente paso.
  3. **WRITING_EFFICIENCY-4.1.3 — Responder una pregunta puntual o de hecho:**

     respuesta directa, sin forzar bloques ni encabezados — el largo lo marca la pregunta, no la plantilla.
  4. **WRITING_EFFICIENCY-4.1.4 — Responder una consulta exploratoria, un debate o una opinión:**

     postura o recomendación corta + el trade-off principal (2-3 oraciones); no implementar ni decidir por el usuario, solo orientar.
  * ❌ *Incorrecto:* ante "¿en qué quedó el Tech-Lead?" responder con "1. Problema: ... 2. Solución: ... 3. Por qué: ..." — no hay problema que resolver, solo estado que reportar.
  * ✅ *Correcto:* "El PROGRESS dice que el frente activo es EPIC-1, pero eso quedó escrito antes del trabajo real de los últimos 3 commits. Lo que pasó: [lista]. Estatus real hoy: [síntesis]. Siguiente paso: [acción]."

* **WRITING_EFFICIENCY-4.2 — Encabezado En Línea Propia:** 
  
  cada bloque lleva una etiqueta corta en su propia línea, con salto de línea antes del texto — nunca pegada al texto.
  * ❌ *Incorrecto:* "[1. Problema] El riesgo es..."
  * ✅ *Correcto:* "### 1. Problema\n\nEl riesgo es..."

* **WRITING_EFFICIENCY-4.3 — Bullets Para 2+ Ítems:** 
  
  cualquier bloque con dos o más riesgos, causas, pasos o datos va en viñetas, nunca en prosa corrida.
  * ❌ *Incorrecto:* "Los riesgos son A, B y C mezclados en una oración."
  * ✅ *Correcto:* "- riesgo A\n- riesgo B\n- riesgo C"

* **WRITING_EFFICIENCY-4.4 — Sintaxis Con Paréntesis Explicativo:** 
  
  cada función, comando o herramienta lleva entre paréntesis qué hace, sobre todo en el bloque de Detalle Técnico.
  * ❌ *Incorrecto:* "Uso `BEGIN IMMEDIATE` para evitar la corrupción."
  * ✅ *Correcto:* "Uso `BEGIN IMMEDIATE` (toma el lock de escritura al abrir la transacción) para evitar la corrupción."

**Precedencia:** N/A — no compite con otra regla; elegir el género correcto (`4.1.1`-`4.1.4`) es obligatorio, no estilístico.

**Filtro de verificación:** `WRITING_EFFICIENCY-6.4`.

**Plantilla de referencia:**
```
Forma 4.1.1 — Explicar un cambio o decisión:

### 1. Problema

- riesgo o causa A
- riesgo o causa B

### 2. Solución

Texto de la solución.

### 3. Por qué esa solución

Texto de la justificación.

### 4. Detalle técnico

- `nombre_de_sintaxis` (qué hace, en paréntesis)

Forma 4.1.2 — Reportar un estado o historial:

**Contexto:** una oración que resume el desfase o el punto de partida.

**Lo que pasó:**
- hecho 1
- hecho 2

**Estatus real hoy:**
- síntesis 1

**Siguiente paso:** acción concreta pendiente.

Formas 4.1.3 y 4.1.4 no llevan plantilla de bloques — su forma es una respuesta directa (4.1.3) o una recomendación corta + trade-off (4.1.4).
```

---

## WRITING_EFFICIENCY-5 — Encabezados Sin Backstory Ni Rutas Literales

Títulos y descripciones rápidas de un documento de referencia dicen qué es y cómo se usa, no por qué nació.

**Aplica a:** todos los roles/skills, al escribir o editar el título y la descripción rápida de cualquier documento.

* **WRITING_EFFICIENCY-5.1 — Sin Historia De Origen Ni Rutas Literales:** 

  el encabezado no lleva por qué nació la regla, qué error corrigió, ni rutas de carpeta exactas cuando una descripción conceptual alcanza. Esa historia va en el commit o en memoria, no en el documento.
  * ❌ *Incorrecto:* "Cómo escribir UNA regla dentro de cualquier documento de `.agents/knowledge/` o `.claude/skills/`... Nace de una corrección real: `base.md` tenía 8 puntos y ninguno seguía la misma forma."
  * ✅ *Correcto:* "Cómo escribir una regla dentro de cualquier documento de instrucciones para agentes o skills, para que sea encontrable por ID y aplicable sin ambigüedad de formato."

**Precedencia:** corolario de `WRITING_EFFICIENCY-2.1` (Prohibido: relleno) — no compite, lo concreta para encabezados.

**Filtro de verificación:** `WRITING_EFFICIENCY-6.5`.

**Documento extendido:** N/A.

**Plantilla de referencia:** N/A.

---

## WRITING_EFFICIENCY-6 — Filtro de Verificación

Antes de enviar cualquier texto, revisa esta lista — es el chequeo interno que las 5 reglas de este documento citan como propio.

**Cuando respondas al usuario confirma: "He leido y aplicaré el filtro de verificación WRITING_EFFICIENCY" HAZLO SIEMPRE, O SERA CONSIDERADO VIOLACIÓN DEL PROTOCOLO DE RESPUESTA**

**Aplica a:** todo texto que un agente produce, como último paso antes de enviarlo.

* **WRITING_EFFICIENCY-6.1:** ¿Empaqueté ítems paralelos con oración de contexto + lista? ¿Corté conectores sin cortar contenido, caso borde o matiz?
* **WRITING_EFFICIENCY-6.2:** ¿Evité jerga sin traducir, relleno, nominalizaciones, voz pasiva evitable, afirmaciones sueltas?
* **WRITING_EFFICIENCY-6.3:** ¿Todo identificador interno lleva su traducción llana, en este mismo mensaje (no solo la primera vez)?
* **WRITING_EFFICIENCY-6.4:** ¿Elegí la forma correcta según el género del intercambio (`WRITING_EFFICIENCY-4.1.1` a `-4.1.4`) y respeté el formato visual de bloques (encabezado en línea propia, bullets, paréntesis explicativo)?
* **WRITING_EFFICIENCY-6.5:** si escribí o edité un documento, ¿el título y la descripción rápida están libres de backstory y rutas literales?

**Precedencia:** N/A — es el filtro, no compite con las reglas que verifica.

**Filtro de verificación:** N/A.

**Documento extendido:** N/A — el detalle de cada chequeo vive en su regla correspondiente (`WRITING_EFFICIENCY-1` a `-5`) dentro de este mismo archivo.

**Plantilla de referencia:** N/A.

---

## WRITING_EFFICIENCY-7 — Ejemplos

### WRITING_EFFICIENCY-7.1 — Casos Compuestos
El mismo texto real escrito varias veces, aplicando distintas reglas de este documento a la vez. No reemplazan los ❌/✅ de cada regla (esos son atómicos, uno por definición) — muestran cómo varias reglas actúan juntas sobre un párrafo real.

**Caso: lista de prohibiciones — demuestra `WRITING_EFFICIENCY-1.1`, `-1.2` y `-2.1` a la vez.**

* **Sin estas reglas (271 tokens):** "Existen varios patrones de escritura que debemos evitar de forma sistemática porque afectan negativamente la comprensión del usuario. En primer lugar, está prohibido el uso de jerga técnica densa sin una explicación adecuada que la acompañe, ya que esto obliga al lector a hacer un esfuerzo adicional de traducción mental. En segundo lugar, se debe evitar la prosa que rellena el texto sin aportar información real, como las frases de apertura aduladoras (por ejemplo, "¡Excelente pregunta!") o los cierres que simplemente repiten lo que ya se dijo antes. En tercer lugar, las nominalizaciones deben evitarse — por ejemplo, en vez de escribir "la consecución de la implementación", es preferible escribir directamente "implementar". En cuarto lugar, la voz pasiva debe usarse con moderación, prefiriendo siempre la voz activa cuando exista una forma más directa de decir lo mismo. Por último, las afirmaciones generales sin un caso concreto que las sostenga también están prohibidas, ya que dejan al lector sin manera de verificar si la afirmación es cierta o aplicable a su situación."

* **Aplicando `WRITING_EFFICIENCY-1.1` (contexto + lista) y `-2.1` (prohibido, empacado) — 89 tokens:**
  ```
  Evita:
  - Jerga sin traducir — obliga a traducir mentalmente.
  - Relleno cordial o cierres que repiten lo ya dicho.
  - Nominalizaciones: usa el verbo directo ("implementar", no "la consecución de la implementación").
  - Voz pasiva cuando existe una forma más directa.
  - Afirmaciones generales sin un caso concreto que las sostenga.
  ```

**Medido (las 3 versiones completas, no solo esta):**

| Caso | Sin estas reglas | Con `WRITING_EFFICIENCY` (relajado + listas) | Con `WRITING_EFFICIENCY` (lineal) |
|---|---|---|---|
| Prohibiciones | 171 palabras / ~271 tokens | 53 palabras / ~89 tokens (67% menos) | 61 palabras / ~100 tokens (63% menos) |
| Protocolo de pasos | 154 palabras / ~239 tokens | 58 palabras / ~91 tokens (62% menos) | 58 palabras / ~93 tokens (61% menos) |
| Checklist | 193 palabras / ~285 tokens | 58 palabras / ~100 tokens (65% menos) | 65 palabras / ~109 tokens (62% menos) |

Mismo contenido verificable en los tres casos — la reducción es ruido cortado, no información perdida (`WRITING_EFFICIENCY-1.2`).
