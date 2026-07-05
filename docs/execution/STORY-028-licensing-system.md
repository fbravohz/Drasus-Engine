# STORY-028 — Licensing System (cimiento #2 del substrato de monetización)

| Campo | Valor |
|---|---|
| **ID** | STORY-028 |
| **Tipo** | Story (código — segundo cimiento del substrato de pricing/SaaS) |
| **Épica (Fase)** | Cimientos de Monetización (transversal, greenfield — va antes de la auditoría retroactiva) |
| **Sprint** | substrato-monetizacion |
| **Estado** | ✅ Completada (gate local; emisor central, plan-tier-quota real y UI diferidos) |
| **Creada** | 2026-07-04 |
| **Feature** | [`licensing-system`](../features/licensing-system.md) |
| **ADRs** | ADR-0144 (cimiento #2) · ADR-0143 (tres planos + tiers) · ADR-0137 (puertos: `ExecutionGate`) · ADR-0141 (esquema) · ADR-0020 (Perfil D) · ADR-0093 (secretos) · ADR-0142 (CLI verify) · ADR-0039/hot-path |

## 1. Objetivo llano

Construir el **gate local de licencia**: la migración de la tabla de licencia, la lógica pura que valida el archivo de licencia firmado contra la huella de hardware y el heartbeat, y el puerto que responde **"¿puedo ejecutar y debo suprimir la telemetría de trabajo?"** devolviendo un `ExecutionGate`. Es el cimiento #2 — consume el `AccountIdentity` de [`central-identity`](../features/central-identity.md) (ya construido, cimiento #1) y prepara el consumo de `PlanLimits` de `plan-tier-quota` (cimiento #3, aún no construido → stub).

**Alcance ahora vs. después (ADR-0144 "puerto + esquema ahora, adaptador después"):**
- **Ahora (esta Story):** esquema + Core (verificación de firma asimétrica + comparación de heartbeat/gracia + derivación del veredicto y de la orden de supresión por tier + conteo de activaciones) + puerto `execution_gate_out` + caché local + CLI verify. La emisión real de licencias por la **Cabina de Mando** todavía NO existe → un **stub local emisor** genera una licencia de desarrollo firmada para las pruebas.
- **Después (adaptadores diferidos):** el emisor real de licencias en la Cabina de Mando; el cableado de `plan_limits_in` al `plan-tier-quota` real (#3); la UI del panel de licencia/tier en el cajón de ajustes.

## 2. Agentes y Modo de Acompañamiento (ADR-0120/0122)

| Agente | Modelo | Modo |
|---|---|---|
| Rust-Engineer | Sonnet | **Docente** |

En Docente el ingeniero implementa el bloque completo por su cuenta y además escribe la lección cero-conocimiento en `docs/lessons/rust/STORY-028-licensing-system.md` (ADR-0124), explicando cada decisión.

## 3. Gate de Coherencia — resultado (contraste bidireccional)

- **Stack:** limpio (Rust + gRPC/CLI; sin tecnologías rechazadas).
- **Esquema (ADR-0141) — corrección obligatoria #1:** la tabla de licencia es **mutable** (el heartbeat refresca la validez en sitio, `updated_at` cambia). Por ADR-0141 lleva **`row_version`** (concurrencia optimista), NO `event_sequence_id UNIQUE` (append-only). El historial de cambios de licencia va al `audit-log` existente, no a esta tabla. Feature doc §8 ya corregido.
- **Reutilización (ADR-0144 FIJO) — corrección obligatoria #2:** `central-identity` ya deriva la huella de hardware (`compute_hardware_fingerprint` → `node_id`). **NO se re-deriva aquí.** La licencia se **valida contra** la huella que llega por el puerto `identity_in` (`AccountIdentity`). Duplicar el Core de huella es un defecto.
- **Criptografía (ADR-0093 + §3 del feature) — corrección obligatoria #3:** §3 prohíbe almacenar claves **privadas** de firma en el cliente. Por tanto la verificación es **asimétrica** (clave pública incrustada que **verifica**; la privada firma en el servidor y jamás sale), NO `HMAC-SHA256` simétrico (la clave incrustada sería la clave de firma → violación). El primitivo asimétrico concreto (Ed25519/RSA) lo elige el ingeniero; el stub emisor local genera su par de claves para pruebas. **Si tu lectura de los ADR de seguridad contradice esto de forma clara → párate y escálame.**
- **Perfil ADR-0020:** Perfil D — Grupo I completo (con `row_version`) + II (`owner_id`, `institutional_tag`, `access_token_id`) + IV (`node_id` = huella de hardware, `process_id`) + V forense (`signature_hash` = firma del archivo de licencia, `compliance_status_id` = estado de la licencia). Campos propios fuera del catálogo, marcados: tier (`TEXT` + `CHECK` Sovereign/Explorer), fecha de expiración del heartbeat (`INTEGER` ns UTC).
- **Puertos (ADR-0137):** `identity_in` ← `AccountIdentity` (real, de #1); `plan_limits_in` ← `PlanLimits` (stub hasta #3); `execution_gate_out` → `ExecutionGate` (veredicto `{Allow/Deny/UpgradeRequired}` + orden de supresión de telemetría). Todos en el catálogo vía enmienda ADR-0144.
- **Ubicación del crate — Gate de Lectura del ingeniero:** `ExecutionGate` es **tipo técnico de plomería** (`textLabel`, ≥2 consumidores: `execute` + `telemetry`, sin puerto de Alpha en el canvas) → vive en `crates/shared` (mismo criterio bendecido que `central-identity`/`audit-log`/`telemetry`, ADR-0137). Confirma leyendo el patrón de `central-identity` ya construido. Si tu lectura dicta lo contrario de forma clara → **párate y escálame**. En Docente, explica tu decisión.
- **Hot-Path (ADR-0039 / feature §3):** PROHIBIDO llamada de red **síncrona** en el gate. El gate lee la licencia **cacheada** (sin I/O de red) y decide con el reloj determinista inyectado. El refresco del heartbeat es asíncrono, fuera del hot-path.
- **Clasificación UI (ADR-0117):** la feature tiene Superficie propia (panel de licencia/tier en ajustes), pero su UI completa + el emisor real son adaptador diferido. Para ESTA Story el observable se verifica por **CLI (Canal #2, ADR-0142)**; la UI queda como deuda de integración.
- **SAD:** SAD-22 ya cubre el substrato. Si al implementar detectas desalineamiento → escala.

## 4. Instrucciones de despacho (prompt exacto)

> Eres el **Rust-Engineer** en **Modo Docente**. Lee primero: `CLAUDE.md`, `.claude/skills/base/SKILL.md`, `.claude/skills/rust-engineer/SKILL.md`, esta Orden completa, la feature `docs/features/licensing-system.md`, la feature ya construida `docs/features/central-identity.md` (patrón de referencia), y los ADR-0144, ADR-0143, ADR-0137, ADR-0141, ADR-0020 (§ADR.md perfiles), ADR-0093, ADR-0142, ADR-0039. Declara que los leíste.
>
> **Gate de Lectura Pre-Código:** (a) confirma la ubicación del crate (`crates/shared`, ver §3) leyendo el `central-identity` ya construido (`domain/central_identity.rs`, `orchestrator/central_identity.rs`, `public_interface.rs`) como plantilla estructural y de puerto+stub; (b) confirma que consumes la huella de hardware vía `AccountIdentity` y **no** la re-derivas.
>
> **Construye (gate local — el emisor real y `plan-tier-quota` son puertos con stub):**
> 1. **Migración greenfield 0008** de la tabla de licencia: Grupo I completo con **`row_version`** (tabla mutable, ADR-0141), Perfil D + `owner_id`/`institutional_tag`/`access_token_id`/`node_id`/`process_id` + forense `signature_hash`/`compliance_status_id`; campos propios marcados (`tier TEXT` con `CHECK (tier IN ('SOVEREIGN','EXPLORER'))`, `heartbeat_expires_at INTEGER`). `STRICT`, PK `TEXT` UUIDv7, timestamps `INTEGER` ns UTC. Baseline editable in-situ (greenfield). Metadatos sensibles cifrados en el almacén local (feature §Contrato de Persistencia).
> 2. **Core (lógica pura, sin I/O):** (a) verificación de **firma asimétrica** del archivo de licencia con clave pública incrustada (NO HMAC — ver §3 corrección #3); (b) validación de que la firma corresponde a la huella de hardware recibida (`node_id` de `AccountIdentity`) — mismatch → licencia inválida; (c) comparación determinista de heartbeat/gracia (usa el reloj determinista inyectado, NO `SystemTime`): dentro de ventana → válida; en ventana de recheck → válida con alerta; pasada la gracia → restringe; (d) derivación del veredicto `ExecutionGate {Allow / Deny / UpgradeRequired}` y de la **orden de supresión de telemetría por tier** (Sovereign al corriente → suprimir; Explorer o vencido → emitir; ADR-0143); (e) conteo de activaciones = **máquinas distintas por huella** (un segundo arranque en la misma máquina comparte huella y NO cuenta doble — feature §3 "una instancia por máquina"). Determinismo bit-a-bit.
> 3. **Shell:** persistencia de la licencia (cifrada); caché local del veredicto con TTL (reutiliza el patrón de `IdentityCache` de central-identity, reloj inyectado); puerto `plan_limits_in` con **implementación stub local** de `PlanLimits` (el adaptador real es `plan-tier-quota`/#3 — coméntalo como tal); **stub local emisor** que firma una licencia de desarrollo con un par de claves generado para las pruebas (el emisor real es la Cabina de Mando — futuro).
> 4. **`public_interface`:** el puerto `execution_gate_out` que responde "¿puedo ejecutar y debo suprimir telemetría?" devolviendo `ExecutionGate`. **Sin exponer secretos** (ADR-0093): el payload NO contiene credenciales de bróker ni IPs live ni la clave de firma.
> 5. **CLI `verify` (Canal #2, ADR-0142):** subcomando que reproduce el observable (veredicto del gate + tier + orden de supresión + activaciones contadas) en JSON, ejecutable por `cargo run -p app -- verify licensing-system --input '<json>'`.
>
> **Pruebas discriminantes (rojo→verde, deben poder fallar):**
> - **Huella no coincide:** licencia firmada para huella A, instancia con huella B → `Deny` (assert). Debe fallar si el gate ignora la huella.
> - **Firma asimétrica:** licencia con firma válida → aceptada; un byte alterado del payload o de la firma → rechazada (assert de desigualdad). Debe fallar si no se verifica la firma.
> - **Heartbeat/gracia (reloj determinista):** dentro de ventana → `Allow`; dentro de recheck → `Allow` + alerta; pasada la gracia → restringe (no `Allow` de trading en vivo). Debe fallar si usa `SystemTime` o no compara.
> - **Supresión por tier (ADR-0143):** Sovereign al corriente → orden de supresión = true; Explorer → false; Sovereign vencido → false (reactiva). Assert por cada caso.
> - **Conteo de activaciones (una instancia por máquina):** 3 huellas distintas → 3 activaciones; segundo arranque con una huella ya vista → sigue contando 3 (assert). Debe fallar si cuenta procesos en vez de máquinas.
> - **`UpgradeRequired` por cuota:** con `PlanLimits` (stub) que fija un límite y un uso que lo excede → `UpgradeRequired`. Debe fallar si nunca emite ese veredicto.
> - **Concurrencia optimista (ADR-0141):** dos refrescos de heartbeat desde el mismo `row_version` → el segundo da conflicto de versión, NO pisa el primero en silencio (assert `rows_affected == 0` → error de conflicto). *(Lección directa de la QA de STORY-027.)*
> - **Guardarraíl ADR-0093:** el payload de `ExecutionGate` NO contiene credenciales de bróker, IPs live ni clave de firma (assert explícito).
> - **Hot-path (ADR-0039):** el gate decide sin I/O de red (inspección/estructura: el método del gate no toca la red; la revalidación es asíncrona).
> - Cobertura del criterio con `cargo llvm-cov --workspace --summary-only`.
>
> **Comentarios en español, identificadores en inglés (ADR-0121).** Entrega en verde con mapeo criterio→prueba.
>
> **Docente:** escribe `docs/lessons/rust/STORY-028-licensing-system.md` (enlace a esta Orden al inicio) explicando cero-conocimiento: qué es una firma asimétrica y por qué el cliente solo tiene la pública, qué es un heartbeat con periodo de gracia, cómo un gate decide sin llamar a la red (caché), qué es la supresión de telemetría por tier, y por qué las activaciones cuentan máquinas y no procesos. Cita el código real que produjiste.
>
> **NO hagas commits** (los hace el Tech-Lead). Al terminar reporta: archivos creados, salida de `cargo test` + `cargo llvm-cov`, salida del `cargo run -p app -- verify licensing-system`, y tu decisión de ubicación del crate con su justificación.

## 5. Criterio de aceptación (verificable)

| # | Criterio | Prueba |
|---|---|---|
| 1 | Migración `STRICT` con Grupo I + Perfil D + `row_version` (no append-only) | inspección del `.sql` + test de esquema |
| 2 | Verificación de firma **asimétrica** (no HMAC); byte alterado → rechazo | test de igualdad/desigualdad (discriminante) |
| 3 | Licencia atada a la huella: huella distinta → `Deny` | test discriminante |
| 4 | Heartbeat/gracia con reloj determinista → Allow/alerta/restringe | test de las 3 ventanas |
| 5 | Orden de supresión de telemetría correcta por tier (ADR-0143) | test por tier (Sovereign corriente/vencido, Explorer) |
| 6 | Activaciones = máquinas distintas, no procesos | test discriminante |
| 7 | Concurrencia optimista real (row_version) en refresco | test de conflicto (`rows_affected == 0`) |
| 8 | `ExecutionGate` sin secretos (ADR-0093) | test + assert |
| 9 | Gate sin I/O de red (hot-path, ADR-0039) | inspección/estructura |
| 10 | CLI `verify licensing-system` devuelve el JSON correcto | `cargo run -p app -- verify licensing-system --input '…'` |
| 11 | Lección Docente escrita | existe `docs/lessons/rust/STORY-028-licensing-system.md` |
| 12 | Verde + cobertura de cada criterio | `cargo test` + `cargo llvm-cov` |

## 6. Comandos de validación (usuario)

```bash
cargo test -p shared
cargo llvm-cov --workspace --summary-only
cargo run -p app -- verify licensing-system --input '{"tier":"SOVEREIGN"}'
```

## 7. Registro de ejecución

- 2026-07-04 · Tech-Lead · Gate corrido (contraste bidireccional). 4 correcciones: (1) tabla mutable → `row_version`; (2) reutilizar la huella de central-identity, no re-derivarla; (3) firma asimétrica en vez de HMAC (ADR-0093 §3); (4) puertos `identity_in`/`plan_limits_in`/`execution_gate_out` añadidos al feature doc desde el catálogo ADR-0137. Orden creada, pendiente de despacho a Rust-Engineer (Sonnet, **Docente**).
- 2026-07-04 · Rust-Engineer (Docente) · Entregado en verde. Migración `0008_licensing_system.sql` (Grupo I + `row_version` + Perfil D + `license_id` propio); Core puro en `crates/shared/src/domain/licensing_system.rs` (firma Ed25519, comparación de huella, heartbeat/gracia, supresión por tier, `derive_execution_gate`); Shell en `crates/shared/src/persistence/licensing_system.rs` (repositorio con concurrencia optimista) y `crates/shared/src/orchestrator/licensing_system.rs` (emisor stub Ed25519, `PlanLimits` stub, `ExecutionGateCache`); puerto `execution_gate_out` + CLI `verify licensing-system` en `public_interface.rs`/`crates/app/src/main.rs`. 45 tests nuevos en verde (`cargo test -p shared`), `cargo clippy --all-targets -- -D warnings` limpio en `shared` y `app`, cobertura de línea: `domain/licensing_system.rs` 99.75 %, `persistence/licensing_system.rs` 100 %, `orchestrator/licensing_system.rs` 100 %. CLI verificado con los tres tiers (`SOVEREIGN`/`EXPLORER`/inválido). Lección Docente en `docs/lessons/rust/STORY-028-licensing-system.md`. Feature doc sellado 🟡 Parcial (adaptadores diferidos: emisor real, `plan-tier-quota`, UI). **Hallazgo auto-cazado en integración:** confusión `id` (PK de la fila de activación) ↔ `license_id` (del payload firmado) e `issued_at` ↔ `created_at`; corregido con columnas propias `license_id`/`issued_at` en la migración, documentado en la lección.
- 2026-07-04 · Tech-Lead · Auditoría independiente (evidencia reproducida): 185 tests workspace / 45 de licensing en verde; clippy `-D warnings` limpio; FCIS 0 violaciones en el Core; esquema `STRICT`+UUIDv7+Grupo I con `row_version`+`audit_chain_hash` NULL génesis+FK `accounts` ON DELETE RESTRICT+CHECK en `tier`/`compliance_status_id`; `UPDATE ... WHERE id=? AND row_version=?` + `rows_affected()==0`→`VersionConflict`; Ed25519 asimétrico (no HMAC); reutiliza `node_id` de `AccountIdentity` sin re-derivar; CLI suprime telemetría solo en Sovereign; prueba anti-secretos real (5 claves fijas). Verde.
- 2026-07-04 · QA-Engineer (Sonnet) · **APTO.** Auditoría de código línea por línea + reproducción + **pruebas de mutación** sobre los 4 puntos de mayor riesgo: neutralizó la verificación Ed25519 (`verify_strict`→`Ok(())`) → 3 tests caen; la guarda de huella (`if false && !hardware_match`) → 2 tests caen; la supresión por tier (`Sovereign=>true` ignorando `Expired`) → cae `sovereign_expired_reactivates_telemetry`; **la concurrencia optimista (quitó `AND row_version=?`)** → cae el test de refresco concurrente exactamente como en la lección de #027, esta vez la guarda es real. Conteo de activaciones con doble cinturón (`COUNT(DISTINCT node_id)` + índice único `(owner_id,node_id)`). Todas las mutaciones restauradas byte a byte, suite de vuelta a 185/185. Observaciones menores no bloqueantes (`expect` de mutex envenenado, patrón estándar). Sin regresiones.
- 2026-07-04 · Tech-Lead · Gate QA cerrado con APTO. **STORY-028 completada** (gate local). Feature `licensing-system` sigue 🟡 Parcial por diseño (emisor real Cabina de Mando + `plan-tier-quota` real #3 + UI diferidos, ADR-0144).

## 8. Deudas / diferidos registrados

- **Emisor real de licencias (Cabina de Mando, diferido, ADR-0144):** ahora es un stub local que firma una licencia de desarrollo. El emisor real llega con la Cabina de Mando.
- **`plan_limits_in` → `plan-tier-quota` real (cimiento #3, diferido):** ahora se cablea a un stub de `PlanLimits`; el adaptador real se enchufa al construir #3.
- **UI del panel de licencia/tier (Superficie propia, diferida):** el panel en el cajón de ajustes es parte del adaptador; deuda de integración. Verificación de esta Story vía CLI Canal #2.
