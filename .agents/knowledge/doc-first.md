# Doc-First — Buscar Antes de Decidir

*"Lo que ya está decidido no se vuelve a decidir. Y no saber que estaba decidido no es excusa: es no haber buscado."* Regla del usuario, 2026-07-19, nacida de tres fallos en una sola sesión: un agente propuso, debatió o construyó algo que ya estaba resuelto en la documentación, sin haber buscado. El precio se paga dos veces — construirlo y deshacerlo.

Slug ID: `DOC-FIRST`.

Tamaño: `2496 caracteres (~624 tokens aprox., ratio 4:1)`.

---

## DOC-FIRST-1 — Búsqueda Obligatoria Antes de Proponer

**Aplica a:** todos los agentes, antes de proponer una regla, un contrato, una fórmula o un cambio de alcance.

* **DOC-FIRST-1.1 — Buscar con `grep`, No de Memoria:**

  antes de proponer, busca si ya está decidido — con `grep` sobre `docs/`, no de memoria. Es barato: una búsqueda cuesta menos que un retrofit.
  * ❌ *Incorrecto:* "Recomiendo fijar como contrato que el motor use el cierre de la barra." ← esto ya existía; nadie buscó.
  * ✅ *Correcto:* "Aplica ADR-0114 (Bar-Open Alignment) — la señal del cierre de N ejecuta en la apertura de N+1. Ya es fijo, no hay nada que decidir."

* **DOC-FIRST-1.2 — Declarar Dónde Se Buscó:**

  toda propuesta de dominio o arquitectura cita el documento que la sostiene o la contradice. "No encontré nada" solo vale si dices dónde buscaste — una afirmación de vacío sin búsqueda declarada es una alucinación de ausencia, tan cara como inventar una regla.
  * ❌ *Incorrecto:* "no hay nada documentado sobre esto" sin decir en qué carpetas se buscó.
  * ✅ *Correcto:* "Sin cobertura documental: busqué en `docs/adr/`, `docs/features/` y `docs/sad/` con `<patrón>`. Cero coincidencias → pregunta abierta."

* **DOC-FIRST-1.3 — Lo Que Este Gate No Prohíbe:**

  no prohíbe improvisar. La iniciativa ha producido aciertos reales. Lo que prohíbe es improvisar sobre preguntas que ya tienen respuesta escrita — terreno virgen documentado como virgen, se puede improvisar y decirlo.

**Filtro de verificación:** `DOC-FIRST-2.1`.

**Documento extendido:** `.agents/knowledge/critical-domain-reasoning.md` — si la propuesta además sella una decisión de dominio cuant/estadístico/financiero/legal, aplica ese gate adicional.

---

## DOC-FIRST-2 — Filtro de Verificación

**Aplica a:** todos los agentes, como último paso antes de proponer una regla, contrato, fórmula o cambio de alcance.

* **DOC-FIRST-2.1:** ¿busqué con `grep` sobre `docs/` antes de proponer, y declaré dónde busqué (encontrara algo o no) (`DOC-FIRST-1`)?
