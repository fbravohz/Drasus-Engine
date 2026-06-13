# STORY-003 · Reloj interno (clock)

| Campo | Valor |
|---|---|
| **ID** | STORY-003 |
| **Título** | Reloj interno determinista (clock) |
| **Tipo** | Story |
| **Épica (Fase)** | EPIC-0 — Fundación |
| **Sprint** | 1 |
| **Estado** | ✅ Completado (núcleo Fase 1 + rastro de auditoría Fase 2) |
| **Responsable** | Rust-Engineer (Sonnet) · auditó Tech-Lead |
| **Creada** | 2026-06-12 |
| **Completada** | 2026-06-12 |

## 1. Especificación de origen
- **Feature:** [`clock`](../features/clock.md) — TTR-001 (timestamp ns), TTR-002 (reloj determinista backtest-ready)
- **ADR(s):** ADR-0003 (FCIS), ADR-0020 V2 (contrato de campos por perfil)

## 2. Objetivo (una frase llana)
Un reloj que funciona en modo real y en modo determinista, para que todo backtest dé exactamente el mismo resultado con las mismas entradas.

## 3. Instrucciones de despacho (la spec ejecutable)
```
Eres el Rust-Engineer de Drasus Engine. Tarea STORY-003 de la Épica 0. STORY-001 (esqueleto) y STORY-002 (migración) aprobadas.

PASOS DE ARRANQUE: 1) Lee base/SKILL.md y declara. 2) Lee rust-engineer/SKILL.md. 3) Lee features/clock.md
COMPLETA (TTR-001 timestamp ns, TTR-002 reloj determinista) — ese es el alcance. 4) Lee los ADRs que cite.
Si menciona persistencia, ADR-0020 V2 (actualizado): 25 campos = contrato lógico con filtro por perfil,
NO 25 columnas calcadas; grupo I universal, resto según perfil. NO calques 25 columnas.

ORDEN: Implementar clock según TTR-001 y TTR-002.
- TTR-001: timestamps de alta precisión (nanosegundos).
- TTR-002: reloj determinista backtest-ready (modo real + modo simulado controlado por semilla/datos;
  mismas entradas -> misma secuencia temporal).
UBICACIÓN: clock es transversal; hogar probable crate shared (según ADR-0003/SAD). FCIS: lógica pura del
reloj en el núcleo (domain), I/O de tiempo real en la cáscara.

CRITERIO: determinismo bit-a-bit (test que demuestre que dos corridas con mismo seed dan secuencia
idéntica); cargo build y cargo test verdes sin warnings nuevos.

LÍMITES: Solo clock (TTR-001 + TTR-002). NO otras features ni lógica de módulos. NO inventes campos/
contratos fuera de clock.md y sus ADRs; si algo es ambiguo, repórtalo como BLOQUEO con cita. NO toques
docs/. NO calques los 25 campos. Código en inglés.

ENTREGABLE: 1) dónde ubicaste clock y por qué; 2) separación núcleo/cáscara; 3) el test de determinismo;
4) salida cargo build/test; 5) ambigüedades/bloqueos.
```

## 3b. Instrucciones de despacho — Fase 2 (rastro de auditoría del reloj)
> Desbloqueada tras el escalamiento al Architect (2026-06-12): `clock.md` ya define el perfil, los 3 eventos auditables y el payload. La bitácora (`audit-log`, STORY-004) ya está en código.
```
Eres el Rust-Engineer de Drasus Engine. Fase 2 de STORY-003 (Épica 0). El núcleo del reloj (TTR-001/002) ya está aprobado; ahora implementas SU RASTRO DE AUDITORÍA, ya definido por el Architect.

PASOS DE ARRANQUE: 1) Lee base/SKILL.md y declara `[base/SKILL.md leído y activo]`. 2) Lee rust-engineer/SKILL.md. 3) Lee features/clock.md COMPLETO, en especial las postcondiciones corregidas de TTR-001/TTR-002 y la sección "Gobernanza y Estándares" (Perfil D, "Granularidad de Auditoría", "Persistencia y Perfil"). 4) Lee la interfaz REAL que vas a consumir (NO inventes otra): crates/shared/src/domain/audit_log.rs (struct AuditEventContent) y crates/shared/src/persistence/audit_log.rs (AuditLogRepository::new(pool, clock) + .append(content)). 5) Lee crates/shared/src/orchestrator.rs (cáscara SystemClock) y crates/shared/src/public_interface.rs (qué se re-exporta).

CONTEXTO DE DISEÑO (del Architect, NO lo cambies): el reloj NO tiene persistencia propia. Emite a la bitácora existente vía AuditEventContent + AuditLogRepository::append. Los tres campos antes huérfanos (ntp_sync_offset, id de proceso virtual, delta real/virtual) NO son columnas del catálogo: van como payload serializado dentro de details_json. Perfil D (Ops/Auditoría): Grupo I lo asigna la bitácora; del Grupo II institutional_tag es obligatorio; del Grupo IV process_id es obligatorio y session_id agrupa la sesión.

ORDEN: Implementa un emisor de eventos de auditoría del reloj en la CÁSCARA de shared (orchestrator.rs o un módulo de cáscara nuevo, p. ej. clock_audit.rs; NUNCA en domain/clock.rs). Debe construir y persistir los 3 eventos vía AuditLogRepository::append, con:
- entity_type = "CLOCK"; entity_id = el session_id de la sesión activa.
- CLOCK_NTP_SYNC (action_type): emitido al verificar la sincronía NTP en el arranque. Payload en details_json: { "ntp_sync_offset_ns": <i64> }.
- CLOCK_MODE_TRANSITION: al pasar de modo REAL <-> SIMULATION. Payload: { "from": "REAL|SIMULATION", "to": "REAL|SIMULATION" }.
- CLOCK_SESSION_CLOSE: al cerrar una sesión de simulación. Payload: { "virtual_process_id": <string>, "real_virtual_delta_ns": <i64> }.
Serializa details_json con serde_json (dep de cáscara; si no está en shared/Cargo.toml, añádela — es Rust puro, sin I/O nuevo). El JSON debe ser determinista (orden de claves estable).

RESTRICCIÓN DE GRANULARIDAD (CRÍTICA): PROHIBIDO emitir auditoría en timestamp_ns(), advance(ns) o tick(). Solo los 3 eventos de arriba. Si tu diseño llama a append desde el hot-path, está MAL.

FCIS: domain/clock.rs NO se toca (su determinismo bit-a-bit ya está verificado). La emisión es I/O (escribe a SQLite) -> vive en la cáscara. Re-exporta el emisor en public_interface.rs si procede.

CRITERIO: cargo build + cargo clippy --workspace --all-targets -- -D warnings limpios; cargo test -p shared verde. Tests nuevos que demuestren: (a) cada uno de los 3 eventos se persiste con su action_type, entity_type="CLOCK" y el payload correcto en details_json; (b) tras emitir los eventos, verify_chain sobre la bitácora sigue Valid (no rompes la cadena); (c) que los eventos se consultan por events_for_entity("CLOCK", session_id). Usa DeterministicClock + pool en memoria como en los tests existentes de audit_log.rs.

LÍMITES: Solo el rastro de auditoría del reloj. NO toques otras features ni módulos. NO inventes campos fuera de clock.md/audit-log.md. NO modifiques docs/ (eso lo hace el Tech-Lead al sellar). NO cambies la migración 0002. Si algo es ambiguo, repórtalo como BLOQUEO con cita textual. Código y comentarios en inglés.

ENTREGABLE: 1) dónde ubicaste el emisor y por qué (cáscara); 2) cómo construyes cada AuditEventContent (qué va en catálogo vs details_json); 3) los tests nuevos (lista + qué prueban); 4) salida de cargo build/clippy/test; 5) confirmación explícita de que NADA en el hot-path (timestamp_ns/advance/tick) emite auditoría; 6) ambigüedades/bloqueos.
```

## 4. Criterio de aceptación
**Fase 1 (núcleo — ✅ aprobada):**
- Determinismo bit-a-bit: mismo seed → misma secuencia temporal idéntica (test).
- `cargo build` / `cargo clippy` / `cargo test` verdes, sin warnings.
- FCIS: el núcleo no toca el reloj real; la cáscara sí.

**Fase 2 (rastro de auditoría):**
- Los 3 eventos (`CLOCK_NTP_SYNC`, `CLOCK_MODE_TRANSITION`, `CLOCK_SESSION_CLOSE`) se emiten vía `AuditLogRepository::append`, con `entity_type="CLOCK"` y el payload correcto en `details_json`.
- Granularidad respetada: el hot-path (`timestamp_ns`/`advance`/`tick`) NO emite auditoría.
- Tras emitir, `verify_chain` sobre la bitácora sigue `Valid`.
- `cargo clippy --workspace --all-targets -- -D warnings` limpio; `cargo test -p shared` verde.
- FCIS: el emisor vive en la cáscara; `domain/clock.rs` intacto.

## 5. Comandos de validación (para el usuario — copy/paste)
```bash
cd /var/home/fbravohz/Documentos/Entornos/Personal/Drasus-Engine
cargo test -p shared                              # incluye deterministic_clock_same_seed_produces_identical_sequence
cargo clippy --workspace --all-targets           # debe salir sin warnings
# Confirmar FCIS: el núcleo NO debe tocar el reloj real (resultado esperado: vacío)
grep -nE "SystemTime|::now\(\)" crates/shared/src/domain/clock.rs
# La cáscara SÍ usa el reloj real (resultado esperado: líneas con SystemTime)
grep -nE "SystemTime|::now\(\)" crates/shared/src/orchestrator.rs
```

## 6. Registro de ejecución (bitácora)
- 2026-06-12 · Rust-Engineer (Sonnet) · **APROBADO (Fase 1, núcleo)** · Auditoría Tech-Lead: build + clippy 0 warnings; 10 tests verdes incluyendo determinismo bit-a-bit; FCIS verificado (`domain/clock.rs` sin acceso a reloj real; `orchestrator.rs` con `SystemTime::now()`). `Clock` (trait), `DeterministicClock` (núcleo), `SystemClock` (cáscara) en `crates/shared`.
- 2026-06-12 · Architect (Opus, escalamiento §3) · **Perfil de auditoría resuelto** · Citas huérfanas de `clock.md` corregidas: `ntp_sync_offset`/proceso virtual/delta real-virtual son payload de `details_json`, no campos del catálogo. Perfil D. 3 eventos auditables definidos. Sin cambios a ADR-0020 V2.
- 2026-06-12 · Rust-Engineer (Sonnet) · **APROBADO (Fase 2, rastro de auditoría)** · Nuevo módulo de cáscara `crates/shared/src/clock_audit.rs`: `emit_ntp_sync`/`emit_mode_transition`/`emit_session_close` emiten vía `AuditLogRepository::append`; payload en `details_json` con `serde_json`; dep `serde_json` añadida a `shared`. Auditoría Tech-Lead (verificación independiente): clippy `--workspace --all-targets -- -D warnings` limpio; 28 tests verdes (21+7); FCIS verificado (`domain/clock.rs` sin referencia a auditoría); granularidad verificada (hot-path `timestamp_ns`/`advance`/`tick` no emite); `verify_chain` sigue `Valid` tras emitir.

## 7. Pendientes derivados / decisiones
- **Cerrado.** El rastro de auditoría del reloj quedó implementado en la Fase 2 tras el escalamiento al Architect. STORY-003 pasa a ✅ Completado. No quedan pendientes derivados.
