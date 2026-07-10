---
name: adr-0020-contrato-logico
description: "ADR-0020 (Inundación de Fundaciones) = contrato lógico con filtro por perfil, NO 25 columnas calcadas en cada tabla."
metadata: 
  node_type: memory
  type: project
  originSessionId: aed28726-733d-44db-a8c3-5b5d78477cc6
---

ADR-0020 ("Inundación de Fundaciones") define un **vocabulario lógico de 25 campos**, NO un molde físico de 25 columnas que se replique en cada tabla. Causó dudas recurrentes durante STORY-002 (migración 0001) hasta que se escaló al Architect el 2026-06-12.

Regla canónica (ahora "tatuada" en los docs):
- **Grupo I (Identidad & Integridad, 6 campos)**: universal, en toda tabla.
- **Grupos II–V**: selectivos según el **Perfil Técnico** de la Feature — A. Datos/Ingest, B. IA/R&D, C. Ops/Hot-Path, D. Ops/Auditoría.
- La **tabla canónica de los 4 perfiles** vive en `docs/ADR.md` (sección ADR-0020, "Resto por Filtro de Relevancia por Perfil") como **fuente única de verdad**. `architect/SKILL.md` y `TEMPLATES.md` la referencian, NO la redefinen.
- La migración `0001_foundation_master_fields.sql` crea la tabla ancla `foundation_master_fields` con las 25 columnas UNA sola vez (catálogo de referencia). Las tablas por feature NUNCA copian las 25.

**Why:** retrofitear campos de auditoría cuesta 10x; pero copy-paste masivo de 25 columnas es lo que el ADR prohíbe explícitamente.

**How to apply:** al diseñar/auditar persistencia de una feature, asigna UN perfil y toma solo los campos concretos de los grupos que ese perfil cubre. Si una feature mezcla perfiles, se documenta explícitamente — no se inventa un quinto perfil. Ejemplos ya correctos: `features/adaptive-volume-indicators.md` (B), `features/broker-connector.md` (C), `features/audit-log.md` + `migrations/0002_audit_log.sql` (D).

**Aprendizaje de la auditoría masiva (137 features, 2026-06-13):**
- Los perfiles son **acumulativos**: B (I+II+III+IV) ⊇ D (I+II+IV). Una feature R&D que necesita rastro forense NO es híbrida — perfil B ya la cubre completa.
- El único caso híbrido real es B vs C: B tiene linaje de datos (III), C tiene latencia (V); no se contienen entre sí.
- El catálogo sigue en **25 campos exactos** (confirmado en ADR + SQL migración 0001 + auditoría 2026-06-20).
- Campos locales legítimos documentados: variantes de latencia (`latency_ns` en pre-trade-validator), campos de dominio único (`active_genome_domain`, `phase_id`) — no promueven al catálogo con menos de 3 features.
