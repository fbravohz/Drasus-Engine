# Brief — Cierre Ejecutivo de Story

Guía para redactar `§9` de la Orden de Trabajo (Cierre Ejecutivo) aplicando el "Formato de Reporte al CEO" ya definido en [[.agents/skills/tech-lead/SKILL.md]] a los datos de cierre de una Story concreta — no define un formato nuevo, dice qué leer y cómo traducirlo bien en este caso puntual.

Slug ID: `BRIEF`.

Tamaño: `3301 caracteres (~825 tokens aprox., ratio 4:1)`.

---

## BRIEF-FV — Filtro de Verificación

**Aplica a:** Tech-Lead, como último paso antes de escribir `§9` de la Orden de Trabajo.

* **BRIEF-FV-1:** ¿leí las 4 fuentes de entrada en el orden indicado antes de redactar (`BRIEF-1`)?
* **BRIEF-FV-2:** ¿mapeé cada campo del Reporte al CEO según su regla de origen, sin inventar contenido (`BRIEF-2`)?
* **BRIEF-FV-3:** ¿escribí el bloque una sola vez en `§9` y lo reproduje literal como mensaje de cierre (`BRIEF-3`)?

De ser satisfactoria cada validación de la lista, declara `BRIEF - FILTRO DE VERIFICACION APROBADO`.

---

## BRIEF-1 — Entrada: Qué Leer, En Este Orden

Este documento no audita ni verifica nada — asume que las 4 fuentes ya están redactadas.

**Aplica a:** Tech-Lead, antes de redactar `§9` de la Orden de Trabajo.

* **BRIEF-1.1 — Registro de Ejecución:** `§7` de la Orden — qué se entregó, tu auditoría, veredicto QA/Quant.

* **BRIEF-1.2 — Pendientes:** `§8` de la Orden — pendientes derivados/decisiones.

* **BRIEF-1.3 — Progreso:** la entrada nueva de esta Story en [[.agents/state/tech-lead/PROGRESS.md]].

* **BRIEF-1.4 — Deuda:** los deltas de [[docs/DEBT.md]] de este cierre.

**Lectura bajo demanda:** [[.agents/skills/tech-lead/SKILL.md]] — para el "Formato de Reporte al CEO" completo, definido ahí; este documento solo mapea sus 4 campos a los datos de cierre.

---

## BRIEF-2 — Cómo Mapear al Formato de Reporte al CEO

Traduce las 4 fuentes de `BRIEF-1` a los 4 campos del Formato de Reporte al CEO.

**Aplica a:** Tech-Lead, al redactar `§9`.

* **BRIEF-2.1 — ESTADO:**

  🟢 si la Story cerró completa con APTO; 🟡 si quedó parcial o generó una decisión pendiente; 🔴 si quedó bloqueada.

* **BRIEF-2.2 — PROGRESO MACRO:**

  la traducción a negocio de lo que dice `§7` — no el veredicto técnico, la capacidad nueva que queda disponible.

* **BRIEF-2.3 — FRICCIONES Y DEUDA:**

  si [[docs/DEBT.md]] no tuvo cambios de esta Story, omite la sección entera. Si los tuvo, una frase de IMPACTO por cada `DEBT-XXX` (qué podría salir mal, bajo qué condición, hacia cuándo pagarlo) — nunca el ID crudo ni el emoji de severidad solo. Cero falsa alarma, cero falsa calma: el nivel de preocupación debe calzar con la severidad real — 🟡 no se infla a "grave", 🔴 no se disuelve en "detalle menor".
  * ❌ *Incorrecto:* listar "DEBT-014, DEBT-015" sin explicar impacto.
  * ✅ *Correcto:* "el guantelete usa placeholders para 3 tipos hasta que #7 entregue el real — sin impacto en producción hoy."

* **BRIEF-2.4 — INPUT REQUERIDO DEL CEO:**

  viene de `§8` si ahí quedó algo pendiente de decidir; si no, `"Ninguna."` — nunca se omite este campo, a diferencia de FRICCIONES.

---

## BRIEF-3 — Salida

* **BRIEF-3.1 — Escritura Única, Reproducción Literal:** el bloque resultante se escribe UNA vez en `§9` de la Orden de Trabajo y se reproduce ahí, literal y sin edición, como mensaje final de chat al cerrar la Story.
