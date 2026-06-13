# Barras Algorítmicas (No-Temporales)

**Carpeta:** `./features/algorithmic-bars/`
**Estado:** Especificación
**Última actualización:** 2026-04-12

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
*   **Postcondición:** Persistencia con `institutional_tag` y `process_id` (ADR-0020 V2).

---

## Persistencia (Inundación de Fundamentos — ADR-0020 V2)

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
