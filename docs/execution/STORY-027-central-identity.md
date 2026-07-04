# STORY-027 — Central Identity (cimiento #1 del substrato de monetización)

| Campo | Valor |
|---|---|
| **ID** | STORY-027 |
| **Tipo** | Story (código — primer cimiento del substrato de pricing/SaaS) |
| **Épica (Fase)** | Cimientos de Monetización (transversal, greenfield — va antes de la auditoría retroactiva) |
| **Sprint** | substrato-monetizacion |
| **Estado** | ✅ Completada (cimiento local; adaptador central + UI diferidos) |
| **Creada** | 2026-07-03 |
| **Feature** | [`central-identity`](../features/central-identity.md) |
| **ADRs** | ADR-0144 (cimiento #1) · ADR-0143 (tres planos) · ADR-0137 (puertos) · ADR-0141 (esquema) · ADR-0020 V2 (perfiles) · ADR-0093 (secretos) · ADR-0142 (CLI verify) |

## 1. Objetivo llano

Construir el **cimiento local** de la cuenta de usuario: la migración de la tabla de cuenta, la lógica pura (huella de hardware + validaciones), el puerto que responde "¿quién es el dueño de esta instancia?" y la caché local. Es la raíz del substrato — `licensing-system`, `usage-metering` y `consent-registry` dependen de su `owner_id`.

**Alcance ahora vs. después (ADR-0144 "puerto + esquema ahora, adaptador después"):** la **Cabina de Mando Central** del proveedor todavía NO existe. Por eso la verificación real de identidad contra el servidor central es el **adaptador diferido**; esta Story entrega todo lo **local** (esquema + Core + puerto + caché) con la llamada al servidor central detrás de un **puerto con implementación stub local**.

## 2. Agentes y Modo de Acompañamiento (ADR-0120/0122)

| Agente | Modelo | Modo |
|---|---|---|
| Rust-Engineer | Sonnet | **Docente** |

En Docente el ingeniero implementa el bloque completo por su cuenta y además escribe la lección cero-conocimiento en `docs/lessons/rust/STORY-027-central-identity.md` (ADR-0124), explicando cada decisión.

## 3. Gate de Coherencia — resultado (contraste bidireccional)

- **Stack:** limpio (Rust + gRPC; sin tecnologías rechazadas).
- **Typo de spec corregido** por el TL (`sp# Central Identity` → `# Central Identity`).
- **Esquema (ADR-0141) — corrección obligatoria:** la tabla de cuenta es **mutable** (el estado de verificación de correo cambia, `updated_at` cambia). Por ADR-0141 debe llevar **`row_version`** (concurrencia optimista), NO `event_sequence_id UNIQUE` (ese patrón es solo para tablas append-only). El resto del Grupo I sí aplica. `STRICT`, UUIDv7, tipos canónicos.
- **Perfil ADR-0020 V2:** Perfil D — Grupo I completo + II (`owner_id`, `institutional_tag`, `access_token_id`) + IV (`node_id` = huella de hardware). Campos propios fuera del catálogo, marcados: estado de verificación de correo (`TEXT` + `CHECK`), proveedor OAuth (`TEXT`).
- **Puerto (ADR-0137):** `identity_out` → `AccountIdentity` (tipo técnico del substrato, ya en el catálogo vía enmienda ADR-0144). Sin puertos de entrada de otros cimientos (es la raíz).
- **Ubicación del crate — decisión del ingeniero en su Gate de Lectura:** `AccountIdentity` es un **tipo técnico de plomería** (análogo a `AuditEvent`/`TelemetrySample`), no un tipo de dominio del canvas. Aplica el criterio de ADR-0137: si califica como infraestructura crosscutting (tipo `textLabel`, ≥2 consumidores, sin puerto de Alpha en el canvas) → vive en `crates/shared`, siguiendo el patrón de `audit-log`/`telemetry`. Si tu lectura de ADR-0137 dicta lo contrario de forma clara → **párate y escálame** (no adivines). En Docente, explica tu decisión de ubicación.
- **Clasificación UI (ADR-0117):** la feature toma input del usuario (registro/login) → tiene **Superficie propia** (panel de cuenta), pero su UI completa + la llamada real al servidor central son el **adaptador diferido**. Para ESTA Story el observable se verifica por **CLI (Canal #2, ADR-0142)**; la UI del panel de cuenta queda registrada como **deuda de integración** contra `licensing-system`.
- **SAD:** sin impacto estructural nuevo (SAD-22 ya cubre el substrato). Si al implementar detectas desalineamiento → escala.

## 4. Instrucciones de despacho (prompt exacto)

> Eres el **Rust-Engineer** en **Modo Docente**. Lee primero: `CLAUDE.md`, `.claude/skills/base/SKILL.md`, `.claude/skills/rust-engineer/SKILL.md`, esta Orden completa, la feature `docs/features/central-identity.md`, y los ADR-0144, ADR-0143, ADR-0137, ADR-0141, ADR-0020 (§ADR.md perfiles), ADR-0093, ADR-0142. Declara que los leíste.
>
> **Gate de Lectura Pre-Código:** confirma la ubicación del crate (ver §3 "Ubicación") y el patrón de `audit-log`/`telemetry` en `crates/shared` como referencia estructural.
>
> **Construye (cimiento local — la llamada al servidor central es un puerto con stub local):**
> 1. **Migración greenfield** de la tabla de cuenta: Grupo I completo (ADR-0020 V2) + `owner_id`, `institutional_tag`, `access_token_id`, `node_id`; campos propios marcados (`email_verification_status TEXT` con `CHECK`, `oauth_provider TEXT`). **`row_version`** (tabla mutable, ADR-0141), NO `event_sequence_id UNIQUE`. `STRICT`, PK `TEXT` UUIDv7, timestamps `INTEGER` ns UTC. Baseline editable in-situ (greenfield).
> 2. **Core (lógica pura, sin I/O):** derivación de la huella de hardware (hash determinista de identificadores de máquina); validación de formato de correo; verificación de firma de token OAuth (dado el material público). Determinismo bit-a-bit.
> 3. **Shell:** persistencia de la cuenta; caché local de identidad con TTL (`IDENTITY_CACHE_TTL`, default 24 h) para operación offline; **puerto/trait para la verificación contra la Cabina de Mando con una implementación stub local** (crea/cachea la cuenta localmente; el adaptador real es futuro — coméntalo como tal).
> 4. **`public_interface`:** el puerto `identity_out` que responde "¿quién es el dueño de esta instancia?" devolviendo `AccountIdentity` (identificador de cuenta + estado de verificación). Sin exponer secretos.
> 5. **CLI `verify` (Canal #2, ADR-0142):** subcomando que reproduce el observable (identidad cacheada + estado) en JSON, ejecutable por `cargo run -p app -- verify central-identity --input '<json>'`.
>
> **Pruebas discriminantes (rojo→verde, deben poder fallar):**
> - Determinismo de huella: mismos identificadores → hash idéntico entre arranques (assert de igualdad); identificador alterado → hash distinto (assert de desigualdad). Debe fallar si la huella no es determinista.
> - Persistencia: la migración crea la tabla `STRICT` con Grupo I + Perfil D y `row_version`; inserta, **actualiza** (verifica que `row_version` incrementa) y relee.
> - Validación de correo: rechaza correos malformados, acepta válidos.
> - Caché TTL: identidad válida dentro del TTL; exige revalidación pasado el TTL (usa el reloj determinista, no `SystemTime`).
> - Guardarraíl ADR-0093: el payload de `AccountIdentity` NO contiene credenciales de bróker ni IPs live (assert explícito).
> - Cobertura del criterio con `cargo llvm-cov --workspace --summary-only`.
>
> **Comentarios en español, identificadores en inglés (ADR-0121).** Entrega en verde con mapeo criterio→prueba.
>
> **Docente:** escribe `docs/lessons/rust/STORY-027-central-identity.md` (enlace a esta Orden al inicio) explicando cero-conocimiento: qué es una huella de hardware y por qué es determinista, qué es `row_version` vs append-only, qué es un puerto hexagonal + stub de adaptador, qué es la caché con TTL. Cita el código real que produjiste.
>
> **NO hagas commits** (los hace el Tech-Lead). Al terminar reporta: archivos creados, salida de `cargo test` + `cargo llvm-cov`, salida del `cargo run -p app -- verify central-identity`, y tu decisión de ubicación del crate con su justificación.

## 5. Criterio de aceptación (verificable)

| # | Criterio | Prueba |
|---|---|---|
| 1 | Migración `STRICT` con Grupo I + Perfil D + `row_version` (no append-only) | inspección del `.sql` + test de esquema |
| 2 | Huella de hardware determinista | test de igualdad/desigualdad (discriminante) |
| 3 | Puerto `identity_out` devuelve `AccountIdentity` sin secretos | test + assert ADR-0093 |
| 4 | Caché con TTL usando reloj determinista | test de expiración |
| 5 | CLI `verify central-identity` devuelve el JSON correcto | `cargo run -p app -- verify central-identity --input '…'` |
| 6 | Lección Docente escrita | existe `docs/lessons/rust/STORY-027-central-identity.md` |
| 7 | Verde + cobertura de cada criterio | `cargo test` + `cargo llvm-cov` |

## 6. Comandos de validación (usuario)

```bash
cargo test -p <crate-de-central-identity>
cargo llvm-cov --workspace --summary-only
cargo run -p app -- verify central-identity --input '{"email":"a@b.com"}' | jq .
```

## 7. Registro de ejecución

- 2026-07-03 · Tech-Lead · Gate corrido (typo corregido, esquema mutable→`row_version`, alcance puerto+stub, UI diferida a adaptador). Orden creada y despachada a Rust-Engineer (Sonnet, **Docente**).
- 2026-07-03 · Rust-Engineer (Sonnet, Docente) · Entregó: migración 0007, Core/Shell en `crates/shared`, puerto + CLI verify, lección. Auditoría TL: esquema/FCIS/132 tests/clippy/CLI verdes.
- 2026-07-03 · QA-Engineer (Sonnet) · **NO APTO.** 2 defectos bloqueantes que los tests no cazaban: (1) `row_version` es cosmético — el UPDATE filtra solo por `id`, sin `WHERE ... AND row_version = ?` ni chequeo de `rows_affected`, así que escrituras concurrentes se pisan en silencio (last-write-wins + bifurcación de la cadena de auditoría); (2) la huella de hardware colisiona con `machine_identifiers` vacío (SHA-256 del buffer vacío = mismo `node_id` para toda máquina). Observación: el correo no se normaliza (case-sensitivity rompe "una cuenta por correo"). Defectos de implementación → devueltos al Rust-Engineer.
- 2026-07-03 · Tech-Lead · Devuelto al mismo Rust-Engineer (SendMessage) con los 3 arreglos + exigencia de pruebas discriminantes de concurrencia/huella-vacía/normalización.
- 2026-07-03 · Rust-Engineer (Sonnet) · Aplicó los 3 arreglos: (1) UPDATE con `WHERE id = ? AND row_version = ?` + chequeo `rows_affected() == 0` → `VersionConflict`; (2) `compute_hardware_fingerprint` → `Result<_, HardwareFingerprintError>` que rechaza lista vacía/en-blanco con `NoUsableIdentifiers`, propagado por la Shell con `?`; (3) `normalize_email` (trim + lowercase) en frontera de escritura y lectura. +8 tests discriminantes (140 en total).
- 2026-07-04 · QA-Engineer (Sonnet) · **APTO.** Re-auditó los 3 arreglos con lectura de código real + reproducción + pruebas de mutación (neutralizó cada guarda y confirmó que los tests caen); restauró los archivos byte a byte. Verificó cadena `audit_hash` consistente en conflicto (0 filas → nada se escribe), propagación sin `unwrap` nuevo en rutas de producción, y normalización idéntica en escritura/lectura. `cargo test -p shared` 140/0, clippy `-D warnings` limpio, CLI de humo normalizando. Sin regresiones.
- 2026-07-04 · Tech-Lead · Gate QA cerrado con APTO. **STORY-027 completada** (cimiento local). Feature `central-identity` sigue 🟡 Parcial por diseño (adaptador Cabina de Mando + UI del panel de cuenta diferidos, ADR-0144).

## 8. Deudas / diferidos registrados

- **Adaptador Cabina de Mando (diferido, ADR-0144):** la verificación real de identidad contra el servidor central se implementa cuando exista la Cabina de Mando. Ahora es un stub local.
- **UI del panel de cuenta (Superficie propia, diferida):** el login/registro en el cajón de ajustes es parte del adaptador; se registra como deuda de integración contra `licensing-system`. Verificación de esta Story vía CLI Canal #2.
