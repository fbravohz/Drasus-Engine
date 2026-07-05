# STORY-004 · Registro de auditoría (audit-log)

| Campo | Valor |
|---|---|
| **ID** | STORY-004 |
| **Título** | Registro de auditoría inmutable (audit-log) |
| **Tipo** | Story |
| **Épica** | EPIC-0 — Fundación |
| **Sprint** | 1 |
| **Estado** | 🟡 Parcial — TTR-001 implementado y auditado; TTR-002 fuera de alcance (EPIC-2+) |
| **Responsable** | Rust-Engineer (Sonnet) · auditó Tech-Lead |
| **Creada** | 2026-06-12 |
| **Completada** | 2026-06-12 (TTR-001) |

## 1. Especificación de origen
- **Feature:** [`audit-log`](../features/audit-log.md) — TTR-001 (Registro de Evento Inmutable, Append-Only + hash chain).
- **Fuera de alcance:** TTR-002 (Reconciliación de rastro Nautilus) — diferido a EPIC-2+ (ROADMAP).
- **ADR(s):** ADR-0015 (audit-log como fuente de verdad), ADR-0005 (hash chain de versionado), ADR-0027 (Event Sourcing), ADR-0020 (campos por perfil — perfil Auditoría).

## 2. Objetivo (una frase llana)
Un libro contable inmutable: cada evento se anexa y se encadena por hash, de modo que alterar un evento pasado se detecta de inmediato.

## 3. Instrucciones de despacho (la spec ejecutable)
```
Eres el Rust-Engineer de Drasus Engine. Tarea STORY-004 de la Épica 0 (Fundación). Ya están aprobadas:
STORY-001 (esqueleto), STORY-002 (migración 0001 + cableado SQLx en crates/shared/src/persistence/),
STORY-003 (clock en crates/shared). El proyecto compila verde.

PASOS DE ARRANQUE: 1) Lee base/SKILL.md y declara que lo aplicarás. 2) Lee rust-engineer/SKILL.md y
adopta el rol. 3) Lee features/audit-log.md COMPLETA — implementa SOLO TTR-001 (Registro de Evento
Inmutable Append-Only + hash chain). TTR-002 (reconciliación Nautilus) está FUERA DE ALCANCE. 4) Lee
los ADRs citados: ADR-0015 (audit-log fuente de verdad), ADR-0005 (hash chain), ADR-0027 (event sourcing).
5) ADR-0020 (actualizado): los 25 campos son contrato lógico con FILTRO POR PERFIL; la tabla de
audit-log usa el perfil Auditoría (grupo I universal + lo forense pertinente). NO calques 25 columnas.

ORDEN: Implementar audit-log TTR-001.
- Registro append-only: los eventos solo se anexan, nunca se modifican ni borran.
- Hash chain: cada evento encadena el hash del anterior (blockchain-lite), de forma que mutar un evento
  histórico rompe la cadena y es detectable.
- Persistencia en SQLite (reusar el cableado SQLx de crates/shared/src/persistence de STORY-002; migración
  nueva si hace falta tabla propia, siguiendo ADR-0006).
- FCIS: la lógica de encadenamiento/verificación de hash es núcleo puro (domain); la persistencia es cáscara.

CRITERIO DE CIERRE (lo auditaré yo mismo):
- Un intento de mutar un evento histórico es RECHAZADO y DETECTADO por verificación de cadena (test que
  lo demuestre: alterar un evento e invalidar la cadena).
- Append y verificación de cadena son deterministas y testeados.
- cargo build / cargo clippy / cargo test verdes, sin warnings nuevos.

LÍMITES: Solo audit-log TTR-001. NO TTR-002. NO otras features. NO inventes campos fuera de audit-log.md
y ADR-0020 (perfil Auditoría); si algo es ambiguo, repórtalo como BLOQUEO con cita. NO toques docs/.
NO calques los 25 campos. Código en inglés.

ENTREGABLE: 1) dónde ubicaste audit-log y por qué; 2) separación núcleo/cáscara (FCIS); 3) el test de
detección de mutación (nombre y qué verifica); 4) salida cargo build/test/clippy; 5) ambigüedades/bloqueos.
```

## 4. Criterio de aceptación
- Mutar un evento histórico se detecta por ruptura de la cadena de hash (test).
- Append-only + verificación de cadena testeados y deterministas.
- `cargo build` / `cargo clippy` / `cargo test` verdes, sin warnings.

## 5. Comandos de validación (para el usuario — copy/paste)
```bash
cd /var/home/fbravohz/Documentos/Entornos/Personal/Drasus-Engine
cargo test -p shared            # incluye el test de detección de mutación de la cadena
cargo clippy --workspace --all-targets   # sin warnings
cargo build --workspace
```

## 6. Registro de ejecución (bitácora)
- 2026-06-12 · Rust-Engineer (Sonnet) · **APROBADO (TTR-001)** · Auditoría Tech-Lead: build + clippy (`-D warnings`) limpios; 22 tests verdes en `shared`, incl. `verify_chain_detects_mutation_of_historical_event` (detecta mutación + posición), triggers SQLite que rechazan UPDATE/DELETE, y determinismo de la cadena. FCIS verificado (núcleo `domain/audit_log.rs` sin I/O). Tabla `audit_events` con perfil Auditoría (no calca 25 columnas). Decisiones aceptadas: dependencia `uuid` (Rust puro, justificada por TTR-001); perfil "Auditoría" tomado de `architect/SKILL.md`.

## 7. Pendientes derivados / decisiones
- **Finalización de STORY-003 (clock):** con audit-log ya disponible, el rastro de auditoría del reloj
  (ntp_sync_offset, virtual_process_id, delta real/virtual) pasa a ser implementable. Requiere ANTES que
  el Architect defina el perfil de persistencia/auditoría de la entidad `clock` (ADR-0020). Es el
  siguiente paso para cerrar STORY-003 de 🟡 Parcial a ✅ Implementado.
