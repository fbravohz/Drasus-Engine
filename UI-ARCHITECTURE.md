# Arquitectura Visual — Drasus Engine

> **Estado:** decisión tomada (2026-06-23). Formalizada en ADR-0136 + ADR-0137.
> **Reemplaza:** ADR-0028 (ZUI Fractal de 3 niveles nominados — SUPERSEDIDO).

---

## Resumen ejecutivo

La interfaz visual de Drasus Engine tiene **dos superficies**:

1. **Dashboard** — centro de monitoreo y navegación. Read-only. Widgets arrastrables (bento grid).
2. **Canvas [Forge/Reactor — TBD]** — lienzo nodal unificado donde se construye toda la lógica: estrategias, portafolios, pipelines de módulos, features.

Los términos MACRO / MESO / MICRO como nombres de nivel quedan descontinuados. El canvas tiene un zoom continuo de dos estados, no tres pantallas fijas.

---

## Las dos superficies

### Dashboard
- Pantalla de inicio de la app.
- Muestra métricas, KPIs, estado del sistema. **Solo lectura.**
- Cada elemento clickeable abre el canvas en el contexto correspondiente.
- El usuario personaliza qué widgets (vistas de monitoreo de features) ve y en qué posición (bento grid).
- No hace zoom. No es parte del canvas.

### Canvas [Forge/Reactor — TBD]

Canvas infinito único. Paradigma: **card-nodes rectangulares** (estilo N8N / React Flow) conectados mediante bezier S-curves con **puertos tipados** (ADR-0137).

**Dos estados de zoom (continuo, no discreto):**

| Estado | Cuándo | Qué se hace aquí |
|---|---|---|
| Vista relacional | Alejado — nodos pequeños, conexiones visibles | Conectar, desconectar, reordenar nodos, crear pipelines |
| Vista interior | Zoom in / doble clic en un nodo | Editar lógica interna del nodo expandido |

**Breadcrumb flotante:** `Cluster A › Portfolio B › Strategy 3` — clic en cualquier segmento vuelve a ese nivel con animación in-place.

---

## Lo que vive en el canvas

### Jerarquía de entidades (nodos anidados)
```
Cluster
  └─ Portfolio × N
       └─ Strategy × N
            └─ Logic Blocks × N  (DAG de señales/indicadores)
```
Hacer zoom en un Cluster → aparecen sus Portfolios. Zoom en un Portfolio → aparecen sus Strategies. Los nodos de un nivel no coexisten con los del siguiente — el zoom los separa.

### Jerarquía de proceso (pipeline de módulos)
```
Pipeline (crypto, forex, fondeo...)
  └─ Módulo (compound node, preset de features)
       └─ Feature (atomic node, con puertos tipados)
```
El usuario puede usar el módulo como caja negra **o** ignorarlo y cablear features directamente.

**Los 8 módulos (Ingest → Withdraw) son el preset default** — la ruta recomendada. No son obligatorios para usuarios expertos.

---

## Cómo se abre cada tipo de nodo

| Tipo de nodo | ¿Tiene sub-nodos? | Cómo se abre |
|---|---|---|
| Cluster | Sí | Zoom in-place en el canvas |
| Portfolio | Sí | Zoom in-place en el canvas |
| Strategy | Sí (logic blocks) | Zoom in-place → DAG de lógica |
| Módulo | Sí (features) | Zoom in-place → mixer view (DAW) o graph view |
| Feature / Logic Block | No (leaf node) | Inspector panel lateral (derecha, como N8N) |

**Regla irrompible:** un feature node siempre abre inspector panel lateral. Nunca una pantalla full separada.

---

## Custom modules

Un custom module es una composición de features guardada por el usuario como nodo compuesto reutilizable. Mismo mecanismo que los módulos predefinidos. El sistema valida compatibilidad de tipos de puerto antes de guardar.

---

## Conexiones tipadas

Cada línea tiene un **tipo de dato** (ADR-0137: `Bars`, `Signal`, `Order`, `ExecutableContainer`, etc.). El color de la línea = color del tipo. Conectar tipos incompatibles → línea `criticalCrimson` + tooltip de error.

---

## Decisiones de diseño clave

| Decisión | Resolución |
|---|---|
| ¿Un canvas o dos? | **Uno unificado.** Focus mode de visibilidad (pipeline vs entidades) sobre el mismo canvas. |
| ¿Tres niveles de zoom? | **No. Dos estados** (relacional / interior) + jerarquía de entidades via anidación. |
| ¿MACRO/MESO/MICRO? | **Descontinuados.** Reemplazados por la jerarquía de entidades + estados de zoom. |
| ¿Features como nodos? | **Sí.** Feature = nodo atómico con puertos tipados. Módulo = preset de composición. |
| ¿Interior de módulo es DAG? | **Opcional.** Toggle: mixer view (canales estilo DAW) ↔ graph view (sub-nodos). |
| ¿Dashboard hace zoom? | **No.** Solo navega. Abre el canvas en contexto al hacer clic. |
| ¿Pipeline de 8 módulos es fijo? | **No.** Es el preset recomendado. El usuario puede reordenarlo o ignorar módulos. |

---

## ADRs relevantes

| ADR | Contenido |
|---|---|
| ADR-0136 | Especificación completa del Canvas [Forge/Reactor] + Dashboard. **Supersede ADR-0028.** |
| ADR-0137 | Feature como unidad hexagonal autónoma + catálogo de Port Types (108 tipos). |
| ADR-0028 | ⚠️ SUPERSEDIDO por ADR-0136. |
| ADR-0117 | Thin Shell bajo Techo Fijo. Actualizado para referenciar ADR-0136. |
| ADR-0135 | `## Cáscara Visual` obligatoria. Clasificación por contexto (ADR-0136) en lugar de MACRO/MESO/MICRO. |
| ADR-0009 | `ExecutableContainer` — tipo omnipresente que viaja por todo el pipeline. |

---

## Pendientes (migración de documentos)

El inventario de referencias a MACRO/MESO/MICRO identificó **~140 instancias en ~25 archivos**. Las actualizaciones de mayor prioridad ya están aplicadas. Pendiente progresivo:

| Archivo | Tipo de cambio | Prioridad |
|---|---|---|
| ~~`docs/features/canvas-navigation.md`~~ | ~~Reescribir feature para el nuevo modelo de canvas~~ | ✅ HECHO (2026-06-23) |
| ~~`docs/features/visual-dag-editor.md`~~ | ~~Reescribir + corregir alias ADR-0038 + eliminar WebGL~~ | ✅ HECHO (2026-06-23) |
| ~~`docs/sad/SAD-06.md`~~ | ~~Actualizar descripción de los 3 niveles~~ | ✅ HECHO (2026-06-23) |
| ~~`docs/adr/ADR-0038.md`~~ | ~~Actualizar alias prohibidos con nueva nomenclatura de ADR-0136~~ | ✅ HECHO (2026-06-23) |
| ~~`docs/modules/*.md` (los 8 módulos)~~ | ~~Cambiar lenguaje "owna features" → "composición preset" (ADR-0137)~~ | ✅ HECHO (2026-06-23) |
| `docs/features/*.md` todos | Añadir `## Puertos de Integración` (progresivo, al construir cada feature en el ROADMAP) | PROGRESIVO |
| Tokens de diseño `zui-nav-pill` → `canvas-breadcrumb`, `zui-zoom-frame` → `canvas-zoom-frame`, `fleet-command-panel` → `dashboard-panel` | Renombrados en DESIGN.md y ui-designer SKILL (2026-06-23) | ✅ HECHO |
