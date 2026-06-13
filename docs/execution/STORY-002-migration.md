# STORY-002 · Base de datos con los 25 campos

| Campo | Valor |
|---|---|
| **ID** | STORY-002 |
| **Título** | Base de datos: migración 0001 con los 25 campos maestros |
| **Tipo** | Story |
| **Épica (Fase)** | EPIC-0 — Fundación |
| **Sprint** | 0 |
| **Estado** | ✅ Implementado |
| **Responsable** | Rust-Engineer (Sonnet) · auditó Tech-Lead · escaló a Architect (Opus) |
| **Creada** | 2026-06-12 |
| **Completada** | 2026-06-12 |

## 1. Especificación de origen
- **ADR(s):** ADR-0006 (migraciones centralizadas SQLx), ADR-0020 V2 (los 25 campos maestros)
- **Fuente:** ROADMAP §EPIC-0, Sprint 0, fila STORY-002

## 2. Objetivo (una frase llana)
Crear la primera migración de base de datos con los 25 campos maestros desde el día uno (agregarlos después cuesta 10x).

## 3. Instrucciones de despacho (la spec ejecutable)
```
Eres el Rust-Engineer de Drasus Engine. Tarea STORY-002 de la Épica 0. STORY-001 (esqueleto) ya está aprobado.

PASOS DE ARRANQUE: 1) Lee base/SKILL.md y declara. 2) Lee rust-engineer/SKILL.md. 3) Lee ADR-0006
(dónde viven las migraciones y cómo). 4) Lee ADR-0020 V2 (LISTA EXACTA de los 25 campos — NO inventes
ni uno). 5) Lee ROADMAP §EPIC-0 Sprint 0 fila STORY-002.

ORDEN: Crear la migración 0001 con SQLx, que cree los 25 campos maestros de ADR-0020 V2 en SQLite WAL.
- Migraciones embebidas en la ubicación que dicte ADR-0006 (centralizadas).
- 25 campos EXACTOS de ADR-0020 V2 (nombres, tipos, semántica).
- Aplica en SQLite WAL. IDEMPOTENTE (correrla dos veces no rompe). Sigue compilando.

LÍMITES: Solo la migración 0001 + cableado SQLx mínimo. NO lógica de negocio. NO inventes campos/tablas/
tipos. Si ADR-0020 V2 es ambiguo en un campo, repórtalo como BLOQUEO con cita. Si ADR-0006 y ADR-0020 V2
se contradicen, repórtalo. Código en inglés.

ENTREGABLE: 1) ubicación+contenido de la migración; 2) los 25 campos con cita del ADR; 3) prueba WAL +
idempotencia; 4) salida cargo build/test; 5) ambigüedades.
```

## 4. Criterio de aceptación
- La migración aplica los 25 campos exactos de ADR-0020 V2 en SQLite WAL.
- Es idempotente (verificado por test).
- `cargo build` / `cargo test` verdes.

## 5. Comandos de validación (para el usuario — copy/paste)
```bash
cd /var/home/fbravohz/Documentos/Entornos/Personal/Drasus-Engine
cat migrations/0001_foundation_master_fields.sql      # revisa los 25 campos
cargo test -p shared                                  # incluye el test de idempotencia + WAL
grep -c "," migrations/0001_foundation_master_fields.sql   # referencia visual de columnas
```

## 6. Registro de ejecución (bitácora)
- 2026-06-12 · Rust-Engineer (Sonnet) · **APROBADO** · Auditoría Tech-Lead: 25 campos verificados uno a uno contra ADR-0020 V2 (líneas 395-399); `migrations/0001_foundation_master_fields.sql`; test `migration_0001_applies_in_wal_mode_and_is_idempotent` verde; WAL e idempotencia confirmadas.
- 2026-06-12 · **Escalamiento a Architect (Opus)** · El ingeniero marcó dudas de diseño (forma del contrato; `transformation_id`). Veredicto: los 25 campos son **contrato lógico + filtro por perfil**, NO molde físico de 25 columnas en cada tabla; la tabla ancla de EPIC-0 es correcta. `transformation_id` es identificador, no flag. Propagado a ADR-0020 V2, SAD §17.9/§20 y 8 módulos (aprobado por el usuario).

## 7. Pendientes derivados / decisiones
- **Implicación para Sprint 1:** las tablas de las Stories STORY-003–STORY-008 NO copian 25 columnas; aplican el Filtro de Relevancia por Perfil (ADR-0020 V2 actualizado).
