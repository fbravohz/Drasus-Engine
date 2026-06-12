# Parallel Coordinates Visualizer

**Carpeta:** `./features/parallel-coordinates-visualizer/`
**Estado:** En Diseño
**Última actualización:** 2026-04-30
**Decisión Arquitectónica Asociada:** ADR-0020 V2

---

## ¿Qué es esta feature?

El **Visualizador de Coordenadas Paralelas** es un componente de análisis visual de alta densidad que permite proyectar optimizaciones de más de 20 parámetros simultáneamente. Cada eje vertical representa un parámetro o métrica de rendimiento (ej. Sharpe Ratio, Drawdown, Profit Factor), y cada backtest individual de la optimización se representa como una línea continua que cruza todos los ejes. 

**Problema que resuelve:** En optimizaciones masivas con docenas de variables, las tablas tradicionales o los heatmaps 2D y 3D colapsan y no permiten ver cómo interactúan los parámetros entre sí.
**Solución:** Permite al usuario "pintar" o aislar rangos en los ejes mediante un brushing interactivo (ej. seleccionar solo el 10% superior del eje del Sharpe Ratio) para ocultar las combinaciones perdedoras y revelar los vecindarios de parámetros robustos.

---

## Comportamientos Observables

- [ ] La interfaz despliega múltiples ejes verticales paralelos que representan el rango completo de cada parámetro y métrica.
- [ ] Al seleccionar un rango en un eje de métricas (ej. Sharpe Ratio > 1.5), todas las líneas que no cumplen el criterio se atenúan o se ocultan.
- [ ] Al aplicar múltiples filtros, la interfaz resalta los clústeres densos de líneas, revelando visualmente las zonas paramétricas más estables.
- [ ] Permite exportar la configuración de los parámetros seleccionados directamente al motor de generación para refinar la población.

---

## Restricciones

- **FIJO:** Los datos de backtests masivos no se cargan completos en la memoria de la UI. Se utiliza un servicio de reducción de resolución (downsampling) para evitar bloqueos del navegador.
- **NUNCA** recalcular las coordenadas en el hilo principal de la UI; el procesamiento se realiza mediante DuckDB en el backend sidecar.

---

## Parámetros Configurables

| Parámetro | Default | Rango | Qué hace | FIJO/CONFIG |
|---|---|---|---|---|
| MAX_VISIBLE_LINES | 5000 | 1000 - 50000 | Número máximo de líneas de backtests renderizadas simultáneamente. | CONFIG |
| DOWNSAMPLING_THRESHOLD | 10000 | 5000 - 100000 | Umbral de registros que activa la reducción de resolución de líneas. | CONFIG |
| BRUSHING_MODE | persistent | dynamic / persistent | Si el filtro visual permanece tras cambiar de página. | CONFIG |

---

## Estructura Interna (FCIS)

- **Core (Lógica Pura):** Procesamiento y filtrado de coordenadas mediante agregaciones DuckDB vectorizadas.
- **Shell (Infraestructura):** Integración con Apache Arrow para transmisión ultrarrápida de datos de backtests hacia el visor Flutter + Canvas.
- **Frontera Pública:** Puertos para consultar conjuntos de datos de backtest filtrados.

---

## Ciclo de Vida de la Feature — Parallel Coordinates Visualizer

### Entrada
- Matriz completa de resultados de optimización (Parquet/DuckDB DataFrame).
- Lista de nombres de parámetros y métricas seleccionadas para los ejes.

### Proceso
- Consulta los datos de backtest masivos desde DuckDB.
- Aplica el downsampling en caso de superar el umbral configurado.
- Serializa la información en formato Apache Arrow para su renderizado en la UI.

### Salida
- Representación visual interactiva en el lienzo de la UI con líneas y ejes.
- Conjunto de parámetros filtrados (`selected_parameter_spaces`) para alimentar el optimizador.

### Contextos de Uso

**Contexto 1: Visualización de Optimizaciones (Módulo Validate)**
- Herramienta principal para que el usuario decida qué combinaciones de parámetros son estables tras una optimización masiva.

---

## Tareas (TTRs)

### **TTR-001: Extracción y Downsampling de Resultados**
*   **¿Cuál es el problema?** Procesar millones de combinaciones de backtests directamente en la UI causa lentitud y bloqueos en el cliente.
*   **¿Qué tiene que pasar?** El sistema orquesta DuckDB para filtrar y reducir la densidad de registros (downsampling) conservando las tendencias y los extremos sin pérdida de información de frontera.
*   **¿Cómo sé que está hecho?**
    - [ ] La UI carga y responde de forma fluida con conjuntos de datos >10,000 backtests.
    - [ ] Se loguea el tiempo de extracción y downsampling de la matriz en el backend.

### **TTR-002: Brushing Interactivo y Filtrado Coordinado**
*   **¿Cuál es el problema?** El usuario necesita aislar visualmente las mejores estrategias sin escribir consultas complejas.
*   **¿Qué tiene que pasar?** Al arrastrar el ratón sobre un eje, la UI aplica un filtro condicional instantáneo sobre los otros ejes.
*   **¿Cómo sé que está hecho?**
    - [ ] El 100% de las líneas que no pasan el filtro visual cambian de color o se ocultan.
    - [ ] El visor actualiza dinámicamente los parámetros resultantes de la zona aislada.

---

## Gobernanza y Estándares (Fijos)

- **Local-First (ADR-0016):** 100% Local.
- **Fidelidad:** Alta fidelidad de filtrado basado en memoria compartida.
- **Inundación de Fundaciones (ADR-0020 V2):** 
    - Perfil: IA / R&D.
    - **I. Identidad & Integridad:** `id`, `created_at`, `audit_hash`, `event_sequence_id`.
    - **II. Soberanía & Propiedad:** `owner_id`, `institutional_tag`, `manifest_id`.
    - **III. Linaje Alpha & Datos:** `version_node_id`, `logic_hash`, `data_snapshot_id`.
    - **IV. Infraestructura & Ops:** `process_id`, `node_id`.
