# Canvas Navigation — Viewport, Zoom, Breadcrumb y Equity Agregada

**Carpeta:** `./features/canvas-navigation/`
**Estado:** Lista para implementar
**Última actualización:** 2026-06-23
**Decisión Arquitectónica Asociada:** ADR-0136 (Canvas [Forge/Reactor] — supersede ADR-0028)

---

## ¿Qué es esta feature?

Canvas Navigation gestiona el estado de viewport del lienzo unificado de Drasus Engine (ADR-0136): posición espacial, factor de zoom, transiciones animadas entre estados y navegación por la jerarquía de entidades (Cluster → Portfolio → Strategy → Logic Blocks). También agrega vectorialmente las curvas de equity de las Strategies activas en cada nivel jerárquico y superpone alertas de correlación Pearson entre nodos de Portfolio.

Sin esta feature el Canvas no tiene memoria de posición ni transiciones suaves — el usuario teletransporta sin contexto y pierde orientación en grafos de decenas de nodos.

---

## Comportamientos Observables

- [ ] Al hacer doble clic en un nodo Cluster → el canvas anima zoom in-place y aparecen sus Portfolios. El breadcrumb flotante muestra `Cluster A`.
- [ ] Al hacer doble clic en un nodo Portfolio → el canvas muestra sus Strategies. El breadcrumb muestra `Cluster A › Portfolio B`.
- [ ] Al hacer doble clic en una Strategy → el canvas entra en Vista Interior y expone sus Logic Blocks (DAG de señales). El breadcrumb muestra `Cluster A › Portfolio B › Strategy 3`.
- [ ] Clic en cualquier segmento del breadcrumb → el canvas anima zoom out in-place al nivel correspondiente.
- [ ] Al alejar el zoom (scroll out), los card-nodes reducen su detalle: solo header + indicador de vitalidad (modo condensado). Al acercar (scroll in), expanden su body con key-values.
- [ ] El Focus Mode es un filtro de visibilidad por tipo de nodo (checkboxes: Cluster, Portfolio, Strategy, Pipeline, Módulo, Feature, Logic Block). No parte el canvas ni restringe el breadcrumb — solo oculta/muestra categorías de nodos para reducir ruido visual. Por defecto, todos visibles.
- [ ] En la vista de Portfolio, los nodos de Strategy conectados muestran líneas de correlación Pearson. Si Pearson > `PEARSON_WARN_THRESHOLD`, la línea parpadea en `alertAmber`.
- [ ] El nodo Portfolio muestra una mini equity curve que es la suma vectorial de las equity curves de sus Strategies activas.
- [ ] La transición de zoom entre niveles completa en < 300ms.
- [ ] Al reabrir la app, el canvas restaura la última posición y nivel de jerarquía de la sesión anterior.

---

## Restricciones

- **NUNCA** bloquear el hilo de renderizado (60fps) durante el cálculo de correlación Pearson o la agregación vectorial de equity — ambos corren en workers Rust y pasan el resultado via FFI.
- **NUNCA** renderizar más de `DOWNSAMPLING_POINTS` puntos por serie de equity — el downsampling LTTB se aplica en el backend antes de pasar al canvas.
- **NUNCA** saltar niveles intermedios de la jerarquía de entidades (no se puede ir de Cluster directo a Logic Block sin pasar por Portfolio y Strategy).
- **Límite Técnico:** Latencia de transición de zoom < 300ms en dispositivo objetivo.

---

## Parámetros Configurables

| Parámetro | Default | Rango | Qué hace | FIJO/CONFIG |
|---|---|---|---|---|
| `PEARSON_WARN_THRESHOLD` | 0.85 | 0.50 – 0.99 | Umbral Pearson para parpadeo `alertAmber` en conexiones de Portfolio | CONFIG |
| `TRANSITION_DURATION_MS` | 250 | 100 – 1000 | Duración de la animación de zoom entre niveles de entidad | CONFIG |
| `DOWNSAMPLING_POINTS` | 1000 | 500 – 5000 | Puntos máximos por serie de equity antes del render | [FIJO] |
| `LOD_CONDENSED_THRESHOLD` | 0.4 | 0.2 – 0.7 | Factor de escala por debajo del cual los nodos entran en modo condensado | CONFIG |

---

## Estructura Interna (FCIS — ADR-0002)

- **Core (Lógica Pura):** Transformaciones matriciales de escala y traslación; lógica de transición de nivel (qué nodo se expande, qué jerarquía se muestra); agregación vectorial de equity; evaluación del umbral Pearson.
- **Shell (Infraestructura):** Conectores FFI para solicitar downsampling LTTB y correlación Pearson al backend Rust; gestor de estado de sesión para persistir posición de viewport en SQLite local.
- **Frontera Pública:** API para teleportar el viewport a un nodo específico (usada cuando el Dashboard abre el canvas en contexto), exponer el breadcrumb path actual, y suscribirse a cambios de nivel de entidad.

---

## Ciclo de Vida de la Feature

### Entrada
- Eventos de gesto del usuario: pinch-to-zoom, scroll wheel, doble clic en nodo, clic en breadcrumb.
- `EquityCurve` de las Strategies activas (para agregación vectorial en nodos Portfolio/Cluster).
- `CorrelationMatrix` entre Strategies activas (para alertas de correlación en conexiones).

### Proceso
- Actualiza coordenadas y factor de escala del viewport.
- Evalúa el nivel de jerarquía de entidades activo y determina qué nodos renderizar.
- Agrega vectorialmente las equity curves de Strategies activas → produce equity curve del Portfolio.
- Evalúa Pearson entre pares de Strategies conectadas → actualiza color y parpadeo de conexiones.
- Determina modo LOD (condensado/completo) según el factor de escala actual.

### Salida
- Estado de viewport actualizado (posición, escala, nivel jerárquico activo).
- Equity curve agregada a nivel Portfolio y Cluster.
- Estado de alerta de correlación por par de nodos Strategy.
- Breadcrumb path para el componente UI flotante.

---

## Tareas (TTRs)

### TTR-001: Viewport Manager y Transiciones de Jerarquía

**¿Cuál es el problema?** El canvas necesita un gestor de estado que mapee gestos de usuario a transformaciones de viewport y coordine animaciones de zoom in-place entre niveles de la jerarquía de entidades.

**¿Qué tiene que pasar?** Implementar controlador de viewport en Flutter que gestione posición, escala y nivel de jerarquía activo. Las transiciones entre niveles (Cluster → Portfolio → Strategy → Logic Blocks) animan el canvas in-place con `AnimationController`. Al hacer zoom out, los sub-nodos se colapsan de vuelta al nodo padre con animación inversa.

**¿Cómo sé que está hecho?**
- [ ] Doble clic en Cluster → aparecen sus Portfolios con animación suave, < 300ms.
- [ ] Clic en `Cluster A` del breadcrumb → canvas vuelve al nivel Cluster con animación inversa.
- [ ] Scroll wheel hace zoom continuo dentro del nivel activo sin saltar jerárquicamente.

### TTR-002: LOD Adaptativo (Nodos Condensados vs Completos)

**¿Cuál es el problema?** Con docenas de nodos visibles, los card-nodes completos saturan el canvas y se vuelven ilegibles.

**¿Qué tiene que pasar?** Implementar dos modos de render de card-node: `condensed` (solo header 32px + indicador de vitalidad) y `full` (header + body con key-values). El cambio de modo es función del factor de escala actual — por debajo de `LOD_CONDENSED_THRESHOLD`, todos los nodos del nivel visible se renderizan en modo condensado.

**¿Cómo sé que está hecho?**
- [ ] Al alejar el zoom al 40% de escala, los nodos muestran solo header + color semántico.
- [ ] Al acercar por encima del umbral, los nodos expanden su body progresivamente.

### TTR-003: Agregación Vectorial de Equity y Alertas de Correlación

**¿Cuál es el problema?** En la vista de Portfolio, el operador necesita ver la equity consolidada del portfolio y detectar redundancia entre strategies.

**¿Qué tiene que pasar?**
1. Implementar sumador vectorial en Rust que combine las equity curves de las Strategies activas del Portfolio visible, aplique downsampling LTTB, y devuelva el array via FFI para renderizado en el nodo Portfolio.
2. Implementar query periódica de correlación Pearson entre Strategies activas del Portfolio, usando DuckDB. Si Pearson > `PEARSON_WARN_THRESHOLD`, la conexión entre esos nodos parpadea en `alertAmber`.

**¿Cómo sé que está hecho?**
- [ ] El nodo Portfolio muestra una mini equity curve que es la suma de sus Strategies activas.
- [ ] Dos Strategies con Pearson > 0.85 muestran su conexión parpadeando en ámbar.
- [ ] El parpadeo desaparece al desactivar una de las dos Strategies.

### TTR-004: Filtro de Visibilidad por Tipo de Nodo y Persistencia de Sesión

**¿Cuál es el problema?** El canvas unificado contiene todos los tipos de nodo (Cluster, Portfolio, Strategy, Pipeline, Módulo, Feature, Logic Block). Con decenas de nodos visibles, el usuario necesita ocultar categorías para enfocarse sin perder conexiones. También quiere que la app recuerde dónde estaba al reabrir.

**¿Qué tiene que pasar?**
1. Implementar panel de filtro en la toolbar del canvas: checkboxes por tipo de nodo (Cluster ☑, Portfolio ☑, Strategy ☑, Pipeline ☑, Módulo ☑, Feature ☑, Logic Block ☑). Por defecto, todos visibles. Ocultar un tipo no rompe conexiones existentes — las líneas hacia nodos ocultos se atenúan pero no desaparecen.
2. El breadcrumb **no se ve afectado por el filtro**: si el usuario navegó Pipeline → Módulo → Feature → Strategy → Logic Block, el breadcrumb muestra ese camino completo independientemente de qué tipos estén filtrados.
3. Persistir `{viewport_x, viewport_y, viewport_scale, breadcrumb_path, visibility_filters}` en SQLite local al cerrar o suspender la app.

**¿Cómo sé que está hecho?**
- [ ] El panel de filtro muestra checkboxes por tipo de nodo; desmarcar "Módulo" oculta todos los nodos de módulo pero sus conexiones permanecen visibles (atenuadas).
- [ ] El breadcrumb refleja el camino completo de navegación aunque algunos tipos estén filtrados.
- [ ] Al reabrir la app, el canvas restaura la última posición, breadcrumb y filtros en < 200ms.

---

## Puertos de Integración (ADR-0137)

| Puerto | ID de tipo | Dirección | Cardinalidad | Descripción |
|---|---|---|---|---|
| `equity_curves_in` | `EquityCurve` | Input | `0..N` | Curvas de equity de Strategies activas; se agregan vectorialmente para el nodo Portfolio/Cluster |
| `correlation_matrix_in` | `CorrelationMatrix` | Input | `0..1` | Matriz Pearson entre Strategies activas; alimenta alertas de color en conexiones del Portfolio |
| `account_states_in` | `AccountState` | Input | `0..N` | Estado de cuenta por Strategy; alimenta indicadores de balance en card-nodes |
| `aggregated_equity_out` | `EquityCurve` | Output | `0..N` | Equity curve agregada a nivel Portfolio o Cluster; consumida por widgets del Dashboard |

---

## Gobernanza y Estándares (Fijos)

- **Local-First (ADR-0016):** 100% Local. DuckDB consulta archivos Parquet locales para correlación Pearson.
- **Inundación de Fundaciones (ADR-0020 V2):** Perfil D (Ops / Auditoría) — persiste preferencias de viewport de sesión.

| Categoría | Campo | Descripción |
|---|---|---|
| **I. Identidad** | `id` | ID de la preferencia de viewport de sesión |
| | `created_at` | Timestamp de creación |
| | `updated_at` | Timestamp de última modificación |
| | `audit_hash` | Hash de integridad de la config de viewport |
| | `audit_chain_hash` | Hash encadenado del historial |
| | `event_sequence_id` | Secuencia de recuperación |
| **II. Soberanía** | `owner_id` | Usuario dueño de la preferencia |
| **IV. Hardware** | `session_id` | Sesión de UI asociada |
| | `node_id` | ID del hardware/cliente |

**Rastro de Evidencia:** Emite latencias de zoom y carga de gráficos para análisis de rendimiento en `feedback`.

---

## Dependencias y Bloqueantes

**Depende de:**
- [`duckdb-sql-engine`](../features/duckdb-sql-engine.md) — para agregación Pearson y downsampling LTTB.
- [`equity-curve-tracker`](../features/equity-curve-tracker.md) — produce las `EquityCurve` que esta feature agrega.
- [`signal-correlation-analyzer`](../features/signal-correlation-analyzer.md) — produce la `CorrelationMatrix`.

**Bloquea:**
- [`visual-dag-editor`](../features/visual-dag-editor.md) — el canvas editor requiere el viewport manager para funcionar.
- [`manage`](../modules/manage.md) — requiere la vista de Portfolio (jerarquía de entidades) para la gestión de portafolios.

**Contrato de Integración UI (ADR-0117):**
- **Superficie propia:** componente core del Canvas [Forge/Reactor]. SVF: el canvas renderiza al menos un nodo y responde a gestos de zoom — el breadcrumb flotante cambia al expandir un nodo, el sub-nodo aparece animado, el estado de viewport se persiste y restaura tras recargar.
