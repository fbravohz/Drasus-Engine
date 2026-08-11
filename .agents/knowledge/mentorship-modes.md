# Modos de Acompañamiento y Protocolo de Lecciones (ADR-0120, ADR-0122, ADR-0124)

Cómo enseñar durante el desarrollo cuando un Ingeniero opera en Modo Mentor, Revisión o Docente, y cómo consolidar lo enseñado para sesiones futuras.

Slug ID: `MENTORSHIP-MODES`.

Tamaño: `3922 caracteres (~980 tokens aprox., ratio 4:1)`.

---

## MENTORSHIP-MODES-FV — Filtro de Verificación

**Aplica a:** los seis Ingenieros, como último paso en Modo Mentor/Revisión/Docente.

* **MENTORSHIP-MODES-FV-1:** ¿expliqué desde cero-conocimiento, sin asumir nada previo (`MENTORSHIP-MODES-1`)?
* **MENTORSHIP-MODES-FV-2:** en Modo Docente, ¿pausé a enseñar antes de avanzar de bloque y agoté las dudas (`MENTORSHIP-MODES-2`)?
* **MENTORSHIP-MODES-FV-3:** ¿consolidé la lección en el archivo de la Story, citando código real y con enlace bidireccional (`MENTORSHIP-MODES-3`)?

De ser satisfactoria cada validación de la lista, declara `MENTORSHIP-MODES - FILTRO DE VERIFICACION APROBADO`.

---

## MENTORSHIP-MODES-1 — Profundidad Cero-Conocimiento (Fijo)

**Aplica a:** los seis Ingenieros (Rust, Flutter, Bridge, QA, Quant, Refactoring) en Modos Mentor/Revisión/Docente.

* **MENTORSHIP-MODES-1.1 — Nunca Asumir Conocimiento Previo:**

  ninguna explicación debe dar por sentado conocimientos previos del lenguaje, framework o disciplina. Se explica desde la base (qué es, por qué existe, qué problema resuelve) antes de aplicarlo al código real. "El usuario ya debe saber esto" nunca es una suposición válida en estos tres Modos.
  * ❌ *Incorrecto:* aplicar un patrón sin explicar qué problema resuelve.
  * ✅ *Correcto:* explicar el problema, luego el patrón, luego el código.

**Filtro de verificación:** `MENTORSHIP-MODES-FV-1`.

---

## MENTORSHIP-MODES-2 — Modo Docente (ADR-0122)

**Aplica a:** los seis Ingenieros, cuando la Orden de Trabajo declara Modo Docente.

* **MENTORSHIP-MODES-2.1 — Implementar, Pausar, Enseñar:**

  el agente implementa el bloque completo de forma autónoma (`Edit`/`Write`). Antes de avanzar al siguiente bloque, se detiene y enseña: explica cada decisión de diseño bajo el principio de cero-conocimiento (`MENTORSHIP-MODES-1`). Invita y responde preguntas del usuario sobre el código escrito. No avanza al siguiente bloque sin agotar las dudas del actual. Granularidad: un bloque lógico (función/struct/widget/fórmula) a la vez.
  * ❌ *Incorrecto:* avanzar al siguiente bloque con una duda del usuario sin resolver.
  * ✅ *Correcto:* agotar las dudas del bloque actual antes de escribir el siguiente.

**Filtro de verificación:** `MENTORSHIP-MODES-FV-2`.

---

## MENTORSHIP-MODES-3 — Protocolo de Lecciones (`docs/lessons/`, ADR-0124)

**Aplica a:** los seis Ingenieros, en Modos Mentor, Revisión o Docente.

* **MENTORSHIP-MODES-3.1 — Un Archivo por Story/Task, Nunca por Tema:**

  toda explicación se destila en un archivo reusable que consolida TODO lo enseñado durante una Story/Task concreta — no por tema de lenguaje suelto. Organización: un archivo por Story/Task bajo la subcarpeta del ingeniero en `docs/lessons/`, con el mismo ID que su Orden de Trabajo (ej. `STORY-007-telemetry.md`).
  * ❌ *Incorrecto:* crear un archivo de lección nuevo para una Story que ya tiene uno.
  * ✅ *Correcto:* extender el archivo existente al final, sin duplicar contenido.

* **MENTORSHIP-MODES-3.2 — Contenido Real, Enlace Bidireccional:**

  cada concepto cita el código real producido (ruta y fragmento), nunca ejemplos genéricos de manual. Enlace bidireccional obligatorio con `docs/execution/<ID>.md`. Estructura mínima: `## Concepto` (una subsección por concepto, anclada a código real) y `## Trucos de Senior` (azúcar sintáctica o atajos reales de esa Story, sin relleno).
  * ❌ *Incorrecto:* explicar un concepto con un ejemplo de manual en vez del código real de la Story.
  * ✅ *Correcto:* citar archivo y fragmento exacto que la Story produjo.

**Filtro de verificación:** `MENTORSHIP-MODES-FV-3`.
