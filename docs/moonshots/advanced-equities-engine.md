# Advanced Equities Engine

**Carpeta:** `./moonshots/`
**Estado:** Moonshot (Fase 5 - Experimental - Prioridad Baja)
**Última actualización:** 2026-06-06
**Decisión Arquitectónica Asociada:** ADR-0030 (Persistencia Soberana "Zero-Docker")

---

## ¿Qué es?

Proporciona soporte completo para la simulación y rebalanceo periódico de portafolios accionarios de alta densidad. Incorpora herramientas de selección de universo (Universe Selection para índices Russell 2000 y S&P 500), gestión de eventos corporativos (splits, dividendos, fusiones y adquisiciones) con ajustes retrospectivos y mitigación del sesgo de supervivencia (survivorship bias) mediante un catálogo histórico de empresas deslistadas y en quiebra.

---

## Comportamientos Observables

- [ ] **Selección de Universo Dinámica:** El usuario configura reglas de selección basadas en capitalización de mercado, volumen transaccionado o pertenencia a índices (Russell 2000 / S&P 500). El motor actualiza el universo de activos en cada intervalo.
- [ ] **Ajuste de Precios por Splits/Dividendos:** Al procesar datos históricos, el motor calcula el impacto de eventos corporativos de forma Point-In-Time (PIT), aplicando ajustes multiplicativos a precios anteriores sin incurrir en fugas de información futura (look-ahead bias).
- [ ] **Simulación de Rebalanceo de Cartera:** Ejecuta rebalanceos de peso periódicos (ej. cada lunes o cada fin de mes), liquidando posiciones y abriendo nuevas según el ranking arrojado por el StockPicker.
- [ ] **Inclusión de Empresas Deslistadas:** Al simular periodos pasados (ej. año 2008), el motor inyecta automáticamente cotizaciones de empresas que quebraron o se deslistaron posteriormente, eliminando el sesgo de optimismo irreal en el rendimiento.

---

## Restricciones

- **OBLIGATORIO:** Mantener la persistencia local de la base de datos de empresas deslistadas y eventos corporativos en archivos Parquet particionados.
- **NUNCA** permitir la inclusión de empresas activas hoy que no existían o no cumplían los criterios de volumen en el momento simulado del pasado.
- **FIJO:** Los ajustes por splits y dividendos deben aplicarse de forma atómica sobre arrays Polars antes de cargarse al matching engine de NautilusTrader.

---

## Parámetros Configurables

| Parámetro | Default | Rango | Qué hace | FIJO/CONFIG |
| :--- | :--- | :--- | :--- | :--- |
| REBALANCE_FREQUENCY | monthly | weekly / monthly / quarterly | Frecuencia de ejecución del StockPicker y rotación de activos | CONFIG |
| SURVIVORSHIP_BIAS_FREE | true | true / false | Habilita o deshabilita la inyección de datos de empresas deslistadas | CONFIG |
| TRANSACTION_TAX_PERCENT | 0.001 | 0.0 - 0.05 | Comisión de corretaje e impuesto a las transacciones por acción | CONFIG |

---

## Estructura Interna (FCIS)

- **Core (Lógica Pura):** Algoritmos de ranking de activos (StockPicker) y fórmulas matemáticas de ajuste de precios retrospectivos.
- **Shell (Infraestructura):** Ingestores de dividendos e históricos de deslistados, persistidos localmente en Hive-Style Parquet y consultados vía DuckDB.

---

## Ciclo de Vida de la Feature — Equities Engine

### Entrada
- Universos accionarios en bruto y registros de eventos corporativos.
- Historial de empresas deslistadas.
- Reglas del StockPicker configuradas por el usuario.

### Proceso
- The Sanitizer filtra y ajusta retrospectivamente la serie temporal ante splits/dividendos detectados en el PIT.
- Evaluación del StockPicker para reordenar y seleccionar las mejores $N$ empresas.
- Generación de órdenes de rotación y rebalanceo para el Autopilot/Simulador.

### Salida
- Portafolio accionario rotado con pesos actualizados.
- Backtest con ajuste institucional libre de sesgo de supervivencia.

---

## Tareas (TTRs)

### **TTR-001: Gestor de Datos Deslistados (Anti-Survivorship Bias)**
*   **¿Cuál es el problema?** Si solo evaluamos nuestro algoritmo en las empresas activas del S&P 500 hoy, el backtest omitirá las compañías que quebraron durante la historia, inflando artificialmente el rendimiento de las estrategias.
*   **¿Qué tiene que pasar?** Diseñar un módulo en Rust que almacene de forma indexada en Parquet el catálogo de empresas deslistadas con sus fechas de cese de operaciones. Al simular, el motor une dinámicamente este catálogo a la consulta de DuckDB para reconstruir el universo exacto disponible en cada fecha del pasado.
*   **¿Cómo sé que está hecho?**
    - [ ] El backtest del año 2001 incluye transacciones de empresas que quebraron en la burbuja dot-com (ej. Enron).
    - [ ] Al desactivar el filtro de deslistados, el número de activos válidos en el universo disminuye significativamente.
*   **¿Qué no puede pasar?**
    - No se deben inyectar cotizaciones de empresas después de su fecha de desliste o quiebra formal.

### **TTR-002: Ajustador Point-In-Time de Eventos Corporativos**
*   **¿Cuál es el problema?** Un split de acciones (ej: 2:1) produce una caída del 50% en el precio nominal del activo de un día a otro, lo cual causaría falsas señales de entrada o detenciones por stop-loss si el precio no se ajusta de forma coherente.
*   **¿Qué tiene que pasar?** Desarrollar un filtro en Polars que aplique factores de ajuste retrospectivos acumulados a los precios de OHLCV únicamente en las fechas anteriores a la fecha del evento corporativo, de forma temporal y aislada para el backtest.
*   **¿Cómo sé que está hecho?**
    - [ ] El gráfico de precios de una acción que sufrió un split no muestra saltos verticales artificiales.
    - [ ] Las órdenes y stop-losses se ejecutan a precios consistentes con los ratios de ajuste calculados.
*   **¿Qué no puede pasar?**
    - No se deben usar factores de ajuste futuros que no hubieran sido conocidos por el trader en la fecha simulada.

---

## Gobernanza y Estándares (ADR-0020 V2)

### Perfil Datos / Ingesta
| Categoría | Campo | Descripción |
| :--- | :--- | :--- |
| **I. Identidad** | `id` | UUID de ejecución de StockPicker |
| | `created_at` | Timestamp de indexación del universo |
| | `audit_hash` | Hash del estado del universo de acciones resultante |
| **III. Linaje Alpha** | `data_snapshot_id` | Identificador de snapshot de eventos corporativos aplicados |
| **IV. Hardware** | `node_id` | ID del hardware local |
| | `process_id` | PID de la tarea de rebalanceo |
| | `execution_latency_ms` | Latencia de cálculo de rankings y factor de splits |
