# Comparación de Tokens — Estilo Claude vs Estilo Gemini

Mismo contenido, escrito dos veces, para medir gasto de texto real (no líneas).

---

## Ejemplo A — Lista de prohibiciones

### 🚫 Prohibido (estilo Claude)

Existen varios patrones de escritura que debemos evitar de forma sistemática porque afectan negativamente la comprensión del usuario. En primer lugar, está prohibido el uso de jerga técnica densa sin una explicación adecuada que la acompañe, ya que esto obliga al lector a hacer un esfuerzo adicional de traducción mental. En segundo lugar, se debe evitar la prosa que rellena el texto sin aportar información real, como las frases de apertura aduladoras (por ejemplo, "¡Excelente pregunta!") o los cierres que simplemente repiten lo que ya se dijo antes. En tercer lugar, las nominalizaciones deben evitarse — por ejemplo, en vez de escribir "la consecución de la implementación", es preferible escribir directamente "implementar". En cuarto lugar, la voz pasiva debe usarse con moderación, prefiriendo siempre la voz activa cuando exista una forma más directa de decir lo mismo. Por último, las afirmaciones generales sin un caso concreto que las sostenga también están prohibidas, ya que dejan al lector sin manera de verificar si la afirmación es cierta o aplicable a su situación.

### 🚫 Prohibido (estilo Gemini relajado, 60% menos)

Evita jerga sin traducir — obliga al lector a traducir mentalmente. Nada de relleno cordial ("¡Excelente pregunta!") ni cierres que repiten lo ya dicho. Las nominalizaciones se cambian por el verbo directo: "implementar", no "la consecución de la implementación". Prefiere voz activa sobre pasiva cuando exista una forma más directa. Y ninguna afirmación general sin un caso concreto que la sostenga.

### 🚫 Prohibido (relajado + listas)

Jerga sin traducir — obliga a traducir mentalmente. Evita:
- Relleno cordial ("¡Excelente pregunta!") o cierres que repiten lo ya dicho.
- Nominalizaciones: usa el verbo directo ("implementar", no "la consecución de la implementación").
- Voz pasiva cuando existe una forma más directa.
- Afirmaciones generales sin un caso concreto que las sostenga.

---

## Ejemplo B — Protocolo de pasos

### Protocolo de Actuación (estilo Claude)

Cuando detectes un caso de bypass del provider, hay una serie de pasos que debes seguir de manera ordenada para asegurar que el problema quede correctamente documentado y resuelto. El primer paso consiste en reportar el hallazgo de inmediato al usuario, indicando de manera precisa el archivo y la línea exacta donde se encontró el problema, para que pueda ser localizado sin ambigüedad. El segundo paso es clasificar la gravedad del hallazgo según una escala de tres niveles: bypass del provider (que requiere corrección inmediata), duplicación de constantes (que requiere migración al token existente), o lógica de negocio en la UI (que requiere escalamiento al Tech Lead, dado que viola el principio de arquitectura FCIS). El tercer y último paso es proponer una solución concreta, aplicando el patrón correcto según el tipo de problema detectado, ya sea delegar al provider, usar un token existente, o mover la lógica a la capa de Rust correspondiente.

### Protocolo de Actuación (estilo Gemini relajado, 60% menos)

Al detectar un bypass del provider, sigue tres pasos. Primero, reporta el hallazgo al usuario con archivo y línea exactos. Segundo, clasifica la gravedad: bypass del provider (corregir de inmediato), constante duplicada (migrar al token), o lógica de negocio en la UI (escalar al Tech Lead, viola FCIS). Tercero, propón la solución concreta según el patrón que corresponda.

### Protocolo de Actuación (relajado + listas)

Al detectar un bypass del provider, realiza:
1. Reporta el hallazgo al usuario con archivo y línea exactos.
2. Clasifica la gravedad: 🟡 bypass del provider (corregir de inmediato), 🟠 constante duplicada (migrar al token), 🔴 lógica de negocio en la UI (escalar al Tech Lead, viola FCIS).
3. Propón la solución concreta según el patrón que corresponda.

---

## Ejemplo C — Checklist de auto-validación

### Auto-Validación (estilo Claude)

Antes de entregar cualquier respuesta o realizar cualquier edición, es fundamental que te detengas a hacer una revisión mental de cinco preguntas clave. La primera pregunta que debes hacerte es si estás violando alguna de tus restricciones fundamentales, lo cual incluye cosas como el uso de pseudocódigo, rutas de archivo inventadas, o jerga técnica que no has explicado adecuadamente. La segunda pregunta es si existe alguna ambigüedad en lo que se te ha pedido — si es así, en vez de asumir algo, debes detenerte y preguntarle directamente al usuario qué es lo que realmente necesita. La tercera pregunta es si la acción que estás a punto de tomar fue realmente solicitada por el usuario, o si es algo que estás haciendo por iniciativa propia sin que nadie te lo haya pedido. La cuarta pregunta es si estás defendiendo adecuadamente los principios de arquitectura del proyecto en caso de que el usuario proponga algo que los contradiga. Y la quinta y última pregunta es una auditoría final en la que te preguntas si has alucinado información, si has simplificado por pereza, y si has seguido el cien por ciento de las reglas establecidas.

### Auto-Validación (estilo Gemini relajado, 60% menos)

Antes de responder o editar, revisa cinco preguntas. Uno: ¿violo mis restricciones? (pseudocódigo, rutas inventadas, jerga sin explicar). Dos: ¿hay ambigüedad? Si asumo algo, pregunto en vez de adivinar. Tres: ¿es una acción que nadie pidió? Si es así, pido permiso primero. Cuatro: ¿estoy defendiendo la arquitectura si el usuario la contradice? Cinco, auditoría final: ¿alucinué algo, simplifiqué por pereza, o me salté alguna regla?

### Auto-Validación (relajado + listas)

Antes de responder, verifica:
1. ¿Violo mis restricciones? (pseudocódigo, rutas inventadas, jerga sin explicar).
2. ¿Hay ambigüedad? Si asumo algo, pregunto en vez de adivinar.
3. ¿Es una acción que nadie pidió? Pido permiso primero.
4. ¿Estoy defendiendo la arquitectura si el usuario la contradice?
5. Auditoría final: ¿alucinué algo, simplifiqué por pereza, o me salté alguna regla?

---

## Medición Real (no estimada)

| Ejemplo | Estilo Claude | Relajado + Listas con contexto (~65% menos) | Gemini Relajado (~61% menos) |
|---|---|---|---|
| A — Prohibiciones | 171 palabras / ~271 tokens | 53 palabras / ~89 tokens (67%) | 61 palabras / ~100 tokens (63%) |
| B — Protocolo | 154 palabras / ~239 tokens | 58 palabras / ~91 tokens (62%) | 58 palabras / ~93 tokens (61%) |
| C — Checklist | 193 palabras / ~285 tokens | 58 palabras / ~100 tokens (65%) | 65 palabras / ~109 tokens (62%) |
