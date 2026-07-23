# Barras Algorítmicas (No-Temporales)

**Carpeta:** `./features/algorithmic-bars/`
**Estado:** Especificación
**Última actualización:** 2026-07-19 (TASK-062 — catálogo cerrado)
**Decisión Arquitectónica Asociada:** ADR-0155 (esquema `Bars` + `CHECK` de `bar_type`)

---

## Catálogo cerrado de `bar_type` (TASK-062, 2026-07-19)

Único sitio canónico del catálogo; `ADR-0155` cita este documento y su `CHECK` debe coincidir exactamente con la columna "Tipo" de esta tabla — si alguna vez difieren, corrige el `CHECK`, no este documento.

| `bar_type` | Regla de cierre | Estado |
|---|---|---|
| `TIME` | Intervalo de tiempo fijo | Construido (`hybrid-data-transformer`, STORY-051) |
| `TICK_COUNT` | Cierra cada N transacciones | Especificado, pendiente de implementación (esta feature) |
| `VOLUME` | Cierra al alcanzar un umbral de volumen acumulado | Especificado, pendiente de implementación |
| `RENKO` | Cierra al moverse un tamaño de ladrillo fijo | Especificado, pendiente de implementación |
| `RANGE` | Cierra cuando el rango High-Low alcanza un valor fijo | Especificado, pendiente de implementación |
| `DOLLAR` | Cierra al cruzar un umbral de valor nocional acumulado (precio × volumen) — barra de dólar, López de Prado (*Advances in Financial Machine Learning*, 2018, cap. 2), más robusta que `VOLUME`/`TICK_COUNT` frente a cambios estructurales de participación de mercado | Añadida al catálogo (faltaba en este documento pese a estar ya en el `CHECK` de ADR-0155 desde 2026-07-18). **Implementación diferida hasta que TASK-061 (ensanche `i128`) esté disponible** — `DOLLAR` acumula precio×volumen toda la sesión, es el tipo que más sufre el techo de magnitud que TASK-061 resuelve |

**Criterio de admisión al catálogo (para futuros candidatos, sin volver a discutirlo cada vez):** un tipo entra a `bar_type` si cumple las tres — (a) es una regla de **cierre** computable causalmente desde el flujo de ticks, sin usar información posterior al cierre de la propia barra; (b) su salida encaja en el esquema `Bars` (`open/high/low/close` + metadatos) sin inventar una forma nueva; (c) respaldo verificable — literatura cuantitativa reconocida (López de Prado) o adopción de plataforma mayor confirmada por fuente (NinjaTrader, TradingView). Cualquier transformación sobre valores de barras **ya cerradas** no cumple (a) — no es `bar_type`, es indicador (ver abajo).

**Heikin Ashi NO es un tipo de barra — es un indicador.** Fórmula recursiva sobre OHLC de barras ya cerradas por otra regla (`HA_close = (O+H+L+C)/4`, etc.); no decide *cuándo* cierra una barra, repinta valores de una que ya cerró. Misma doctrina que sacó el precio medio ponderado de la ingesta (DEBT-028): materia prima vs. indicador. Pertenece al motor de indicadores (EPIC-2/EPIC-3), que no existe todavía — no se registra una feature nueva para esto ahora, mismo tratamiento que dieron los puntos 2/3 de DEBT-028 a su propio indicador. **Advertencia para cuando se construya:** Heikin Ashi es recursivo (necesita semilla/warm-up) y su `close` es un promedio — nunca un precio negociado. Ningún simulador debe usar precios Heikin Ashi como fills reales (violaría el Contrato de Consistencia Conservadora, ADR-0114).

**Kagi, Point & Figure y Line Break — NO admitidos, decisión diferida.** Confirmado que NinjaTrader 8 los ofrece, junto con "Order Flow Volumetric/Delta Bars" (que NO es un tipo nuevo: es una vista sobre `DepthLevel`/`OrderEvent`, ya canonizados en ADR-0155 — no requiere entrada en `bar_type`). Los tres primeros no encajan en el esquema OHLC de `Bars` — su salida nativa es líneas/cajas sin eje de tiempo uniforme (fallan el criterio de admisión (b) de arriba). Admitirlos exige decidir primero su tipo de dato de salida (¿aproximar a OHLC perdiendo la esencia del método, o un tipo nuevo distinto de `Bars`?) — **sin caso de uso concreto que lo demande hoy, no se inventa esa forma por adelantado** (Inundación de Fundaciones aplica a campos dentro de un perfil ya decidido, no a arquitecturas de dato completas sin consumidor). **Disparador para reabrir esta decisión:** el propietario declara necesidad concreta de un estilo de gráfico discrecional visual, o una Story de EPIC-8 (Canvas/Pulido) los requiere para paridad de UX con NinjaTrader.

**Barras de desequilibrio y de "run" (López de Prado, cap. 2.3.2–2.4) — candidatas para el ROADMAP, no para esta especificación.** Mismo esquema `Bars`, pero con umbral **adaptativo** (EWMA de barras previas) — mejores propiedades estadísticas que las cuatro simples. Riesgo de implementación a vigilar cuando se construyan: el umbral de la barra N debe derivarse solo de `1..N-1`, nunca de la barra en curso (look-ahead bias). No se nombran por el propietario ni tienen EPIC asignado — quedan anotadas aquí para que no se pierdan, sin comprometer secuenciación.

**Advertencia de sesgo estadístico por tipo de barra — para cualquier feature futura de validación (WFA/CPCV/Monte Carlo/DSR) que permita elegir la fuente de barras (EPIC-4):**
- `TIME` (el único construido hoy): sobremuestrea baja actividad y submuestrea ráfagas — retornos heterocedásticos, no gaussianos, con autocorrelación serial inducida. Es la peor opción estadísticamente estudiada, y hoy es la única fuente de datos del guantelete.
- `TICK_COUNT`/`VOLUME`: mejoran sobre `TIME`, pero no son estacionarios en horizontes largos ante el crecimiento estructural del trading algorítmico.
- `DOLLAR`: la más robusta a cambio estructural de las cuatro simples — mejor default para backtests de horizonte largo.
- `RENKO`/`RANGE`: filtran ruido visual, no buscan robustez estadística — inducen autocorrelación artificial en los movimientos que sí registran. Si el guantelete corre alguna vez sobre estas series, señalar explícitamente en la UI que los supuestos de independencia de CPCV/WFA/DSR son más propensos a violarse.

---

## ¿Qué es?

Es un procesador de datos de mercado que transforma el flujo de ticks en barras basadas en eventos de precio o volumen, en lugar de intervalos de tiempo fijos. 

Permite capturar la estructura micro del mercado que se pierde en las barras estándar (1m, 5m), eliminando el ruido durante periodos de baja actividad y enfocándose en la volatilidad real.

---

## Comportamientos Observables

- [ ] Genera barras **Renko** basadas en un tamaño de ladrillo (brick size) fijo.
- [ ] Genera barras de **Rango (Range Bars)** donde cada barra tiene un High-Low igual al rango definido.
- [ ] Genera barras de **Volumen (Volume Bars)** que cierran cuando se alcanza un umbral de volumen acumulado.
- [ ] Genera barras de **Tick (Tick Bars)** que cierran cada N transacciones.
- [ ] Genera barras de **Dólar (Dollar Bars)** que cierran al cruzar un umbral de valor nocional acumulado (precio × volumen) — ver catálogo cerrado abajo, implementación diferida a TASK-061.

---

## Restricciones

- **ALTA FIDELIDAD:** El generador siempre debe usar Ticks como fuente primaria, nunca reconstruir desde barras de tiempo.
- **DETERMINISMO:** El cierre de una barra algorítmica debe ser idéntico en backtest y en tiempo real para evitar sesgos de ejecución.
- **PERFORMANCE (ADR-0019):** Las barras generadas deben emitirse en formato **Apache Arrow** para permitir transferencia zero-copy a NautilusTrader.

---

## Parámetros Configurables

| Parámetro | Default | Qué hace |
|---|---|---|
| BRICK_SIZE | configurable | Tamaño del movimiento de precio para barras Renko |
| RANGE_PIPS | configurable | Rango mínimo para cerrar una Range Bar |
| VOLUME_THRESHOLD | configurable | Volumen necesario para cerrar una Volume Bar |
| TICK_COUNT | configurable | Número de transacciones por Tick Bar |
| DOLLAR_THRESHOLD | configurable | Valor nocional (precio × volumen) necesario para cerrar una Dollar Bar |

---

## Ciclo de Vida
*   **Entrada:** `ingest` (tick stream crudo de brokers/Nautilus).
*   **Proceso:** Acumulación por umbral (Precio/Volumen/Ticks) → Cierre de Barra → Normalización OHLCV.
*   **Salida:** Stream de barras algorítmicas en formato **Apache Arrow**, rastro de secuencia.

---

## Tareas (TTRs)

### **TTR-001: Implementar Agregador Vectorizado (Rust SIMD-Ready)**
*   **Descripción:** Lógica de agrupación de ticks que minimice el uso de loops; compatible con Rust SIMD/Rayon para performance sub-milisegundo.
*   **Reglas de Negocio:**
    * Los precios deben manejarse como `int64` (centavos/ticks) para evitar errores de coma flotante (ADR-0002).
    * Toda barra generada DEBE incluir el `data_provenance_hash` del stream de ticks original.
*   **Entrada:** `tick_stream` (Polars/Arrow), `bar_config` (Renko/Range/Volume).
*   **Salida:** `algo_bars` (DataFrame con OHLCV + Metadata).
*   **Precondición:** Stream de ticks libre de gaps y ordenado por `ntp_timestamp`.
*   **Postcondición:** Emisión de `audit_hash` para la secuencia de barras generadas.

### **TTR-002: Sincronización Causales y Time-Travel Debugging**
*   **Descripción:** Asegura que barras de múltiples símbolos mantengan la causalidad temporal estricta.
*   **Reglas de Negocio:**
    * El timestamp de cierre de la barra algorítmica DEBE ser el del último tick que activó el cierre.
    * Los metadatos deben incluir `tick_count` y `duration_ms` para análisis de liquidez posterior.
*   **Entrada:** `multiple_symbol_streams`.
*   **Salida:** `synchronized_algo_stream`.
*   **Precondición:** Reloj del sistema sincronizado vía NTP (ADR-0013).
*   **Postcondición:** Persistencia con `institutional_tag` y `process_id` (ADR-0020).

---

## Persistencia (Inundación de Fundamentos — ADR-0020)

Toda barra algorítmica generada registra el set de relevancia técnica para Datos:

| Categoría | Campo | Descripción |
| :--- | :--- | :--- |
| **I. Identidad** | `id` | Identificador único de la barra |
| | `created_at` | Timestamp de cierre (nanosegundos) |
| | `updated_at` | Timestamp de última modificación del registro |
| | `audit_hash` | Hash de integridad de la barra OHLCV |
| | `audit_chain_hash` | Hash de la secuencia de barras |
| | `event_sequence_id` | Secuencia de recuperación (event-sourcing) |
| **III. Linaje** | `data_snapshot_id` | Ref al stream de ticks origen |
| | `transformation_id` | ID del tipo de barra (Renko/Range) |
| | `logic_hash` | Hash del motor de construcción |
| **IV. Hardware** | `node_id` | ID del hardware físico |
| | `process_id` | PID del proceso de construcción |
| | `execution_latency_ms` | Tiempo de construcción de la barra |

## Gobernanza y Estándares (Fijos)

---

## Dependencias
**Depende de:**
- [`infrastructure-setup`](../features/infrastructure-setup.md) — para almacenamiento OLAP (Parquet/DuckDB).

**Consumido por:**
- [`ingest`](../modules/ingest.md) — para la generación de feeds no-temporales.
