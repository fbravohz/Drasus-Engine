# UMAP Scatter Visualizer — Visualizador Multidimensional de Robustez

**Carpeta:** `./features/umap-scatter-visualizer/`
**Estado:** Lista para implementar
**Última actualización:** 2026-06-06
**Decisión Arquitectónica Asociada:** ADR-0097 (Renderizado Gráfico Multidimensional Nativo sin WebViews)

---

## ¿Qué es esta feature?

El UMAP Scatter Visualizer es una herramienta del lienzo Meso/Micro que permite al operador explorar visualmente el espacio de robustez de miles de estrategias candidatas optimizadas mediante reducción de dimensionalidad UMAP.

Para evitar la latencia y sobrecarga de memoria de los motores WebViews (Plotly/Deck.gl), el visualizador renderiza en 2D o 3D de forma nativa mediante GPU (Flutter CustomPainter e Impeller), consumiendo arrays binarios de Apache Arrow pasados desde Rust FFI. Habilita una herramienta de selección por lazo (lasso select) o caja para aislar grupos de puntos (clústeres), actualizando instantáneamente la tabla de estrategias para drill-down semántico y rehidratación de AST.

---

## Comportamientos Observables

- [ ] Al cargar los resultados de optimización del Databank, la UI renderiza una nube de puntos (scatter plot 2D/3D) donde cada punto representa una estrategia.
- [ ] La coloración de los puntos se realiza dinámicamente según la métrica Sharpe Ratio (RdYlGn colormap) u otros parámetros de robustez seleccionables por el usuario.
- [ ] El operador puede arrastrar el cursor utilizando la herramienta de lazo (lasso2d) o caja (select2d) para seleccionar un conjunto de puntos.
- [ ] Al cerrar la selección, la UI resalta los puntos interceptados y actualiza una tabla adyacente con el listado de estrategias correspondientes a dichos puntos.
- [ ] Al hacer clic en una estrategia de la tabla, la UI realiza zoom dinámico y rehidrata su AST en el Strategy Inspector del Nivel 3.
- [ ] La UI permite animar la evolución de las estrategias a lo largo de las generaciones del motor genético moviendo un slider temporal.

---

## Restricciones

- **NUNCA** incrustar WebViews, motores HTML o dependencias JS en el visualizador; el scatter plot debe ser dibujado al 100% en el lienzo nativo de Flutter CustomPainter.
- **NUNCA** realizar cálculos pesados de coordenadas UMAP en el hilo de Dart; las coordenadas tridimensionales deben calcularse en Rust y transmitirse pre-procesadas.
- **Límite Técnico:** Renderizado interactivo estable a 120 FPS / 60 FPS con nubes de puntos de hasta 100,000 elementos en hardware local.

---

## Parámetros Configurables

| Parámetro | Default | Rango | Qué hace | FIJO/CONFIG |
|---|---|---|---|---|
| UMAP_DIMENSIONS | 3 | 2 - 3 | Dimensiones del scatter plot interactivo | CONFIG |
| MAX_VISIBLE_POINTS | 50000 | 1000 - 200000 | Límite máximo de puntos renderizados simultáneamente en lienzo | CONFIG |
| DART_COLLISION_LATENCY_CAP | 16ms | 5ms - 50ms | Tiempo de procesamiento máximo para resolver intersecciones de lazo | [FIJO] |

---

## Estructura Interna (FCIS — ADR-0002)

- **Core (Lógica Pura):** Algoritmo de mapeo de coordenadas espaciales, escala cromática (colormap mapper) e intersección geométrica de polígonos de lazo en Dart.
- **Shell (Infraestructura):** API Rust para la carga y proyección de coordenadas UMAP en archivos Parquet (`strategies_umap_cached.parquet`), serialización de arrays binarios Arrow y enlace FFI.
- **Frontera Pública:** Interfaz para solicitar proyecciones (`get_umap_projection`) y emitir identificadores seleccionados a la interfaz del orquestador.

---

## Ciclo de Vida de la Feature

### Entrada
- Identificador de pipeline y número de generación.
- Archivo Parquet indexado con coordenadas del embedding y métricas (`strategies_umap_cached.parquet`).
- Coordenadas de trazado de lazo emitidas por el ratón/gestos del usuario.

### Proceso
- Lee de forma Out-of-Core las coordenadas y métricas Sharpe en Rust.
- Realiza downsampling estocástico rápido si el total supera el límite visible.
- Transmite el payload en formato binario Arrow a Flutter.
- Renderiza en CustomPainter los puntos con su coloración y rotación tridimensional.
- Resuelve colisiones geométricas de lazo sobre las coordenadas proyectadas.

### Salida
- Coordenadas 3D renderizadas en pantalla en tiempo real.
- Listado de `strategy_ids` seleccionados por brushing.

---

## Tareas (TTRs)

### **TTR-001: API de Proyección y Caché de Embeddings UMAP (Rust Shell & Parquet Cache)**
*   **¿Cuál es el problema?** Calcular proyecciones UMAP para 100,000 estrategias candidato al vuelo o procesar objetos JSON de coordenadas introduce una latencia inaceptable y desborda la UI Dart.
*   **¿Qué tiene que pasar?** Desarrollar un subsistema en Rust que actúe como un gestor de caché de proyecciones (equivalente a `embeddings_cache` que persiste embeddings calculados en Parquet). Este debe leer de forma perezosa el archivo `strategies_umap_cached.parquet` (o regenerarlo si cambian parámetros), aplicar un filtro de densidad estocástico rápido si es necesario, y emitir los vectores de coordenadas 2D/3D directamente a través de Apache Arrow en memoria compartida.
*   **¿Cómo sé que está hecho?**
    - [ ] La consulta de proyecciones UMAP para 50,000 estrategias retorna el array binario en menos de 50ms desde la caché Parquet sin recalcular.

### **TTR-002: Canvas de Dispersión GPU en Flutter (Dart UI)**
*   **¿Cuál es el problema?** Las librerías de gráficos web (Plotly JS) en WebViews degradan el rendimiento, consumen RAM masiva y no se integran de forma frameless.
*   **¿Qué tiene que pasar?** Programar un renderizador de scatter plot 3D/2D nativo en Flutter CustomPainter que use buffers de vértices optimizados para la GPU mediante el motor Impeller.
*   **¿Cómo sé que está hecho?**
    - [ ] El gráfico 3D gira y responde al zoom a 120 FPS constantes con 50,000 puntos cargados.

### **TTR-003: Algoritmo de Brushing y Selección por Lazo (Dart Core)**
*   **¿Cuál es el problema?** Resolver la intersección de un polígono irregular (lazo) dibujado por el usuario con 50,000 puntos proyectados en pantalla puede causar congelamientos en la UI.
*   **¿Qué tiene que pasar?** Implementar un algoritmo de detección de colisiones geométricas eficiente en Dart (ej. Ray-Casting o R-Tree simplificado) que evalúe qué puntos caen dentro del lazo cerrado en menos de 16ms.
*   **¿Cómo sé que está hecho?**
    - [ ] El arrastre del lazo resalta los puntos de inmediato y actualiza la tabla de clúster sin retrasos perceptibles.

---

## Gobernanza y Estándares (Fijos)

- **Local-First (ADR-0016):** 100% Local. Las consultas se realizan sobre el databank Parquet local.
- **Inundación de Fundaciones (ADR-0020 V2):** Perfil IA / R&D. Registra `id`, `created_at`, `audit_hash`, `version_node_id`, `logic_hash`, `node_id`.
- **Rastro de Evidencia:** Emite logs de telemetría sobre la latencia de renderizado y el número de puntos escaneados para el módulo de `feedback`.

---

## Dependencias

**Depende de:**
- [`databank-lake`](../features/databank-lake.md) — para la lectura de datos Parquet.
- [`binary-arrow-transport`](../features/binary-arrow-transport.md) — para la transmisión de datos zero-copy.

**Bloquea:**
- [`validate`](../modules/validate.md) — el análisis interactivo de clústeres de optimización en el módulo de validación depende de este componente.
