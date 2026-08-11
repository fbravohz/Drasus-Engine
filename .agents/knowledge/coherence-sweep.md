# Barrido de Coherencia Pre-Orden

Qué se barre, cómo se responde y qué se entrega antes de redactar una Orden de Trabajo, para que ningún ingeniero improvise sobre una decisión que ya está tomada por escrito.

Slug ID: `COHERENCE-SWEEP`.

Tamaño: `6729 caracteres (~1682 tokens aprox., ratio 4:1)`.

---

## COHERENCE-SWEEP-0 — Lecturas Obligatorias

| Documento | De qué va | Cuándo leerlo (disparador) | Alcance |
|---|---|---|---|
| [[.agents/knowledge/governance.md]] | Gobernanza meta, supremacía absoluta. | `SIEMPRE` al iniciar cualquier skill. | Todos los agentes |
| [[.agents/knowledge/doc-first.md]] | Cómo se busca en `docs/` antes de proponer, y por qué "no encontré nada" solo vale declarando dónde se buscó. | `SIEMPRE` durante el barrido — es el método de búsqueda que este protocolo aplica. | Architect, TL |

---

## COHERENCE-SWEEP-FV — Filtro de Verificación

**Aplica a:** el Architect al entregar el barrido, y al Tech-Lead antes de redactar la Orden.

* **COHERENCE-SWEEP-FV-1:** ¿el barrido corrió para esta feature, sin omitirlo por parecer simple? (`COHERENCE-SWEEP-1`)
* **COHERENCE-SWEEP-FV-2:** ¿arranqué desde los hallazgos del especialista de dominio, en vez de desde cero? (`COHERENCE-SWEEP-2`)
* **COHERENCE-SWEEP-FV-3:** ¿cada respuesta cita documento y línea, sin una sola respondida de memoria? (`COHERENCE-SWEEP-3.3`)
* **COHERENCE-SWEEP-FV-4:** ¿clasifiqué lo no resuelto en los tres cubos, sin mezclarlos ni tapar una contradicción? (`COHERENCE-SWEEP-4`)
* **COHERENCE-SWEEP-FV-5:** si quedó un bloqueo duro sin resolver, ¿paré en vez de redactar la Orden? (`COHERENCE-SWEEP-5`)

De ser satisfactoria cada validación de la lista, declara `COHERENCE-SWEEP - FILTRO DE VERIFICACION APROBADO`.

---

## COHERENCE-SWEEP-1 — Activación Obligatoria, No Reactiva

El barrido corre para **toda** feature o TTR, antes de que se escriba una sola línea de su Orden de Trabajo.

**Aplica a:** el Tech-Lead, que lo despacha, y el Architect, que lo ejecuta.

* **COHERENCE-SWEEP-1.1 — Sin Excepciones:**

  no es opcional y no se omite porque la feature parezca simple. Se despacha inmediatamente después del veredicto de viabilidad experta.
  * ❌ *Incorrecto:* saltar el barrido en una feature de superficie pequeña y redactar la Orden directo.
  * ✅ *Correcto:* despacharlo siempre, incluso cuando el alcance se ve trivial.

* **COHERENCE-SWEEP-1.2 — El Architect No Está Dormido Aquí:**

  es la única etapa en que el Architect se activa por defecto, sin esperar un escalamiento. Fuera de ella vuelve a estado pasivo.

**Filtro de verificación:** `COHERENCE-SWEEP-FV-1`.

---

## COHERENCE-SWEEP-2 — Insumo Obligatorio del Especialista de Dominio

El barrido no arranca de cero: parte de los hallazgos del especialista que ya revisó la feature.

**Aplica a:** el Architect, al recibir el despacho.

* **COHERENCE-SWEEP-2.1 — Los Hallazgos Viajan Íntegros:**

  el Tech-Lead despacha primero al especialista de dominio (Quant-Engineer para materia cuantitativa) y entrega sus preguntas y hallazgos completos, sin resumir. El trabajo del Architect empieza donde termina el suyo: confirma lo que citó, corrige lo que citó mal, y añade lo que el especialista no puede ver — coherencia entre ADRs, propiedad de datos, secuenciación, contradicciones cruzadas entre documentos.

* **COHERENCE-SWEEP-2.2 — Despacho Sin Hallazgos:**

  si el despacho llega sin ellos y la feature toca dominio especializado, pídelos antes de barrer. Repetir ese trabajo es caro; perderlo es peor.

**Filtro de verificación:** `COHERENCE-SWEEP-FV-2`.

---

## COHERENCE-SWEEP-3 — Qué se Hace

Cinco pasos en orden.

**Aplica a:** el Architect, durante el barrido.

* **COHERENCE-SWEEP-3.1 — Recorrer lo Ya Decidido:**

  lee la feature y sus TTRs, y recorre el índice de ADRs, el SAD, las features relacionadas y el modelo de datos buscando todo lo ya decidido que toca este trabajo.

* **COHERENCE-SWEEP-3.2 — Producir las Preguntas que Llevan a Improvisar:**

  no las obvias. Cuatro categorías:
  - **Fronteras de fase:** ¿esto es materia prima o cálculo derivado? ¿pertenece a esta fase del pipeline o a una posterior?
  - **Propiedad del dato:** ¿quién es dueño de este campo? ¿quién puede escribirlo?
  - **Convenciones no escritas:** ¿qué se asume aquí que nadie ha declarado?
  - **Casos borde del dominio:** ¿qué pasa con el activo raro, la sesión partida, el símbolo con formato distinto?

* **COHERENCE-SWEEP-3.3 — Responder Citando Documento y Línea:**

  una respuesta "de memoria" o "por sentido común" no cuenta. Si no hay documento que la sostenga, la pregunta sigue abierta y pasa a `COHERENCE-SWEEP-4`.
  * ❌ *Incorrecto:* "esto normalmente se resuelve calculándolo en ingesta" sin cita.
  * ✅ *Correcto:* "ADR-XXXX lo declara FIJO, línea N" — o dejarla abierta.

* **COHERENCE-SWEEP-3.4 — Señalar lo Ya Especificado que Nadie Recuerda:**

  para que la Orden no mande construir algo ya decidido ni lo contradiga.

**Filtro de verificación:** `COHERENCE-SWEEP-FV-3`.

---

## COHERENCE-SWEEP-4 — Los Tres Cubos

Todo lo que quedó sin respuesta se clasifica en exactamente uno de tres cubos, sin mezclarlos.

**Aplica a:** el Architect, al cerrar el barrido.

* **COHERENCE-SWEEP-4.1 — Resoluble por el Architect:**

  lo canoniza ahora, como enmienda o ADR nuevo, dentro de esta misma etapa.

* **COHERENCE-SWEEP-4.2 — Contradicción entre Documentos Vigentes:**

  se reporta al Tech-Lead sin taparla. Él decide o escala.

* **COHERENCE-SWEEP-4.3 — Bloqueo Duro de Producto o Negocio:**

  se para y se consulta al propietario. Jamás se improvisa una decisión de producto para desbloquear una implementación — es la regla que esta etapa existe para hacer cumplir.

**Filtro de verificación:** `COHERENCE-SWEEP-FV-4`.

---

## COHERENCE-SWEEP-5 — Entregable y Regla de Bloqueo

**Aplica a:** el Architect al entregar, y al Tech-Lead al recibir.

* **COHERENCE-SWEEP-5.1 — Entregable:**

  una sección lista para pegar en la Orden de Trabajo, titulada **"Preguntas resueltas antes de implementar"**: cada pregunta, su respuesta, y la cita del documento que la sostiene. Esa sección viaja con el despacho al ingeniero — es literalmente lo que le impide improvisar.

* **COHERENCE-SWEEP-5.2 — Bloqueo Duro Sin Resolver Detiene la Orden:**

  si el barrido devuelve un bloqueo duro sin resolver, el Tech-Lead **no redacta la Orden**. Consulta al propietario primero. Redactar una Orden sobre una pregunta abierta es exactamente el error que esta etapa existe para impedir.

**Precedencia:** `COHERENCE-SWEEP-5.2` gana sobre cualquier presión de calendario o de alcance — una Orden redactada sobre una pregunta abierta se deshace y se rehace, y cuesta el doble.

**Filtro de verificación:** `COHERENCE-SWEEP-FV-5`.
