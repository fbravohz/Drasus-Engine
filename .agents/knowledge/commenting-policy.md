# Política de Comentarios — Las 4 Capas

El propietario del proyecto debe poder leer cualquier archivo de código y entender cada sección sin ser experto en el tema o lenguaje. Prioriza el contexto sobre las convenciones de "clean code" restrictivas.

Slug ID: `COMMENTING-POLICY`.

Tamaño: `13187 caracteres (~3297 tokens aprox., ratio 4:1)`.

---

## COMMENTING-POLICY-0 — Lecturas Obligatorias

| Documento | De qué va | Cuándo leerlo (disparador) | Alcance |
|---|---|---|---|
| [[.agents/knowledge/governance.md]] | Gobernanza meta, supremacía absoluta. | `SIEMPRE` al iniciar cualquier skill. | Todos los agentes |

---

## COMMENTING-POLICY-FV — Filtro de Verificación

**Aplica a:** todo código o comentario que el agente produce, como último paso.

* **COMMENTING-POLICY-FV-1:** ¿declaraste que leíste este archivo? (`COMMENTING-POLICY-1`)
* **COMMENTING-POLICY-FV-2:** ¿cada función/método público tiene su comentario de contrato? (`COMMENTING-POLICY-3`)
* **COMMENTING-POLICY-FV-3:** ¿la lógica borde o peligrosa tiene su comentario de línea? (`COMMENTING-POLICY-4`)
* **COMMENTING-POLICY-FV-4:** ¿cada comentario describe el resultado, sin historial, sin IDs sin explicar, sin abstracciones sin anclaje? (`COMMENTING-POLICY-5`)
* **COMMENTING-POLICY-FV-5:** ¿cada `unwrap`/`expect`/equivalente tiene su justificación de imposibilidad de fallo? (`COMMENTING-POLICY-6`)
* **COMMENTING-POLICY-FV-6:** ¿si hay simplificación deliberada con techo, tiene su comentario `ponytail:`? (`COMMENTING-POLICY-7`)
* **COMMENTING-POLICY-FV-7:** ¿si hay aplazamiento con disparador externo, está registrado en [[docs/DEBT.md]] y no como comentario en el código? (`COMMENTING-POLICY-8`)
* **COMMENTING-POLICY-FV-8:** ¿ningún comentario es prosa defensiva? (`COMMENTING-POLICY-9`)
* **COMMENTING-POLICY-FV-9:** ¿ningún comentario repite lo que el código ya dice? (`COMMENTING-POLICY-10`)

De ser satisfactoria cada validación de la lista, declara `COMMENTING-POLICY - FILTRO DE VERIFICACION APROBADO`.

---

## COMMENTING-POLICY-1 — Declaración de Lectura

* **COMMENTING-POLICY-1.1 — Declaración de Lectura:** declara explícitamente que leíste este archivo antes de aplicar sus reglas a la tarea en curso.

**Filtro de verificación:** `COMMENTING-POLICY-FV-1`.

---

## COMMENTING-POLICY-2 — Jerarquía de Capas y Precedencia

Cuatro capas jerárquicas para comentarios de código, cada una con su lugar y regla propia. Todas coexisten: Ponytail (Capa 3, que minimiza complejidad/líneas) no invalida las Capas 1–2.

**Aplica a:** todos los roles, al escribir o revisar comentarios de código.

* **COMMENTING-POLICY-2.1 — Capa 1, Contrato:**

  qué hace la función y qué devuelve. Obligatorio siempre. Va antes de `fn`/`struct`/`impl`, 1–2 líneas. Ejemplo: `// Valida permisos; devuelve Error si denegado.`

* **COMMENTING-POLICY-2.2 — Capa 2, Lógica No Obvia:**

  por qué es seguro un `unwrap`, qué pasa en un caso borde, por qué esa condición. Obligatorio si aplica. Inline en la línea o bloque, 1 línea. Ejemplo: `// Pool ya inicializó; imposible ser None.`

* **COMMENTING-POLICY-2.3 — Capa 3, Simplificación Deliberada:**

  qué se simplificó y cuándo cambiarlo. Recomendado si hay techo conocido. Va en un comentario `ponytail:`, 1 línea. Ejemplo: `// ponytail: sin caché. Escalar si >1000 req/s.`

* **COMMENTING-POLICY-2.4 — Capa 4, Deuda Técnica:**

  aplazamiento con disparador externo. Obligatorio si aplica. Va en [[docs/DEBT.md]] como `DEBT-XXX`, nunca en el código.

**Precedencia:** Capas 1–2 (`COMMENTING-POLICY-3`, `COMMENTING-POLICY-4`) siempre ganan. Ponytail (Capa 3, `COMMENTING-POLICY-7`) añade metaanotación, no reemplaza. Deuda (Capa 4, `COMMENTING-POLICY-8`) va en su archivo canónico, no en el código.

---

## COMMENTING-POLICY-3 — Comentario de Contrato Antes de Cada Función

Describe en una frase qué hace la función y qué devuelve.

**Aplica a:** todos los roles, al escribir o revisar funciones y métodos públicos.

* **COMMENTING-POLICY-3.1 — Una Frase, Qué y Qué Devuelve:**

  el comentario de bloque antes de cada función/método describe qué hace y qué devuelve, no una paráfrasis del nombre.
  * ❌ *Incorrecto:*
    ```rust
    /// Registra un evento.
    pub fn record_audit_event(entry: &AuditEntry) -> Result<AuditEvent, DbError> {
    ```
  * ✅ *Correcto:*
    ```rust
    /// Registra un evento de auditoría atómicamente en el ledger append-only.
    /// Retorna el evento con event_sequence_id y audit_hash asignados.
    pub fn record_audit_event(entry: &AuditEntry) -> Result<AuditEvent, DbError> {
    ```

**Filtro de verificación:** `COMMENTING-POLICY-FV-2`.

---

## COMMENTING-POLICY-4 — Comentario de Línea en Lógica No Obvia

Guardas de error, condiciones de borde, cálculos complicados, `match`/`switch` multirrama: una línea que explica el por qué.

**Aplica a:** todos los roles, al escribir o revisar lógica no obvia.

* **COMMENTING-POLICY-4.1 — Una Línea, el Por Qué:**

  el comentario inline explica por qué la línea es segura o necesaria, no qué hace literalmente — eso ya lo dice el código.
  * ❌ *Incorrecto:*
    ```rust
    let pool = POOL.get().expect("pool initialized"); // get pool

    // La secuencia es importante
    let seq_id = ...
    ```
  * ✅ *Correcto:*
    ```rust
    // Pool ya inicializó en main(); imposible ser None.
    let pool = POOL.get().expect("pool initialized");

    // Derivar sequence_id como MAX+1 dentro de transacción.
    // Si colisionan dos escritores, uno recibe UNIQUE y debe reintentar.
    let seq_id = txn.query_scalar::<_, i64>(
        "SELECT MAX(event_sequence_id) + 1 FROM audit_events"
    ).await? as u64;
    ```

**Filtro de verificación:** `COMMENTING-POLICY-FV-3`.

---

## COMMENTING-POLICY-5 — Contenido del Comentario: Resultado, No Proceso

El comentario describe el resultado de la operación y los casos que maneja, no el proceso que llevó a escribirla.

**Aplica a:** todos los roles, al redactar el texto de cualquier comentario.

* **COMMENTING-POLICY-5.1 — Resultado, No Historial:**

  nunca el por qué histórico ("cuando arreglamos el bug de X") — eso vive en Git, no en el comentario.
  * ✅ *Correcto:*
    ```rust
    // Calcula el hash de auditoría sobre los campos de dominio (scope + outcome + override).
    // Excluye Grupo I para evitar circularidad.
    fn compute_audit_hash(entry: &AuditEntry) -> String {
    ```

* **COMMENTING-POLICY-5.2 — Prohibido Referenciar IDs Sin Explicar:**

  una referencia a un documento externo (ADR, Story, ticket) siempre va acompañada de la explicación inline — el ID solo, sin contexto, no le sirve a quien lee el código sin abrir otro documento.
  * ❌ *Incorrecto:* `// STORY-009` — `// ADR-0003`
  * ✅ *Correcto:* `// Auditoría de permisos implementada en STORY-009 (ver [[docs/execution/STORY-009-cli-app.md]])`

* **COMMENTING-POLICY-5.3 — Prohibido Abstracciones Sin Anclaje:**

  un término técnico abstracto se describe por lo que hace, no se nombra dándolo por entendido.
  * ❌ *Incorrecto:* `// Append-only`
  * ✅ *Correcto:* `// Solo permite insertar; borrar o modificar lanzará error de base de datos`

**Filtro de verificación:** `COMMENTING-POLICY-FV-4`.

---

## COMMENTING-POLICY-6 — Gestión de Pánicos: unwrap, expect y Equivalentes

Cada `unwrap()`, `expect()` o equivalente requiere un comentario que justifique por qué es matemática o lógicamente imposible que falle.

**Aplica a:** todos los roles, en cualquier código que use `unwrap`/`expect`/equivalentes.

* **COMMENTING-POLICY-6.1 — Justificación de Imposibilidad de Fallo:**

  el comentario prueba, no afirma, por qué el panic es inalcanzable.
  * ❌ *Incorrecto:*
    ```rust
    let pool = POOL.get().unwrap(); // sin comentario

    let prev_hash = ...).await.unwrap(); // "puede ser None"
    ```
  * ✅ *Correcto:*
    ```rust
    // Pool ya inicializó antes de llamar a esta función; no puede ser None.
    let pool = POOL.get().expect("pool initialized");

    // Tabla siempre tiene al menos 1 fila (bootstrap); unwrap es seguro.
    let prev_hash = txn.query_scalar::<_, String>(
        "SELECT audit_hash FROM audit_events ORDER BY event_sequence_id DESC LIMIT 1"
    ).await.unwrap();
    ```

**Filtro de verificación:** `COMMENTING-POLICY-FV-5`.

---

## COMMENTING-POLICY-7 — Regla Ponytail

Metaanotación de una simplificación deliberada, con techo conocido y umbral medible para cuándo escalarla.

**Aplica a:** todos los roles, cuando una simplificación tiene un tradeoff claro.

* **COMMENTING-POLICY-7.1 — Formato y Condición de Uso:**

  usa `// ponytail: [qué se simplificó]. [Cuándo cambiar, umbral medible].`, máximo 1 línea. Aplica solo si hay un techo conocido o tradeoff claro — no es una justificación defensiva. Si la simplificación es trivial, omítelo.
  * ❌ *Incorrecto:* `// Idealmente usaríamos Redis, pero no tenemos presupuesto aún...`
  * ✅ *Correcto:*
    ```rust
    // ponytail: sin caché. Escalar a Redis si >1000 checks/s.
    let row = db.query_scalar("SELECT accepted FROM consents WHERE user_id = ?")
        .bind(user_id)
        .fetch_optional()
        .await?;
    ```

**Filtro de verificación:** `COMMENTING-POLICY-FV-6`.

**Lectura bajo demanda:** [[.agents/knowledge/ponytail.md]] — siempre que quieras el detalle completo de la Capa 3.

---

## COMMENTING-POLICY-8 — Regla DEBT

Un aplazamiento con disparador externo o dependencia (otra épica, módulo aún no existente, estadística de producción) va a [[docs/DEBT.md]], nunca como comentario en el código.

**Aplica a:** todos los roles, al detectar un aplazamiento con disparador externo.

* **COMMENTING-POLICY-8.1 — Externo va a DEBT, Acotado va a Ponytail:**

  un aplazamiento acotado al módulo actual con umbral medible va a `ponytail:` (`COMMENTING-POLICY-7`), no a `DEBT.md`. Nunca dejes un aplazamiento con disparador externo solo en código comentado sin registrarlo en [[docs/DEBT.md]].

**Filtro de verificación:** `COMMENTING-POLICY-FV-7`.

**Lectura bajo demanda:** [[.agents/knowledge/debt-management.md]] — siempre que quieras el detalle completo de la Capa 4.

---

## COMMENTING-POLICY-9 — Prohibido Prosa Defensiva

**Aplica a:** todos los roles.

* **COMMENTING-POLICY-9.1 — Sin Justificaciones ni Excusas:**

  el comentario no narra la duda o la limitación de presupuesto/tiempo; usa `ponytail:` (`COMMENTING-POLICY-7`) si es una simplificación deliberada.
  * ❌ *Incorrecto:* `// Idealmente usaríamos X, pero por ahora Y es suficiente...`
  * ✅ *Correcto:* `// ponytail: Y sin overhead. Cambiar a X si Z ocurre.`

**Filtro de verificación:** `COMMENTING-POLICY-FV-8`.

---

## COMMENTING-POLICY-10 — Prohibido Comentarios que Repiten el Código

**Aplica a:** todos los roles.

* **COMMENTING-POLICY-10.1 — El Comentario Aporta Información Nueva:**

  un comentario que solo parafrasea la línea siguiente no aporta nada; debe explicar el por qué, no repetir el qué.
  * ❌ *Incorrecto:*
    ```rust
    fn validate_input(x: i32) {
      // Valida la entrada x
      if x < 0 { ... }
    ```
  * ✅ *Correcto:*
    ```rust
    fn validate_input(x: i32) {
      // Rechaza valores negativos; el negocio requiere x >= 0.
      if x < 0 { ... }
    ```

**Filtro de verificación:** `COMMENTING-POLICY-FV-9`.

---

## COMMENTING-POLICY-EJ — Ejemplos

### COMMENTING-POLICY-EJ-1 — Reconciliación de las Cuatro Capas en una Función Real

Caso compuesto: una sola función real donde 2+ reglas de este documento actúan juntas — no reemplaza los ❌/✅ atómicos de cada regla.

**Caso: `record_audit_event` — demuestra `COMMENTING-POLICY-3.1`, `COMMENTING-POLICY-4.1`, `COMMENTING-POLICY-7.1`.**

* **Aplicando las tres reglas:**
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

* Doc-comment (líneas 1–2): Capa 1, `COMMENTING-POLICY-3.1`.
* `ponytail:` (líneas 4–5): Capa 3, `COMMENTING-POLICY-7.1`.
* Líneas 7–8 y 11–13: Capa 2, `COMMENTING-POLICY-4.1`.
* Total: ~4 líneas de comentario legible, sin prosa.
