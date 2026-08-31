# Cross-Filtering Visualizer

**Carpeta:** `./features/cross-filtering-visualizer/`
**Estado:** En Diseño
**Última actualización:** 2026-04-30
**Decisión Arquitectónica Asociada:** ADR-0020

---

## ¿Qué es esta feature?

El **Visualizador de Vistas Coordinadas** (Cross-Filtering) es un componente de análisis que presenta múltiples histogramas interactivos sincronizados en tiempo real. Cada histograma representa la distribución de un parámetro o métrica de rendimiento de una optimización (ej. eje X es el valor del parámetro, eje Y es la cantidad de backtests rentables).

**Problema que resuelve:** Permite al usuario comprender la relación causa-efecto entre múltiples variables sin tener que graficar todo en una sola dimensión.
**Solución:** Al seleccionar un rango en un histograma, los otros histogramas se actualizan instantáneamente para reflejar únicamente el subconjunto de datos condicionados por el primer filtro, permitiendo "esculpir" la estrategia paso a paso.

---

## Narrativa de Producto

**Capacidad:** `CAP-09` — Una interfaz donde se ve el razonamiento

**Historia de usuario:** Como investigador de estrategias, quiero ver varias distribuciones de parámetros a la vez y aislar un rango en una para ver cómo reaccionan las demás al instante, para esculpir una configuración robusta en vez de adivinarla.

**El problema (técnico):** entender la relación entre varias variables de una optimización exige ver más de una distribución a la vez, y ver cada una por separado no revela cómo se condicionan entre sí. Sin coordinación entre las vistas, cada gráfica cuenta una historia aislada que no refleja la relación real entre los parámetros.

**La solución (técnica):** una cuadrícula de histogramas sincronizados, uno por parámetro o métrica. Aislar un rango en cualquiera de ellos actualiza de inmediato las distribuciones de todos los demás, mostrando solo el subconjunto de datos que cumple ese filtro — con un botón para reiniciar y volver al conjunto completo.

**Explicado sin tecnicismos:** imagina una fila de básculas, cada una pesando algo distinto de la misma canasta de frutas. Si aprietas el filtro en una báscula ("solo las manzanas de más de 200 gramos"), las demás básculas se reacomodan al instante para mostrarte el peso de esas mismas frutas en las otras categorías.

Con esta pieza haces lo mismo con los parámetros de una optimización: aíslas un rango en un histograma y ves de inmediato cómo se ven todos los demás para ese mismo subconjunto de estrategias. Vas apretando filtro tras filtro hasta que te queda, a la vista, la combinación exacta que buscabas — sin escribir una sola consulta.

---

## Comportamientos Observables

- [ ] La UI despliega una cuadrícula de histogramas interactivos (uno por parámetro/métrica).
- [ ] El usuario hace clic y arrastra sobre el Histograma 1 para aislar un rango de valores.
- [ ] Instantáneamente, los histogramas del 2 al N cambian sus distribuciones de frecuencia para reflejar los datos condicionados.
- [ ] Ofrece un botón de "Reiniciar filtros" para devolver las distribuciones a su estado inicial.

---

## Restricciones

- **FIJO:** Los datos están vinculados por transiciones atómicas; nunca se permite que un filtro deje la UI en un estado inconsistente.
- **NUNCA** bloquear la interfaz durante la actualización cruzada de histogramas; el cálculo se realiza de forma vectorizada en backend o mediante estructuras eficientes en el cliente.

---

## Parámetros Configurables

| Parámetro | Default | Rango | Qué hace | FIJO/CONFIG |
|---|---|---|---|---|
| HISTOGRAM_BINS | 20 | 5 - 50 | Número de divisiones por histograma. | CONFIG |
| CROSSFILTER_REFRESH_MS | 50 | 10 - 250 | Tiempo de espera entre actualizaciones visuales tras filtrar. | CONFIG |

---

## Estructura Interna (FCIS)

- **Core (Lógica Pura):** Agregación y conteo por buckets sobre datos filtrados dinámicamente mediante DuckDB.
- **Shell (Infraestructura):** Integración con la capa de datos DuckDB para consultas locales rápidas sobre archivos Parquet.
- **Frontera Pública:** Puertos para aplicar filtros cruzados a datasets de optimización.

---

## Ciclo de Vida de la Feature — Cross-Filtering Visualizer

### Entrada
- Matriz completa de resultados de la optimización (Parquet/DuckDB DataFrame).
- Índices de las dimensiones a graficar como histogramas.

### Proceso
- Segmenta los datos de cada dimensión en el número de bins configurado.
- Mantiene una máscara de bits para rastrear los registros que pasan todos los filtros activos.
- Recalcula los conteos de bins para todas las dimensiones basándose en la máscara activa.

### Salida
- Matriz de bins y frecuencias actualizada para el frontend.
- Lista de registros que satisfacen el filtro cruzado actual (`matching_backtests`).

### Contextos de Uso

**Contexto 1: Inspección de Robustez de Optimizaciones (Módulo Validate)**
- Permite al usuario aplicar múltiples filtros secuenciales para encontrar la combinación de parámetros que optimice las métricas de retorno-riesgo.

---

## Tareas (TTRs)

### **TTR-001: Agregación en Bins Sincronizada**
*   **¿Cuál es el problema?** Recalcular todas las distribuciones de frecuencias sobre miles de backtests cada vez que se mueve un filtro puede degradar la experiencia visual.
*   **¿Qué tiene que pasar?** El sistema utiliza consultas optimizadas de DuckDB para evaluar la intersección de filtros en paralelo.
*   **¿Cómo sé que está hecho?**
    - [ ] El 100% de los histogramas se actualizan en menos de 50 ms tras aplicar un brushing en cualquier vista.

### **TTR-002: Exportación de Máscaras y Sub-Datasets**
*   **¿Cuál es el problema?** Después de filtrar, el usuario necesita guardar o enviar ese subconjunto exacto de estrategias a la siguiente fase del pipeline.
*   **¿Qué tiene que pasar?** Al confirmar la selección, la feature genera una lista de identificadores únicos de backtests que pasan el filtro.
*   **¿Cómo sé que está hecho?**
    - [ ] Se emite un array inmutable de IDs para ser inyectados en la base de datos de candidatos promovidos.

---

## Gobernanza y Estándares (Fijos)

- **Local-First (ADR-0016):** 100% Local.
- **Fidelidad:** Alta fidelidad de agregación de datos por bins.
- **Inundación de Fundaciones (ADR-0020):** 
    - Perfil: IA / R&D.
    - **I. Identidad & Integridad:** `id`, `created_at`, `updated_at`, `audit_hash`, `audit_chain_hash`, `event_sequence_id`.
    - **II. Soberanía & Propiedad:** `owner_id`, `institutional_tag`, `manifest_id`.
    - **III. Linaje Alpha & Datos:** `version_node_id`, `logic_hash`, `data_snapshot_id`.
    - **IV. Infraestructura & Ops:** `process_id`, `node_id`.
