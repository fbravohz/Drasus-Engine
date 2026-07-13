---
name: arquitectura-visual-canvas
description: "Decisión de arquitectura visual 2026-06-23 — Dashboard + Canvas unificado [Forge/Reactor TBD] reemplaza ZUI de 3 niveles. ADR-0136 + ADR-0137 formalizan el cambio."
metadata: 
  node_type: memory
  type: project
  originSessionId: 72bbbfa4-9d39-44ce-a77e-bcf513f67bb3
---

## ADR-0028 supersedido por ADR-0136 (2026-06-23)

Los términos MACRO/MESO/MICRO como niveles de navegación están descontinuados. No usarlos.

**Nueva arquitectura:**
- **Dashboard**: monitoreo read-only + bento grid de widgets de features. No hace zoom. Navega al canvas.
- **Canvas [Forge/Reactor — nombre TBD]**: lienzo infinito único. Card-nodes estilo N8N/React Flow. Dos estados: Vista Relacional (nodos pequeños, conexiones) y Vista Interior (zoom in-place, edición). Zoom continuo con breadcrumb flotante.
- Jerarquía de entidades: Cluster → Portfolio → Strategy → Logic Blocks (anidación in-place)
- Jerarquía de proceso: **Workspace** → Pipeline → Módulo (compound node) → Feature (atomic node, abre inspector panel lateral) — nivel Workspace añadido 2026-07-12, ver [[banco-estrategias-portafolios-clusters]]
- Módulos = presets de composición de features, no wrappers obligatorios

**ADR-0137**: Feature como unidad hexagonal autónoma con puertos tipados. 108 tipos identificados (Bars, Signal, Order, ExecutableContainer, RobustnessScore, IncubationVerdict, etc.). `ExecutableContainer` es el tipo omnipresente del pipeline. Ver ADR-0137 para catálogo completo.

**Why:** ZUI de 3 niveles no reflejaba cómo el usuario piensa el sistema. El DAG pertenece a todos los "niveles". La nueva arquitectura expresa la jerarquía via anidación de card-nodes, no nombres de pantalla fijos.

**How to apply:** Al crear features, clasificar por contexto de superficie (Dashboard widget / Canvas Vista Relacional / Canvas Vista Interior / Inspector Panel) en lugar de MACRO/MESO/MICRO. Añadir `## Puertos de Integración` a cada feature doc.

## El Canvas DAG es una pieza CENTRAL de Drasus (énfasis del usuario 2026-06-28)

No es un visualizador secundario: es el corazón de la UX. El usuario **arrastra y suelta nodos (features) y los interconecta** para armar flujos de trabajo custom. Esto **rompe la dependencia de los módulos como piezas monolíticas de orquestación**: el usuario puede extraer solo las features clave que necesita en vez de correr todo el pipeline de un módulo, y tiene libertad total de componer su propio flujo. Los módulos quedan como *presets* de composición, no wrappers obligatorios.

**Tres manifestaciones de UI distintas (no confundir — ocurren en momentos distintos):**
1. **SVF / Cáscara Delgada (tab en el Panel Operativo Fundacional)** — ADR-0117. Se entrega CON cada feature (misma Story), incluida la plomería. Es el canal de verificación del humano: un botón que dispara la operación real + el resultado real (FFI) + un observable persistido. Patrón canónico: `ui/lib/tabs/clock_tab.dart`, `jobs_tab.dart`, `audit_tab.dart`. ESTE es el debug sin tocar código. Ver [[verification-surface-svf]].
2. **Dashboard widget read-only (bento grid)** — monitoreo personalizable; `ui/lib/tabs/dashboard_registry.dart` (hoy `available:false`). Más adelante.
3. **Canvas DAG node + inspector panel** (drag-drop, interconexión) — la UX de producción. **EPIC-8** (ADR-0117 redefine EPIC-8 = construir el sistema de card-nodes ADR-0137). NO es ahora.

**Regla derivada:** ninguna feature (ni plomería) se cierra sin su superficie de verificación (manifestación 1).

**Migración completada (2026-06-30):** `zui-navigation.md`, `visual-dag-editor.md`, `SAD-06.md`, `ADR-0038.md`, los 8 módulos — todos actualizados. `UI-ARCHITECTURE.md` eliminado; el conocimiento vive en ADR-0136 + ADR-0137. Pendiente solo progresivo: `## Puertos de Integración` en ~138 features (Tech Lead lo completa al procesar cada feature en ROADMAP).
