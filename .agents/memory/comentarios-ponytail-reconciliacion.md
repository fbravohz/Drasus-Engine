---
name: comentarios-ponytail-reconciliacion
description: Reconciliación de las 4 capas de comentarios (base.md + Ponytail + DEBT.md) — cómo coexisten sin conflicto
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 8320c909-d535-467d-b886-9c112ec0d98f
---

## Decisión

**Los comentarios operan en 4 capas jerárquicas que coexisten sin conflicto.** Ponytail NO invalida base.md; añade una capa de metaannotación sobre simplificaciones deliberadas.

## Las 4 Capas

| Capa | Regla | Obligatorio | Dónde | Líneas |
|---|---|---|---|---|
| 1. Contrato | Qué hace la función, qué devuelve | ✅ Siempre | Antes de `fn` | 1–2 |
| 2. Lógica No Obvia | Por qué es seguro `unwrap`, qué pasa en borde | ✅ Si aplica | Inline en línea | 1 |
| 3. Simplificación (Ponytail) | Qué se simplificó, cuándo cambiar (umbral) | 🟡 Si hay techo | `ponytail:` | 1 |
| 4. Deuda Técnica | Aplazamiento con disparador externo | ✅ Si aplica | `docs/DEBT.md`, NO en código | En DEBT.md |

## Jerarquía

- **base.md §4.1 gana siempre** sobre Ponytail: Capas 1–2 (contrato + lógica no obvia) nunca se negocian.
- **Ponytail añade Capa 3**, no reemplaza: metaannotación de decisiones deliberadas con techo medible.
- **DEBT.md es Capa 4**: para aplazamientos con causa raíz externa o módulo futuro (NO acotados al módulo actual).

## Distinción: `ponytail:` vs. DEBT-XXX

**Usa `ponytail:` si:**
- Eres dueño del techo ("escalar si >1000 req/s", "si dataset >10k").
- Aplica a este módulo/función.
- Triviál cambiar cuando llegue el umbral.

**Usa DEBT-XXX si:**
- Depende de otra EPIC, módulo futuro, decisión del Architect.
- Tiene causa raíz que NO puedes arreglar hoy.
- Necesita disparador externo (estadística de prod, evento de negocio).

**Ejemplo de distinción:**
```rust
// ponytail: sin caché. Escalar si >1000 req/s. ← Eres dueño, métrica clara.
```

vs.

```
DEBT-015: espera tipo BacktestResult de EPIC-3 (`institutional-report-engine` #7)
```

## Por Qué Esta Reconciliación Importa

**Sin ella:** tensión entre "base.md dice comenta mucho" vs. "Ponytail dice simplifica" → confusión sobre qué es obligatorio.

**Con ella:** cada capa tiene función clara, regla precisa, precedencia definida → código legible, anticipado, rastreable.

## Cómo Aplicar

**Orden al escribir un función:**

1. Contrato (base.md Capa 1): "qué, qué devuelve" → 1–2 líneas obligatorias.
2. Lógica no obvia (base.md Capa 2): "por qué es seguro este punto" → 1 línea si aplica.
3. Simplificación (Ponytail Capa 3): "qué se simplificó, cuándo cambiar" → 1 línea si hay techo.
4. Deuda (DEBT.md Capa 4): si aplazamiento externo, regístralo allá, NO en código.

**Total esperado:** 3–4 líneas de comentario legible, no prosa.

## Archivos Que Lo Formalizan

- **`.agents/knowledge/commenting-policy.md`** — Política de Comentarios completa (4 capas, reglas, ejemplos, prohibiciones).
- **`.agents/knowledge/debt-management.md`** — Gestión de Deuda Técnica (cuándo `ponytail:` vs. DEBT-XXX, severidad, ejemplos).
- **`.agents/knowledge/base.md`** — Resumen + referencias a los archivos anteriores (§4.1 y §4.2).
- **`.agents/knowledge/ponytail.md`** — Sección "Relación con Políticas de Comentarios" (reconciliación explícita).
- **`CLAUDE.md`** — Mapa de orientación (referencia a base de conocimiento).
- **Esta memoria** — Para que skills y sesiones futuras lo encuentren.

## Precedencia

Si hay conflicto de interpretación:
1. `.agents/knowledge/base.md` (mapa de referencias, ley suprema).
2. `.agents/knowledge/commenting-policy.md` (capas 1–2 obligatorias, Capa 3 si aplica).
3. `.agents/knowledge/debt-management.md` (cuándo DEBT-XXX vs. ponytail:).
4. `.agents/knowledge/ponytail.md` (Capa 3: simplificación deliberada).
5. `docs/DEBT.md` (registro canónico de deuda, ortogonal).

**Why:** el usuario necesita código legible (commenting-policy), eficiente (ponytail), y rastreable (debt-management + DEBT.md). Las 4 capas coexisten. Cada archivo se lee bajo demanda para minimizar tokens.

**How to apply:** al principio de cada Story/Task, skills y el usuario leen el archivo que necesitan (no todos). Nunca simplificar Capas 1–2 en nombre de Ponytail. Nunca dejar deuda sin registrar en DEBT.md si tiene disparador externo.
