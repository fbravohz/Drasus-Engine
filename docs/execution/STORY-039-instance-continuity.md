# STORY-039 — Instance Continuity (cimiento #11 del substrato · respaldo cifrado + maestro itinerante)

| Campo | Valor |
|---|---|
| **ID** | STORY-039 |
| **Tipo** | Story (código — undécimo cimiento del substrato de pricing/SaaS) |
| **Épica (Fase)** | Cimientos de Monetización (transversal, greenfield) |
| **Sprint** | substrato-monetizacion |
| **Estado** | 🟡 En curso (Core cripto + gate de custodia + esquema + puertos + CLI; adaptador de almacén de objetos y liberación forzada central diferidos) |
| **Creada** | 2026-07-06 |
| **Feature** | [`instance-continuity`](../features/instance-continuity.md) |
| **ADRs** | ADR-0146 (cimiento #11 — rector) · ADR-0093 (secretos jamás salen, cifrado client-side) · ADR-0143 (tres planos, Cabina de Mando como almacén diferido) · ADR-0145 (motivo de urgencia — atestación soberana irremplazable) · ADR-0137 (puertos) · ADR-0141 (append-only + optimistic concurrency) · ADR-0020 (Perfil D) · ADR-0002 (FCIS + determinismo) |

## 1. Objetivo llano

Construir la **continuidad y portabilidad de instancia**: (1) un **respaldo cifrado client-side** de la DB local (el proveedor guarda bytes opacos que jamás puede leer) y (2) un **relevo de custodia (maestro itinerante)** que garantiza que **exactamente una máquina** es la titular escritora de la cadena de auditoría en cada instante, con detección de conflicto. Motivo de urgencia (ADR-0146): en el tier de pago la telemetría de trabajo se suprime en origen (ADR-0143), así que el historial soberano de #10 **no** está en el proveedor — un disco muerto lo borra irreversiblemente.

**Alcance ahora vs. después (ADR-0146 "contrato/esquema ahora, adaptador después"):**
- **Ahora (esta Story):** Core (KDF desde el secreto maestro + cifrado/descifrado **AES-256-GCM autenticado** + cálculo del delta a respaldar **excluyendo secretos** + verificación determinista de titularidad de custodia) + esquema (registro de respaldos append-only + estado de custodia mutable) + consumo real de `AccountIdentity` (#1) + puertos `identity_in`/`backup_blob_out`/`custody_status_out` + CLI verify.
- **Después (diferidos):** el **adaptador de almacén de objetos** (S3/R2 subida/descarga real); la **liberación forzada de titularidad** desde el panel de cuenta central (Cabina de Mando, diferida); el **cajón de UI** (toggle de respaldo + indicador de titularidad, SVF).

## 2. Agentes y Modo de Acompañamiento (ADR-0120/0122)

| Agente | Modelo | Modo |
|---|---|---|
| Rust-Engineer | Sonnet | **Docente** |

Docente: conceptos nuevos — cifrado autenticado client-side (KDF desde secreto maestro, AES-256-GCM, por qué el tag GCM detecta manipulación), el **nonce inyectado y sembrado** (por qué un nonce GCM jamás se reutiliza y aun así los tests deben ser deterministas — mismo principio que el RNG sembrado de #9), y el **gate de titularidad de custodia** (concurrencia optimista a **nivel de instancia** en vez de a nivel de fila). Lección en `docs/lessons/rust/STORY-039-instance-continuity.md`.

## 3. Gate de Coherencia — resultado (contraste bidireccional)

- **Stack:** limpio. Crate `crates/shared` (plomería crosscutting, excepción ADR-0137: tipos técnicos nuevos del cimiento #11 — blob cifrado de respaldo + estado de titularidad; consumidos por ≥2 dominios (arranque de la app + `licensing-system`), sin puerto de Alpha en el canvas). Añade dependencias de cripto vetadas (`aes-gcm` para AES-256-GCM + un KDF estándar — `argon2` o `hkdf`); `rand 0.8` y `sha2 0.10` ya están.
- **Cifrado client-side real y autenticado — regla obligatoria #1 (ADR-0093 FIJO):** el blob se cifra con **AES-256-GCM** (cifrado autenticado); la clave se deriva del **secreto maestro** del usuario por un **KDF estándar**. La clave y el secreto maestro **NUNCA** se persisten, **NUNCA** salen de la máquina, **NUNCA** aparecen en ninguna columna ni salida. Pruebas: (a) round-trip `encrypt→decrypt` recupera el plaintext exacto; (b) alterar **un solo byte** del ciphertext (o del tag) hace **fallar** el descifrado (la autenticación GCM detecta manipulación) — no devuelve basura silenciosa.
- **Nonce inyectado y sembrado — regla obligatoria #2 (ADR-0002 determinismo):** el **nonce** de AES-GCM se **inyecta** (fuente/puerto de nonce), **sembrado y determinista en tests** (`StdRng::seed_from_u64`, mismo patrón que el ruido gaussiano de #9), **aleatorio en producción**. NUNCA `rand::thread_rng()` embebido en el Core. El nonce **se almacena junto con el blob** (no es secreto — es requisito para descifrar después); jamás se reutiliza un nonce con la misma clave (nonce-reuse en GCM es catastrófico — documentarlo en la lección). Prueba: mismo plaintext + misma clave + mismo nonce sembrado → mismo ciphertext (reproducible); nonces distintos → ciphertexts distintos.
- **Secretos jamás en el blob ni en el registro — regla obligatoria #3 (ADR-0093):** el cálculo del **delta a respaldar EXCLUYE** las credenciales de bróker y las IPs de servidores live (mismas clases de secreto que se excluyen de la telemetría). El registro de respaldos **NUNCA** guarda la clave de cifrado ni el secreto maestro — solo: marca de tiempo del último snapshot, **hash** del blob, tamaño (bytes, INTEGER), `node_id` titular. Guardarraíl estructural con test (como #8/#9/#10): ninguna columna ni salida porta secreto ni clave.
- **Gate de titularidad exclusiva — regla obligatoria #4 (el mecanismo NUEVO, ADR-0146 FIJO):** en cada instante **exactamente una** máquina es la titular escritora de la cadena de auditoría de la cuenta. Reclamar la titularidad usa **concurrencia optimista a nivel de instancia**: un contador `custody_epoch` monotónico; reclamar desde un epoch vencido → error tipado `CustodyConflict` (la segunda máquina queda **bloqueada**, NUNCA escribe en paralelo). Verificación **determinista y pura** `is_current_titular(node_id, custody_state) -> bool`. El relevo **NO** exige que la máquina anterior esté viva (la liberación forzada central es el adaptador diferido). Pruebas: (a) dos reclamos desde el **mismo** epoch → uno gana (epoch+1), el otro `CustodyConflict`; (b) `is_current_titular` es `true` solo para el `node_id` titular vigente.
- **Dos tablas — regla obligatoria #5 (ADR-0141):**
  - **Registro de respaldos APPEND-ONLY ATÓMICA** (cada snapshot subido es un hecho histórico permanente) → `event_sequence_id UNIQUE` + triggers anti UPDATE/DELETE + `BEGIN IMMEDIATE`+reintento+`WriteContention`. Grupo I + Perfil D. Campos propios: timestamp del snapshot, hash del blob, tamaño (INTEGER bytes), `node_id` que respaldó, nonce (no secreto). **Prueba de 2 escritores obligatoria** (archivo temporal, no `:memory:`).
  - **Estado de custodia MUTABLE** (la titularidad cambia con el tiempo) → `custody_epoch` (concurrencia optimista, `UPDATE ... WHERE owner_id=? AND custody_epoch=?` + `rows_affected()==0` → `CustodyConflict`). Grupo I + Perfil D. Campos propios: `titular_node_id`, `custody_epoch`, estado.
  - STRICT, UUIDv7. `audit_hash`/`audit_chain_hash` en ambas (integridad).
- **Determinismo y FCIS — regla obligatoria #6 (ADR-0002):** KDF, cifrado/descifrado, cálculo del delta y verificación de titularidad son **puros** dado sus inputs (con el nonce inyectado). Sin reloj de sistema ni aleatoriedad sin semilla en el Core. Reloj inyectado en la Shell.
- **Puertos (ADR-0137):** `identity_in ← AccountIdentity` (Input 1, tipo **real** de #1 — NO placeholder), `backup_blob_out → <blob cifrado>` (Output 0..1), `custody_status_out → <estado titularidad>` (Output 1). Bajo `public_interface::instance_continuity`. Nombres de `struct` los fija el ingeniero.
- **NO acoplar features entre sí (ADR-0137):** `custody_status_out` lo **consumirá** `licensing-system` (#2) más adelante — es consumidor downstream, NO una dependencia. #11 **no** importa `licensing-system`. Solo depende de `shared` (incl. el tipo real de #1).
- **Anti-`tenant_id` (ADR-0144):** todo bajo `owner_id`/`node_id`; PROHIBIDO calcar `tenant_id`.
- **Perfil ADR-0020:** Perfil D — Grupo I + II (`owner_id`) + IV (`node_id` — qué máquina es titular). Sin Grupo III (no hay linaje genómico).
- **Diferidos:** adaptador de almacén de objetos (S3/R2), liberación forzada desde el panel central (Cabina de Mando), UI toggle. NO los cablees.

## 4. Instrucciones de despacho (prompt exacto)

> Eres el **Rust-Engineer** en **Modo Docente**. Lee primero: `CLAUDE.md`, `.claude/skills/base/SKILL.md`, `.claude/skills/rust-engineer/SKILL.md` (§4 atomicidad de ledgers + placeholders→DEBT), **esta Orden completa (STORY-039)**, la feature `docs/features/instance-continuity.md`, el ADR-0146 (`docs/adr/ADR-0146.md`), el tipo real `AccountIdentity` de #1 en `crates/shared/src/domain/central_identity.rs`, el patrón de **RNG sembrado inyectado** de #9 en `crates/shared/src/domain/data_aggregation.rs` (para el nonce), el patrón **append atómico** `crates/shared/src/persistence/enriched_domain_events.rs`, el patrón **concurrencia optimista** (`row_version`) `crates/shared/src/persistence/central_identity.rs` (lo adaptarás a `custody_epoch` a nivel de instancia), y los ADR-0146, ADR-0093, ADR-0143, ADR-0137, ADR-0141, ADR-0020, ADR-0002. Declara que los leíste.
>
> **Gate de Lectura Pre-Código:** (a) confirma `crates/shared`; (b) confirma el tipo real `AccountIdentity` de #1 que consumirás en `identity_in`; (c) elige crates de cripto **vetados** (`aes-gcm` para AES-256-GCM; KDF `argon2` o `hkdf`) y decláralos; (d) copia el patrón de RNG sembrado inyectado (#9) para el nonce, el de append atómico (#6) y el de concurrencia optimista (#1); (e) confirma que NO cableas el adaptador de almacén de objetos, ni la liberación forzada central, ni la UI.
>
> **Construye (Core + esquema + puertos; adaptadores diferidos):**
> 1. **Migración `migrations/00NN_instance_continuity.sql`** (usa el siguiente número libre) con DOS tablas: (a) registro de respaldos APPEND-ONLY (`event_sequence_id UNIQUE` + triggers anti UPDATE/DELETE; timestamp, hash del blob, tamaño INTEGER, `node_id`, nonce no-secreto, `audit_chain_hash`); (b) estado de custodia MUTABLE (`custody_epoch` para concurrencia optimista; `titular_node_id`, estado). Grupo I + Perfil D. `STRICT`, UUIDv7. **NINGUNA columna guarda la clave ni el secreto maestro.**
> 2. **Core `domain/instance_continuity.rs`:** derivación de clave por KDF desde el secreto maestro; cifrado/descifrado **AES-256-GCM autenticado** (nonce inyectado); cálculo del delta a respaldar **excluyendo secretos de bróker/IPs live**; `is_current_titular` (verificación determinista de titularidad); tipos de los puertos `backup_blob_out`/`custody_status_out`. Puro, determinista, sin I/O.
> 3. **Shell:** `persistence/instance_continuity.rs` — repo del registro de respaldos **append-only atómico** (`BEGIN IMMEDIATE`+reintento+`WriteContention`) + repo del estado de custodia **mutable** (`custody_epoch`, reclamo con concurrencia optimista → `CustodyConflict`); `orchestrator/instance_continuity.rs` — flujo: tomar snapshot lógico → derivar clave → cifrar delta (nonce inyectado) → registrar el respaldo + resolver/reclamar titularidad. Reloj y fuente de nonce inyectados.
> 4. **`public_interface`:** submódulo `instance_continuity` con los tres puertos. Sin secretos ni claves (ADR-0093).
> 5. **CLI `verify`:** `cargo run -p app -- verify instance-continuity --input '<json>'` que, dado un plaintext + secreto maestro + nonce sembrado + estado de custodia, reproduce el observable (round-trip de cifrado y veredicto de titularidad) en JSON — **sin** emitir la clave ni el secreto.
>
> **Pruebas discriminantes (rojo→verde):**
> - **Round-trip de cifrado:** `encrypt→decrypt` con la misma clave/nonce recupera el plaintext exacto.
> - **Autenticación GCM:** alterar un byte del ciphertext/tag hace **fallar** el descifrado (error tipado, no basura). Assert.
> - **Nonce sembrado determinista:** mismo plaintext+clave+nonce sembrado → mismo ciphertext; nonces distintos → distintos. (Patrón #9.)
> - **Secretos/clave nunca persistidos ni en salida:** ninguna columna ni struct de salida porta la clave, el secreto maestro, credenciales de bróker ni IPs live. Assert estructural (ADR-0093).
> - **Delta excluye secretos:** el conjunto a respaldar no incluye las clases de secreto. Assert.
> - **Titularidad exclusiva:** dos reclamos desde el mismo `custody_epoch` → uno gana (epoch+1), el otro `CustodyConflict`; `is_current_titular` true solo para el titular vigente.
> - **Append atómico + concurrencia:** 16 escritores sobre el registro de respaldos (archivo temporal) → N filas, `event_sequence_id` 1..=N denso. Cae sin `BEGIN IMMEDIATE`.
> - **Append-only** (UPDATE/DELETE rechazados por trigger), `event_sequence_id` UNIQUE, `audit_chain_hash` encadenado, STRICT.
> - Cobertura con `cargo llvm-cov --workspace --summary-only`.
>
> **Comentarios en español, identificadores en inglés (ADR-0121).** Gate: `cargo test --workspace` verde + `cargo clippy --workspace --all-targets -- -D warnings` cero warnings. Si consumes algún tipo de #1 como placeholder en vez del real, cruza a DEBT (regla del skill) — pero debes consumir el `AccountIdentity` **real**.
>
> **Docente:** `docs/lessons/rust/STORY-039-instance-continuity.md` cero-conocimiento: qué es el cifrado autenticado y por qué el tag GCM prueba integridad+confidencialidad, qué es un KDF y por qué la clave se deriva del secreto maestro (que nunca sale), por qué el nonce jamás se reutiliza pero aun así se siembra en tests (determinismo, ADR-0002), y qué es el gate de titularidad de custodia (concurrencia optimista a nivel de instancia, por qué evita dos escritores de la cadena de auditoría). Cita el código real.
>
> **NO construyas el adaptador de almacén de objetos (S3/R2), ni la liberación forzada central, ni la UI. NO toques migraciones existentes (solo crea la nueva). NO hagas commits. NO toques archivos personales ni los 6 del Architect** (`docs/ROADMAP.md`, `docs/features/licensing-system.md`, `docs/features/usage-metering.md`, `docs/features/central-identity.md`, `docs/README.md`, `docs/moonshots/encrypted-local-backup.md`). Al terminar reporta: archivos, crates de cripto elegidos, `cargo test --workspace` + `clippy` + `llvm-cov`, salida del `verify`, y confirma que ninguna columna/salida porta clave ni secreto.

## 5. Criterio de aceptación (verificable)

| # | Criterio | Prueba |
|---|---|---|
| 1 | Migración `STRICT`: registro de respaldos append-only (triggers + UNIQUE) + custodia mutable (`custody_epoch`) + Grupo I + Perfil D; sin clave/secreto en ninguna columna | inspección + tests |
| 2 | Cifrado AES-256-GCM: round-trip recupera plaintext; manipular un byte → descifrado falla | tests de cripto |
| 3 | Nonce inyectado y sembrado (determinista en test, aleatorio en prod); nunca `thread_rng` en el Core | test de reproducibilidad |
| 4 | Clave/secreto maestro/credenciales/IPs jamás persistidos ni en salida | assert estructural (ADR-0093) |
| 5 | Delta a respaldar excluye secretos de bróker/IPs live | assert |
| 6 | Gate de titularidad: dos reclamos desde el mismo epoch → uno gana, el otro `CustodyConflict`; `is_current_titular` correcto | test de concurrencia de instancia |
| 7 | Registro de respaldos append-only atómico + 2 escritores | test de concurrencia (cae sin la tx) |
| 8 | Consume el `AccountIdentity` **real** de #1 (no placeholder) | inspección |
| 9 | CLI `verify instance-continuity` sin emitir clave/secreto | `cargo run -p app -- verify …` |
| 10 | Lección Docente | existe el archivo |
| 11 | Verde + cobertura | `cargo test --workspace` + `cargo llvm-cov` |

## 6. Comandos de validación (usuario)

```bash
cargo test -p shared instance_continuity
cargo clippy --workspace --all-targets -- -D warnings
cargo run -p app -- verify instance-continuity --input '{"master_secret":"correct horse battery staple","plaintext":"snapshot-bytes","nonce_seed":42,"custody":{"titular_node_id":"node-A","custody_epoch":3},"my_node_id":"node-A"}'
```

## 7. Registro de ejecución

- 2026-07-06 · Tech-Lead · Gate de Coherencia corrido (contraste bidireccional). Reglas obligatorias: (1) cifrado AES-256-GCM autenticado client-side, clave por KDF desde el secreto maestro que jamás sale (ADR-0093); (2) nonce inyectado y sembrado (determinismo ADR-0002, patrón #9, nonce nunca reutilizado); (3) secretos/clave jamás en el blob ni en el registro; (4) gate de titularidad exclusiva por concurrencia optimista a nivel de instancia (`custody_epoch` → `CustodyConflict`); (5) dos tablas (registro de respaldos append-only atómica + estado de custodia mutable); (6) determinismo + FCIS. Perfil D. Consume `AccountIdentity` real de #1; NO acopla `licensing-system` (consumidor downstream). Anti-`tenant_id`. Adaptador de almacén de objetos + liberación forzada central + UI diferidos. Orden creada, despacho a Rust-Engineer (Sonnet, **Docente**).
- 2026-07-06 · Rust-Engineer (Sonnet, Docente) · Entregado. Migración `0017_instance_continuity.sql` (dos tablas STRICT — `instance_backups` append-only atómica + `custody_state` mutable `custody_epoch`), Core `domain/instance_continuity.rs` (KDF `Argon2id` real, cifrado `Aes256Gcm` autenticado, nonce `StdRng::seed_from_u64` inyectado, `compute_backup_delta`/`is_excluded_from_backup` excluye secretos, `decide_custody_claim`/`is_current_titular`), Shell `persistence/instance_continuity.rs` (repo append-only atómico `BEGIN IMMEDIATE`+`WriteContention` + repo custodia mutable con `CustodyConflict` + prueba de 16 escritores en archivo), orquestador, `public_interface::instance_continuity` (3 puertos) + `verify instance-continuity`, CLI en `crates/app/src/main.rs`, lección Docente. Deps `aes-gcm = "0.10"` + `argon2 = "0.5"`. Consume `AccountIdentity` real de #1 (no placeholder). 33 tests de la feature. Sin placeholders.
- 2026-07-06 · Tech-Lead · **Auditoría independiente APROBADA** (reproducción propia: `cargo test -p shared instance_continuity` 33 verdes, clippy 0 warnings; verificado en el código real: cripto REAL — `Aes256Gcm::new`+`Aead` y `Argon2::hash_password_into`, no un juguete; nonce sembrado/inyectado sin `thread_rng` en el Core; tests de manipulación GCM (un byte → falla), clave-errónea → falla, y guardarraíl estructural de que ninguna columna/salida porta clave/secreto/credencial/IP; `compute_backup_delta` excluye secretos; gate de custodia `CustodyConflict` real; append-only atómico con 16 escritores). Cobertura 96–99% en los 3 archivos.
- 2026-07-06 · QA (mutación, `cargo-mutants` sobre el Core, ejecutada por el TL) · **APTO**. 38 mutantes: 32 cazados, 3 inviables, 3 sobrevivientes — los 3 son la MISMA función `canonical_delta_bytes` (serialización del delta a bytes) sin test de valor-dorado. La lógica de seguridad quedó 100% cazada (cifrado autenticado, `CustodyConflict`, filtro de exclusión de secretos, hashes de auditoría). Sobrevivientes = completitud de datos, no seguridad → **DEBT-015** (no bloqueante).
- 2026-07-06 · Tech-Lead · **CIMIENTO #11 CERRADO — SUBSTRATO 11/12.** Feature sellada. DEBT-015 registrada. Pendiente de autorización: commit agrupado.

## 8. Deudas / diferidos registrados

- **Adaptador de almacén de objetos (S3/R2):** la subida/descarga real del blob contra la Cabina de Mando es adaptador de red diferido (disparador: primer cobro real, tras EPIC-5); aquí se modela el blob cifrado y el puerto, no el transporte.
- **Liberación forzada de titularidad desde el panel central:** cuando la máquina anterior está muerta, el reclamo forzado corre en la Cabina de Mando (diferida) reutilizando el self-service de `licensing-system`; aquí se modela el gate de titularidad local + la detección de conflicto, no el flujo central.
- **Superficie propia (Canal #1):** toggle de respaldo + indicador de titularidad en el cajón de ajustes → tanda de UI final (DEBT-005). Feature con Superficie propia (no plomería): su SVF es parte de esa tanda.
- **DEBT-015 (QA):** `canonical_delta_bytes` sin test de valor-dorado (3 sobrevivientes de mutación) → añadir el golden-value **antes** del adaptador de subida real. No bloqueante (greenfield, sin respaldo real aún).
