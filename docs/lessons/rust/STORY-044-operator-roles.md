# STORY-044 — Operator Roles: lecciones de Rust

> **Story:** [STORY-044 — Operator Roles (cimiento #14, y ÚLTIMO, del substrato de monetización)](../../execution/STORY-044-operator-roles.md).
> **Archivos que esta Story produjo y que se citan abajo:** `migrations/0020_operator_roles.sql`, `crates/shared/src/domain/operator_roles.rs`, `crates/shared/src/persistence/operator_roles.rs`, `crates/shared/src/orchestrator/operator_roles.rs`, `crates/shared/src/public_interface.rs`, `crates/app/src/main.rs`.
> **Modo de Acompañamiento:** Docente (STORY-044 declara "Rust-Engineer (Sonnet, Docente)" en su cabecera) — este archivo consolida, siguiendo el protocolo de Lecciones (ADR-0122/ADR-0124), lo no obvio de cada bloque que se implementó.

## Concepto

### Por qué una capacidad es una CLAVE DE TEXTO en un mapa, no una variante de un `enum`

El catálogo de puertos de ADR-0137 tiene ~108 tipos y sigue creciendo con cada Feature nueva. La tentación "tipada" sería modelar cada capacidad como una variante de un `enum Capability { GenerateRunSearch, ExecuteSendOrder, ... }` — el compilador te obligaría a enumerar todas, y un `match` exhaustivo te avisaría si te olvidas una. Pero esa tentación esconde un costo real: cada Feature nueva del sistema obligaría a **recompilar `shared`** solo para añadir una línea al `enum`. Esa es exactamente la textura de acoplamiento que ADR-0137 quiere evitar entre el catálogo de puertos (que vive y crece en `docs/`) y el código que lo consume.

La solución de este cimiento es tratar la capacidad como **dato**, no como código — el mismo principio que ya usó `plan_tier_quota` para el catálogo de planes ("un plan es dato configurable, no código"):

```rust
pub const CAPABILITY_MANAGE_ROLES: &str = "operator-roles.manage_roles";
pub const CAPABILITY_CREATE_CHILD_ACCOUNT: &str = "operator-roles.create_child_account";
```

(`crates/shared/src/domain/operator_roles.rs`). Solo DOS capacidades tienen un nombre reservado en Rust — porque el Core necesita reconocerlas para el invariante "último admin en pie" y el gate de cuota. El resto del universo de ~108 capacidades nunca aparece en el código Rust: vive como texto libre dentro de la matriz JSON de cada rol (`capability_matrix`), y una Feature nueva simplemente empieza a aparecer como clave posible sin que `shared` se entere ni se recompile.

El precio de esta libertad es que el compilador ya NO te protege de un typo (`"generate.run_serach"` compilaría igual que `"generate.run_search"`, y simplemente nunca coincidiría con nada). La compensación es `CapabilityMatrix::allows` (ver más abajo): denegado por defecto ante CUALQUIER clave ausente, typo incluido — el peor caso de un error de tecleo es "se deniega de más", nunca "se concede de más".

### `BTreeMap`, no `HashMap` — el orden de iteración es parte del CONTRATO, no un detalle de implementación

```rust
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(transparent)]
pub struct CapabilityMatrix(BTreeMap<String, bool>);
```

(`crates/shared/src/domain/operator_roles.rs`). `HashMap` de Rust deliberadamente ALEATORIZA su orden de iteración entre ejecuciones del proceso (protección contra ataques de colisión de hash, `SipHash` con semilla aleatoria por defecto) — es una decisión de diseño del lenguaje, no un accidente. Eso significa que si `capability_matrix` fuera un `HashMap`, serializar el MISMO contenido lógico dos veces (misma cuenta, mismas capacidades, en dos ejecuciones distintas del programa) podría producir DOS strings JSON distintos según el orden en que el `HashMap` decida iterar sus claves ese día.

Ese detalle, que parece cosmético, rompe algo concreto: `compute_role_audit_hash` incluye el JSON de la matriz como uno de sus campos. Si el JSON no es determinista, el `audit_hash` de la MISMA fila lógica cambiaría entre ejecuciones sin que el contenido real haya cambiado — violando ADR-0002 ("mismo input, mismo output, bit a bit") y rompiendo cualquier verificación de integridad que compare hashes calculados en momentos distintos.

`BTreeMap` resuelve esto por construcción: itera SIEMPRE en orden ascendente de clave. El test `capability_matrix_json_is_deterministic_regardless_of_insertion_order` lo fija directamente:

```rust
let mut a = CapabilityMatrix::new();
a.set("zeta.op", true);
a.set("alpha.op", false);

let mut b = CapabilityMatrix::new();
b.set("alpha.op", false);
b.set("zeta.op", true);

assert_eq!(a.to_json(), b.to_json()); // mismo contenido, distinto orden de inserción -> MISMO JSON
```

La regla general que deja este episodio: cuando un mapa va a entrar a un hash de auditoría (o a cualquier cálculo que deba ser reproducible), la pregunta correcta no es "¿qué tan rápido es el mapa?" sino "¿su orden de iteración es parte de mi contrato?". Si la respuesta es sí, `HashMap` está descartado de entrada — es la misma razón por la que `rust-engineer/SKILL.md` prohíbe `HashMap` "sin ordenar en un hash" como regla FIJA del Core, no como sugerencia de estilo.

### "Último admin en pie" es una función PURA que aplica el cambio a una COPIA, no un flag guardado en una fila

El diseño ingenuo de este invariante sería una columna `is_protected: bool` en el rol ADMIN, marcada una vez y consultada en cada intento de modificarlo. Ese diseño falla por una razón sutil: protege a UN rol específico, pero el invariante real ("la cuenta nunca se queda sin nadie con `MANAGE_ROLES`") puede violarse por CUATRO caminos distintos que no pasan por ese rol en particular — reasignar al único admin a otro rol, revocar su asignación, revocar el rol ADMIN completo, o editar la matriz del rol ADMIN para quitarle la capacidad. Un flag estático solo cubriría uno de los cuatro.

La solución de STORY-044 es una función pura que **simula** el cambio sobre una copia en memoria del estado, y cuenta el resultado:

```rust
pub fn admins_remaining_after(roles: &[RoleView], assignments: &[AssignmentView], change: &ProposedChange) -> usize {
    let mut roles: Vec<RoleView> = roles.to_vec();
    let mut assignments: Vec<AssignmentView> = assignments.to_vec();

    match change {
        ProposedChange::UpdateRoleMatrix { role_id, new_matrix } => { /* reemplaza la matriz de ESE rol en la copia */ }
        ProposedChange::SetAssignment { access_token_id, role_id } => { /* upsert en la copia */ }
        ProposedChange::RevokeAssignment { access_token_id } => { /* retain que la excluye */ }
        ProposedChange::RevokeRole { role_id } => { /* retain que excluye el rol */ }
    }

    assignments.iter()
        .filter(|a| roles.iter().find(|r| r.role_id == a.role_id).map(|r| r.matrix.allows(CAPABILITY_MANAGE_ROLES)).unwrap_or(false))
        .count()
}
```

(`crates/shared/src/domain/operator_roles.rs`). Es pura en el sentido estricto de ADR-0002: no toca disco, no muta `roles`/`assignments` originales (trabaja sobre `.to_vec()`), y el mismo `(roles, assignments, change)` produce SIEMPRE el mismo número. `check_last_admin_standing` es solo la envoltura que convierte "el conteo dio 0" en un `Err`. Las CUATRO vías quedan cubiertas por el MISMO cuerpo de función porque las cuatro terminan respondiendo la MISMA pregunta ("¿cuántos admins quedan tras esto?") sobre la MISMA proyección de estado — no hace falta un caso especial por vía, y añadir una quinta vía el día de mañana (si aparece) sería una variante nueva de `ProposedChange` que reutiliza el mismo conteo.

**Property test, no solo casos de borde manuales** (ADR-0133 Capa 3): `property_accepted_changes_always_leave_at_least_one_admin` recorre EXHAUSTIVAMENTE una combinación de (número de admins, roles de ruido adicionales, las cuatro vías de cambio) y afirma, para cada combinación, que si `check_last_admin_standing` aceptó el cambio, `admins_remaining_after` sobre el MISMO estado da `>= 1` — y si lo rechazó, da exactamente `0`. Esto fija la COHERENCIA entre el guardarraíl y la función que lo sostiene, no solo un puñado de ejemplos elegidos a mano. El workspace no tenía el crate `proptest` en ningún otro punto — en vez de añadir una dependencia nueva para un solo test, el recorrido exhaustivo con `for` anidados cubre el mismo espacio de forma determinista (y, para un espacio de este tamaño, más fuerte que una muestra aleatoria).

### Por qué el guardarraíl vive DENTRO de `BEGIN IMMEDIATE`, no como una verificación previa

Una verificación previa ingenua sería: "antes del `UPDATE`, hago un `SELECT` que cuenta los admins, si el conteo tras el cambio sería 0, no ejecuto el `UPDATE`". El problema es la ventana de tiempo ENTRE ese `SELECT` y el `UPDATE`: si otro proceso revoca al segundo admin en ese hueco, las dos operaciones (que individualmente parecían seguras) pueden dejar la cuenta sin ningún admin — un caso clásico de *race condition* de "verificar-luego-actuar" (TOCTOU).

`try_update_role_matrix_once` (`crates/shared/src/persistence/operator_roles.rs`) cierra esa ventana metiendo la lectura Y la escritura dentro de la MISMA transacción `BEGIN IMMEDIATE`:

```rust
let mut tx = self.pool.begin_with("BEGIN IMMEDIATE").await?; // toma el lock de escritura YA

let roles = load_active_roles_tx(&mut tx, &current.owner_id).await?;         // lee DENTRO de la tx
let assignments = load_active_assignments_tx(&mut tx, &current.owner_id).await?;

check_last_admin_standing_views(&roles, &assignments, &change)?;              // valida DENTRO de la tx
                                                                                // (si falla, `tx` se descarta sin commit)
let result = sqlx::query("UPDATE operator_roles SET ... WHERE id = ? AND row_version = ?")
    .execute(&mut *tx).await?;                                                // escribe DENTRO de la MISMA tx

let event = insert_event_in_tx(&mut tx, ...).await?;                          // el evento de auditoría, TAMBIÉN dentro
tx.commit().await?;
```

`BEGIN IMMEDIATE` (a diferencia de `BEGIN DEFERRED`) toma el lock de ESCRITURA de SQLite desde el primer statement, no cuando la transacción intenta su primer `UPDATE`. Eso significa que, mientras esta transacción está abierta, NINGÚN otro escritor puede colarse entre la lectura y la escritura — la lectura que alimenta `check_last_admin_standing` es exactamente el estado que la escritura va a modificar, sin ventana. Si el invariante se rechaza, la función devuelve el `Err` ANTES de llamar `tx.commit()` — y `sqlx::Transaction` implementa `Drop` haciendo `ROLLBACK` automático de cualquier transacción que se abandona sin `commit()` explícito, así que no hace falta escribir el `ROLLBACK` a mano: simplemente NO commitear ya deshace todo.

El test `revoking_the_only_admin_assignment_is_rejected_and_writes_nothing` verifica las DOS mitades de esta garantía: que la asignación sigue `ACTIVE` tras el intento rechazado, Y que el ledger (`operator_role_events`) no ganó ninguna fila nueva — si el rollback fuera parcial (por ejemplo, si el evento se hubiera insertado en una transacción DISTINTA de la que valida), este test lo detectaría.

### Por qué `evaluate_operator_call` COMPONE `mcp_gateway::evaluate_permission` en vez de reimplementarlo o modificarlo

`mcp_gateway.rs` (#8, ADR-0123) ya es código sellado: implementa una función pura `evaluate_permission(req: &PermissionRequest) -> PermissionOutcome` que decide si un pipeline (`Ingest`/`Generate`/.../`Withdraw`) está abierto, condicionado o bloqueado, sin saber NADA sobre roles de operador (ese concepto no existía cuando se escribió). ADR-0149 es explícito: el rol es una compuerta ADICIONAL, nunca sustituta — la evaluación de riesgo de pipeline sigue aplicando exactamente igual, tenga el operador el rol que tenga.

La forma equivocada de implementar esto sería copiar la lógica de `evaluate_permission` dentro de `operator_roles.rs` y añadirle el chequeo de rol en el medio — eso duplicaría la matriz de riesgo de ADR-0123 en dos lugares, y el día que esa matriz cambie (un pipeline nuevo, una regla nueva) alguien tendría que acordarse de actualizar AMBAS copias.

La forma correcta, y la que implementa este cimiento, es COMPONER las dos funciones puras sin tocar ninguna:

```rust
pub fn evaluate_operator_call(
    matrix: &CapabilityMatrix,
    capability_key: &str,
    permission_request: &crate::domain::mcp_gateway::PermissionRequest,
) -> CombinedVerdict {
    if let RoleVerdict::Denied { reason } = evaluate_role_capability(matrix, capability_key) {
        return CombinedVerdict::DeniedByRole { reason };
    }

    match crate::domain::mcp_gateway::evaluate_permission(permission_request) {
        crate::domain::mcp_gateway::PermissionOutcome::Granted => CombinedVerdict::Granted,
        crate::domain::mcp_gateway::PermissionOutcome::Denied { reason } => CombinedVerdict::DeniedByPipeline { reason },
    }
}
```

(`crates/shared/src/domain/operator_roles.rs`). Ni una línea de `mcp_gateway.rs` cambió. `evaluate_operator_call` simplemente IMPORTA la función existente y la llama como el segundo paso de su propia decisión — el orden importa por eficiencia (si el rol ya deniega, ni siquiera se construye/evalúa el chequeo de pipeline), pero la SEMÁNTICA es simétrica: `Granted` exige que AMBAS funciones puras hayan dicho que sí, de forma completamente independiente. El test `evaluate_operator_call_denies_by_pipeline_when_role_grants_but_pipeline_blocks` fija exactamente esta independencia: un rol que SÍ permite la capacidad, contra un pipeline bloqueado (`Execute` sin `production_override_active`), debe denegar por la compuerta de pipeline — si alguien "optimizara" el código para saltarse la llamada a `evaluate_permission` cuando el rol ya concede, este test se caería.

Esta es la misma técnica de composición (nunca modificación) que el resto del substrato ya usa para código sellado de otras Features — por ejemplo `master_account_hierarchy::issue_override` reutiliza `consent_registry::resolve_consent_verdict` sin tocar `consent_registry.rs`.

## Trucos de Senior

### El reborrow `&mut **tx` cuando la función recibe `&mut Transaction` en vez de la `Transaction` dueña

Cuando una función recibe una transacción PRESTADA (`tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>`) en vez de dueña (`let mut tx = pool.begin_with(...).await?`), pasarla a un método de `sqlx` como `.fetch_all(&mut *tx)` da un error de compilación confuso ("`&mut Transaction` no implementa `Executor`"). La razón: `*tx` desreferencia el parámetro UNA vez, dando el valor `Transaction` en sí (no su conexión interna); `&mut *tx` vuelve a `&mut Transaction`, que sigue sin ser lo que `sqlx::Executor` espera.

La solución es un desreferencia adicional: `&mut **tx`. La primera `*` quita la capa de "esto es un parámetro prestado" (`&mut Transaction` → `Transaction`); la segunda `*` atraviesa el propio `DerefMut` de `Transaction` hacia su conexión interna (`Transaction` → `Connection`), que SÍ implementa `Executor`. Cuando la transacción es dueña de la función (no un parámetro prestado), una sola `*` ya basta porque no hay capa extra de referencia que atravesar primero — por eso `insert_event_in_tx`/`load_active_roles_tx`/`load_active_assignments_tx`/`find_assignment_row_tx` (que SÍ reciben `&mut Transaction` como parámetro) usan `&mut **tx`, mientras que los métodos que abren su propia transacción con `let mut tx = ...begin_with(...)` (como en `crates/shared/src/persistence/data_portability.rs`) usan `&mut *tx`. La regla mnemotécnica: cuenta cuántas capas de "esto es una referencia" hay entre lo que tienes en la mano y el valor que `Executor` necesita, y desreferencia esa cantidad de veces.

### UPSERT dentro de una transacción ya abierta, sin `ON CONFLICT`, cuando el invariante de negocio necesita leer ANTES de decidir

`operator_assignments` tiene `UNIQUE(owner_id, access_token_id)` — "un operador tiene un rol vigente por cuenta". La forma más corta de expresar "alta o reemplazo" en SQL sería `INSERT ... ON CONFLICT(owner_id, access_token_id) DO UPDATE SET ...`. Pero `try_set_assignment_once` NO usa esa forma — hace un `SELECT` explícito primero (`find_assignment_row_tx`) y bifurca a `INSERT` o `UPDATE` en Rust. La razón no es estilo: el guardarraíl "último admin en pie" necesita el `row_version` ACTUAL de la fila (si existe) para construir el `audit_hash` encadenado (`compute_assignment_audit_hash(..., Some(&current.audit_hash), ...)`) y para incrementar `row_version` correctamente — información que un `ON CONFLICT` puro no te da de vuelta sin una cláusula `RETURNING` combinada con lógica condicional que SQLite no expresa limpiamente para este caso (encadenar un hash sobre el valor previo). Leer primero, dentro de la MISMA transacción `BEGIN IMMEDIATE` que ya tomó el lock de escritura, es tan seguro contra condiciones de carrera como el `ON CONFLICT` atómico — la ventana entre el `SELECT` y el `INSERT/UPDATE` está cerrada por el mismo lock que protege el guardarraíl del invariante, así que no se paga ningún costo de seguridad por preferir la forma explícita.

## Endurecimiento por mutación: por qué "57 tests verdes + cobertura alta" dejó 44 survivors (cierre TL, 2026-07-08)

La primera entrega pasaba 57 tests, pero el gate de mutación (`cargo-mutants`, ADR-0133 capa 8) dejó **44 sobrevivientes** — CERO de lógica (el código era correcto), todos cobertura de test faltante. El episodio deja tres reglas concretas para cualquier repositorio con varias operaciones mutables:

### 1. Cada bucle de reintento INLINE necesita su propia prueba de contención

Las 5 operaciones mutables (`create_role`, `update_role_matrix`, `revoke_role`, `set_assignment`, `revoke_assignment`) repiten el MISMO patrón de bucle `attempt += 1` / `if attempt < MAX_GUARDED_ATTEMPTS { continue }`, pero cada una lo tiene ESCRITO inline en su propio cuerpo (no comparten un helper genérico). `cargo-mutants` muta cada sitio por separado — mutar el `+=` a `*=` o el `<` a `==`/`>` en `revoke_role` es un mutante DISTINTO del mismo cambio en `create_role`. Un solo test de contención sobre una operación no vigila las otras cuatro: eso dejó 20 survivors. La cura es un test de contención por operación (`crates/shared/src/persistence/operator_roles.rs`, tests `*_exhausts_max_attempts_under_sustained_write_lock`), todos calcando el mismo patrón: un `lock_pool` retiene `BEGIN IMMEDIATE` con `busy_timeout(Duration::from_millis(0))`, la operación bajo prueba (en un `repo_pool` aparte, también con `busy_timeout=0`) choca de inmediato, agota `MAX_GUARDED_ATTEMPTS` y devuelve `WriteContention { attempts: MAX }`. La lección de arquitectura de tests: cuando un patrón se repite inline en N sitios (en vez de extraerse a un helper), tu suite necesita N pruebas — la duplicación del CÓDIGO obliga a duplicar la COBERTURA.

### 2. Toda función que construye su retorno con `..current.clone()` necesita una aserción por campo sobrescrito

Las proyecciones de fila devueltas (`OperatorRoleRow { updated_at_ns, audit_hash, audit_chain_hash, row_version, status, ..current.clone() }`) tenían ~13 survivors: un mutante que borra `audit_hash` (dejándolo como el viejo de `current`) es invisible para un test que solo relee de la DB (la DB tiene el valor correcto; la proyección en memoria es un SEGUNDO camino que el mutante ataca). La cura son los tests `*_returned_row_reflects_*` que asertan sobre la fila DEVUELTA (no releída): `audit_hash != anterior`, `audit_chain_hash == Some(anterior)`, `row_version == anterior + 1` (esto también mata `+ 1 → - 1/* 1`), `updated_at_ns == now del reloj`, y `status`/`operator_type` correctos. Una aserción por cada campo que la proyección sobrescribe — releer de la base NO basta.

### 3. La guarda `rows_affected() == 0` y el cuerpo `Ok(vec![])` se matan con el camino feliz correcto

El `== → !=` en `if result.rows_affected() == 0 { return Err(VersionConflict) }` se mata con cualquier test que haga un UPDATE EXITOSO y afirme la fila resultante: con `!=`, un update exitoso (`rows_affected == 1`) entraría en la rama de error y el `.expect(...)` del test caería. Y el mutante que reemplaza el cuerpo de `load_assignments` por `Ok(vec![])` se mata con un test que crea ≥1 asignación y afirma que la lista devuelta NO es vacía y trae el `access_token_id`/`role_id` correctos. Regla general: un getter que devuelve una colección necesita al menos un test con datos reales que afirme contenido, no solo "no truena".

### 4. Código muerto = mutante gratis: bórralo (YAGNI)

`CapabilityMatrix::from_map` existía "por si acaso" pero no lo usaba ni producción ni test (la deserialización va por `#[serde(transparent)]`). `cargo-mutants` lo reportó como survivor porque ninguna prueba lo ejercía — y la cura correcta NO es escribirle un test, es ELIMINARLO. Sin código no hay mutante, y un método público sin un solo llamador es superficie de API que hay que mantener a cambio de nada. Antes de escribir un test para matar un survivor, pregúntate si la línea merece existir.
