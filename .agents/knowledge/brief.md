## [CONSULTA CONDICIONAL — SOLO AL CERRAR UNA STORY]

Este archivo NO se lee al iniciar el skill ni en cada turno, a diferencia de `base.md`. Se lee ÚNICAMENTE cuando el Tech-Lead ejecuta el "Barrido de Cierre Documental" (`.agents/skills/tech-lead/SKILL.md`, punto 2 del checklist) y va a redactar `§9 Cierre ejecutivo` de una Orden de Trabajo. Fuera de ese momento exacto, no leas este archivo.

**`.agents/knowledge/base.md` conserva supremacía total.**

---

# Brief — Cierre ejecutivo de Story

## Propósito
Redactar `§9` de la Orden de Trabajo aplicando el "Formato de Reporte al CEO" ya definido en `.agents/skills/tech-lead/SKILL.md` a los datos de cierre de ESTA Story concreta. Este archivo no define un formato nuevo — usa el que ya existe. Solo dice qué leer y cómo traducirlo bien en el caso específico del cierre de una Story.

## Entrada (ya redactada — este archivo no audita ni verifica nada)
Lee, en este orden:
1. `§7` de la Orden (Registro de ejecución) — qué se entregó, tu auditoría, veredicto QA/Quant.
2. `§8` de la Orden (Pendientes derivados/decisiones).
3. La entrada nueva de `.agents/state/tech-lead/PROGRESS.md` de esta Story.
4. Los deltas de `docs/DEBT.md` de este cierre.

## Cómo mapear al Formato de Reporte al CEO
- **ESTADO:** 🟢 si la Story cerró completa con APTO; 🟡 si quedó parcial o generó una decisión pendiente; 🔴 si quedó bloqueada.
- **PROGRESO MACRO:** la traducción a negocio de lo que dice `§7` — no el veredicto técnico, la capacidad nueva que queda disponible.
- **FRICCIONES Y DEUDA:** si `DEBT.md` no tuvo cambios de esta Story, omite la sección. Si los tuvo, una frase de IMPACTO por cada `DEBT-XXX` (qué podría salir mal, bajo qué condición, hacia cuándo pagarlo) — nunca el ID crudo ni el emoji de severidad solo. **Cero falsa alarma, cero falsa calma:** el nivel de preocupación debe calzar con la severidad real (🟡 no se infla a "grave", 🔴 no se disuelve en "detalle menor").
- **INPUT REQUERIDO DEL CEO:** viene de `§8` si ahí quedó algo pendiente de decidir; si no, `"Ninguna."` (nunca se omite este campo, a diferencia de FRICCIONES).

## Salida — dónde vive
El bloque resultante se escribe UNA vez en `§9` de la Orden de Trabajo y se reproduce ahí, literal y sin edición, como mensaje final de chat al cerrar la Story.
