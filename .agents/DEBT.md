# DEBT.md — Índice del Registro de Deuda de Agentes/Documentación

> **Propósito:** el **único lugar descubrible** donde vive la deuda de gobernanza de agentes — skills, knowledge docs, hooks, plantillas — deliberadamente aplazada. Registro hermano de [[docs/DEBT.md]] (deuda de código/producto): mismo patrón, mismo protocolo (`DEBT-MANAGEMENT-11`), dominio distinto.
> **Estructura (patrón ADR):** este archivo es el **índice**; la ficha completa de cada deuda vive en `.agents/debt/AGENT-DEBT-XXX.md`. Para leer una deuda concreta, abre su archivo — no hace falta cargar todo el registro.
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
| AGENT-DEBT-001 | 🔴 Alta | 6 skills conservan el formato legacy pese a que el commit 972bab5 declara "armonización de 13" — bloquean el hook `validate-agent-docs.py` | Abierta | [detalle](./debt/AGENT-DEBT-001.md) |
| AGENT-DEBT-002 | 🔴 Alta | `debt-management.md` referencia una lectura de `.agents/memory/` que `memory-policy.md` prohíbe expresamente | Abierta | [detalle](./debt/AGENT-DEBT-002.md) |
| AGENT-DEBT-003 | 🟠 Media | `ui-designer/SKILL.md` (worktree) — pérdida de salvaguarda FFI, prohibición absoluta debilitada a condicional, ADRs vinculantes perdidos | Abierta | [detalle](./debt/AGENT-DEBT-003.md) |
| AGENT-DEBT-004 | 🟡 Baja | 5 knowledge docs nuevos huérfanos — ningún skill los cita por wikilink, la copia vieja duplicada sigue siendo la que se lee | Abierta | [detalle](./debt/AGENT-DEBT-004.md) |
| AGENT-DEBT-005 | 🟡 Baja | Referencias muertas a `ui/COMPONENTS.md` en `flutter-engineer` y `ui-designer` (worktree) tras el reset `ui/`→`ui-old/` | Abierta | [detalle](./debt/AGENT-DEBT-005.md) |
| AGENT-DEBT-006 | 🟡 Baja | IDs de ítem que reutilizan el ID del bloque padre en `commenting-policy.md` y `partner/SKILL.md` | Abierta | [detalle](./debt/AGENT-DEBT-006.md) |
| AGENT-DEBT-007 | 🟠 Media | Etapas de `tech-lead/SKILL.md` mal numeradas — bloque obligatorio sin número entre Etapa 6 y 7; revisión completa pendiente | Abierta | [detalle](./debt/AGENT-DEBT-007.md) |
| AGENT-DEBT-008 | 🟡 Baja | `qa-engineer/SKILL.md` en formato legacy — sin bloque `-FV`, rutas sin wikilink, lenguaje de bitácora (avisos blandos del hook) | Abierta | [detalle](./debt/AGENT-DEBT-008.md) |

---

## Deudas pagadas (resumen)

Ninguna todavía — registro abierto el 2026-08-20 a partir de la auditoría del commit `972bab5` y del worktree de `.agents/` en curso ese día.

> Las fichas de las deudas pagadas se conservan en `.agents/debt/` con su historia completa y Estado ✅ Pagada.
