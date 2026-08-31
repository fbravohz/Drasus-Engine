# Trade Analysis BI Suite

**Carpeta:** `./features/trade-analysis-bi-suite/`
**Estado:** Lista para implementar
**Última actualización:** 2026-06-06
**Decisión Arquitectónica Asociada:** ADR-0106 (Paradigma de Interfaz de Usuario y Dashboards Visuales de Alta Precisión)

---

## ¿Qué es esta feature?

El `Trade Analysis BI Suite` es una colección integrada de gráficos estadísticos avanzados y cuadros de control analíticos pre-calculados en el backend. Su objetivo es diagnosticar la estructura interna del rendimiento de una estrategia o portafolio. Permite auditar la distribución de las ganancias y pérdidas según temporales físicas (días, horas, meses), la asimetría operativa y la correlación entre el tiempo de retención (duración) y la rentabilidad del trade.

---

## Narrativa de Producto

**Capacidad:** `CAP-08` — El ciclo que se cierra solo

**Historia de usuario:** Como investigador que necesita entender por qué una estrategia gana o pierde, quiero un conjunto de gráficos que desglose el desempeño por hora, día, mes y dirección, para encontrar el patrón detrás del número agregado en vez de quedarme solo con el resultado final.

**El problema (técnico):** una métrica agregada —el Sharpe, la ganancia total— esconde la estructura interna del comportamiento. Una estrategia puede ganar en promedio y aun así perder sistemáticamente los lunes, o solo funcionar del lado corto, o depender de operaciones que se mantienen abiertas más tiempo del que parece razonable. Sin desagregar el resultado por estas dimensiones, esos patrones quedan invisibles hasta que el investigador tropieza con ellos por accidente, si es que lo hace.

**La solución (técnica):** una colección de visualizaciones pre-calculadas en el backend —cascada de ganancias y pérdidas acumuladas, distribución por hora/día/mes, comparación de aciertos por día de la semana, desempeño largo contra corto, y un mapa de dispersión que cruza duración de la operación contra su resultado— con filtrado dinámico por símbolo y rango de fechas. Todo el cálculo estadístico pesado ocurre en el motor nativo; la interfaz solo pinta lo que ya viene resuelto.

**Explicado sin tecnicismos:** imagina que un doctor solo te dijera "tu salud general está bien" sin nunca mostrarte los resultados de cada análisis por separado. No sabrías si el problema está en el colesterol, en la presión o en ningún lado.

Esta pieza es el desglose completo de análisis: te muestra si tu estrategia gana los martes y pierde los viernes, si las operaciones largas son las que sostienen el resultado mientras las cortas lo drenan, o si las posiciones que se quedan abiertas más tiempo son justamente las que más pierden. No te dice qué hacer con esa información — te la entrega desglosada para que tú, o el propio sistema en etapas posteriores, decidan qué corregir.

---

## Comportamientos Observables

- [ ] El usuario interactúa con los siguientes gráficos integrados:
  - **Gráfico de Cascada P/L por Año:** Muestra el efecto acumulativo de las ganancias y pérdidas consecutivas.
  - **Distribución de Retornos por Hora/Día/Mes:** Gráfico de barras que aísla en qué periodos cronológicos la estrategia tiene mejor desempeño.
  - **Win/Loss por Weekday:** Comparación del ratio de operaciones ganadas/perdidas por día de la semana.
  - **Long vs Short P/L:** Visualización segmentada del comportamiento direccional.
  - **Scatter Plot (PnL Growth vs Duration):** Gráfico de dispersión que mapea cada transacción en función de su duración (tiempo abierto) vs su resultado final en dólares o ticks.
- [ ] Incorpora filtrados dinámicos por símbolo o rango temporal en la parte superior.

---

## Restricciones

- **NUNCA** procesar la distribución estadística pesada en el hilo principal de Flutter; todas las matrices y dispersiones se calculan en el Core de Rust vía Polars y se envían optimizadas.
- **NUNCA** graficar puntos individuales de dispersión por encima de 10,000 en el frontend sin aplicar un filtro de muestreo o reducción de resolución (downsampling).

---

## Parámetros Configurables

| Parámetro | Default | Rango | Qué hace | FIJO/CONFIG |
|---|---|---|---|---|
| SCATTER_DOWNSAMPLE_LIMIT | 5000 | 1000 - 20000 | Límite máximo de puntos de transacciones a pintar en el gráfico de dispersión | CONFIG |
| DISTRIBUTION_BIN_SIZE | Hour | Minute/Hour/Day | Agrupación cronológica base para histogramas | CONFIG |

---

## Estructura Interna (FCIS — ADR-0002)

- **Core (Lógica Pura):** Algoritmos de cálculo de distribuciones, agrupaciones temporales de transacciones y cálculo de coordenadas de dispersión.
- **Shell (Infraestructura):** Consultas DuckDB y dataframes de Polars sobre los archivos Parquet de transacciones históricas.
- **Frontera Pública:** Contrato que provee payloads serializados de coordenadas de dispersión y agregados para histogramas.

---

## Ciclo de Vida de la Feature — Trade Analysis BI Suite

### Entrada
- Catálogo de transacciones (trades) con marcas temporales (apertura/cierre) y P&L neto.

### Proceso
- Agrupa los trades por hora del día, día de la semana, mes.
- Mapea la duración de cada trade en segundos y la correlaciona con su P&L neto.
- Aplica downsampling estadístico si supera el límite configurable.

### Salida
- Arrays de visualización para gráficos de cascada, barras y dispersión en Flutter.

---

## Tareas (TTRs)

### **TTR-001: Calculador Estadístico de Transacciones (Rust)**
*   **¿Cuál es el problema?** Procesar relaciones cruzadas como la duración del trade vs rentabilidad en la base de datos local puede introducir latencia al cambiar de dashboard.
*   **¿Qué tiene que pasar?** Implementar funciones analíticas en Rust utilizando Polars para pre-agrupar transacciones en cubetas y calcular correlaciones de Pearson en menos de 10ms.
*   **¿Cómo sé que está hecho?**
    - [ ] Rust entrega el payload estructurado y pre-agrupado listo para ser consumido por la interfaz.

### **TTR-002: Suite de Gráficos BI (Flutter)**
*   **¿Cuál es el problema?** El renderizado de múltiples gráficos complejos (cascada, dispersión de miles de puntos) puede provocar caídas de FPS en la interfaz.
*   **¿Qué tiene que pasar?** Construir widgets personalizados en Flutter optimizados por GPU para pintar el scatter plot y cascadas usando CustomPainter nativo.
*   **¿Cómo sé que está hecho?**
    - [ ] El dashboard carga y permite realizar zoom/pan sobre el scatter plot sin congelamientos visuales.

---

## Gobernanza y Estándares (Fijos)
- **Local-First (ADR-0016):** 100% Local.
- **Inundación de Fundaciones (ADR-0020):**
  - **Perfil D (Ops / Auditoría, persiste):** BI analítico sobre trades, no ingesta de mercado. Aplica el mantra "ante la duda, tenerlo": persiste los reportes de análisis.
  - **I. Identidad & Integridad:** `id`, `created_at`, `updated_at`, `audit_hash`, `audit_chain_hash`, `event_sequence_id`.
  - **II. Soberanía & Propiedad:** `owner_id`, `manifest_id`.
  - **IV. Infraestructura & Ops:** `node_id`, `process_id`.
- **Rastro de Evidencia:** Emite métricas de asimetría direccional al módulo de `feedback`.

---

## Dependencias
- **Depende de:** `/features/duckdb-sql-engine.md`, `/features/equity-curve-tracker.md`
- **Bloquea:** `/modules/validate.md`
