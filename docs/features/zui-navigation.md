# ZUI Navigation — Navegación Fractal de 3 Niveles

**Carpeta:** `./features/zui-navigation/`
**Estado:** Lista para implementar
**Última actualización:** 2026-06-06
**Decisión Arquitectónica Asociada:** ADR-0028 (ZUI Navigation)

---

## ¿Qué es esta feature?

La **Zoomable User Interface (ZUI)** es el paradigma de navegación espacial y contextual de Drasus Engine. En lugar de pantallas y menús aislados tradicionales que causan desorientación en el operador, la aplicación presenta un lienzo infinito continuo y tridimensional renderizado por GPU (Impeller) con tres niveles de zoom fractal.

El sistema permite transitar en milisegundos desde el monitoreo global del portafolio hasta la inspección del genoma lógico de una estrategia individual. A nivel macro, integra la gestión de portafolios (Fleet Command), la correlación cruzada dinámica en tiempo real y la visualización de la equity agregada.

---

## Niveles de Navegación

### **Nivel 1: Fleet Command (Visión de Flota - Macro)**
*   **Contenido:** Lienzo infinito con la red de portafolios activos y agentes operando como bloques autónomos.
*   **Métricas y Análisis:**
    - **Matriz de Correlación Dinámica:** Consulta de coeficientes Pearson calculados al vuelo mediante DuckDB. Si dos estrategias superan un umbral crítico (Pearson > 0.85), la línea conectora en el grafo parpadea en ámbar como alerta de riesgo de concentración.
    - **Curva de Equity Global:** Suma vectorial en tiempo real de todas las curvas de balance de las estrategias activas.
    - **Inspector Contextual (Macro):** Parámetros y reglas de portafolio global (ej: Max Drawdown global, asignación de margen, optimización de varianza media).
*   **Interacción:** Clic o desplazamiento profundo (Zoom In) sobre un portafolio específico transiciona suavemente hacia el Nivel 2.

### **Nivel 2: Orchestrator (Visual Editor DAG - Meso)**
*   **Contenido:** El lienzo de diseño visual de pipelines de estrategias (Visual DAG Editor).
*   **Métricas:** Barra de progreso y ETA de jobs de simulación y optimización genética.
*   **Interacción:** "Zoom In" clickeando en una estrategia aprobada o "Zoom Out" para regresar a la flota global.

### **Nivel 3: Strategy Inspector (Inspector de Estrategia - Micro)**
*   **Contenido:** Máxima granularidad analítica de una única estrategia.
*   **Métricas:** Gráfico de equity a 60fps, cono de confianza Monte Carlo, veredictos de robustez en lenguaje natural y visualización del genoma (AST) o código de scripting seguro (Rhai).
*   **Interacción:** Panel de editor de código embebido nativo Flutter para inyección y gráficos nativos Flutter CustomPainter/Impeller para visualización interactiva.

---

## Restricciones

- **NUNCA** bloquear el renderizado visual durante cálculos matemáticos de correlación o suma de curvas (deben ejecutarse de forma asíncrona en workers Rust/DuckDB).
- **NUNCA** permitir desbordamiento de memoria al renderizar más de 10,000 puntos en gráficos históricos de equity (aplicar downsampling LTTB en backend antes de pasar a Dart FFI).
- **Límite Técnico:** Latencia de transición entre niveles de zoom debe ser inferior a 300ms.

---

## Parámetros Configurables

| Parámetro | Default | Rango | Qué hace | FIJO/CONFIG |
|---|---|---|---|---|
| PEARSON_WARN_THRESHOLD | 0.85 | 0.50 - 0.99 | Umbral de correlación para disparar alertas visuales en Fleet | CONFIG |
| TRANSITION_DURATION_MS | 250 | 100 - 1000 | Duración del efecto de zoom animado entre capas | CONFIG |
| DOWNSAMPLING_POINTS | 1000 | 500 - 5000 | Puntos máximos a renderizar por serie de equity | [FIJO] |

---

## Estructura Interna (FCIS — ADR-0002)

- **Core (Lógica Pura):** Gestión de coordenadas de escala, cálculo vectorial de equity agregada y lógica de transiciones de escala.
- **Shell (Infraestructura):** Conectores FFI para solicitar downsampling del backend, y consultas SQL a DuckDB para correlación Pearson.
- **Frontera Pública:** Interfaz de navegación para mover viewport a coordenadas específicas e invocar renders de nivel.

---

## Ciclo de Vida de la Feature

### Entrada
- Datos de rendimiento y balances históricos de DuckDB.
- Matrices de correlación Pearson.
- Eventos de entrada de ratón/gestos de zoom de la UI.

### Proceso
- Aplica transformaciones matriciales de escala y traslación en el canvas GPU.
- Monitorea umbrales de escala para gatillar transiciones de estado visual (Macro -> Meso -> Micro).
- Agrega vectorialmente curvas de balance individuales.

### Salida
- Coordenadas de Viewport y escala actualizadas.
- Renderizado de capas con nivel de detalle adaptativo.

---

## Tareas (TTRs)

### **TTR-001: Viewport Manager y Transición de Escalas**
*   **¿Cuál es el problema?** Cambiar de pantalla mediante menús rompe el contexto operativo y es lento.
*   **¿Qué tiene que pasar?** Desarrollar controlador de Viewport en Flutter que escale y desplace el lienzo suavemente manejando niveles de detalle (LOD) dinámicos según el factor de escala de zoom.
*   **¿Cómo sé que está hecho?**
    - [ ] El paso de Fleet Command a Orchestrator se siente continuo, sin saltos abruptos de UI.
    - [ ] Latencia medida de renderizado de transición es <300ms.

### **TTR-002: Matriz de Correlación Dinámica Pearson (DuckDB)**
*   **¿Cuál es el problema?** El operador puede no notar si dos estrategias son redundantes y se correlacionan en pérdidas.
*   **¿Qué tiene que pasar?** Desarrollar query periódica a DuckDB para evaluar la correlación Pearson en retornos diarios de estrategias activas, y enlazarla a la UI para parpadear en ámbar los cables virtuales correspondientes si Pearson > Umbral.
*   **¿Cómo sé que está hecho?**
    - [ ] Clic en Fleet Command muestra líneas de interconexión parpadeando en ámbar entre estrategias con Pearson > 0.85.

### **TTR-003: Agregador Vectorial de Equity Curve**
*   **¿Cuál es el problema?** Visualizar el rendimiento conjunto de la flota requiere combinar miles de registros históricos eficientemente.
*   **¿Qué tiene que pasar?** Implementar sumador vectorial en Rust que combine las curvas de equidad de las estrategias seleccionadas, aplique downsampling y devuelva el array a través de Dart FFI para visualización inmediata.
*   **¿Cómo sé que está hecho?**
    - [ ] Fleet Command renderiza la curva global consolidada en tiempo real al activar o desactivar estrategias del portafolio.

---

## Gobernanza y Estándares (Fijos)
- **Local-First (ADR-0016):** 100% Local. DuckDB realiza consultas directo en archivos Parquet locales.
- **Inundación de Fundaciones (ADR-0020 V2):** Perfil Ops / Auditoría. Registra `owner_id`, `session_id`, `node_id`, `execution_latency_ms`.
- **Rastro de Evidencia:** Loguea latencias de zoom y carga de gráficos para análisis de rendimiento en `feedback`.

---

## Dependencias
**Depende de:**
- [`duckdb-sql-engine`](../features/duckdb-sql-engine.md) — para la agregación rápida y Pearson.
- [`visual-dag-editor`](../features/visual-dag-editor.md) — para el lienzo del Nivel 2.

**Bloquea:**
- [`ingest`](../modules/ingest.md) — requiere visualización de datasets.
- [`manage`](../modules/manage.md) — requiere Fleet Command para la gestión de portafolios.
