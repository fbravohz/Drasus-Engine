# Monthly Performance Heatmap

**Carpeta:** `./features/monthly-performance-heatmap/`
**Estado:** Lista para implementar
**Última actualización:** 2026-06-06
**Decisión Arquitectónica Asociada:** ADR-0106 (Paradigma de Interfaz de Usuario y Dashboards Visuales de Alta Precisión)

---

## ¿Qué es esta feature?

El `Monthly Performance Heatmap` es un componente visual analítico que muestra el rendimiento porcentual mensual de una estrategia o portafolio en una matriz interactiva de Años × Meses. Incluye la sumatoria anual del rendimiento acumulado (YTD - Year-To-Date) y permite la segmentación dinámica instantánea por dirección (Solo Largos, Solo Cortos) y tipo de datos de validación (Muestra Completa, In-Sample, Out-Of-Sample).

---

## Narrativa de Producto

**Capacidad:** `CAP-08` — El ciclo que se cierra solo

**Historia de usuario:** Como operador que revisa el historial de una estrategia o portafolio, quiero ver de un vistazo qué meses y qué años fueron buenos y cuáles fueron malos, para identificar patrones de temporada o rachas sin tener que leer una tabla de números mes por mes.

**El problema (técnico):** una curva de capital muestra la tendencia general, pero esconde la estructura mensual: si el resultado del año lo sostuvo un solo mes excepcional, si hay una temporada recurrente de pérdidas, o si el desempeño cambia según se mire solo el lado largo o solo el corto. Leer esa estructura en una tabla de números obliga a hacer el trabajo mental de comparar celda por celda.

**La solución (técnica):** una matriz de años por meses donde cada celda se colorea según la intensidad y dirección del retorno de ese mes, con una columna de acumulado anual, y selectores para aislar solo muestra dentro o fuera de validación, y solo posiciones largas o cortas. Al pasar el cursor sobre una celda aparece el detalle exacto: retorno neto, porcentaje y número de operaciones. Todo el cálculo ocurre en el motor nativo; la interfaz solo colorea lo que ya viene resuelto.

**Explicado sin tecnicismos:** es como el calendario de calificaciones de la escuela, pero para el dinero: cada mes es una casilla, y el color te dice de inmediato si fue un buen mes o uno malo, sin que tengas que sumar nada tú mismo.

De un vistazo ves si diciembre siempre es flojo, si el año pasado lo salvó un solo mes bueno, o si la estrategia se porta distinto cuando solo compra contra cuando solo vende. Es la diferencia entre leer una lista larga de números y ver el patrón completo pintado en una sola imagen.

---

## Comportamientos Observables

- [ ] El usuario visualiza una tabla matricial donde cada fila es un año de datos y cada columna es un mes del año, más una columna final con el acumulado YTD.
- [ ] Las celdas se colorean cromáticamente: azul oscuro o verde intenso para ganancias récord, rojo profundo para pérdidas máximas, y colores neutros para resultados cercanos a cero.
- [ ] Incorpora selectores interactivos superiores para conmutar la vista:
  - **Muestra:** `Full` / `In-Sample` / `Out-Of-Sample`.
  - **Dirección:** `All` / `Long Only` / `Short Only`.
- [ ] Al pasar el cursor por encima de una celda, muestra el retorno neto en dólares, porcentaje de ganancia y cantidad de trades ejecutados en ese mes.

---

## Restricciones

- **NUNCA** calcular los agregados matemáticos mensuales en Dart; el cálculo se realiza en Rust utilizando Polars sobre el catálogo de transacciones (trades) y se transmite como un payload estructurado JSON/Arrow.
- **NUNCA** permitir la alteración del ledger histórico para maquillar el mapa de rendimiento; los datos son de solo lectura.

---

## Parámetros Configurables

| Parámetro | Default | Rango | Qué hace | FIJO/CONFIG |
|---|---|---|---|---|
| COLOR_SCHEME_PALETTE | ClassicGreenRed | Classic/BlueRed/HSL-Tailored | Paleta de colores para la escala cromática del heatmap | CONFIG |
| RETRUN_CALCULATION_METHOD | Compounded | Simple/Compounded | Método para calcular el retorno acumulado mensual | [FIJO] |

---

## Estructura Interna (FCIS — ADR-0002)

- **Core (Lógica Pura):** Algoritmos de agregación cronológica de rentabilidades mensuales y cálculo de YTD compuesto.
- **Shell (Infraestructura):** Consultor DuckDB sobre la tabla de transacciones persistida en Parquet o SQLite.
- **Frontera Pública:** Contrato de consulta que acepta filtros de muestra y dirección, y retorna la matriz estructurada.

---

## Ciclo de Vida de la Feature — Monthly Performance Heatmap

### Entrada
- Lista de transacciones (trades) con timestamps de cierre, P&L neto y volumen.
- Segmentos temporales de validación (fechas IS / OOS).

### Proceso
- Agrupa y acumula los retornos por año y mes.
- Aplica el filtro dinámico de dirección del trade.
- Calcula el acumulado anual YTD.

### Salida
- Matriz serializada de rendimiento mensual y anual con metadatos.

---

## Tareas (TTRs)

### **TTR-001: Agregador de Rendimiento Cronológico (Rust)**
*   **¿Cuál es el problema?** Computar la rentabilidad histórica mensual compuesta en tiempo real puede ralentizar la UI si hay decenas de miles de trades.
*   **¿Qué tiene que pasar?** Implementar consultas agregadas en DuckDB/Polars optimizadas para procesar el catálogo de transacciones filtrando por dirección y muestra.
*   **¿Cómo sé que está hecho?**
    - [ ] La consulta retorna en menos de 5ms la matriz formateada por año/mes para 50,000 transacciones.

### **TTR-002: Matriz Cromática Dinámica (Flutter)**
*   **¿Cuál es el problema?** Representar visualmente el rendimiento mensual de forma intuitiva requiere una escala cromática flexible sin saturación.
*   **¿Qué tiene que pasar?** Desarrollar el widget de tabla en Flutter que asigne dinámicamente colores de fondo según la intensidad y dirección del retorno porcentual.
*   **¿Cómo sé que está hecho?**
    - [ ] El widget renderiza la matriz y actualiza visualmente al cambiar las segmentaciones dinámicas de dirección o muestra.

---

## Gobernanza y Estándares (Fijos)
- **Local-First (ADR-0016):** 100% Local.
- **Inundación de Fundaciones (ADR-0020):** Perfil Datos / Ingest. Registra `id`, `created_at`, `updated_at`, `audit_hash`, `audit_chain_hash`, `event_sequence_id` (Grupo I universal).
- **Rastro de Evidencia:** Emite retornos mensuales consolidados para el módulo de `feedback`.

---

## Dependencias
- **Depende de:** `/features/equity-curve-tracker.md`, `/features/duckdb-sql-engine.md`
- **Bloquea:** `/modules/validate.md`
