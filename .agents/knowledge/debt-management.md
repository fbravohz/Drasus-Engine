# Gestión de Deuda Técnica

La deuda técnica deliberada es sana en greenfield: permite avanzar en el camino crítico sin frenar por cosas que aún no muerden, siempre que quede registrada con su causa, impacto y disparador de pago.

Registro canónico: [[docs/DEBT.md]] es el índice (una fila por deuda) y [[docs/debt/DEBT-XXX.md]] la ficha completa de cada una (patrón, un archivo por deuda) — el único lugar descubrible.

Slug ID: `DEBT-MANAGEMENT`.

Tamaño: `17369 caracteres (~4342 tokens aprox., ratio 4:1)`.

---

## DEBT-MANAGEMENT-FV — Filtro de Verificación

**Aplica a:** todos los roles, antes de abrir o cerrar cualquier entrada de deuda técnica.

* **DEBT-MANAGEMENT-FV-1:** ¿tiene causa raíz clara — depende de X, espera Y, falta Z (`DEBT-MANAGEMENT-5.2`, `DEBT-MANAGEMENT-6`)?
* **DEBT-MANAGEMENT-FV-2:** ¿tiene disparador de pago medible, no una fecha vaga (`DEBT-MANAGEMENT-1`, `DEBT-MANAGEMENT-6`)?
* **DEBT-MANAGEMENT-FV-3:** ¿es un aplazamiento deliberado con justificación, no una lista de deseos ni evasión de correctitud (`DEBT-MANAGEMENT-1`, `DEBT-MANAGEMENT-5`)?
* **DEBT-MANAGEMENT-FV-4:** ¿está registrada en [[docs/DEBT.md]] y su ficha en `docs/debt/`, no solo en código o chat (`DEBT-MANAGEMENT-8`)?
* **DEBT-MANAGEMENT-FV-5:** ¿la severidad es realista, no minimizada (`DEBT-MANAGEMENT-7`)?
* **DEBT-MANAGEMENT-FV-6:** ¿el impacto actual es honesto — "nulo" solo si es verdad, "limitación" si es verdad (`DEBT-MANAGEMENT-6`)?

De ser satisfactoria cada validación de la lista, declara `DEBT-MANAGEMENT - FILTRO DE VERIFICACION APROBADO`.

---

## DEBT-MANAGEMENT-1 — Regla de Oro: Disparador Escrito Obligatorio

Un aplazamiento sin disparador escrito está olvidado.

**Aplica a:** todos los roles, ante cualquier aplazamiento deliberado.

* **DEBT-MANAGEMENT-1.1 — Sin Registro No Está Rastreada:**

  si no está en [[docs/DEBT.md]] con causa raíz y disparador, no está rastreada.
  * ❌ *Incorrecto:* `DEBT-X: Refactorizar este módulo` — "refactorizar algún día" no es un disparador.
  * ✅ *Correcto:* especificar "cuando la métrica X llegue a Y" o "cuando el módulo Z exista".

* **DEBT-MANAGEMENT-1.2 — No Es Evasión de Correctitud:**

  DEBT es para aplazamientos deliberados, no para evadir trabajo que debería hacerse hoy. Si el hueco es de correctitud, se corrige ahora, no se registra como deuda.
  * ❌ *Incorrecto:* `DEBT-Y: Falta validación exhaustiva` sin más justificación.
  * ✅ *Correcto:* corregir la validación faltante hoy mismo, si es correctitud.

**Filtro de verificación:** `DEBT-MANAGEMENT-FV-2`, `DEBT-MANAGEMENT-FV-3`.

---

## DEBT-MANAGEMENT-2 — Regla de Promoción al Cumplirse el Disparador

Un disparador que se cumple y nadie promueve queda dormido — indistinguible de uno que aún no aplica.

**Aplica a:** Tech-Lead, al cerrar cualquier hito o al responder "¿qué sigue?".

* **DEBT-MANAGEMENT-2.1 — Barrido Obligatorio al Cerrar un Hito:**

  al cerrar cualquier hito (un substrato, una auditoría, una EPIC), es obligatorio barrer [[docs/DEBT.md]] y evaluar cada disparador contra el estado nuevo. Toda deuda cuyo disparador ese hito acaba de cumplir se promueve: se le sube la severidad si aplica, se marca el disparador como `✅ CUMPLIDO`, y se mueve al marcador de "Siguiente" en `PROGRESS.md`.

* **DEBT-MANAGEMENT-2.2 — Barrido Antes de Responder Qué Sigue:**

  al responder "¿qué sigue?" no se contesta desde la lista de EPICs del ROADMAP. Primero se barre `DEBT.md` por disparadores cumplidos. Una deuda que es gap de Definición de Terminado (falta la SVF o la conexión al Banco de Pruebas de ADR-0117) va ARRIBA del siguiente EPIC, porque sin ella la feature no está Terminada.

* **DEBT-MANAGEMENT-2.3 — Severidad Honesta:**

  una deuda que bloquea la Definición de Terminado (superficie de verificación) NO es 🟡 Baja "cosmética"; es al menos 🟠 Media. 🟡 Baja se reserva para lo verdaderamente cosmético o sin impacto en correctitud ni en la verificabilidad por el propietario.

---

## DEBT-MANAGEMENT-3 — DEBT vs. TASK — Los Dos Registros No Se Mezclan

Sin un criterio escrito para distinguir una TASK de una DEBT, la elección queda a intuición. Este bloque fija el criterio: no se fusionan porque responden a preguntas distintas.

**Aplica a:** todos los roles, al detectar cualquier hallazgo o hueco.

* **DEBT-MANAGEMENT-3.1 — Criterio de Distinción:**

  | | **DEBT-XXX** (`docs/debt/`) | **TASK-XXX** (`docs/execution/`) |
  |---|---|---|
  | **Qué es** | Algo ya construido que tiene un hueco conocido | Algo aún no decidido o no construido |
  | **Pregunta que responde** | ¿Qué le falta a lo que ya existe? | ¿Qué hay que decidir o hacer a continuación? |
  | **Existe código hoy** | Sí — funciona, pero incompleto o frágil | No necesariamente; a menudo es solo una decisión |
  | **Campo que la gobierna** | Disparador de pago (condición verificable) | Bloqueante (quién debe actuar) |
  | **Cierra con** | Una Story que la salda | Una decisión canonizada (ADR/enmienda) o un entregable |
  | **Si nadie la toca** | El sistema sigue funcionando, con el hueco latente | El trabajo que depende de ella no puede empezar |

  La prueba de una línea: ¿esto es un hueco en algo que ya corre, o una pregunta que alguien tiene que contestar? Hueco → DEBT. Pregunta → TASK.

* **DEBT-MANAGEMENT-3.2 — Casos Reales para Calibrar:**

  - El sello de auditoría hereda la temporalidad equivocada al reagregar → `DEBT-030`. El código existe y corre; le falta una derivación.
  - La escala entera no admite cripto de oferta extrema → `DEBT-033` (el hueco) y `TASK-061` (la decisión de qué escala adoptar). Ambas, legítimamente: el hueco es de código, la elección es de arquitectura. Cuando un hallazgo tiene las dos caras, se registran las dos y se enlazan.
  - El catálogo de tipos de barra no coincide entre la base de datos y la especificación → `TASK-062`. No hay código que arreglar: hay un catálogo que decidir.
  - Falta muestra real de Forex/CFD → `TASK-060`. No es un hueco de código: es un insumo que solo el propietario puede entregar.

---

## DEBT-MANAGEMENT-4 — Accionables Bloqueados en el Propietario

Todo accionable que dependa de una decisión o un insumo del propietario se registra como TASK con `🔴 BLOQUEADO POR EL USUARIO` en el encabezado, y aparece marcado así en `PROGRESS.md` y en el ROADMAP.

**Aplica a:** todos los roles, al detectar un accionable que solo el propietario puede resolver.

* **DEBT-MANAGEMENT-4.1 — Nunca Solo en Prosa:**

  nunca se deja en prosa dentro de un chat. Un bloqueo que solo existe en una conversación se pierde con la conversación, y el trabajo que dependía de él se descubre parado semanas después.

---

## DEBT-MANAGEMENT-5 — Cuándo Usar `ponytail:` vs. Abrir DEBT-XXX

Dos capas de aplazamiento deliberado, según quién controla el techo del problema.

**Aplica a:** todos los roles, al detectar un aplazamiento deliberado en código.

* **DEBT-MANAGEMENT-5.1 — Usa `ponytail:` (Capa 3, en el código):**

  si el aplazamiento está acotado al módulo o función actual y tiene un umbral medible.

  Criterios: eres dueño del techo (no depende de otra EPIC); métrica clara dentro de este módulo; trivial cambiar cuando llegue el umbral; el código funciona correctamente hoy sin ello.

  ```rust
  // ponytail: sin caché. Escalar si >1000 req/s.
  let result = db.query(...).await?;
  ```

  Ejemplos: "Sin paginación; escalar si >1000 registros/mes." / "O(n) aceptable ahora; cambiar a HashMap si dataset >10k." / "Sin transacción de reintento (greenfield); añadir si jobs concurrentes."

* **DEBT-MANAGEMENT-5.2 — Usa DEBT-XXX (Capa 4, en DEBT.md):**

  si el aplazamiento tiene disparador externo o dependencia clara.

  Criterios: depende de otra EPIC, módulo futuro o decisión del Architect; causa raíz que NO puedes arreglar ahora; necesita evento externo (estadística de prod, tipo real que aún no existe, infraestructura compartida); hoy el código es incompleto o necesita cambio arquitectónico.

  ```markdown
  DEBT-003: Falta paginación en user-fetch
  - Disparador: cuando otra EPIC construya el backend de paginación
  - Impacto: limita la SVF, no la correctitud
  ```

  Ejemplos: "Tipo `BacktestResult` aún no construido; espera EPIC de validación." / "Fan-out al event bus diferido; el bus aún no existe (ADR-0085)." / "Fetch de datos reales del servidor diferido; el adaptador se construye en EPIC-5."

**Filtro de verificación:** `DEBT-MANAGEMENT-FV-3`.

**Lectura bajo demanda:** [[.agents/knowledge/commenting-policy.md]] — al escribir un comentario de Capa 1-3 en código. [[.agents/knowledge/ponytail.md]] — al usar la sintaxis `ponytail:` (Capa 3). [[.agents/memory/comentarios-ponytail-reconciliacion.md]] — si necesitas el historial de reconciliación de comentarios `ponytail:` heredados.

---

## DEBT-MANAGEMENT-6 — Estructura de una Entrada DEBT

Toda entrada de deuda, en su ficha [[docs/debt/DEBT-XXX.md]] (patrón, un archivo por deuda), lleva estos campos.

**Aplica a:** todos los roles, al abrir una ficha de deuda nueva.

* **DEBT-MANAGEMENT-6.1 — Campos Obligatorios:**

  Severidad (🔴 Alta / 🟠 Media / 🟡 Baja), Origen (dónde se detectó: Story, revisión de código, QA), Descripción (qué falta o es incompleto), Impacto actual (qué se rompe HOY: nulo, limitación, riesgo), Causa raíz (por qué está diferida: dependencia, decisión de arquitecto, greenfield), Disparador de pago (evento o condición que detona su construcción), Plan (pasos concretos para pagarla), Estado (Abierta / En pago / Pagada).

**Filtro de verificación:** `DEBT-MANAGEMENT-FV-1`, `DEBT-MANAGEMENT-FV-2`, `DEBT-MANAGEMENT-FV-6`.

**Plantilla de referencia:**
```
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

## DEBT-MANAGEMENT-7 — Niveles de Severidad

**Aplica a:** todos los roles, al asignar severidad a una entrada de deuda.

* **DEBT-MANAGEMENT-7.1 — Tabla de Niveles:**

  | Nivel | Significado | Cuándo pagar |
  |---|---|---|
  | 🔴 Alta | Puede corromper datos o violar un invariante bajo condiciones alcanzables. | Pronto (antes del release). |
  | 🟠 Media | Fallo seguro (sin corrupción) pero con pérdida de función/correctitud bajo condiciones aún no presentes. | Antes de que la condición llegue. |
  | 🟡 Baja | Cosmético o diferido por secuenciación; sin riesgo de correctitud. | Bajo demanda o al cierre de la fase. |

**Filtro de verificación:** `DEBT-MANAGEMENT-FV-5`.

---

## DEBT-MANAGEMENT-8 — Cómo Registrar una DEBT Nueva

**Aplica a:** todos los roles, al detectar un aplazamiento deliberado que cumple los criterios de `DEBT-MANAGEMENT-5.2`.

* **DEBT-MANAGEMENT-8.1 — Detecta el Aplazamiento:**

  identifica el hallazgo, ej. "Falta el tipo BacktestResult — espera EPIC-3".

* **DEBT-MANAGEMENT-8.2 — Decide ponytail: o DEBT-XXX:**

  | Pregunta | Respuesta | Acción |
  |---|---|---|
  | ¿Depende de otra EPIC/módulo futuro? | Sí | → DEBT-XXX |
  | ¿Tienes un umbral medible para cambiarlo? | Sí | → `ponytail:` |
  | ¿Es causa raíz externa? | Sí | → DEBT-XXX |
  | ¿Puedes arreglarlo hoy si quisieras? | No | → DEBT-XXX |

* **DEBT-MANAGEMENT-8.3 — Registra la Deuda (Índice + Ficha):**

  crea la ficha en [[docs/debt/DEBT-NNN.md]] (patrón, un archivo por deuda) con la estructura completa (`DEBT-MANAGEMENT-6`), y añade una fila al índice [[docs/DEBT.md]] (`ID | Severidad | Resumen | Estado | Ficha`).

* **DEBT-MANAGEMENT-8.4 — Enlaza desde el Documento que la Consume:**

  si la deuda condiciona un trabajo futuro, añade el puntero a `DEBT-NNN` desde el documento de dominio que lo consume — la feature, el ADR o la bitácora de [[.agents/state/tech-lead/PROGRESS.md]]. Escribir en `.agents/memory/` está prohibido (`MEMORY-POLICY-14`).

* **DEBT-MANAGEMENT-8.5 — Prohibido Dejar el Aplazamiento Solo en Código Comentado:**

  el comentario apunta a [[docs/DEBT.md]], donde vive el motivo de fondo y el disparador — nunca se deja la justificación completa solo en el código.
  * ❌ *Incorrecto:* `// "Falta el tipo BacktestResult, lo completamos cuando venga de EPIC-3"` seguido de `let metrics = BTreeMap::new();`, o `// TODO: añadir caché cuando haya tiempo` seguido de `let result = fetch_data();`.
  * ✅ *Correcto:* `let metrics = BTreeMap::new(); // placeholder; ver DEBT-009`.

**Filtro de verificación:** `DEBT-MANAGEMENT-FV-4`.

**Lectura bajo demanda:** [[.agents/state/tech-lead/PROGRESS.md]] — al registrar un hallazgo nuevo, para enlazar la entrada de la bitácora operativa correspondiente.

**Plantilla de referencia:** la ficha usa la estructura completa de `DEBT-MANAGEMENT-6` — los ocho campos, sin recortar `Causa raíz` ni `Plan`. La fila del índice [[docs/DEBT.md]] es:
```
| DEBT-NNN | 🟡 Baja | [resumen en una línea] | Abierta | [ficha](./debt/DEBT-NNN.md) |
```

---

## DEBT-MANAGEMENT-9 — Cierre de DEBT (Pago)

**Aplica a:** todos los roles, al pagar una deuda registrada.

* **DEBT-MANAGEMENT-9.1 — Abre una Story:**

  para su corrección (ej. STORY-041).

* **DEBT-MANAGEMENT-9.2 — Edita el Estado en [[docs/DEBT.md]]:**

  debe enlazar a la Story que la saldó — trazabilidad es obligatoria.
  * ❌ *Incorrecto:* `- **Estado:** ✅ Pagada.` sin enlace a la Story.
  * ✅ *Correcto:* `- **Estado:** ✅ **Pagada** — [STORY-041](../execution/STORY-041-...md)`.

* **DEBT-MANAGEMENT-9.3 — Referencia en PROGRESS.md:**

  la bitácora del Tech-Lead referencia la Story que saldó la deuda.

---

## DEBT-MANAGEMENT-10 — Referencia por Documento Canónico, No por Ejemplo

Cuando una Orden de Trabajo diga "sigue el patrón de X", se refiere al documento de knowledge que define X, nunca a "la feature que ya lo tiene implementado".

**Aplica a:** todos los roles, al citar un patrón a seguir en una Orden de Trabajo.

* **DEBT-MANAGEMENT-10.1 — El Molde Es el Documento, No el Código:**

  ningún archivo del repositorio es "el ejemplo a seguir" completo: el código es la foto de lo que alguien entendió un día concreto; el molde es el documento. Elegir referencia por antigüedad — "la primera que se construyó" — propaga defectos por copia a través de las implementaciones siguientes. Si un ejemplo de código se cita, se cita para un punto concreto y auditado, nunca como plantilla entera.
  * ❌ *Incorrecto:* "copia cómo lo hizo la feature Y" para implementar un ledger append-only.
  * ✅ *Correcto:* seguir el documento de knowledge del patrón como molde; citar la feature Y solo para un punto concreto ya auditado.

---

## DEBT-MANAGEMENT-EJ — Ejemplos

Casos reales completos — decisión de registro y estructura de la ficha aplicadas juntas —, no solo la sintaxis aislada de `DEBT-MANAGEMENT-5` o `DEBT-MANAGEMENT-6`.

### DEBT-MANAGEMENT-EJ-1 — DEBT con Causa Raíz Externa

Demuestra `DEBT-MANAGEMENT-5.2` (criterio DEBT-XXX) y `DEBT-MANAGEMENT-6.1` (estructura de entrada) actuando juntos.

```markdown
### DEBT-009 · Placeholders de tipos del guantelete en #7
- **Severidad:** 🟡 Baja
- **Origen:** STORY-034 (institutional-report-engine).
- **Descripción:** `institutional-report-engine` (#7) consume
  `BacktestResult`/`RobustnessScore` que hoy son placeholders
  (`pub struct X;`); la firma es reproducible, pero el mapeo desde
  los tipos reales del guantelete aún no existe.
- **Impacto actual:** ninguno — el puerto es estable; es un mapeo
  pendiente, no un bug.
- **Disparador de pago:** cuando el guantelete produzca los tipos reales
  (EPIC de validación/ejecución).
- **Estado:** Abierta.
```

Por qué va a [[docs/DEBT.md]]: depende de otra EPIC (validación/ejecución), no se puede arreglar hoy.

### DEBT-MANAGEMENT-EJ-2 — Simplificación Acotada con `ponytail:`

Demuestra `DEBT-MANAGEMENT-5.1` (criterio ponytail) con el razonamiento completo de por qué NO es DEBT.

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

Por qué es `ponytail:`: eres dueño del techo (si llega 1000/s, añades Redis); no depende de nada externo.

### DEBT-MANAGEMENT-EJ-3 — DEBT con Causa Raíz Externa y Disparador Claro

Demuestra `DEBT-MANAGEMENT-5.2`, `DEBT-MANAGEMENT-6.1` y `DEBT-MANAGEMENT-7.1` (severidad) actuando juntos.

```markdown
### DEBT-015 · #11 canonical_delta_bytes sin test de valor-dorado
- **Severidad:** 🟡 Media
- **Origen:** QA por mutación de STORY-039.
- **Descripción:** `compute_backup_delta` sí está cazado; lo que falta
  es un test de valor-dorado sobre `canonical_delta_bytes` que ancle
  los bytes exactos producidos.
- **Impacto actual:** nulo (fase greenfield; el adaptador de subida
  S3/R2 está diferido).
- **Disparador de pago:** ANTES de construir el adaptador de almacén
  de objetos (antes de que exista un respaldo real que pudiera salir vacío).
- **Plan:** añadir test de valor-dorado con semilla fija.
- **Estado:** Abierta.
```

Por qué va a [[docs/DEBT.md]]: el disparador es externo ("antes de que S3 exista"); hoy es cosmético.
