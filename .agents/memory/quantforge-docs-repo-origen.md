---
name: quantforge-docs-repo-origen
description: quantforge-docs es el repo de documentación original (antes del fork limpio a Drasus-Engine); su git conserva artefactos borrados recuperables.
metadata: 
  node_type: memory
  type: reference
  originSessionId: ffd1c5dd-a9fc-4b7f-9711-396d9ddd4feb
---

`../quantforge-docs/` (ruta: `/var/home/fbravohz/Documentos/Entornos/Personal/quantforge-docs`) es el repositorio de **documentación original** del proyecto. Drasus-Engine se bifurcó de él para empezar limpio, **sin el histórico de commits de documentación**.

**Por qué importa:** features/ADRs/docs borrados en el pasado siguen recuperables ahí vía git. Ejemplo real (2026-06-14): `features/alpha-purity-analyzer.md` se recuperó del commit `87faa08` (borrado luego en `2b8f43a`) usando `git -C ../quantforge-docs show 87faa08:documentation/features/alpha-purity-analyzer/FEATURE.md`.

**Cómo usar:** `git -C ../quantforge-docs log --all --oneline -- '*<nombre>*'` para localizar, y `git show <ref>:<path>` para extraer. Estructura legacy: `documentation/features/<feat>/FEATURE.md` (formato antiguo: Python/`logic.py`, sin Perfil Técnico [[adr-0020-contrato-logico]]) — al recuperar, adaptar a plantillas vigentes.
