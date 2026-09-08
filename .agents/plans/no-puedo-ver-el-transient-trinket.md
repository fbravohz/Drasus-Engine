# Plan — Hacer autosuficiente el plan de auditoría (dueños + decisiones)

## Contexto

El plan de auditoría ya existe y está completo en cuanto a hallazgos: `.agents/plans/auditoria-forense-commits-972bab5-3007f92.md` (13 hallazgos nuevos + las 8 fichas `CONTEXT-DEBT` verificadas). Pero hoy no se puede retomar en frío: no dice **quién arregla qué** ni separa **lo que el propietario tiene que decidir** de lo que un agente puede ejecutar solo.

Objetivo: que el propietario abra ese archivo dentro de un mes, en una sesión limpia, y sepa exactamente a qué skill despachar cada cosa y qué se le está preguntando a él — sin depender de este chat.

## Cambio

Insertar **dos secciones nuevas** en el plan de auditoría existente. No se toca ningún hallazgo ni el veredicto; solo se añade la capa de ejecución.

**Archivo a modificar (único):** `.agents/plans/auditoria-forense-commits-972bab5-3007f92.md`

---

### Sección 1 — va inmediatamente después de "Veredicto global"

```markdown
## ⚠️ Decisiones que solo el propietario puede tomar

Ningún agente puede resolver estas cinco. Todo lo demás en este plan es ejecutable sin consultar.

### D-1 · ¿Se acepta que el motor muera con la UI? (hallazgo N-1) — BLOQUEANTE

Hoy, en el modo por defecto, cerrar la ventana de Flutter mata el motor de ejecución en vivo. Ningún ADR lo reconoce. Elige una:

- **(a) Aceptarlo y escribirlo.** El Architect redacta un ADR que declara el trade-off como decisión consciente para la fase local.
- **(b) Registrarlo como deuda con disparador.** Ficha en `docs/DEBT.md` con disparador "antes del primer despliegue con dinero real".
- **(c) Adelantar la separación de procesos** que ADR-0119 difiere a EPIC-9+.

Bloquea a: **Architect**. Sin tu respuesta no puede escribir nada.

### D-2 · ¿Cómo se repara `general-counsel`? (hallazgo N-2)

Se quedó sin definición operable de sus Modos de Acompañamiento.

- **(a) Restaurar en el skill** su definición propia de rol legal. *Recomendada* — es fiel a lo que se perdió y no toca ADR-0120.
- **(b) Ampliar el alcance de `mentorship-modes.md`** a roles no-ingeniero. Toca ADR-0120, requiere al Architect.

Si no respondes, se ejecuta la (a).

### D-3 · ¿Qué dice el runbook mientras no exista `ui/`? (hallazgo N-3)

`docs/TEST.md` manda un gate obligatorio de QA que hoy falla.

- **(a) Apuntar temporalmente a `ui-old/`** si el catálogo viejo aún sirve para correr algo.
- **(b) Marcar el bloque como suspendido** hasta que exista el `ui/` nuevo.

Depende de cuándo reconstruyas la UI — por eso es tuya.

### D-4 · Ratificar que `base.md` se queda

El plan original ordenaba borrarlo; durante la ejecución se decidió conservarlo y el hook lo exime a propósito, para no romper enlaces de planes ya cerrados. La decisión nunca se escribió. Confirma que se queda de forma permanente y se corrige el plan viejo.

### D-5 · Alcance de la canonización de `tech-lead`

Son 85 489 caracteres y 36 marcas de bitácora — el archivo más cargado del sistema.

- **(a) Solo formato:** cabecera, bloque de verificación, purga de bitácora.
- **(b) Formato + partirlo**, extrayendo el Pipeline de Ejecución y el Gate de Coherencia a knowledge docs propios, como proponía el plan original.

La (b) es más trabajo pero es la que resuelve el problema de fondo.
```

---

### Sección 2 — va inmediatamente antes de "Parte 4 — Remediación priorizada"

```markdown
## Parte 3-bis — Quién arregla qué

Regla de dominio, verificada en cada skill:

- **`context-engineer`** edita `.agents/skills/` y `.agents/knowledge/` — formato, redacción y restauración de contenido que ya existía.
- **`architect`** edita `docs/` — ADR, SAD, features, módulos, `ROADMAP`, `docs/debt/`.
- **`tech-lead`** mantiene `docs/TEST.md` (paso 4 de su Barrido de Cierre Documental).
- **`context-auditor`** nunca corrige: solo audita y reporta. Se usa para verificar, no para arreglar.
- **`qa-engineer`** valida que el runbook reparado corra de verdad.

| Ítem | Dueño | Nota |
|---|---|---|
| D-1 → ADR o ficha de deuda | `architect` | Bloqueado por D-1 |
| N-2 · Modos de `general-counsel` | `context-engineer` | Si eliges D-2 (b), entra también `architect` |
| N-3 · refs `ui/` en `docs/` (ROADMAP, ADR-0117/0138/0152, features, `DEBT.md`) | `architect` | |
| N-3 · refs `ui/` en `.agents/skills/` y `.agents/knowledge/` | `context-engineer` | |
| N-3 · `docs/TEST.md` | `tech-lead` | Bloqueado por D-3; `qa-engineer` valida después |
| N-4 · punteros temáticos en `bridge`/`qa`/`quant`/`rust` | `context-engineer` | |
| N-5 · evidencia perdida en 3 knowledge docs | `context-engineer` | |
| N-7 · reformatear `DEBT-046` | `architect` | Vive en `docs/debt/` |
| N-8 · sincronizar el plan viejo con la realidad | `context-engineer` | Bloqueado por D-4 |
| N-9 · wikilinks faltantes | `context-engineer` | |
| N-10 · ruta rota en 4 skills | `context-engineer` | |
| N-11 · indexar `brief.md` | `context-engineer` | |
| N-12 · dos referencias rotas | `context-engineer` | |
| N-13 · `PARTNER-FV` y hueco de `refactoring-engineer` | `context-engineer` | |
| Canonizar los 5 skills legacy | `context-engineer` | Bloqueado por D-5 solo para `tech-lead` |
| `CONTEXT-DEBT-001` · corregir cifras | `context-engineer` | |
| `CONTEXT-DEBT-002` · contradicción de memoria | `context-engineer` | |
| `CONTEXT-DEBT-003` · restaurar FFI, revertir `7.3`, reponer ADRs | `context-engineer` | La ficha ya especifica qué restaurar |
| `CONTEXT-DEBT-004` · wikilinks a los 5 knowledge nuevos | `context-engineer` | |
| `CONTEXT-DEBT-006` · IDs colisionados | `context-engineer` | |
| `CONTEXT-DEBT-007` · numeración de `tech-lead` | `context-auditor` audita → `context-engineer` corrige | La ficha nombra al auditor para el diagnóstico |
| `CONTEXT-DEBT-008` · corregir alcance + canonizar `qa-engineer` | `context-engineer` | |
| Abrir `CONTEXT-DEBT-009` (refs `ui/`) | quien lo pague primero | El registro admite que cualquier rol abra ficha |
| N-6 · commit masivo | — | No se arregla retroactivamente; es regla para el futuro |

**Lectura práctica:** el 80 % de la lista es un solo despacho a `context-engineer`, que puede hacerse en lotes. `architect` tiene 3 ítems. `tech-lead` uno. Nada exige coordinar a los tres a la vez.
```

---

### Cambio menor en "Parte 4"

En el encabezado de **Prioridad 0**, sustituir el texto actual por un puntero a la sección nueva:

```markdown
### Prioridad 0 — Decisiones del propietario

Las cinco decisiones (D-1 a D-5) están arriba, en su propia sección. **D-1 bloquea al Architect**; el resto tiene ruta por defecto si no respondes.
```

## Verificación

- Abrir `.agents/plans/auditoria-forense-commits-972bab5-3007f92.md` y comprobar que las decisiones aparecen antes de la Parte 0, y la tabla de dueños antes de la Parte 4.
- Cada ítem de la Parte 4 tiene dueño localizable en la tabla nueva.
- El archivo sigue siendo el único plan del tema — no se crea un segundo documento.
