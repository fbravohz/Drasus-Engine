# Sovereign Data Fetcher

**Carpeta:** `./features/sovereign-data-fetcher/`
**Estado:** En Diseño
**Última actualización:** 2026-04-13
**Decisión Arquitectónica Asociada:** ADR-0034 (Ingesta Híbrida Soberana)

## ¿Qué es?

Es el componente encargado de saturar el ancho de banda para la obtención masiva de históricos. Resuelve el problema de la lentitud de las APIs REST (que son 100x más lentas) mediante una estrategia híbrida: descarga archivos comprimidos masivos (Bulk) y usa la API solo para el "Delta" final (datos recientes).

**Problema:** Descargar 5 años de datos por API REST puede tomar días y causar bloqueos por Rate Limit.
**Solución:** Descargar volcados mensuales de S3 en segundos y rellenar los últimos minutos vía API.

## Comportamientos Observables

- [ ] Usuario solicita histórico de BTC de 2020 a hoy.
  - El sistema identifica volcados en `data.binance.vision`.
  - Descarga archivos `.zip` concurrentemente usando todos los hilos disponibles.
  - Al terminar, conecta con la API REST para descargar las barras que faltan desde el último volcado hasta el "ahora".
- [ ] La interfaz muestra una barra de progreso indicando "Descargando Bulk (80%)" y luego "Sincronizando Delta (100%)".
- [ ] Si un archivo Bulk falla, el sistema intenta descargarlo de nuevo automáticamente.

## Restricciones

- NUNCA se usa la API REST para periodos que ya existen en volcados Bulk.
- NUNCA se inicia la ingesta si el espacio en disco es insuficiente para el tamaño estimado del Bulk.
- La descarga debe ser asíncrona y no bloquear el hilo principal de la aplicación.

## Parámetros Configurables

| Parámetro | Default | Rango | Qué hace | FIJO/CONFIG |
|---|---|---|---|---|
| CONCURRENT_DOWNLOADS | 5 | 1 - 20 | Cuántos archivos descargar simultáneamente | CONFIG |
| BULK_SOURCE_URL | Binance Vision | - | URL base para buscar volcados S3 | [FIJO] |
| DELTA_SYNC_RETRY | 3 | 1 - 10 | Reintentos para la sincronización REST | CONFIG |

## Estructura Interna (FCIS — ADR-0002)

- **Core (Lógica Pura):** Algoritmos de reconciliación de timestamps, detección de huecos (Gaps) y priorización de descargas en `fetcher_core.rs`.
- **Shell (Infraestructura):** Cliente HTTP asíncrono, descompresor de archivos y gestión de sistema de archivos local.
- **Frontera Pública:** Contrato `fetch_data(symbol, timeframe, range)`.

## Ciclo de Vida de la Feature — Sovereign Data Fetcher

### Entrada
- Símbolo (ej: BTCUSDT), Intervalo (1m), Rango de fechas.
- Credenciales de API (solo para Delta).

### Proceso
- Consulta el inventario de archivos Bulk en el servidor remoto.
- Descarga y descomprime archivos en segundo plano.
- Identifica el punto de corte (último timestamp del Bulk).
- Solicita formalmente el Delta a la API REST del broker.

### Salida
- Stream de datos crudos (CSV/JSON) listos para la capa de normalización.
- Reporte de éxito/fallo por cada bloque temporal.

### Contextos de Uso

**Contexto 1: Ingesta Inicial (Hydro-Ingest)**
- El sistema descarga años de historia para alimentar la generación de alfas (Ingest).

**Contexto 2: Reconexión Live**
- Si el sistema se apaga 2 horas, el fetcher usa la API Delta para rellenar el hueco sin intervención humana.

## Tareas (TTRs)

### **TTR-001: Descargador Asíncrono de Bulk (S3)**
- Implementa la lógica de descarga concurrente de archivos comprimidos optimizada para ancho de banda alto.

### **TTR-002: Reconciliador de Delta (REST)**
- Implementa la conexión con la API REST para descargar el segmento de datos faltante entre el Bulk y el presente.

### **TTR-003: Alternative Data Webhook Listener**
- **Qué tiene que pasar:** Implementar un receptor local de HTTP Webhooks en Rust que exponga un puerto seguro. Permite a plataformas como n8n y Zapier inyectar datos alternativos (ej: puntajes de sentimiento, feeds de noticias de impacto) como señales en tiempo real para el generador y motor de ejecución.

### **TTR-004: Alternative Time-Series Converter (Backtestable Data)**
- **¿Cuál es el problema?** Los datos alternativos asíncronos (sentimiento de mercado, noticias fundamentales, análisis macroeconómicos) no son útiles para investigar si no se estructuran históricamente, impidiendo su backtesting.
- **¿Qué tiene que pasar?** Implementar un alineador en Rust que normalice, indexe y asocie eventos asíncronos alternativos a las marcas de tiempo Point-in-Time (PIT) de las velas de mercado en los archivos Parquet locales (Hive-Style), haciéndolos completamente backtesteables sin sesgo de look-ahead.
- **¿Cómo sé que está hecho?**
  - [ ] Puedo correr un backtest que cargue columnas de sentimiento histórico y verificar que el motor reaccione con precisión determinista a eventos pasados.

---

## Gobernanza y Estándares (Fijos)

- **Local-First (ADR-0016):** 100% Local (los datos se descargan y procesan en el disco del usuario).
- **Fidelidad (ADR-0017):** Alta (maneja Ticks y Barras de 1M).
## Persistencia (Inundación de Fundamentos — ADR-0020 V2)

Cada descarga registra el set de relevancia técnica para Datos:

| Categoría | Campo | Descripción |
| :--- | :--- | :--- |
| **I. Identidad** | `id` | Identificador único del job de descarga |
| | `created_at` | Timestamp de inicio |
| | `audit_hash` | Hash de integridad del archivo comprimido |
| | `audit_chain_hash` | Hash de la secuencia de descarga |
| **II. Linaje** | `source_id` | URL/Endpoint de la fuente Bulk/REST |
| | `data_snapshot_id` | Ref al snapshot del broker |
| | `logic_hash` | Hash del driver del fetcher |
| **III. Hardware** | `node_id` | ID del hardware físico |
| | `process_id` | PID del worker de descarga |
| | `execution_latency_ms` | Tiempo total de descarga |
