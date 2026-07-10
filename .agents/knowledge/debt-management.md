# Gestión de Deuda Técnica

La deuda técnica deliberada es sana en greenfield — permite avanzar en el camino crítico sin frenar por cosas que aún no muerden, **siempre que quede registrada aquí con su causa, impacto y disparador de pago**.

📖 **Registro canónico:** `docs/DEBT.md` (es el único lugar descubrible).

---

## Regla de Oro

**Un aplazamiento sin disparador escrito está olvidado.**

> Si no está en `docs/DEBT.md` con causa raíz + disparador, no está rastreada.

---

## Cuándo Abrir DEBT-XXX vs. Usar `ponytail:`

### ✅ Usa `ponytail:` (Capa 3 — en el código)

Si el aplazamiento está **acotado al módulo/función actual** y **tienes un umbral medible**:

```rust
// ponytail: sin caché. Escalar si >1000 req/s.
let result = db.query(...).await?;
```

**Criterios:**
- Eres dueño del techo (no depende de otra EPIC).
- Métrica clara dentro de este módulo.
- Triviál cambiar cuando llegue el umbral.
- El código funciona correctamente hoy sin ello.

**Ejemplos:**
- "Sin paginación; escalar si >1000 registros/mes."
- "O(n) aceptable ahora; cambiar a HashMap si dataset >10k."
- "Sin transacción de reintento (greenfield); añadir si jobs concurrentes."

---

### ✅ Usa DEBT-XXX (Capa 4 — en DEBT.md)

Si el aplazamiento tiene **disparador externo o dependencia clara**:

```markdown
DEBT-003: Falta paginación en user-fetch
- Disparador: cuando otra EPIC construya el backend de paginación
- Impacto: limita la SVF, no la correctitud
```

**Criterios:**
- Depende de otra EPIC, módulo futuro, decisión del Architect.
- Causa raíz que **NO puedes arreglar ahora**.
- Necesita evento externo (estadística de prod, tipo real que aún no existe, infraestructura compartida).
- Hoy el código es incompleto o necesita cambio arquitectónico.

**Ejemplos:**
- "Tipo `BacktestResult` aún no construido; espera EPIC de validación."
- "Fan-out al event bus diferido; el bus aún no existe (ADR-0085)."
- "Fetch de datos reales del servidor diferido; el adaptador se construye en EPIC-5."

---

## Estructura de una Entrada DEBT

```markdown
### DEBT-XXX · [Nombre corto]
- **Severidad:** 🔴 Alta / 🟠 Media / 🟡 Baja
- **Origen:** dónde se detectó (Story, revisión de código, QA).
- **Descripción:** qué falta o es incompleto.
- **Impacto actual:** qué se rompe HOY (nulo, limitación, riesgo).
- **Causa raíz:** por qué está diferida (dependencia, decisión de arquitecto, greenfield).
- **Disparador de pago:** evento o condición que detonará su construcción.
- **Plan:** pasos concretos para pagarla.
- **Estado:** Abierta / En pago / Pagada.
```

---

## Severity Levels

| Nivel | Significado | Cuándo Pagar |
|---|---|---|
| 🔴 **Alta** | Puede corromper datos o violar un invariante bajo condiciones alcanzables. | Pronto (antes del release). |
| 🟠 **Media** | Fallo seguro (sin corrupción) pero con pérdida de función/correctitud bajo condiciones aún no presentes. | Antes de que la condición llegue. |
| 🟡 **Baja** | Cosmético o diferido por secuenciación; sin riesgo de correctitud. | Bajo demanda o al cierre de la fase. |

---

## Ejemplos Reales

### Ejemplo 1: DEBT con Causa Raíz Externa (debe ir a DEBT.md)

```markdown
### DEBT-009 · Placeholders de tipos del guantelete en #7
- **Severidad:** 🟡 Baja
- **Origen:** STORY-034 (institutional-report-engine).
- **Descripción:** `institutional-report-engine` (#7) consume 
  `BacktestResult`/`RobustnessScore` que hoy son **placeholders** 
  (`pub struct X;`); la firma es reproducible, pero el mapeo desde 
  los tipos **reales** del guantelete aún no existe.
- **Impacto actual:** ninguno — el puerto es estable; es un mapeo 
  pendiente, no un bug.
- **Disparador de pago:** cuando el guantelete produzca los tipos reales 
  (EPIC de validación/ejecución).
- **Estado:** Abierta.
```

→ **Por qué DEBT.md:** depende de otra EPIC (validación/ejecución), no puedes arreglarlo hoy.

---

### Ejemplo 2: Simplificación Acotada (debe ir en `ponytail:`)

```rust
// ponytail: sin caché Redis. Escalar si >1000 validaciones/s.
fn validate_consent(user_id: &str) -> Result<bool, Error> {
    let row = db.query_scalar("SELECT accepted FROM consents WHERE user_id = ?")
        .bind(user_id)
        .fetch_optional()
        .await?;
    Ok(row.unwrap_or(false))
}
```

→ **Por qué `ponytail:`:** eres dueño del techo (si llega 1000/s, añades Redis); no depende de nada externo.

---

### Ejemplo 3: DEBT con Causa Raíz Externa + Disparador Claro

```markdown
### DEBT-015 · #11 canonical_delta_bytes sin test de valor-dorado
- **Severidad:** 🟡 Media
- **Origen:** QA por mutación de STORY-039.
- **Descripción:** `compute_backup_delta` sí está cazado; lo que falta 
  es un test de valor-dorado sobre `canonical_delta_bytes` que ancle 
  los bytes exactos producidos.
- **Impacto actual:** nulo (fase greenfield; el adaptador de subida 
  S3/R2 está diferido).
- **Disparador de pago:** **ANTES de construir el adaptador de almacén 
  de objetos** (antes de que exista un respaldo real que pudiera salir vacío).
- **Plan:** añadir test de valor-dorado con semilla fija.
- **Estado:** Abierta.
```

→ **Por qué DEBT.md:** el disparador es externo ("antes de que S3 exista"); hoy es cosmético.

---

## Cómo Registrar una DEBT Nueva

### 1. Detectas un Aplazamiento
```
"Falta el tipo BacktestResult — espera EPIC-3"
```

### 2. Decide: ¿`ponytail:` o DEBT-XXX?

| Pregunta | Respuesta | Acción |
|----------|----------|--------|
| ¿Depende de otra EPIC/módulo futuro? | Sí | → DEBT-XXX |
| ¿Tienes un umbral medible para changarlo? | Sí | → `ponytail:` |
| ¿Es causa raíz externa? | Sí | → DEBT-XXX |
| ¿Puedes arreglarlo hoy si quisieras? | No | → DEBT-XXX |

### 3. Registra en `docs/DEBT.md`

```markdown
### DEBT-NNN · [Nombre corto]
- **Severidad:** 🟡 Baja
- **Origen:** STORY-YYY.
- **Descripción:** [qué falta].
- **Impacto actual:** [nulo / limitación / riesgo].
- **Disparador de pago:** [cuándo arreglarlo].
- **Estado:** Abierta.
```

### 4. Linkea desde `.agents/memory/MEMORY.md`
Si es importante para future-you, añade un pointer.

### 5. **PROHIBIDO:** No dejes un aplazamiento solo en código comentado
```rust
// ❌ MALO:
// "Falta el tipo BacktestResult, lo completamos cuando venga de EPIC-3"
let metrics = BTreeMap::new();

// ✅ CORRECTO:
let metrics = BTreeMap::new(); // placeholder; ver DEBT-009
```

→ El comentario apunta a DEBT.md, donde vive la causa raíz + disparador.

---

## Cierre de DEBT (Pago)

Cuando pagues una deuda:

1. **Abre una Story** para su corrección (ej. STORY-041).
2. **Edita la entrada en `docs/DEBT.md`:**
   ```markdown
   - **Estado:** ✅ **Pagada** — [STORY-041](./execution/STORY-041-...md) (2026-07-07)
   ```
3. **Referencia en `PROGRESS.md`** (bitácora del Tech-Lead) la Story que la saldó.

---

## Relación con Otros Archivos

- **`docs/DEBT.md`** — Registro canónico (si no está ahí, no existe).
- **`.agents/knowledge/commenting-policy.md`** — Capas 1–3 (en código).
- **`.agents/knowledge/ponytail.md`** — Capa 3 (`ponytail:`).
- **`.agents/memory/comentarios-ponytail-reconciliacion.md`** — Reconciliación histórica.
- **`.agents/state/tech-lead/PROGRESS.md`** — Bitácora de hallazgos (apunta a DEBT-XXX).

---

## Checklist: "¿Es una DEBT válida?"

- ✅ Tiene **causa raíz clara** (depende de X, espera Y, falta Z).
- ✅ Tiene **disparador medible** (cuándo arreglarlo, qué evento lo detona).
- ✅ **NO es una lista de deseos.** Es un aplazamiento deliberado con justificación.
- ✅ Está en `docs/DEBT.md`, no solo en código/chat.
- ✅ Severidad correcta (realista).
- ✅ Impacto actual honesto ("nulo" si es verdad; "limitación" si es verdad).

---

## Anti-Patrones

### ❌ DEBT sin disparador
```markdown
DEBT-X: Refactorizar este módulo
```
→ "Refactorizar algún día" no es disparador. Especifica: "cuando X métrica llegue a Y" o "cuando módulo Z exista".

### ❌ DEBT por pereza
```markdown
DEBT-Y: Falta validación exhaustiva
```
→ Si es correctitud, corrígelo hoy. DEBT es para aplazamientos deliberados, no para evasión.

### ❌ Aplazamiento en código sin DEBT.md
```rust
// TODO: añadir caché cuando haya tiempo
let result = fetch_data();
```
→ Si es aplazamiento externo (depende de otra EPIC), regístralo en DEBT.md. Si es local (techo medible), usa `ponytail:`.

### ❌ DEBT cerrada sin Story
```markdown
DEBT-Z: ... Estado: ✅ Pagada.
```
→ Debe linkar a la Story que la saldó. Trazabilidad es obligatoria.
