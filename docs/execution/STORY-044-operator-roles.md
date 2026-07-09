# STORY-044 — Operator Roles (cimiento #14 del substrato de monetización)

> **Orden de trabajo del Tech-Lead** · Ingeniero: Rust-Engineer (Sonnet, Docente) · ADR-0149 (cimiento #14) · Depende de #1 `central-identity` (`owner_id`/`access_token_id`), #3 `plan-tier-quota` (`max_child_accounts`, YA construido), extiende ADR-0123 (`mcp_gateway::evaluate_permission`), reutiliza el canal de #12 · Reutiliza ADR-0020 (Perfil D), ADR-0141 (append-only atómico + pseudonimización sobre DELETE), ADR-0137 (catálogo de puertos = universo de capacidades).

## 1. Objetivo observable

Dentro de **una sola** cuenta maestra, el dueño crea **roles de operador a la carta** (nombre libre + matriz de capacidades permitido/denegado por **puerto de Feature**, NO por módulo) y los asigna a operadores (`HUMAN` login o `AGENT` vía MCP). Una llamada de un operador a un puerto se concede solo si **su rol la permite** (compuerta #14) **Y** pasa el evaluador de riesgo de pipeline existente (ADR-0123). Invariante **"último admin en pie"**: ningún cambio deja la cuenta con cero operadores capaces de "gestionar roles". Solo un ADMIN crea cuentas hijas, con tope `max_child_accounts` de #3.

Se cimenta AHORA: catálogo de roles (mutable) + asignaciones (mutable) + ledger append-only de cambios (auditoría) + los **dos evaluadores puros** (gate de rol compuesto con ADR-0123; y "último admin en pie") + el chequeo de cuota de cuentas hijas + CLI `verify`. Se **difieren**: el transporte de red de la cascada de autoridad del fondo (relé ADR-0143) y la integración cross-máquina completa de la doble atestación de #12 (se modela la decisión/registro local), y la UI.

## 2. Ubicación (ADR-0137, excepción crosscutting)

`crates/shared`, NO crate propio — infraestructura de gobernanza transversal (produce tipos `textLabel` de plomería, consumida por ≥2 dominios, sin puerto de Alpha), igual que #5/#13/`mcp_gateway`. Archivos:
- `crates/shared/src/domain/operator_roles.rs` — Core puro.
- `crates/shared/src/persistence/operator_roles.rs` — repositorios.
- `crates/shared/src/orchestrator/operator_roles.rs` — composición.
- `migrations/0020_operator_roles.sql` — esquema.
- Cableado en `domain/mod.rs`, `orchestrator/mod.rs`, `persistence/mod.rs`, `public_interface.rs`, `crates/app/src/main.rs`.

## 3. Esquema — `migrations/0020_operator_roles.sql` (STRICT, UUIDv7)

### 3.1 `operator_roles` — catálogo de roles por cuenta (MUTABLE)
- Grupo I (`id`, `created_at`, `updated_at`, `audit_hash`, `audit_chain_hash`) + `row_version` (mutable, ADR-0141) — un rol se edita (se reclasifica su matriz).
- Grupo II: `owner_id` (cuenta dueña del catálogo), `institutional_tag`.
- Propios: `role_name TEXT NOT NULL`, `capability_matrix TEXT NOT NULL CHECK (json_valid(capability_matrix))` (JSON objeto `{ "<capability_key>": true|false, ... }`). **Sin flag de inmutabilidad** (la protección es dinámica en el Core, no un flag — ADR-0149 corrección 2026-07-07).
- `UNIQUE(owner_id, role_name)` — no se duplican nombres de rol dentro de una cuenta.

### 3.2 `operator_assignments` — asignación operador↔rol (MUTABLE)
- Grupo I + `row_version` + Grupo II: `owner_id`, `institutional_tag`, `access_token_id` (ancla de atribución del operador, ADR-0020).
- Propios: `operator_type TEXT NOT NULL CHECK (operator_type IN ('HUMAN','AGENT'))`, `role_id TEXT NOT NULL REFERENCES operator_roles(id) ON DELETE RESTRICT` (FK, jamás CASCADE — ADR-0141 M7; índice obligatorio en la FK).
- `UNIQUE(owner_id, access_token_id)` — un operador tiene UN rol vigente por cuenta.

### 3.3 `operator_role_events` — APPEND-ONLY ATÓMICA (ADR-0141) — auditoría de cambios
- Grupo I + `event_sequence_id INTEGER NOT NULL UNIQUE` + `audit_chain_hash` encadenado + **triggers BEFORE UPDATE/DELETE que abortan**. **Copiar EXACTAMENTE de `0018`/`0019`.**
- Grupo II: `owner_id`, `institutional_tag`; Grupo IV: `node_id`; subset V: `compliance_status_id` (nullable).
- Propios: `change_type TEXT NOT NULL CHECK (change_type IN ('ROLE_CREATED','ROLE_UPDATED','ROLE_REVOKED','ASSIGNMENT_SET','ASSIGNMENT_REVOKED','AUTHORITY_OVERRIDE'))`, `subject_ref TEXT NOT NULL` (el `role_id` o `access_token_id` afectado), `detail TEXT CHECK (detail IS NULL OR json_valid(detail))` (nullable).
- Índice obligatorio en `event_sequence_id` (ADR-0141 M8) y en `owner_id`.

## 4. Core — `domain/operator_roles.rs` (lógica pura, ADR-0002)

1. **`OperatorType`** enum `Human`/`Agent` (`as_str`→`HUMAN`/`AGENT`, `from_str_value`).
2. **Capacidades como clave string** (no enum de los 108 puertos — `dato, no código`). Constantes reservadas: **`CAPABILITY_MANAGE_ROLES`** (ej. `"operator-roles.manage_roles"` — la capacidad ADMIN) y **`CAPABILITY_CREATE_CHILD_ACCOUNT`** (ej. `"operator-roles.create_child_account"`).
3. **`CapabilityMatrix`** — envuelve un **`BTreeMap<String, bool>`** (ordenado — CRÍTICO para el hash determinista; nunca `HashMap`). `allows(&self, capability_key) -> bool` = **denegado por defecto** si la clave no está o es `false` (refuerza el bloqueo-por-defecto de ADR-0123). Serde a/desde el JSON de la columna `capability_matrix` (objeto ordenado).
4. **`RoleVerdict`** enum `Granted`/`Denied { reason }`. **`evaluate_role_capability(matrix: &CapabilityMatrix, capability_key: &str) -> RoleVerdict`** — puro.
5. **`evaluate_operator_call(matrix, capability_key, permission_request: &PermissionRequest) -> CombinedVerdict`** — **compone** el gate de rol (punto 4) **con `crate::domain::mcp_gateway::evaluate_permission`** (ADR-0123, ya existe): `Granted` **solo si AMBOS** conceden; si el rol niega → `DeniedByRole`; si el pipeline niega → `DeniedByPipeline`. **NO modifiques `mcp_gateway.rs`** — impórtalo y compón (ADR-0149 "adicional, no sustituta"; código sellado de #8).
6. **Invariante "último admin en pie"** (función pura, property-testable):
   - Vistas mínimas: `RoleView { role_id, matrix }`, `AssignmentView { access_token_id, role_id }`.
   - `ProposedChange` enum: `UpdateRoleMatrix { role_id, new_matrix }`, `SetAssignment { access_token_id, role_id }`, `RevokeAssignment { access_token_id }`, `RevokeRole { role_id }`.
   - **`admins_remaining_after(roles: &[RoleView], assignments: &[AssignmentView], change: &ProposedChange) -> usize`** — aplica el cambio propuesto sobre copias en memoria y cuenta cuántos operadores quedan cuyo rol asignado tiene `CAPABILITY_MANAGE_ROLES == true`.
   - **`check_last_admin_standing(...) -> Result<(), LastAdminViolation>`** — `Err` si el conteo resultante es `0`. Cubre las cuatro vías (editar matriz ADMIN, revocar asignación, reasignar, revocar rol).
7. **`can_create_child_account(actor_matrix: &CapabilityMatrix, current_child_count: i64, max_child_accounts: i64) -> ChildAccountVerdict`** — puro: `Granted` solo si el actor tiene `CAPABILITY_MANAGE_ROLES` **Y** `current_child_count < max_child_accounts`; si no, `DeniedNotAdmin` o `DeniedQuotaExceeded`. Reutiliza el `max_child_accounts` de `plan_tier_quota::PlanLimits` (#3), NO reinventes cuota.
8. **Hashes**: `compute_role_audit_hash`, `compute_assignment_audit_hash`, `compute_event_audit_hash` — SHA-256 sobre mapa canónico ordenado (estilo `compute_override_audit_hash` de #12 / `compute_request_audit_hash` de #13). La matriz entra al hash **serializada de forma ordenada** (BTreeMap).

> **Prohibido** en el Core: I/O, `thread_rng`, `HashMap` sin ordenar en un hash, `f64`.

## 5. Persistencia — `persistence/operator_roles.rs`

- **`OperatorRoleRepository`** (MUTABLE): `create_role`, `update_role_matrix` (concurrencia optimista `row_version`→`VersionConflict`), `load_roles(owner_id)`, `get_role(id)`, `revoke_role` (marca/archiva — **NUNCA DELETE físico**, ADR-0141; modela revocación como estado o como baja lógica, no `DELETE`; la FK `ON DELETE RESTRICT` protege asignaciones vivas).
- **`OperatorAssignmentRepository`** (MUTABLE): `set_assignment` (alta/cambio del rol de un operador; `row_version`), `revoke_assignment`, `load_assignments(owner_id)`.
- **`OperatorRoleEventRepository`** (APPEND-ONLY ATÓMICA): `record_event` = bucle de reintento acotado (MAX=5) → `try_record_once` con `pool.begin_with("BEGIN IMMEDIATE")` envolviendo `load_tail` + INSERT; `WriteContention { attempts }`; `is_transient_write_conflict` con el criterio canónico. **Copiar EXACTAMENTE de `DataPortabilityRequestRepository` (#13).**
- **Guardarraíl transaccional del invariante (CRÍTICO):** toda mutación que pueda afectar "último admin en pie" (`update_role_matrix` sobre un rol con la capacidad ADMIN, `set_assignment`, `revoke_assignment`, `revoke_role`) debe, **dentro de una transacción `BEGIN IMMEDIATE`**, (1) cargar el estado admin-relevante vigente (roles + asignaciones del `owner_id`), (2) llamar a `check_last_admin_standing` del Core con el cambio propuesto, (3) abortar con `LastAdminViolation` si el Core lo rechaza, (4) solo entonces escribir. El chequeo cross-fila NO puede depender solo del `row_version` de una fila — por eso va en la transacción que serializa los cambios admin-afectantes. Registra el cambio en `operator_role_events` en la MISMA transacción.

## 6. Orquestador — `orchestrator/operator_roles.rs` (composición, sin lógica)

- `define_role(identity, role_name, matrix)` — crea un rol + evento `ROLE_CREATED`.
- `assign_operator(identity, access_token_id, operator_type, role_id)` — set_assignment (con guardarraíl del invariante) + evento `ASSIGNMENT_SET`.
- `revoke_assignment` / `revoke_role` / `update_role_matrix` — cada uno pasa por el guardarraíl transaccional + su evento.
- `evaluate_call(identity, access_token_id, capability_key, permission_request)` — resuelve el rol asignado del operador, carga su matriz, y devuelve el `CombinedVerdict` del Core (rol + ADR-0123). Si el operador no tiene rol asignado → **denegado** (ADR-0149: un `AGENT` nunca opera sin rol explícito).
- `request_child_account(identity, actor_access_token_id, current_child_count, plan_limits)` — resuelve la matriz del actor y devuelve `can_create_child_account(...)` con el `max_child_accounts` del `PlanLimits` (#3). NO crea la cuenta hija (eso es #12); aquí se decide y se registra la autorización.
- `seed_admin_bootstrap(identity)` — **stub**: siembra el rol ADMIN inicial (matriz con `CAPABILITY_MANAGE_ROLES=true` + `CAPABILITY_CREATE_CHILD_ACCOUNT=true`) y lo asigna al `owner_id` raíz como primer operador `HUMAN` (el "primer admin por defecto" de ADR-0149). Idempotente.
- `apply_authority_override(...)` — **modela la decisión/registro local** de una cascada del fondo sobre una asignación de una hija: aplica el cambio de asignación + evento `AUTHORITY_OVERRIDE`. El **transporte de red** (relé ADR-0143) y la doble atestación cross-máquina de #12 quedan **diferidos** (adaptador); documenta el diferido en el código.

## 7. Reglas FIJAS (ADR-0149)
1. NUNCA se gatea a nivel de módulo — la unidad es el puerto de Feature (clave de capacidad). El módulo sería solo plantilla de conveniencia en la UI (no se implementa aquí).
2. NUNCA un cambio deja la cuenta con **cero** operadores con `CAPABILITY_MANAGE_ROLES` — "último admin en pie" (protege la capacidad, no la persona; el `owner_id` raíz es reasignable si queda ≥1 más).
3. NUNCA un operador sin `CAPABILITY_MANAGE_ROLES` crea una cuenta hija; NUNCA se excede `max_child_accounts` (#3, fijado por el proveedor).
4. NUNCA un `AGENT` (MCP) opera sin rol explícito asignado — sin rol = denegado (refuerza el bloqueo-por-defecto de ADR-0123, no lo relaja).
5. NUNCA "eliminar" un rol es un DELETE físico si hay asignaciones vivas — baja lógica / `ON DELETE RESTRICT` (ADR-0141).
6. El gate de rol es **adicional** al de ADR-0123, nunca sustituto — `evaluate_operator_call` exige AMBOS.

## 8. Tests obligatorios (ADR-0133, incl. capa 8 mutación)
- **Concurrencia — 16 escritores** sobre `operator_role_events` en **DB de archivo temporal** (NO `:memory:`): `event_sequence_id` únicos/contiguos, sin pérdida, `WriteContention` reintentado.
- **Ledger append-only — las 3 pruebas que exige la mutación (ADR-0133 capa 8, regla companion):** (1) **contención sostenida**: segundo escritor reteniendo `BEGIN IMMEDIATE` con `busy_timeout=0` → `record_event` agota `MAX` y afirma `WriteContention { attempts: MAX }`; (2) **`is_transient_write_conflict` directo** con violación UNIQUE de PK (no de `event_sequence_id`) → `false`; (3) **fidelidad de la fila devuelta** por `update_role_matrix`/`set_assignment` (audit_hash recomputado, chain encadenado, updated_at avanzado). **Copiar el patrón de `persistence/data_portability.rs` (STORY-043).**
- **"Último admin en pie" (property/unit):** revocar/reasignar al ÚNICO admin ⇒ `LastAdminViolation`; con ≥2 admins, reasignar a uno ⇒ OK; editar la matriz del rol ADMIN para quitar `MANAGE_ROLES` cuando es el único ⇒ `LastAdminViolation`. Property test: para cualquier conjunto generado, tras un cambio aceptado siempre queda ≥1 admin.
- **Gate compuesto:** un rol que permite la capacidad pero cuyo `PermissionRequest` es `Execute`/`Withdraw` (ADR-0123 bloqueado) ⇒ `DeniedByPipeline`; un rol que NIEGA la capacidad con pipeline abierto ⇒ `DeniedByRole`; ambos conceden ⇒ `Granted`. **Fija que se exigen los dos.**
- **Operador sin rol ⇒ denegado.** **`AGENT` sin rol ⇒ denegado** (mismo camino que humano).
- **Cuota de cuentas hijas:** no-admin ⇒ `DeniedNotAdmin`; admin con `current_child_count == max_child_accounts` ⇒ `DeniedQuotaExceeded`; admin bajo cuota ⇒ `Granted`. Borde exacto.
- **Matriz denegada-por-defecto:** capacidad ausente en la matriz ⇒ denegada.
- **JSON no filtra secretos** (ADR-0093): output del CLI sin credenciales/tokens crudos — allowlist de claves (`*_json_never_leaks_secret_fields`).
- **Hash determinista** con matriz ordenada (BTreeMap) — mismo input, mismo hash; property donde aplique.

## 9. Cableado del CLI (Canal #2, ADR-0142)
- `public_interface.rs`: submódulo `operator_roles` (re-export domain/orchestrator/persistence) + `OperatorRolesVerifyInput/Output` + `verify_operator_roles(input)`, forma de `verify_data_portability`. Input JSON: identidad (`owner_id`/`institutional_tag`/`node_id`), `access_token_id` del operador, `capability_key` invocada, y el `pipeline`/`institutional_tag` para el gate de ADR-0123. Output: `verdict` (`GRANTED`/`DENIED_BY_ROLE`/`DENIED_BY_PIPELINE`), el rol resuelto, `audit_hash`, `event_sequence_id`. Sin secretos.
- `crates/app/src/main.rs`: rama `"operator-roles"` en el `match` + mensaje si falta `--input` + añadir a la lista de features soportadas.

**Comando de ejemplo (debe funcionar):** tras `seed_admin_bootstrap`, un operador ADMIN invocando una capacidad permitida en pipeline abierto ⇒ `GRANTED`:
```bash
cargo run -p app -- verify operator-roles --input '{"owner_id":"acc-1","institutional_tag":"LIVE","node_id":"node-A","access_token_id":"tok-owner","capability_key":"generate.run_search","pipeline":"GENERATE"}'
```

## 10. Lección Docente (ADR-0122)
Al cerrar, `docs/lessons/rust/STORY-044-operator-roles.md`: por qué la capacidad es una clave de dato (matriz JSON) y no un enum de puertos; por qué "último admin en pie" es una función pura sobre el estado propuesto (no un flag de inmutabilidad) y por qué su guardarraíl vive DENTRO de la transacción; y por qué se compone con el evaluador de ADR-0123 en vez de mutarlo.

## 11. Prohibiciones
- **NO** commitees nada. **NO** modifiques `mcp_gateway.rs` (#8, sellado — solo se importa/compone). **NO** toques los 6 archivos protegidos del Architect ni otras features (#1–#13 solo se consumen).
- **NO** construyas el transporte de red de la cascada ni la UI — diferidos. **NO** uses modelos/agentes Opus.

---

## §12. Registro de cierre (lo llena el Tech-Lead al auditar)
- **Ingeniero:** Rust-Engineer (Sonnet, Docente). Entregó Core (`domain/operator_roles.rs`), persistencia (3 repos), orquestador, cableado, migración `0020` y lección. `CapabilityMatrix(BTreeMap<String,bool>)` `#[serde(transparent)]`; enums `OperatorType`/`LifecycleStatus`(`ACTIVE`/`REVOKED`)/`OperatorRoleChangeType`(6); veredictos `RoleVerdict`/`CombinedVerdict`(`Granted`/`DeniedByRole`/`DeniedByPipeline`)/`ChildAccountVerdict`; baja lógica vía columna `status` + `ON DELETE RESTRICT` (nunca DELETE físico).
- **Auditoría TL independiente:** ✅ reproducida: `cargo test -p shared --lib` = **630 verdes**; `cargo clippy -p shared -p app --all-targets -- -D warnings` = 0 warnings; `mcp_gateway.rs` **intacto** (composición, no mutación — verificado por `git status`); CLI `verify operator-roles` con los 3 veredictos correctos: ADMIN+pipeline abierto ⇒ `GRANTED`; operador sin rol ⇒ `DENIED_BY_ROLE`; ADMIN sobre `EXECUTE` ⇒ `DENIED_BY_PIPELINE` (prueba el gate compuesto rol AND ADR-0123). El CLI es harness de demostración (inyecta la `capability_key` en la matriz admin antes de evaluar, para exhibir la dimensión de pipeline).
- **QA por mutación:** ✅ **APTO, 0 survivors** — `cargo-mutants` sobre los 3 archivos: **169 mutantes**. La 1ª corrida completa tuvo **44 survivors** (todos de cobertura de tests, ninguno de lógica — el patrón atómico de retry/proyección de fila replicado en los 5 repos mutables + `from_map` código muerto). Muertos así: el ingeniero añadió 8 tests (5 de contención sostenida, 2 de fidelidad de fila revoke, 1 de `load_assignments` no-vacío) + eliminó `from_map` → bajó a **2 survivors** (`operator_type`/`status` de la proyección de `try_set_assignment_once`, invisibles porque el upsert usaba el mismo valor); el TL los remató con 2 escenarios discriminantes (reasignar `Human→Agent`; reactivar un operador `REVOKED→ACTIVE`) → **0 survivors** (corrida acotada a `try_set_assignment_once`: 27 caught, 1 unviable, 0 missed; los otros 166 ya caught/unviable/timeout-caught en la corrida completa). 66 tests de la feature.
- **Nota de infraestructura:** el subagente dejó viva su propia corrida de `cargo-mutants` que colisionó con la del TL sobre `mutants.out/` (lock contention) y corrompió ambos resultados; saneado (procesos huérfanos terminados, corridas del TL desde entonces con `--output` dedicado, gitignorado). El sandbox del subagente además mata `cargo-mutants` a ~67s, por lo que el gate de mutación lo corre SIEMPRE el TL, no el ingeniero.
- **Barrido de Cierre Documental:** ✅ feature `operator-roles.md` sellada 🟡 Parcial (transporte de red de la cascada + doble atestación cross-máquina de #12 + UI diferidos); `docs/TEST.md` con la entrada de #14; lección `docs/lessons/rust/STORY-044-operator-roles.md` (con la sección de endurecimiento por mutación); memoria + PROGRESS actualizados a **substrato 14/14 COMPLETO**. Sin DEBT nueva (los diferidos son "puerto ahora, adaptador después" ADR-0144, sin placeholders de tipo).
- **Estado:** ✅ **CERRADA** (2026-07-08). Backend del cimiento #14 completo (catálogo de roles + asignaciones + ledger de auditoría + evaluador compuesto + invariante "último admin en pie" + gate de cuota de cuentas hijas); transporte de cascada y UI diferidos. **Substrato de monetización 14/14 COMPLETO.**
