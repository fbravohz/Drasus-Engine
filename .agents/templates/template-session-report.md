# Plantilla de Documento — Reporte de Sesión

Cómo estructurar un archivo de `.agents/sessions/` y cada una de sus entradas. Las reglas de comportamiento — qué va al archivo, qué queda en el chat, cuándo se omite una sección — viven en `.agents/knowledge/session-reporting.md`; esta plantilla solo da la forma copiable.

---

## Estructura del Archivo

Se crea una sola vez, al abrir la sesión. La cabecera se completa al abrir; el índice y el cierre crecen con la sesión.

```
# Sesión <YYYY-MM-DD> — <Título llano del tema>

| Campo | Valor |
|---|---|
| **Archivo** | `.agents/sessions/<YYYY-MM-DD>-<slug>.md` |
| **Abierta** | <YYYY-MM-DD HH:MM> |
| **Rol principal** | <Tech-Lead / Architect / etc.> |
| **Otros agentes** | <lista, o se omite la fila si no hubo> |
| **Estado** | 🟢 en curso · 🟡 esperando al usuario · 🔴 bloqueada · ⚪ cerrada |

---

## Índice

- [Turno 1 — <título corto>](#turno-1--hhmm--título-corto)
- [Turno 2 — <título corto>](#turno-2--hhmm--título-corto)

---

<aquí se acumulan las entradas, en orden>

---

## Cierre de sesión

**Estado final:** <una frase>

**Sin resolver:** <lista, o se omite si no queda nada>

**Siguiente paso:** <acción concreta>
```

---

## Estructura de una Entrada

Se añade una por turno del usuario, al final del archivo. El orden de las secciones es fijo; una sección sin contenido real se omite entera.

```
## Turno <N> — <HH:MM> — <Título corto>

### Qué pediste

<La petición en una o dos líneas, en tus palabras — no una transcripción literal.>

### Qué hice

<La narración que iría al chat: contexto primero, detalle después. Sin límite de
extensión — es una conversación escrita, no un resumen.>

### Archivos tocados

| Archivo | Qué cambió | Por qué |
|---|---|---|
| `ruta/al/archivo.ext` | <una línea> | <una línea> |
| `ruta/nueva.ext` *(nuevo)* | <una línea> | <una línea> |
| `ruta/vieja.ext` *(eliminado)* | <una línea> | <una línea> |

### Comprobado

<Qué se revisó y qué salió — la prueba de que lo afirmado funciona. En lenguaje
llano: "Suite de pruebas de interfaz | 37 de 37 en verde", no el comando pelado
ni vocabulario interno.>

| Qué revisé | Resultado |
|---|---|
| <qué se comprobó, legible sin contexto> | <resultado real, nunca el esperado> |

### Decisiones y por qué

- **<Decisión>:** <por qué esa y no la alternativa>

### Pendientes y bloqueos

- **<Pendiente>:** <qué falta y de quién depende>

### Preguntas para ti

- <Pregunta 1 — se repite en el chat, es la única sección duplicada a propósito>

### Otros

- <Lo que no encaja en ninguna sección de arriba>
```

---

## Estructura de la Sección de un Subagente

Un especialista despachado escribe dentro de la entrada del turno, con su propio encabezado de tercer nivel y las mismas secciones.

```
### <Rol del especialista> — <título de su encargo>

**Qué se le pidió:** <una línea>

**Qué encontró / qué hizo:** <su texto, escrito por él>

**Archivos tocados:** <tabla, con el mismo formato de arriba>

**Su conclusión:** <una línea>
```

Debajo, quien despachó añade su propia valoración como texto aparte — nunca editando el texto del especialista.

---

## Formato del Eco en el Chat

Tres líneas, siempre la misma forma. Es lo único que el usuario ve en la terminal cuando no hay preguntas.

```
<emoji de estado> <Estado> — <qué pasó, una frase>
<Una segunda frase solo si hay algo que el usuario deba saber ya>
→ .agents/sessions/<YYYY-MM-DD>-<slug>.md (turno <N>)
```

Emojis de estado, sin inventar otros: 🟢 hecho · 🟡 hecho con reservas o esperando al usuario · 🔴 detenido · ⚪ sin cambios.

---

## Checklist Antes de Guardar una Entrada

- [ ] ¿Escribí la entrada completa en el archivo antes de responder en el chat?
- [ ] ¿El encabezado lleva número de turno correlativo, hora y título corto?
- [ ] ¿Las secciones van en el orden fijo, y omití enteras las vacías en vez de escribir `N/A`?
- [ ] ¿Cada archivo tocado tiene su fila con qué cambió **y por qué**, sin volcados de código?
- [ ] ¿Los archivos nuevos o eliminados están marcados como tales?
- [ ] ¿La sección "Comprobado" dice en llano qué se revisó, con resultados reales y no los esperados?
- [ ] ¿Escribí en estilo Gemini desde el principio — hecho primero, cada dato una vez, sin metacomentario?
- [ ] ¿Las preguntas de la entrada están también en el chat?
- [ ] ¿Dejé intactas todas las entradas anteriores, escribiendo cualquier rectificación como entrada nueva?
- [ ] Si hubo subagentes, ¿cada uno tiene su sección con su nombre y su texto sin reescribir?
- [ ] ¿El eco del chat son tres líneas y no repite el cuerpo del reporte?
- [ ] ¿Actualicé el índice y el estado de la cabecera?
