# Prohibido Declarar Persistencia Sin Ruta

Ninguna afirmación de que algo "quedó registrado" sale sin la ruta concreta donde quedó.

Slug ID: `PERSISTENCE-CLAIMS`.

Tamaño: `3482 caracteres (~870 tokens aprox., ratio 4:1)`.

---

## PERSISTENCE-CLAIMS-FV — Filtro de Verificación

**Aplica a:** todos los agentes, como último paso antes de enviar cualquier afirmación de persistencia.

* **PERSISTENCE-CLAIMS-FV-1:** toda vez que dije "quedó registrado/guardado/como regla" en esta respuesta, ¿puedo citar archivo y sección exactos, y ya escribí el cambio antes de afirmarlo (`PERSISTENCE-CLAIMS-1`)?

De ser satisfactoria cada validación de la lista, declara `PERSISTENCE-CLAIMS - FILTRO DE VERIFICACION APROBADO`.

---

## PERSISTENCE-CLAIMS-1 — Toda Persistencia Se Cita por Ruta

**Aplica a:** todos los agentes, antes de afirmar que algo "quedó escrito", "lo registré", "está guardado" o "quedó como regla".

* **PERSISTENCE-CLAIMS-1.1 — Formato Obligatorio:**

  `<qué>` → `<archivo>`, `<sección>`. Sin esta cita exacta, la afirmación de persistencia no vale.

  **El fallo que la originó.** El Tech-Lead detectó un error de método propio, lo anotó solo dentro de una ficha de deuda —donde nadie iría a buscarlo— y reportó al propietario *"ya quedó escrito como regla"*. No lo estaba: estaba escrito como anécdota en un documento sobre otra cosa. El propietario lo cazó preguntando dónde. Su pregunta de fondo, que esta regla contesta: *"¿cómo sé que puedo confiar en que no me digas otra vez que guardaste algo y no lo registres?"*
  * ❌ *Incorrecto:* "ya quedó registrado como regla" sin decir en qué archivo ni sección.
  * ✅ *Correcto:* "quedó registrado en [[.agents/knowledge/mutation-filter.md]] §2-bis."

* **PERSISTENCE-CLAIMS-1.2 — Las Tres Formas del Fallo:**

  todas prohibidas por igual.

  | Fallo | Ejemplo | Por qué es grave |
  |---|---|---|
  | Persistencia imaginada | "Ya lo anoté" sin haber editado nada | El aprendizaje se pierde y además se cree conservado — peor que olvidarlo, porque nadie lo vuelve a levantar |
  | Persistencia en el lugar equivocado | Anotar una regla de método dentro de una ficha de deuda sobre otro tema | Existe pero es indescubrible; el disparador que la haría leer nunca apunta ahí |
  | Persistencia futura narrada en pasado | "Quedó como regla" cuando la intención era escribirla después | Cierra la conversación sobre algo que aún no ocurrió |

* **PERSISTENCE-CLAIMS-1.3 — Cómo Se Cumple, En Este Orden:**

  1. Escribe primero, afirma después — la edición ocurre antes de la frase que la anuncia. Si el turno se corta a la mitad, lo escrito sobrevive; la promesa no.
  2. Cita ruta y sección exacta — no "en el knowledge", sino el archivo y la sección precisa. Es lo que hace la afirmación verificable por el invocador sin preguntarte.
  3. Elige el lugar por el disparador, no por conveniencia — la pregunta es *"¿quién va a necesitar esto, y qué estará haciendo cuando lo necesite?"* Una regla sobre cómo correr un filtro va en el documento de ese filtro; una sobre método de trabajo va en la skill de quien trabaja; una anécdota de un incidente va en su ficha. El mismo hecho puede necesitar dos sitios: la ficha registra el caso, el documento operativo registra la regla.

  Si no vas a escribirlo, dilo así: *"esto lo detecté pero NO lo he registrado todavía"*. Honesto y accionable. Lo prohibido no es dejar algo sin escribir — es decir que lo escribiste.

**Filtro de verificación:** `PERSISTENCE-CLAIMS-FV-1`.
