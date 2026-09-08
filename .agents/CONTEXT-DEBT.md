# CONTEXT-DEBT.md — Índice del Registro de Deuda de Agentes/Documentación

> **Propósito:** el **único lugar descubrible** donde vive la deuda de gobernanza de agentes — skills, knowledge docs, hooks, plantillas — deliberadamente aplazada. Registro hermano de [[docs/DEBT.md]] (deuda de código/producto): mismo patrón, mismo protocolo (`DEBT-MANAGEMENT-11`), dominio distinto.
> **Estructura (patrón ADR):** este archivo es el **índice**; la ficha completa de cada deuda vive en `.agents/debt/CONTEXT-DEBT-XXX.md`. Para leer una deuda concreta, abre su archivo — no hace falta cargar todo el registro.
> **Mantenimiento:** cualquier rol crea/cierra la ficha en `.agents/debt/` al abrir o pagar una deuda y actualiza la fila del índice. Estado: `Abierta` · `En pago` · `Pagada` (con enlace a la Story o edición que la saldó).

## Convención de severidad

| Severidad | Significado |
|---|---|
| 🔴 Alta | Contradice o viola una regla de gobernanza vigente bajo condiciones alcanzables hoy; pagar pronto. |
| 🟠 Media | Sin violación activa, pero pérdida de función/corrección o riesgo de desincronización bajo condiciones aún no presentes; pagar antes de que la condición llegue. |
| 🟡 Baja | Cosmético / diferido por secuenciación; sin riesgo de corrección ni de gobernanza. |

---

## Índice

| ID | Severidad | Resumen | Estado | Ficha |
|---|---|---|---|---|
| CONTEXT-DEBT-001 | 🔴 Alta | Skills en formato legacy que bloqueaban el hook `validate-agent-docs.py` — los 6 originales, incluido `rust-engineer`, ya canonizados | ✅ Pagada | [detalle](./debt/CONTEXT-DEBT-001.md) |
| CONTEXT-DEBT-002 | 🔴 Alta | `debt-management.md` referencia una lectura de `.agents/memory/` que `memory-policy.md` prohíbe expresamente | ✅ Pagada | [detalle](./debt/CONTEXT-DEBT-002.md) |
| CONTEXT-DEBT-003 | 🟠 Media | `ui-designer/SKILL.md` — pérdida de salvaguarda FFI, prohibición absoluta debilitada a condicional, ADRs vinculantes perdidos | ✅ Pagada | [detalle](./debt/CONTEXT-DEBT-003.md) |
| CONTEXT-DEBT-004 | 🟡 Baja | 5 knowledge docs nuevos huérfanos — ningún skill los cita por wikilink, la copia vieja duplicada sigue siendo la que se lee | 🟠 Parcial | [detalle](./debt/CONTEXT-DEBT-004.md) |
| CONTEXT-DEBT-005 | 🟡 Baja | Referencias muertas a `ui/COMPONENTS.md` en `flutter-engineer` y `ui-designer` tras el reset `ui/`→`ui-old/` | ✅ Pagada | [detalle](./debt/CONTEXT-DEBT-005.md) |
| CONTEXT-DEBT-006 | 🟡 Baja | IDs de ítem que reutilizan el ID del bloque padre en `commenting-policy.md` y `partner/SKILL.md` | ✅ Pagada | [detalle](./debt/CONTEXT-DEBT-006.md) |
| CONTEXT-DEBT-007 | 🟠 Media | Etapas de `tech-lead/SKILL.md` mal numeradas — bloque obligatorio sin número entre Etapa 6 y 7; revisión completa pendiente | ✅ Pagada | [detalle](./debt/CONTEXT-DEBT-007.md) |
| CONTEXT-DEBT-008 | 🟡 Baja | `qa-engineer/SKILL.md` en formato legacy — sin bloque `-FV`, rutas sin wikilink, lenguaje de bitácora (avisos blandos del hook) | ✅ Pagada | [detalle](./debt/CONTEXT-DEBT-008.md) |
| CONTEXT-DEBT-009 | 🟡 Baja | `brief.md` referencia secciones de la Orden de Trabajo con el numeral `§N`, notación baneada — falta criterio de reemplazo | Abierta | [detalle](./debt/CONTEXT-DEBT-009.md) |
| CONTEXT-DEBT-010 | 🟡 Baja | Citas huérfanas a sub-IDs de `CONTEXT-ENGINEER` retirados por el aligeramiento de formato del skill (7 archivos) | Abierta | [detalle](./debt/CONTEXT-DEBT-010.md) |
| CONTEXT-DEBT-011 | 🟠 Media | `hard-prohibitions.md` declarado `SIEMPRE` sin ningún paso que fuerce su lectura, en 14 skills y 16 stubs | ✅ Pagada | [detalle](./debt/CONTEXT-DEBT-011.md) |
| CONTEXT-DEBT-012 | 🟡 Baja | Cita huérfana a sub-ID de `TECH-LEAD` retirado por la migración al formato ligero del skill (1 archivo) | Abierta | [detalle](./debt/CONTEXT-DEBT-012.md) |

---

## Deudas pagadas (resumen)

Registro abierto el 2026-08-20 a partir de la auditoría del commit `972bab5` y del worktree de `.agents/` en curso ese día.

- **CONTEXT-DEBT-002** — la contradicción entre `debt-management.md` y la suspensión de memoria quedó resuelta marcando el puntero como histórico suspendido.
- **CONTEXT-DEBT-003** — `ui-designer` recupera la verificación de binding FFI, la prohibición absoluta y las tres citas de ADR. Su disparador ("antes de comitear") se había incumplido: la pérdida llegó a `HEAD` y se pagó después.
- **CONTEXT-DEBT-006** — colisión de IDs resuelta en sus dos mitades.
- **CONTEXT-DEBT-005** — las referencias al catálogo de componentes quedan declaradas como suspendidas hasta que exista el `ui/` nuevo, en vez de apuntar a la nada.
- **CONTEXT-DEBT-001** — pagada: `architect`, `social-strategist`, `bridge-engineer`, `quant-engineer`, `tech-lead` y `rust-engineer` (el último, cierre) salieron del formato legacy. 0 violaciones duras en todo `.agents/skills/`.
- **CONTEXT-DEBT-007** — resuelta dentro de la canonización de `tech-lead`: el Barrido de Cierre Documental tiene ID y trigger propios en `execution-protocol.md`.
- **CONTEXT-DEBT-008** — pagada junto con `CONTEXT-DEBT-001`: `qa-engineer` reestructurado a formato canónico completo, 0 avisos.

> Las fichas de las deudas pagadas se conservan en `.agents/debt/` con su historia completa y Estado ✅ Pagada.
