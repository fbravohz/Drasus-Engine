# Cierre de Trabajo — Sellado, Validación y Diferidos

Tres reglas obligatorias al cerrar cualquier unidad de trabajo — un TTR, una Feature, un Módulo, una Story, o una decisión de ADR llevada a código.

Slug ID: `CLOSING-WORK`.

Tamaño: `2868 caracteres (~717 tokens aprox., ratio 4:1)`.

---

## CLOSING-WORK-FV — Filtro de Verificación

**Aplica a:** todos los roles, como último paso al cerrar cualquier unidad de trabajo.

* **CLOSING-WORK-FV-1:** ¿sellé con fecha solo lo verificado, entregué comandos de validación copiables, y registré todo diferido en su lugar canónico con disparador (`CLOSING-WORK-1`)?

De ser satisfactoria cada validación de la lista, declara `CLOSING-WORK - FILTRO DE VERIFICACION APROBADO`.

---

## CLOSING-WORK-1 — Sellado, Validación y Trabajo Diferido

**Aplica a:** todos los roles, al cerrar cualquier unidad de trabajo.

* **CLOSING-WORK-1.1 — Sellar lo Implementado (con fecha):**

  al completar un TTR, Feature, Módulo o decisión de ADR, añade un sello visible en su documento fuente: `> ✅ **Implementado** YYYY-MM-DD · Orden de trabajo [<ID>](../execution/<ID>-<slug>.md)`, o si es parcial: `> 🟡 **Parcial** YYYY-MM-DD · <qué falta> · Orden [<ID>](...)`. Nunca marques implementado algo que no verificaste.
  * ❌ *Incorrecto:* marcar como implementado algo que no se corrió ni se verificó.
  * ✅ *Correcto:* sello con fecha solo después de verificar el resultado.

* **CLOSING-WORK-1.2 — Entrega de Comandos de Validación:**

  al cerrar cualquier trabajo, entrega al usuario los comandos exactos (listos para copiar y pegar) para reproducir y validar de forma autónoma (builds, tests, lints). Quedan registrados también en la sección 5 de la Orden de Trabajo.
  * ❌ *Incorrecto:* cerrar una Story sin entregar los comandos de validación.
  * ✅ *Correcto:* comandos de `cargo test`/`flutter test` listos para copiar, en la Orden de Trabajo.

* **CLOSING-WORK-1.3 — Registro Obligatorio de Trabajo Diferido:**

  prohibido dejar un aplazamiento vivo únicamente en la conversación o en el razonamiento del agente. Se registra en su lugar canónico junto con su disparador: fase o entrega nueva/aplazada → [[docs/ROADMAP.md]] (dominio del Architect); deuda técnica granular deliberada → [[docs/DEBT.md]] como `DEBT-XXX` (dominio del Tech-Lead, ver [[.agents/knowledge/debt-management.md]]); pendiente acotado a una feature → banner `> 🟡 **Parcial** … · Pendiente: …` en `docs/features/*.md`, que luego se enrola en [[docs/DEBT.md]].
  * ❌ *Incorrecto:* mencionar en el chat que algo queda pendiente sin registrarlo en su lugar canónico.
  * ✅ *Correcto:* `DEBT-XXX` en [[docs/DEBT.md]] con causa raíz y disparador, antes de cerrar la conversación.

**Filtro de verificación:** `CLOSING-WORK-FV-1`.

**Lectura bajo demanda:** [[.agents/knowledge/debt-management.md]] — al registrar deuda técnica granular deliberada (`DEBT-XXX`).
