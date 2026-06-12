# Data Normalization Layer

**Carpeta:** `./features/data-normalization-layer/`
**Estado:** En Diseño
**Última actualización:** 2026-04-13
**Decisión Arquitectónica Asociada:** ADR-0008 (Configurabilidad Universal)

## ¿Qué es?

Es la capa encargada de unificar el caos de diferentes formatos de exchanges y brokers en un estándar interno único. Resuelve el problema de que Binance llama a un par `BTCUSDT`, pero Interactive Brokers lo llama de otra forma, y cada uno tiene su propia escala de precios y volumen.

**Problema:** Cada fuente de datos habla su propio "idioma" (símbolos, decimales, nombres de columnas).
**Solución:** Un traductor universal que estructura todo bajo un objeto `Symbol { base, quote, broker_symbol, tick_size }`.

## Comportamientos Observables

- [ ] El sistema recibe datos de Binance y Oanda; ambos se guardan en la DB interna con el mismo formato de columnas y nomenclatura.
- [ ] Si un activo tiene una escala de precios inusual (ej: centavos de Yen), la capa de normalización la escala a una representación interna consistente (Enteros int64 si se requiere ADR-0004).
- [ ] Cualquier módulo (Backtest, Execute) pide datos usando el "Símbolo Normalizado" (ej: `BTC/USDT`) y el sistema sabe a qué tabla de broker corresponde.

## Restricciones

- NUNCA se permiten símbolos duplicados con diferentes configuraciones de normalización.
- NUNCA se procesan barras que no hayan pasado por la normalización de tipado (float64 para precios, datetime64[ns] para tiempo).
- La normalización debe ser "Zero-Loss": los datos originales crudos deben poder reconstruirse o estar disponibles en una columna de auditoría.

## Parámetros Configurables

| Parámetro | Default | Rango | Qué hace | FIJO/CONFIG |
|---|---|---|---|---|
| INTERNAL_TIMEZONE | UTC | - | Zona horaria para normalizar todos los timestamps | [FIJO] |
| PRICE_PRECISION | 8 | 0 - 15 | Decimales máximos para normalización de precio | CONFIG |
| SYMBOL_MAPPING | {} | - | Mapa de traducción BrokerSymbol -> InternalSymbol | CONFIG |

## Ciclo de Vida de la Feature — Data Normalization Layer

### Entrada
- DataFrame crudo desde Ingest (Polars/Pandas).
- Metadatos del broker (símbolo original, convención de nombres).

### Proceso
- Aplica el mapeo de nombres de columnas a (Open, High, Low, Close, Volume, Timestamp).
- Convierte timestamps a UTC datetime64[ns] de alta precisión.
- Escala precios y volúmenes según las reglas del activo.
- Inyecta metadatos institucionales (base, quote).

### Salida
- DataFrame normalizado de Polars.
- Objeto de identidad de símbolo único.

### Contextos de Uso

**Contexto 1: Ingesta (ETL)**
- Estandarización de datos recién descargados antes de persistirlos en Parquet.

**Contexto 2: Ejecución Real (Módulo Execute)**
- Traducción de la orden interna al "Broker Symbol" específico para colocar la operación.

## Tareas (TTRs)

### **TTR-001: Mapeador Universal de Activos**
- Implementa el diccionario dinámico y lógica de traducción de símbolos entre brokers y el sistema interno.

### **TTR-002: Estandarizador de Tipado y Escala**
- Implementa la transformación física de tipos de datos y escalado de precios para asegurar coherencia numérica total.

## Gobernanza y Estándares (Fijos)

- **Local-First (ADR-0016):** 100% Local (lógica de transformación).
- **Fidelidad (ADR-0017):** Alta (normalización sin pérdida de precisión).
## Persistencia (Inundación de Fundamentos — ADR-0020 V2)

Cada mapeo de símbolo registra el set de relevancia técnica para Datos:

| Categoría | Campo | Descripción |
| :--- | :--- | :--- |
| **I. Identidad** | `id` | Identificador único del mapeo |
| | `created_at` | Timestamp de registro |
| | `audit_hash` | Hash de la configuración del símbolo |
| **II. Linaje** | `source_id` | ID del broker fuente |
| | `transformation_id" | ID del mapeo institucional |
| | `logic_hash` | Hash del traductor de esquemas |
| **III. Hardware** | `node_id` | ID del hardware físico |
| | `process_id` | PID del normalizador |
