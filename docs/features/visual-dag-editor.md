# Visual DAG Editor — Canvas Nodal de Construcción de Lógica

**Carpeta:** `./features/visual-dag-editor/`
**Estado:** Lista para implementar
**Última actualización:** 2026-06-23
**Decisión Arquitectónica Asociada:** ADR-0022 (Pipeline No-Lineal), ADR-0136 (Canvas [Forge/Reactor])

---

## ¿Qué es esta feature?

El Visual DAG Editor implementa el lienzo nodal interactivo del Canvas [Forge/Reactor] (ADR-0136). Permite al usuario construir y modificar pipelines de estrategias mediante **card-nodes rectangulares** estilo N8N / React Flow: arrastrar nodos desde una paleta, conectar puertos tipados con bezier S-curves, expandir módulos en sus features internas, y editar la lógica interna de una Strategy mediante bloques micro (Logic Blocks).

La feature también implementa el **Motor de Invalidación Reactiva**: cualquier cambio en parámetros de Logic Blocks invalida el caché de validación de la estrategia padre y lo marca visualmente en el canvas, impidiendo el despliegue a producción con resultados obsoletos.

---

## Comportamientos Observables

- [ ] El usuario arrastra un nodo de Módulo al canvas y lo conecta a otro mediante su puerto de salida — el canvas valida que los tipos de puerto sean compatibles (ADR-0137) antes de aceptar la conexión.
- [ ] Si el usuario intenta conectar un puerto `Bars` a un input `Signal`, la línea aparece en `criticalCrimson` con tooltip "tipos incompatibles: Bars → Signal".
- [ ] Doble clic en un nodo de Módulo (compound) → canvas anima zoom in-place y expone sus Features internas, con toggle entre graph view y mixer view (estilo DAW).
- [ ] Doble clic en una Strategy → canvas expone sus Logic Blocks (DAG de señales e indicadores micro).
- [ ] Clic en un Feature node (nodo atómico hoja) → abre Inspector Panel lateral derecho sin salir del canvas.
- [ ] Al editar un parámetro en el Inspector Panel de un Logic Block, todos los nodos de validación de esa strategy en el canvas se colorean en `criticalCrimson` con etiqueta "Caché de Validación Inválido".
- [ ] Un parámetro con el toggle "Exponer al Optimizador" activado queda resaltado en el Inspector Panel y se inyecta en el espacio de búsqueda del motor genético.
- [ ] El usuario activa "Modo Rendimiento" y los nodos del canvas se colorean con gradiente `reactorGreen → alertAmber → criticalCrimson` según sus microsegundos de latencia medidos.
- [ ] El usuario activa "Vista Diff" al comparar dos versiones del DAG — los nodos eliminados muestran overlay `criticalCrimson`, los añadidos muestran `reactorGreen`.
- [ ] En el Inspector Panel de un Logic Block, el usuario escribe una función Rhai, define dos pines tipados, y el nodo se ejecuta a velocidad nativa en Rust.

---

## Restricciones

- **NUNCA** permitir el despliegue a producción de una strategy con nodos de validación marcados como inválidos (caché stale).
- **NUNCA** conectar tipos de datos incompatibles sin nodo convertidor explícito — la validación de tipos es inmediata en el canvas (ADR-0137).
- **NUNCA** usar WebGL, WebView, DOM, HTML ni SVG. Todo el rendering del canvas es `CustomPainter` / Impeller GPU nativo (ADR-0097).
- **NUNCA** usar Python u otros intérpretes externos — el Rhai Escape Hatch evalúa código en el runtime de scripting Rhai embebido en Rust.
- **Límite Técnico:** Invalidación visual y actualización de estado del canvas en < 16ms (1 frame a 60fps).

---

## Parámetros Configurables

| Parámetro | Default | Rango | Qué hace | FIJO/CONFIG |
|---|---|---|---|---|
| `AUTO_INVALIDATE_CACHE` | `true` | true/false | Si la edición de un Logic Block invalida automáticamente el caché de validación | [FIJO] |
| `MAX_OPT_PARAMETERS` | 20 | 5 – 100 | Límite de parámetros expuestos simultáneamente al optimizador genético | CONFIG |
| `MAX_VISIBLE_NODES` | 200 | 50 – 500 | Máximo de nodos visibles sin culling en el canvas | CONFIG |
| `BEZIER_TENSION` | 0.5 | 0.2 – 0.9 | Curvatura de las bezier S-curves de conexión | CONFIG |

---

## Estructura Interna (FCIS — ADR-0002)

- **Core (Lógica Pura):** Gestión del grafo DAG (aciclicidad via `petgraph`); validación de tipos de puerto en conexiones; motor de invalidación de caché (comparación de `logic_hash`); mapeo de parámetros expuestos al optimizador; evaluación segura de nodos Rhai.
- **Shell (Infraestructura):** Renderer del canvas (`CustomPainter`/Impeller): card-nodes, bezier S-curves, port handles, dot-grid background; gestor de drag-and-drop; Inspector Panel lateral en Flutter; lector de manifiestos JSON AST; motor de templates.
- **Frontera Pública:** Contrato de importación/exportación de `ExecutableContainer` / `StrategyManifest` y bus de eventos de invalidación de caché.

---

## Ciclo de Vida de la Feature

### Entrada
- Interacciones del usuario en el canvas (drag, connect, doble clic, edits en Inspector Panel).
- `ExecutableContainer` existentes cargados desde los módulos `manage` o `generate`.
- `BacktestResult` / `RobustnessScore` para colorear el estado de validación de los nodos.
- `TelemetrySample` de latencia para el modo heatmap de rendimiento.

### Proceso
- Mapea elementos gráficos del canvas 1:1 con nodos y aristas del JSON AST global.
- Valida compatibilidad de tipos en cada conexión nueva (ADR-0137).
- Evalúa dependencias de parámetros y marca invalidaciones en cascada cuando se edita un Logic Block.
- Gestiona el estado de cada nodo (valid / stale / running / error) basado en eventos del pipeline.

### Salida
- `ExecutableContainer` modificado / ensamblado tras la edición en el canvas.
- `StrategyVersionNode` creado al guardar cambios (versionado git-like).
- Bus de eventos de invalidación hacia el módulo `validate`.

---

## Tareas (TTRs)

### TTR-001: Canvas de Card-Nodes (N8N / React Flow)

**¿Cuál es el problema?** El operador necesita construir y reconfigurar pipelines de validación y estrategia de forma visual, conectando módulos y features como si fueran bloques.

**¿Qué tiene que pasar?** Implementar el canvas interactivo en Flutter `CustomPainter` con:
- Card-nodes: header `32–36px` (nombre, ícono, borde izquierdo coloreado por dominio), body (key-values del nodo).
- Port handles: círculos de `10px` en los laterales izquierdo (inputs) y derecho (outputs).
- Conexiones: bezier S-curves coloreadas por tipo de dato (ADR-0137), con arrowhead en el target.
- Fondo: dot-grid `borderPanel #1B2440` 1.5px @ 20px sobre `deepSpace #080A18`.
- Drag-and-drop de nodos desde una paleta lateral.
- Layout automático con algoritmos jerárquicos (Dagre) como opción de auto-organización.

**¿Cómo sé que está hecho?**
- [ ] Puedo arrastrar un nodo de Feature desde la paleta y soltarlo en el canvas.
- [ ] Puedo conectar el output de un nodo al input de otro — la línea es bezier S-curve del color del tipo.
- [ ] Puedo reordenar nodos y reconectar sin que el DAG acíclico se rompa.

### TTR-002: Logic Block Editor (Canvas Interior de Strategy)

**¿Cuál es el problema?** Diseñar reglas de señal específicas requiere granularidad de indicadores, patrones de velas y operadores matemáticos.

**¿Qué tiene que pasar?** Al expandir una Strategy (zoom in-place al canvas interior), exponer una paleta de micro-bloques clasificados: `Data Source`, `Indicator Blocks`, `Signal Blocks` (Candle Patterns, Time Filters, Position State), `Execution Blocks`. Los bloques se conectan con validación de tipos en el canvas interior.

**¿Cómo sé que está hecho?**
- [ ] Puedo diseñar un cruce de EMA con filtrado por patrón Doji y día de la semana.
- [ ] Los tipos de conexión entre micro-bloques se validan igual que en el canvas principal.

### TTR-003: Motor de Invalidación Reactiva

**¿Cuál es el problema?** Alterar parámetros de Logic Blocks sin re-evaluar la robustez conduce a falsos positivos en producción.

**¿Qué tiene que pasar?** Implementar motor de invalidación en Flutter que compare `logic_hash` de la strategy antes y después de cada edición. Si difieren, todos los nodos de validación de esa strategy en el canvas (WFA, MC, CPCV) cambian a estado stale: `criticalCrimson` con etiqueta "Re-evaluación Mandatoria".

**¿Cómo sé que está hecho?**
- [ ] Cambio el período de un RSI en el Inspector Panel y los nodos WFA y MC pasan de `optimaCyan` a `criticalCrimson` con la advertencia.
- [ ] El sistema bloquea el despliegue a producción mientras haya nodos en estado stale.

### TTR-004: Visualizadores de Optimización (CustomPainter — sin WebGL)

**¿Cuál es el problema?** Identificar agrupamientos y estabilidad de parámetros optimizados entre miles de iteraciones es inmanejable en tablas.

**¿Qué tiene que pasar?** En el Inspector Panel de una Strategy, implementar dos visualizadores de análisis de optimización con `CustomPainter` nativo (ADR-0097 — prohibido WebGL):
1. **Parallel Coordinates** — para evaluar regímenes estables de parámetros (referencia: `ParameterSensitivityPainter` en la galería).
2. **Scatter UMAP 2D** — dispersión espacial de robustez de candidatos. Las coordenadas UMAP se calculan en Rust; el canvas solo renderiza puntos 2D. Soporta brushing (lasso) para drill-down. Referencia: `RiskReturnScatterPainter` en la galería.

**¿Cómo sé que está hecho?**
- [ ] El Inspector Panel muestra el scatter UMAP 2D de candidatos optimizados en < 50ms.
- [ ] Puedo seleccionar un clúster con lasso y el canvas resalta esos nodos candidatos.
- [ ] El parallel coordinates chart muestra los parámetros de los candidatos seleccionados.

### TTR-005: Rhai Escape Hatch (Nodo de Código Embebido)

**¿Cuál es el problema?** El catálogo de bloques puede carecer de operadores específicos que el operador desea programar a mano.

**¿Qué tiene que pasar?** Crear un tipo de nodo especial inyectable con editor de código embebido nativo Flutter. El código (Rhai) se evalúa en el runtime de scripting Rust. El usuario declara explícitamente los tipos de puerto de entrada y salida del nodo para que el canvas pueda validar conexiones (ADR-0137).

**¿Cómo sé que está hecho?**
- [ ] Escribo una función de filtro matemático en el editor Rhai del nodo, expongo un input `Bars` y un output `Signal`, y el nodo se conecta y ejecuta sin intérpretes externos.

### TTR-006: Heatmap de Latencia de Nodos

**¿Cuál es el problema?** Identificar cuellos de botella entre decenas de nodos es difícil sin perfiles visuales.

**¿Qué tiene que pasar?** En "Modo Rendimiento", colorear los card-nodes con gradiente `reactorGreen → alertAmber → criticalCrimson` según sus microsegundos de ejecución medidos via `TelemetrySample`.

**¿Cómo sé que está hecho?**
- [ ] Al activar el Modo Rendimiento, los nodos lentos muestran su color de heatmap y un tooltip con los microsegundos.

### TTR-007: Git Visual — Diff de Grafo

**¿Cuál es el problema?** Comparar cambios entre versiones lógicas del canvas a nivel de texto JSON es propenso a errores cognitivos.

**¿Qué tiene que pasar?** Vista de diff del canvas: nodos eliminados con overlay `criticalCrimson`, nodos añadidos con `reactorGreen`, nodos modificados con `alertAmber`. Alimentada por `strategy-versioning`.

**¿Cómo sé que está hecho?**
- [ ] Al comparar dos `StrategyVersionNode`, el canvas resalta el diff gráfico en colores semánticos.

### TTR-008: Canvas de Meta-estrategias (Vista Interior de Portfolio)

**¿Cuál es el problema?** Orquestar múltiples strategies con lógicas de control de capital cruzadas requiere un nivel de abstracción superior.

**¿Qué tiene que pasar?** En la Vista Interior de un nodo Portfolio (jerarquía de entidades), el canvas expone un grafo donde los nodos son Strategies completas y las aristas representan reglas de rebalanceo y distribución de margen (ej: si Strategy A Drawdown > 15%, desactiva Strategy B y transfiere margen a C). Las reglas se evalúan en Rust durante la ejecución.

**¿Cómo sé que está hecho?**
- [ ] Puedo diseñar un grafo de portfolio conectando dos nodos de strategy a una regla global de balance.
- [ ] La regla dispara los ajustes de margen correspondientes en el runtime.

---

## Puertos de Integración (ADR-0137)

| Puerto | ID de tipo | Dirección | Cardinalidad | Descripción |
|---|---|---|---|---|
| `container_in` | `ExecutableContainer` | Input | `0..N` | Containers de strategy/portfolio que se cargan como nodos en el canvas |
| `manifest_in` | `StrategyManifest` | Input | `0..1` | Manifiesto de strategy para renderizar su estado en el canvas |
| `backtest_results_in` | `BacktestResult` | Input | `0..N` | Resultados de validación para colorear el estado (valid/stale/running) de los nodos |
| `robustness_score_in` | `RobustnessScore` | Input | `0..N` | Score de robustez para el indicador de vitalidad en el header del card-node |
| `telemetry_in` | `TelemetrySample` | Input | `0..N` | Muestras de latencia para el heatmap de rendimiento (TTR-006) |
| `container_out` | `ExecutableContainer` | Output | `0..1` | Container ensamblado/modificado tras la edición en el canvas |
| `version_node_out` | `StrategyVersionNode` | Output | `0..1` | Nodo de versión creado al guardar cambios en el DAG |

---

## Gobernanza y Estándares (Fijos)

- **Local-First (ADR-0016):** 100% Local.
- **Inundación de Fundaciones (ADR-0020 V2):** Perfil B (IA / R&D). Registra `id`, `created_at`, `updated_at`, `audit_hash`, `audit_chain_hash`, `event_sequence_id`, `logic_hash`, `manifest_id`, `version_node_id`, `node_id`.

**Rastro de Evidencia:** Emite `logic_hash` de invalidación y auditoría de parámetros expuestos para el módulo `feedback`.

---

## Dependencias y Bloqueantes

**Depende de:**
- [`strategy-versioning`](../features/strategy-versioning.md) — para el DAG de versiones del AST y TTR-007.
- [`robustness-score-aggregator`](../features/robustness-score-aggregator.md) — para actualizar el estado de nodos de validación.
- [`canvas-navigation`](../features/canvas-navigation.md) — para el viewport manager y las transiciones de zoom in-place.

**Bloquea:**
- [`generate`](../modules/generate.md) — requiere el canvas para la construcción y visualización de Logic Blocks.

**Contrato de Integración UI (ADR-0117):**
- **Superficie propia:** el Canvas [Forge/Reactor] es la superficie central de la app. SVF: un nodo arrastrable al canvas, una conexión válida entre dos nodos, y el `ExecutableContainer` resultante persistido y visible tras recargar.
