# Política de Comentarios — Las 4 Capas

El propietario del proyecto debe poder leer cualquier archivo de código y entender cada sección sin ser experto en el tema/lenguaje. Esta política prioriza el contexto sobre las convenciones de "clean code" restrictivas.

Los comentarios operan en **4 capas jerárquicas**, cada una con su lugar, regla y precedencia. **Todas coexisten** — Ponytail (que minimiza complejidad/líneas) NO invalida las capas 1–2 de esta política.

---

## Jerarquía de Capas (Orden de Precedencia)

| Capa | Qué | Obligatorio | Dónde | Ejemplo | Líneas | Referencia |
|---|---|---|---|---|---|---|
| **1. Contrato** | Qué hace la función, qué devuelve | ✅ Siempre | Antes de `fn`/`struct`/`impl` | `// Valida permisos; devuelve `Error` si denegado.` | 1–2 | base.md Capas 1–2 |
| **2. Lógica No Obvia** | Por qué es seguro un `unwrap`, qué pasa en borde, por qué esa condición | ✅ Si aplica | Inline en la línea/bloque | `// Pool ya inicializó; imposible ser None.` | 1 | base.md Capas 1–2 |
| **3. Simplificación Deliberada** | Qué se simplificó, cuándo cambiar | 🟡 Recomendado si hay techo | Comentario `ponytail:` | `// ponytail: sin caché. Escalar si >1000 req/s.` | 1 | ponytail.md |
| **4. Deuda Técnica** | Aplazamiento con disparador externo | ✅ Si aplica | `docs/DEBT.md`, **NO en código** | `DEBT-003: falta paginación cuando >1000 registros` | En DEBT.md | debt-management.md |

**Regla de Oro:** Capas 1–2 (base.md) **siempre ganan**. Ponytail (Capa 3) añade metaannotación, no reemplaza. Deuda (Capa 4) va en archivo canónico, no en código.

---

## Reglas de Aplicación (Capas 1–2)

### 1. Comentario de Bloque Antes de Cada Función/Método
Describe en **una frase** qué hace la función y qué devuelve.

**✅ Ejemplo:**
```rust
/// Registra un evento de auditoría atómicamente en el ledger append-only.
/// Retorna el evento con event_sequence_id y audit_hash asignados.
pub fn record_audit_event(entry: &AuditEntry) -> Result<AuditEvent, DbError> {
```

**❌ Malo:**
```rust
/// Registra un evento.
pub fn record_audit_event(entry: &AuditEntry) -> Result<AuditEvent, DbError> {
```

### 2. Comentario de Línea en Lógica No Obvia
Guardas de error, condiciones de borde, cálculos complicados, `match`/`switch` multirrama. **Una línea** que explique el **por qué**.

**✅ Ejemplo:**
```rust
// Pool ya inicializó en main(); imposible ser None.
let pool = POOL.get().expect("pool initialized");

// Derivar sequence_id como MAX+1 dentro de transacción.
// Si colisionan dos escritores, uno recibe UNIQUE y debe reintentar.
let seq_id = txn.query_scalar::<_, i64>(
    "SELECT MAX(event_sequence_id) + 1 FROM audit_events"
).await? as u64;
```

**❌ Malo:**
```rust
let pool = POOL.get().expect("pool initialized"); // get pool

// La secuencia es importante
let seq_id = ...
```

### 3. Contenido del Comentario
Describe el **RESULTADO** de la operación y los casos que maneja. **NO incluyas:**
- Por qué histórico ("cuando arreglamos el bug de X") → eso va en Git.
- Referencias a documentos externos sin explicar ("ver ADR-0003", "STORY-009") → explica inline.
- Abstracciones sin anclaje ("Append-only") → describe qué hace ("solo permite insertar; borrar lanzará error").

**✅ Ejemplo:**
```rust
// Calcula el hash de auditoría sobre los campos de dominio (scope + outcome + override).
// Excluye Grupo I para evitar circularidad.
fn compute_audit_hash(entry: &AuditEntry) -> String {
```

**❌ Malo:**
```rust
// ADR-0020 requiere hash encadenado (ver issue #234)
fn compute_audit_hash(entry: &AuditEntry) -> String {
```

### 4. Gestión de Pánicos (`unwrap()`, `expect()` o Equivalentes)
Cada uno requiere un comentario justificando por qué es **matemáticamente o lógicamente imposible** que falle.

**✅ Ejemplo:**
```rust
// Pool ya inicializó antes de llamar a esta función; no puede ser None.
let pool = POOL.get().expect("pool initialized");

// Tabla siempre tiene al menos 1 fila (bootstrap); unwrap es seguro.
let prev_hash = txn.query_scalar::<_, String>(
    "SELECT audit_hash FROM audit_events ORDER BY event_sequence_id DESC LIMIT 1"
).await.unwrap();
```

**❌ Malo:**
```rust
let pool = POOL.get().unwrap(); // sin comentario

let prev_hash = ...).await.unwrap(); // "puede ser None"
```

---

## Regla Ponytail (Capa 3 — Si Aplica)

📖 **Documento completo:** `ponytail.md`

Usa `// ponytail: [qué se simplificó]. [Cuándo cambiar, umbral medible].` Máximo 1 línea.

- Aplica **SOLO si hay un techo conocido** o tradeoff claro.
- NO es una justificación defensiva. Es **metaannotación** de decisión deliberada.
- Si el simplificación es triviál, omítelo.

**✅ Ejemplo:**
```rust
// ponytail: sin caché. Escalar a Redis si >1000 checks/s.
let row = db.query_scalar("SELECT accepted FROM consents WHERE user_id = ?")
    .bind(user_id)
    .fetch_optional()
    .await?;
```

**❌ Malo:**
```rust
// Idealmente usaríamos Redis, pero no tenemos presupuesto aún...
let row = db.query_scalar(...).await?;
```

---

## Regla DEBT (Capa 4 — Si Aplica)

📖 **Documento completo:** `debt-management.md`

Un aplazamiento **con disparador externo o dependencia** (otra EPIC, módulo aún no existente, estadística de producción) va a `docs/DEBT.md`, **NO en código como comentario**.

Un aplazamiento **acotado al módulo actual** con umbral medible va a `ponytail:` (Capa 3).

**Nunca dejes un aplazamiento solo en código comentado sin registrarlo en `DEBT.md` si tiene causa raíz externa.**

---

## Prohibiciones Absolutas

### 🚫 NO referenciar IDs de tickets sin explicación
```rust
// ❌ // STORY-009
// ❌ // ADR-0003
// ✅ // Auditoría de permisos implementada en STORY-009 (ver docs/execution/STORY-009.md)
```

### 🚫 NO usar términos técnicos abstractos sin definir
```rust
// ❌ // Append-only
// ✅ // Solo permite insertar; borrar o modificar lanzará error de base de datos
```

### 🚫 NO escribir prosa defensiva
```rust
// ❌ // Idealmente usaríamos X, pero por ahora Y es suficiente...
// ✅ // ponytail: Y sin overhead. Cambiar a X si Z ocurre.
```

### 🚫 NO comentarios que repiten el código
```rust
// ❌ fn validate_input(x: i32) {
//   // Valida la entrada x
//   if x < 0 { ... }

// ✅ fn validate_input(x: i32) {
//   // Rechaza valores negativos; el negocio requiere x >= 0.
//   if x < 0 { ... }
```

---

## Ejemplo Completo: Reconciliación de Capas

```rust
/// Registra un evento de auditoría atómicamente en el ledger append-only.
/// Retorna el evento con event_sequence_id y audit_hash asignados.
pub fn record_audit_event(entry: &AuditEntry) -> Result<AuditEvent, DbError> {
    // ponytail: BEGIN IMMEDIATE sin reintento (greenfield, monoproceso).
    // Escalar a reintento con busy_timeout si jobs concurrentes.
    let txn = db.begin_immediate()?;
    
    // Derivar sequence_id como MAX(event_sequence_id)+1 dentro de transacción.
    // Si colisionan dos escritores, uno recibe error UNIQUE y debe reintentar.
    let seq_id = txn.query_scalar::<_, i64>(
        "SELECT MAX(event_sequence_id) + 1 FROM audit_events"
    ).await? as u64;
    
    // El hash previo encadena esta fila a la anterior; garantiza trazabilidad.
    // Pool ya inicializó; ok devolver default si tabla vacía al inicio.
    let prev_hash = txn.query_scalar::<_, String>(
        "SELECT audit_hash FROM audit_events ORDER BY event_sequence_id DESC LIMIT 1"
    ).await.unwrap_or_default();
    
    let audit_hash = compute_hash(&entry, &prev_hash);
    txn.execute(
        "INSERT INTO audit_events (event_sequence_id, audit_hash, ...) VALUES (?, ?, ...)",
        params![seq_id, audit_hash],
    ).await?;
    
    txn.commit().await?;
    Ok(AuditEvent { seq_id, audit_hash, ...*entry })
}
```

**Análisis:**
- **Doc-comment (líneas 1–2):** Capa 1 (contrato).
- **`ponytail:` (líneas 4–5):** Capa 3 (simplificación deliberada).
- **Líneas 7–8, 11–13:** Capa 2 (lógica no obvia).
- **Total:** ~4 líneas de comentario legible, sin prosa.

---

## Checklist al Entregar

- ✅ Cada función pública tiene Capa 1 (contrato).
- ✅ Lógica borde/peligrosa tiene Capa 2 (lógica no obvia).
- ✅ Si hay simplificación con techo, tiene Capa 3 (`ponytail:`).
- ✅ Si hay aplazamiento externo, está en `DEBT.md` (Capa 4).
- ✅ Sin referencias a tickets/ADRs sin explicar.
- ✅ Sin prosa defensiva.
- ✅ Sin comentarios que repiten el código.

---

## Relación con Otros Archivos

- **`.agents/knowledge/base.md`** — Gobernanza meta. Referencia a este archivo.
- **`.agents/knowledge/ponytail.md`** — Skill de simplificación (Capa 3).
- **`.agents/knowledge/debt-management.md`** — Gestión de deuda (Capa 4).
- **`docs/DEBT.md`** — Registro canónico de deuda técnica rastreada.
- **`.agents/memory/comentarios-ponytail-reconciliacion.md`** — Decisión histó­rica de esta reconciliación.
