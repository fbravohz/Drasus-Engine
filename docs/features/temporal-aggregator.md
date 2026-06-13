# Agregador Temporal Custom

**Carpeta:** `./features/temporal-aggregator/`
**Estado:** Especificación
**Última actualización:** 2026-04-12

---

## ¿Qué es?

Es un motor de procesamiento de series temporales encargado de agrupar ticks o barras de alta frecuencia en intervalos de tiempo arbitrarios y no estándar (ej: 17 minutos, 3 horas 21 minutos, 45 segundos). 

A diferencia de los agregadores estándar que se limitan a potencias de tiempo fijas (1m, 5m, 1h), esta feature permite a los investigadores explorar periodicidades "ruidosas" o específicas que pueden revelar patrones invisibles en marcos temporales convencionales.

---

## Comportamientos Observables

- [ ] Genera barras OHLCV a partir de ticks usando un intervalo de tiempo configurable en segundos/minutos.
- [ ] Soporta alineación de sesión (ej: "comenzar a contar desde la apertura de NY").
- [ ] Permite "Rolling Aggregation" (ventanas deslizantes de tiempo).
- [ ] El sistema detecta y rellena huecos (gaps) según la política configurada (omitir o repetir último precio).

---

## Restricciones

- **DUCKDB-POWERED (ADR-0036):** El remuestreo se realiza mediante consultas SQL vectorizadas directamente sobre archivos Parquet en disco, eliminando la necesidad de persistir múltiples temporalidades físicamente.
- **ALINEACIÓN ATÓMICA:** Las barras siempre deben alinearse con el inicio del día o de la sesión para garantizar reproducibilidad.
- **DETERMINISMO:** El timestamp de cierre de la barra agregada debe ser predecible y consistente entre backtest y vivo.
- **REGLA DE MÚLTIPLES:** El sistema solo permite remuestreo ascendente (ej: de 1m a 17m). El remuestreo descendente (ej: de 1h a 1m) está prohibido por falta de granularidad.

---

## Parámetros Configurables

| Parámetro | Default | Qué hace |
|---|---|---|
| AGGREGATION_INTERVAL | configurable | El intervalo de tiempo deseado (en segundos o minutos) |
| ALIGNMENT_ANCHOR | midnight | Punto de referencia para iniciar el conteo (medianoche, apertura, etc.) |
| GAP_FILL_POLICY | skip | Qué hacer si no hay ticks en un intervalo (skip, forward_fill) |

---

## Ciclo de Vida de la Feature — Temporal Aggregator

### Entrada
- Flujo de datos fuente (Ticks o Barras de 1 minuto).
- Definición del intervalo (`AGGREGATION_INTERVAL`).
- Reglas de alineación de sesión.

### Proceso
1. **Bucketing:** Agrupa los datos fuente en "cubetas" temporales basadas en el ancla definida.
2. **OHLCV Synthesis:** Calcula Open (primer precio), High (máximo), Low (mínimo), Close (último) y Volumen (suma) para cada cubeta.
3. **Validation:** Verifica integridad de cada barra resultante.

### Salida
- Serie temporal de barras OHLCV en la periodicidad solicitada.
- Metadatos de la agregación (frecuencia fuente, gaps detectados).

### Contextos de Uso

**Contexto 1: Ingesta de Datos (Ingest)**
- Entrada: Ticks en tiempo real.
- Uso: Crea el dataset primario para una estrategia que opera en periodicidades raras.

**Contexto 2: Validación y Backtesting (Validate)**
- Entrada: Barras de 1m históricas de larga duración.
- Uso: Sintetiza barras de mayor temporalidad para reducir el ruido en las pruebas de robustez.

---

---

## Tareas (TTRs)

### **TTR-001: Motor de Bucketizado Temporal (Time-Binned Aggregation)**
*   **Descripción:** Asigna cada tick o barra fuente a su intervalo correspondiente basado en el `AGGREGATION_INTERVAL`.
*   **Reglas de Negocio:**
    * El precio debe manejarse como `int64` (centavos/ticks) para evitar drift acumulado (ADR-0002).
    * El ancla de alineación debe ser inmutable durante la sesión para evitar barras "saltarinas".
*   **Entrada:** `source_data` (ticks/1m bars), `interval_seconds`, `anchor_point`.
*   **Salida:** `aggregated_ohlcv` (Arrow/DataFrame).
*   **Precondición:** Datos fuente validados por `data-validator`.
*   **Postcondición:** Emisión de `audit_hash` del resultado final para reconciliación (ADR-0020 V2).

### **TTR-002: Lógica de Continuidad y Relleno de Gaps**
*   **Descripción:** Gestiona periodos sin actividad de mercado según la `GAP_FILL_POLICY`.
*   **Reglas de Negocio:**
    * Si la política es `forward_fill`, la barra resultante debe marcarse con `is_synthetic=True`.
    * Toda barra sintética debe incluir el `process_id` del job generador (ADR-0020 V2).
*   **Entrada:** `aggregated_ohlcv`, `gap_policy`.
*   **Salida:** `continuous_ohlcv_stream`.
*   **Precondición:** TTR-001 finalizado.
*   **Postcondición:** Persistencia con `institutional_tag` y `version_node_id` vinculado al feed.

---

## Persistencia (Inundación de Fundamentos — ADR-0020 V2)

Cada barra temporal regenerada registra el set de relevancia técnica para Datos:

| Categoría | Campo | Descripción |
| :--- | :--- | :--- |
| **I. Identidad** | `id` | Identificador único de la barra |
| | `created_at` | Timestamp de agregación |
| | `updated_at` | Timestamp de última modificación del registro |
| | `audit_hash` | Hash del resultado OHLCV |
| | `audit_chain_hash` | Hash de integridad del bloque |
| | `event_sequence_id` | Secuencia de recuperación (event-sourcing) |
| **III. Linaje** | `data_snapshot_id` | Ref al precio fuente (Ticks/1m) |
| | `transformation_id` | ID de la temporalidad (Custom TF) |
| | `logic_hash` | Hash del motor agregador |
| **IV. Hardware** | `node_id` | ID del hardware físico |
| | `process_id` | PID del worker de agregación |

- **Decisión Arquitectónica Asociada:**
    - ADR-0002: Desacoplamiento de Persistencia (Precios enteros).
    - ADR-0013: Stack Tecnológico (NautilusTrader ready).
    - ADR-0020 V2: Inundación de Fundaciones.

---

## Dependencias
**Depende de:**
- [`data-validator`](../features/data-validator.md) — para limpieza de la fuente.

**Consumido por:**
- [`ingest`](../modules/ingest.md) — para la síntesis de periodicidades arbitrarias.
