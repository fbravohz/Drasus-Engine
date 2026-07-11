# Pipeline Registry — Definición Reutilizable y Versionada de Pipelines

**Carpeta:** `./features/pipeline-registry/`
**Estado:** En Diseño
**Última actualización:** 2026-07-11
**Decisión Arquitectónica Asociada:** ADR-0150 (Expedition — Ledger de Ejecución + Linaje + Pipeline Versionado) · ADR-0005 (mecanismo de versionado git-like reutilizado) · ADR-0137 (formaliza el "custom module")

---

## ¿Qué es esta feature?

El Pipeline Registry es el dueño de la **definición** de un Pipeline: el flujo custom, nombrado y ordenado, de features/módulos que el usuario arma en el Canvas — la "ruta" reutilizable. Persiste esa definición y la **versiona** con el mismo patrón git-like de ADR-0005: cada cambio en la topología crea un nodo de versión inmutable content-addressed, de modo que dos corridas distintas (Expeditions) pueden apoyarse en versiones distintas de la misma ruta y ese cambio es diffeable.

**Problema que resuelve:** hoy el "custom module" de ADR-0137 (una composición de features guardada) es un concepto sin entidad persistida ni historial. Sin un Pipeline versionado no se puede saber *con qué ruta exacta* corrió una Expedition, ni diffear la ruta entre la corrida N y la N+1. Esta feature aporta ese eslabón.

**Qué NO es:** no versiona el artefacto (eso es `strategy-versioning`, ADR-0005); no ejecuta nada (eso lo instancia `expedition-ledger`). Versiona la *ruta*, no el *resultado* ni la *corrida*.

---

## Comportamientos Observables

- [ ] El usuario guarda un flujo armado en el Canvas → se crea el nodo raíz de una definición de Pipeline: `version_hash` determinista por contenido, `parent_hash = NULL`, nombre libre.
- [ ] El usuario modifica la topología (añade/quita nodos, recablea puertos) y vuelve a guardar → nuevo nodo de versión hijo del anterior; la cabecera del Pipeline apunta a la versión vigente; el nodo antiguo permanece.
- [ ] El sistema diffea dos versiones de un Pipeline → muestra qué nodos/conexiones cambiaron.
- [ ] Una Expedition referencia una versión concreta del Pipeline (`pipeline_version_hash`) → esa versión queda *locked*: nunca se reescribe.
- [ ] El sistema valida la compatibilidad de tipos de todas las conexiones (ADR-0137) antes de persistir una versión.

## Restricciones

- **NUNCA** un nodo de versión se modifica o borra tras crearse (inmutable, append-only).
- **NUNCA** el grafo de versiones forma un ciclo.
- **NUNCA** una Expedition referencia una versión de Pipeline inexistente (referencia verificada al crear la Expedition).
- **NUNCA** se reimplementa el mecanismo de versionado: se reutiliza el patrón de ADR-0005 (content-addressed + `parent_hash`).

## Parámetros Configurables (ADR-0008)

| Parámetro | Default | Rango | Qué hace | FIJO/CONFIG |
|---|---|---|---|---|
| PIPELINE_DEPTH_LIMIT | 1000 | 100 - 10000 | Nodos máximos en el grafo de versiones de un Pipeline antes de sugerir archivado | CONFIG |

## Estructura Interna (FCIS — ADR-0002)

- **Core (Lógica Pura):** cálculo del `version_hash` determinista por contenido (topología canónicamente serializada); validación de aciclicidad; cómputo del diff entre dos versiones.
- **Shell (Infraestructura):** repositorio SQLite de la cabecera mutable + nodos de versión append-only (`BEGIN IMMEDIATE`, atomicidad estado+auditoría, ADR-0141).
- **Frontera Pública:** puerto para guardar/consultar una definición de Pipeline y para resolver una `pipeline_version_hash` a su topología.

## Tareas (TTRs)

### TTR-001: Nodo de versión de Pipeline (content-addressed, DAG)
Persistir una topología de Pipeline como nodo inmutable con `version_hash` determinista y `parent_hash`, reutilizando el patrón de ADR-0005.

### TTR-002: Diff de versiones de Pipeline
Calcular en el Core la diferencia de topología (nodos/conexiones añadidos, quitados, recableados) entre dos `version_hash`.

## Puertos de Integración (ADR-0137)

| Puerto | ID de tipo | Dirección | Cardinalidad | Descripción |
|---|---|---|---|---|
| `pipeline_definition_out` | `PipelineDefinition` (tipo de dominio nuevo — se cataloga en ADR-0137 con color de procedencia al construir el nodo Canvas, patrón progresivo) | Output | `1` | Definición vigente de un Pipeline (cabecera + versión activa). Consumida por `expedition-ledger`, `visual-dag-editor` y `event-driven-pipeline-triggers`. |
| `pipeline_version_out` | `PipelineVersion` (tipo de dominio nuevo — cableado de Canvas diferido, ídem) | Output | `1..N` | Nodos de versión inmutables de la ruta; referencia *locked* de cada Expedition. |

> Los nombres canónicos de `struct`/tipo Rust los fija el ingeniero (anti-alucinación, ADR-0144). El cableado en Canvas de estos tipos se difiere a EPIC-8 (ADR-0136 §Enmienda 2026-06-28); el subsistema no depende del Canvas para existir.

## Cáscara Visual (Thin Shell)

> Pendiente de la Etapa 0.5 (UI-Designer). Superficie prevista: en el Canvas [Forge/Reactor], guardar/nombrar/versionar una ruta; panel de diff entre dos versiones. El Architect NO rellena esta sección.

## Gobernanza y Estándares (Fijos)

- **Local-First (ADR-0016):** 100% local (SQLite WAL).
- **Inundación de Fundaciones (ADR-0020):** **Perfil D (Ops/Auditoría)** — Grupo I completo (6 campos, incl. `event_sequence_id`) + Soberanía (`owner_id`, `manifest_id`). La definición de ruta es metadato auditable, no R&D numérico.

## Persistencia (Inundación de Fundamentos — ADR-0020)

Cabecera **mutable** (`row_version`, puntero a la versión vigente) + nodos de versión **append-only** (`event_sequence_id UNIQUE`, `version_hash` PK content-addressed, `parent_hash`, snapshot de topología `TEXT` JSON con `CHECK (json_valid(...))`). `STRICT`, PK UUIDv7 en la cabecera. Ver ADR-0150 y ADR-0141 para el detalle de esquema.

**Rastro de Evidencia:** emite hacia `feedback` la versión de ruta que cada Expedition consumió (causalidad ruta→corrida).

## Dependencias y Bloqueantes

**Depende de:** [`clock`](../features/clock.md), [`audit-log`](../features/audit-log.md).
**Consumido por:** [`expedition-ledger`](../features/expedition-ledger.md) (referencia la versión que corrió), [`visual-dag-editor`](../features/visual-dag-editor.md) (arma/edita la ruta), [`event-driven-pipeline-triggers`](../features/event-driven-pipeline-triggers.md) (dispara la ruta), módulo [`validate`](../modules/validate.md) (orquestación en EPIC-2).
