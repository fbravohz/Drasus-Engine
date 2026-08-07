# Modo de Escritura: Claude, Gemini Relajado, Gemini Relajado + Listas

Tres registros para escribir el mismo contenido, mismo 100% de información, distinto empaque. Se activa por orden explícita del usuario en cualquier punto de la conversación y se mantiene mientras dure la conversación o el agente.

Slug ID: `WRITE-LIKE-GEMINI`.

Tamaño: `12976 caracteres (~3244 tokens aprox., ratio 4:1)`.

---

## WRITE-LIKE-GEMINI-0 — Lecturas Obligatorias

| Documento | De qué va | Cuándo leerlo (disparador) | Alcance |
|---|---|---|---|
| [[.agents/knowledge/governance.md]] | Cómo se reconoce la frase de activación y se declara el modo activo cada ronda. | `SIEMPRE` — `GOVERNANCE-1.7` dispara la lectura de este documento. | Todos los agentes |
| [[.agents/knowledge/writing-efficiency.md]] | Fuente única de reglas de redacción; `WRITING-EFFICIENCY-7` activa el Modo Gemini Relajado + Listas automáticamente al reportar un estado. | Al escribir un reporte de estado, resumen de turno o reporte de sesión. | Todos los agentes |
| [[.agents/knowledge/session-reporting.md]] | Qué va al archivo de sesión y qué queda en el eco de tres líneas del chat. | Al escribir cualquier entrada de `.agents/sessions/`. | Todos los agentes |

---

## WRITE-LIKE-GEMINI-FV — Filtro de Verificación

**Aplica a:** todos los agentes, en cada respuesta o archivo escrito mientras un modo distinto de Claude esté activo.

* **WRITE-LIKE-GEMINI-FV-1:** ¿el modo activo es el último que pidió el usuario — no revertí a Claude sin instrucción, ni seguí en Gemini si pidió volver a Claude? (`WRITE-LIKE-GEMINI-2`)
* **WRITE-LIKE-GEMINI-FV-2:** ¿declaré el modo activo en su propio bracket esta ronda, sin agruparlo con otras citas? (`WRITE-LIKE-GEMINI-3`, `GOVERNANCE-1.5`)
* **WRITE-LIKE-GEMINI-FV-3:** ¿lo que escribí replica el movimiento del ejemplo del modo activo — verbo directo, sin relleno, listas donde el modo las pide — en vez de parafrasear la regla? (`WRITE-LIKE-GEMINI-1`, `WRITE-LIKE-GEMINI-EJ`)

De ser satisfactoria cada validación de la lista, declara `WRITE-LIKE-GEMINI - FILTRO DE VERIFICACION APROBADO`.

---

## WRITE-LIKE-GEMINI-1 — Los Tres Modos

Mismo contenido, tres formas de empacarlo — ninguna resume ni corta dato técnico, caso borde o matiz.

**Aplica a:** todos los roles/skills, en cualquier respuesta o archivo mientras `GOVERNANCE-1.7` tenga un modo activo.

* **WRITE-LIKE-GEMINI-1.1 — Modo Claude (default):**

  prosa completa con conectores narrativos ("en primer lugar", "es fundamental que"), cada idea envuelta en contexto. Es lo que sale sin activar nada — nunca se declara con bracket porque es el estado de reposo.

* **WRITE-LIKE-GEMINI-1.2 — Modo Gemini Relajado (~61% menos, medido):**

  mismo contenido, sin narrar el propósito de decirlo, sin frases de transición, verbo directo. Sigue siendo prosa — no fuerza listas — pero cada oración carga un solo dato nuevo, sin relleno alrededor.

* **WRITE-LIKE-GEMINI-1.3 — Modo Gemini Relajado + Listas (~65% menos, medido):**

  igual que `-1.2`, y todo conjunto de 2+ ítems paralelos (prohibiciones, pasos, causas) se separa en viñetas o numerales en vez de encadenarse en prosa. Modo recomendado por defecto al escribir o reescribir un documento completo (knowledge, skill, ADR): sostiene el 100% del contenido en 60-100 líneas donde el modo Claude necesitaría 300.

**Precedencia:** `WRITING-EFFICIENCY-1.2` (cortar conectores, no contenido) manda sobre los tres modos.

**Filtro de verificación:** `WRITE-LIKE-GEMINI-FV-3`.

---

## WRITE-LIKE-GEMINI-2 — Activación y Persistencia

Cómo se enciende y apaga un modo, y cuánto dura una vez encendido.

**Aplica a:** todos los roles/skills.

* **WRITE-LIKE-GEMINI-2.1 — Frases de Activación:**

  cualquier variante de "escribe/reescríbelo como Gemini", "modo gemini relajado", "modo gemini relajado + listas" o "modo Claude" — dicha en cualquier punto de la conversación, incluso pegada a la invocación de otro skill (ej. "/tech-lead escribe como gemini, vamos a...") — activa el modo correspondiente desde esa misma respuesta.
  * ❌ *Incorrecto:* tratar la frase de activación como contexto conversacional y seguir en el modo anterior.
  * ✅ *Correcto:* la respuesta donde aparece la frase ya sale en el modo pedido.

* **WRITE-LIKE-GEMINI-2.2 — Alcance y Duración:**

  el modo activo aplica a toda escritura — respuesta de chat y archivo (documentación, memoria, comentarios, reportes) — mientras dure la conversación o el agente, sin importar cuántos turnos o skills se invoquen de por medio. Nunca revierte solo porque cambió el tema o el skill activo.
  * ❌ *Incorrecto:* volver a modo Claude en la siguiente respuesta sin que el usuario lo haya pedido.
  * ✅ *Correcto:* seguir en modo Gemini diez turnos después de activarlo, aunque la conversación haya cambiado de skill.

* **WRITE-LIKE-GEMINI-2.3 — Reversión:**

  "modo Claude" apaga cualquier modo Gemini activo y vuelve al estilo por defecto — mismo mecanismo de `-2.1`, dirección contraria.

**Filtro de verificación:** `WRITE-LIKE-GEMINI-FV-1`.

**Lectura bajo demanda:** [[.agents/knowledge/governance.md]] — `GOVERNANCE-1.7` es donde cualquier agente reconoce la frase de activación, sin importar el skill invocado.

---

## WRITE-LIKE-GEMINI-3 — Declaración Obligatoria por Ronda

**Aplica a:** todos los roles/skills, en cada respuesta o edición mientras un modo Gemini esté activo.

* **WRITE-LIKE-GEMINI-3 — Bracket por Ronda:** cada respuesta o edición hecha bajo un modo Gemini activo lleva su propio bracket, mismo formato de `GOVERNANCE-1.5`: `[WRITE-LIKE-GEMINI-N, sigo en modo <nombre>]`, con `N` = `-1.2` o `-1.3` según el modo activo. El modo Claude, al ser el estado de reposo, no se declara.

**Filtro de verificación:** `WRITE-LIKE-GEMINI-FV-2`.

---

## WRITE-LIKE-GEMINI-EJ — Ejemplos (Medición Real)

Tres casos reales, mismo contenido, escrito en los tres modos — relee estos antes de escribir en modo Gemini: son el patrón a replicar, no la regla a parafrasear.

### WRITE-LIKE-GEMINI-EJ-1 — Lista de Prohibiciones (demuestra `WRITE-LIKE-GEMINI-1.1` a `-1.3`)

* **Modo Claude (171 palabras / ~271 tokens):** "Existen varios patrones de escritura que debemos evitar de forma sistemática porque afectan negativamente la comprensión del usuario. En primer lugar, está prohibido el uso de jerga técnica densa sin una explicación adecuada que la acompañe, ya que esto obliga al lector a hacer un esfuerzo adicional de traducción mental. En segundo lugar, se debe evitar la prosa que rellena el texto sin aportar información real, como las frases de apertura aduladoras (por ejemplo, "¡Excelente pregunta!") o los cierres que simplemente repiten lo que ya se dijo antes. En tercer lugar, las nominalizaciones deben evitarse — por ejemplo, en vez de escribir "la consecución de la implementación", es preferible escribir directamente "implementar". En cuarto lugar, la voz pasiva debe usarse con moderación, prefiriendo siempre la voz activa cuando exista una forma más directa de decir lo mismo. Por último, las afirmaciones generales sin un caso concreto que las sostenga también están prohibidas, ya que dejan al lector sin manera de verificar si la afirmación es cierta o aplicable a su situación."

* **Modo Gemini Relajado (61 palabras / ~100 tokens, 63% menos):** "Evita jerga sin traducir — obliga al lector a traducir mentalmente. Nada de relleno cordial ("¡Excelente pregunta!") ni cierres que repiten lo ya dicho. Las nominalizaciones se cambian por el verbo directo: "implementar", no "la consecución de la implementación". Prefiere voz activa sobre pasiva cuando exista una forma más directa. Y ninguna afirmación general sin un caso concreto que la sostenga."

* **Modo Gemini Relajado + Listas (53 palabras / ~89 tokens, 67% menos):**
  ```
  Jerga sin traducir — obliga a traducir mentalmente. Evita:
  - Relleno cordial ("¡Excelente pregunta!") o cierres que repiten lo ya dicho.
  - Nominalizaciones: usa el verbo directo ("implementar", no "la consecución de la implementación").
  - Voz pasiva cuando existe una forma más directa.
  - Afirmaciones generales sin un caso concreto que las sostenga.
  ```

### WRITE-LIKE-GEMINI-EJ-2 — Protocolo de Pasos (demuestra `WRITE-LIKE-GEMINI-1.1` a `-1.3`)

* **Modo Claude (154 palabras / ~239 tokens):** "Cuando detectes un caso de bypass del provider, hay una serie de pasos que debes seguir de manera ordenada para asegurar que el problema quede correctamente documentado y resuelto. El primer paso consiste en reportar el hallazgo de inmediato al usuario, indicando de manera precisa el archivo y la línea exacta donde se encontró el problema, para que pueda ser localizado sin ambigüedad. El segundo paso es clasificar la gravedad del hallazgo según una escala de tres niveles: bypass del provider (que requiere corrección inmediata), duplicación de constantes (que requiere migración al token existente), o lógica de negocio en la UI (que requiere escalamiento al Tech Lead, dado que viola el principio de arquitectura FCIS). El tercer y último paso es proponer una solución concreta, aplicando el patrón correcto según el tipo de problema detectado, ya sea delegar al provider, usar un token existente, o mover la lógica a la capa de Rust correspondiente."

* **Modo Gemini Relajado (58 palabras / ~93 tokens, 61% menos):** "Al detectar un bypass del provider, sigue tres pasos. Primero, reporta el hallazgo al usuario con archivo y línea exactos. Segundo, clasifica la gravedad: bypass del provider (corregir de inmediato), constante duplicada (migrar al token), o lógica de negocio en la UI (escalar al Tech Lead, viola FCIS). Tercero, propón la solución concreta según el patrón que corresponda."

* **Modo Gemini Relajado + Listas (58 palabras / ~91 tokens, 62% menos):**
  ```
  Al detectar un bypass del provider, realiza:
  1. Reporta el hallazgo al usuario con archivo y línea exactos.
  2. Clasifica la gravedad: 🟡 bypass del provider (corregir de inmediato), 🟠 constante duplicada (migrar al token), 🔴 lógica de negocio en la UI (escalar al Tech Lead, viola FCIS).
  3. Propón la solución concreta según el patrón que corresponda.
  ```

### WRITE-LIKE-GEMINI-EJ-3 — Checklist de Auto-Validación (demuestra `WRITE-LIKE-GEMINI-1.1` a `-1.3`)

* **Modo Claude (193 palabras / ~285 tokens):** "Antes de entregar cualquier respuesta o realizar cualquier edición, es fundamental que te detengas a hacer una revisión mental de cinco preguntas clave. La primera pregunta que debes hacerte es si estás violando alguna de tus restricciones fundamentales, lo cual incluye cosas como el uso de pseudocódigo, rutas de archivo inventadas, o jerga técnica que no has explicado adecuadamente. La segunda pregunta es si existe alguna ambigüedad en lo que se te ha pedido — si es así, en vez de asumir algo, debes detenerte y preguntarle directamente al usuario qué es lo que realmente necesita. La tercera pregunta es si la acción que estás a punto de tomar fue realmente solicitada por el usuario, o si es algo que estás haciendo por iniciativa propia sin que nadie te lo haya pedido. La cuarta pregunta es si estás defendiendo adecuadamente los principios de arquitectura del proyecto en caso de que el usuario proponga algo que los contradiga. Y la quinta y última pregunta es una auditoría final en la que te preguntas si has alucinado información, si has simplificado por pereza, y si has seguido el cien por ciento de las reglas establecidas."

* **Modo Gemini Relajado (65 palabras / ~109 tokens, 62% menos):** "Antes de responder o editar, revisa cinco preguntas. Uno: ¿violo mis restricciones? (pseudocódigo, rutas inventadas, jerga sin explicar). Dos: ¿hay ambigüedad? Si asumo algo, pregunto en vez de adivinar. Tres: ¿es una acción que nadie pidió? Si es así, pido permiso primero. Cuatro: ¿estoy defendiendo la arquitectura si el usuario la contradice? Cinco, auditoría final: ¿alucinué algo, simplifiqué por pereza, o me salté alguna regla?"

* **Modo Gemini Relajado + Listas (58 palabras / ~100 tokens, 65% menos):**
  ```
  Antes de responder, verifica:
  1. ¿Violo mis restricciones? (pseudocódigo, rutas inventadas, jerga sin explicar).
  2. ¿Hay ambigüedad? Si asumo algo, pregunto en vez de adivinar.
  3. ¿Es una acción que nadie pidió? Pido permiso primero.
  4. ¿Estoy defendiendo la arquitectura si el usuario la contradice?
  5. Auditoría final: ¿alucinué algo, simplifiqué por pereza, o me salté alguna regla?
  ```

**Medición real (los tres casos completos, `wc -w -m`, no estimada):**

| Caso | Modo Claude | Modo Gemini Relajado | Modo Gemini Relajado + Listas |
|---|---|---|---|
| Prohibiciones | 171 palabras / ~271 tokens | 61 palabras / ~100 tokens (63% menos) | 53 palabras / ~89 tokens (67% menos) |
| Protocolo | 154 palabras / ~239 tokens | 58 palabras / ~93 tokens (61% menos) | 58 palabras / ~91 tokens (62% menos) |
| Checklist | 193 palabras / ~285 tokens | 65 palabras / ~109 tokens (62% menos) | 58 palabras / ~100 tokens (65% menos) |

Medido con `wc -w -m` sobre el texto real de cada variante — no estimado.
