# Data Import Wizard

**Carpeta:** `./features/data-import-wizard/`
**Estado:** En Diseño
**Última actualización:** 2026-04-13
**Decisión Arquitectónica Asociada:** ADR-0008 (Configurabilidad Universal) · ADR-0154 (vía 3 de ingesta: exportación manual desde el terminal del usuario) · ADR-0155 (esquema canónico de campos — resuelve el pendiente citado abajo)

## ¿Qué es?

Es el componente que permite al usuario incorporar datos externos (EJ: CSV de MetaTrader, TXT de NinjaTrader) al sistema Drasus Engine. Automatiza la detección de columnas y el mapeo de campos, traduciendo cualquier formato externo al estándar institucional del "Ingest".

> **Rol arquitectónico (ADR-0154):** esta feature materializa la **tercera vía de ingesta** — la única en la que Drasus no hace red. El usuario exporta desde su propio terminal y entrega el archivo; Drasus lee, valida, normaliza y particiona. Es la vía que garantiza el precio del bróker de ejecución real, con compra y venta, sin depender de que ese bróker publique volcado alguno.

**Problema:** Cada broker exporta CSVs con nombres de columnas y formatos de fecha distintos. Mapearlos a mano es lento y propenso a errores.
**Solución:** Un asistente inteligente que detecta automáticamente qué columna es el precio, cuál es la fecha y qué formato tiene.

## Comportamientos Observables

- [ ] El usuario arrastra un archivo CSV; el sistema previsualiza las primeras 10 filas.
- [ ] El sistema propone un mapeo automático (Columna 1 -> Timestamp, Columna 2 -> Open, etc.).
- [ ] El usuario confirma el mapeo y el sistema valida el tipo de datos antes de iniciar la importación.
- [ ] El sistema detecta automáticamente el delimitador (coma, punto y coma, tabulación).

## Restricciones

- NUNCA se descarta un campo que el archivo de origen entregue. El esquema de importación es **abierto por arriba**: marca de tiempo y precio son el mínimo exigido, pero todo lo demás que traiga la fuente —compra, venta, volúmenes por lado, banderas de sesión, identificadores de orden— se conserva y se persiste (ADR-0154, ADR-0020). Reducir la importación a "OHLCV + Timestamp" está **prohibido**: es lo que convierte el spread simulado en un parámetro inventado en vez de un dato observado.
  > ✅ **Pendiente resuelto — ADR-0155 (2026-07-18):** el esquema canónico de campos de la operación y de la vela ya existe (`QuoteTick`/`TradeTick`/`Bars`/`DepthLevel`/`OrderEvent`). Un CSV de MT5 que mezcla cotización y ejecución en las mismas columnas (`Bid, Ask, Last, Volumen`) se importa crudo aquí tal cual llega; el demultiplexado a `QuoteTick`/`TradeTick` es responsabilidad de [`data-normalization-layer`](data-normalization-layer.md) aguas abajo, no de este wizard. El mapeo de columnas del asistente debe reconocer los nombres canónicos de ADR-0155 (`bid`/`ask`/`price`/`trade_volume`/`aggressor_side`/`spread_reported`/`volume_traded`/`volume_ticks`/`trade_count`/`volume_buy`) además de los ya cubiertos (OHLCV/Timestamp).
- NUNCA se sobreescriben datos existentes sin una confirmación explícita del usuario.
- El proceso de importación masiva debe ejecutarse en segundo plano (vía Download Manager) para no bloquear la UI.

## Parámetros Configurables

| Parámetro | Default | Rango | Qué hace | FIJO/CONFIG |
|---|---|---|---|---|
| AUTO_DETECT_COLUMNS | True | True/False | Intentar mapear columnas por nombre | CONFIG |
| PREVIEW_ROWS | 10 | 5 - 100 | Cuántas filas mostrar en la previsualización | CONFIG |
| STRICT_DATE_PARSING | True | True/False | Falla si una sola fecha no cumple el formato | CONFIG |

## Ciclo de Vida de la Feature — Data Import Wizard

### Entrada
- Archivo plano (CSV, TXT, TSV).
- Metadatos manuales (Exchange, Symbol).

### Proceso
- Análisis estadístico del contenido para detectar tipos de datos.
- Mapeo de columnas a la estructura interna.
- Conversión via Polars para máxima velocidad de ingestión.
- Sanitización inmediata mediante el `Data Sanitizer Pipeline`.

### Salida
- Datos integrados en la estructura Hive-Style Parquet.
- Reporte de filas importadas vs rechazadas.

### Contextos de Uso

**Contexto 1: Migración de Datos**
- Traer históricos desde otras plataformas al laboratorio de Drasus Engine.

## Tareas (TTRs)

### **TTR-001: Motor de Inferencia de Esquema**
- Implementa la lógica que analiza el encabezado y las primeras filas para adivinar el formato de fecha y el rol de cada columna.

### **TTR-002: Ingestor Masivo Polars-CSV**
- Desarrolla el puente que convierte el archivo plano a Parquet particionado usando el motor de Polars.

## Persistencia (Inundación de Fundamentos — ADR-0020)

Cada importación manual registra el set de relevancia técnica:

| Categoría | Campo | Descripción |
| :--- | :--- | :--- |
| **I. Identidad** | `id` | Identificador único de la importación |
| | `created_at` | Timestamp de inicio |
| | `updated_at` | Timestamp de última modificación del registro |
| | `audit_hash` | Hash del archivo crudo importado |
| | `audit_chain_hash` | Hash de la secuencia de asimilación |
| | `event_sequence_id` | Secuencia de recuperación (event-sourcing) |
| **III. Linaje** | `data_snapshot_id` | Nombre/Ruta del archivo original |
| | `logic_hash` | Hash del parser utilizado |
| **IV. Hardware** | `node_id` | ID del hardware físico de importación |
| | `process_id` | PID del worker de parseo |
