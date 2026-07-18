# Data Normalization Layer

**Carpeta:** `./features/data-normalization-layer/`
**Estado:** En Diseño
**Última actualización:** 2026-04-13
**Decisión Arquitectónica Asociada:** ADR-0008 (Configurabilidad Universal) · ADR-0141 (tipos canónicos) · ADR-0154 (dueña de la tabla de capacidades por bróker) · ADR-0155 (dueña del demultiplexado `QuoteTick`/`TradeTick`)

> **Dueña de la Tabla de Capacidades (ADR-0154).** Esta feature es la propietaria del conocimiento declarado de "qué soporta cada fuente". La unidad es la combinación **bróker × vía de ingesta × clase de activo × granularidad** — nunca una fila por bróker, porque el mismo bróker ofrece cosas distintas por su interfaz de programación que por exportación de terminal. Cada fila declara: clases de activo, granularidades, si entrega compra y venta o solo último precio, profundidad de libro, método de autenticación, profundidad histórica estimada y si la licencia de esa fuente permite que el dato alimente la agregación anonimizada (cimiento #9).
>
> **Declarado vs. observado — regla rectora:** la fila es una **hipótesis**. Al conectar por primera vez, el conector verifica lo declarado contra lo que la fuente entrega de verdad y persiste el resultado. **Lo declarado gobierna qué se ofrece antes de conectar; lo observado gobierna después y gana siempre que discrepen.** La profundidad histórica se **mide**, no se cree: el valor declarado solo ordena la interfaz.
>
> **Costo de añadir un bróker, sin adorno:** si reutiliza un protocolo ya implementado, es una fila y nada de código. Si trae protocolo nuevo, es una fila **más** un adaptador con sus pruebas. La tabla desacopla la declaración del código; no elimina el adaptador.

## ¿Qué es?

Es la capa encargada de unificar el caos de diferentes formatos de exchanges y brokers en un estándar interno único. Resuelve el problema de que Binance llama a un par `BTCUSDT`, pero Interactive Brokers lo llama de otra forma, y cada uno tiene su propia escala de precios y volumen.

**Problema:** Cada fuente de datos habla su propio "idioma" (símbolos, decimales, nombres de columnas).
**Solución:** Un traductor universal que estructura todo bajo un objeto `Symbol { base, quote, broker_symbol, tick_size }`.

## Comportamientos Observables

- [ ] El sistema recibe datos de Binance y Oanda; ambos se guardan en la DB interna con el mismo formato de columnas y nomenclatura.
- [ ] Si un activo tiene una escala de precios inusual (ej: centavos de Yen), la capa de normalización la escala a una representación interna consistente (Enteros int64 si se requiere ADR-0004).
- [ ] Cualquier módulo (Backtest, Execute) pide datos usando el "Símbolo Normalizado" (ej: `BTC/USDT`) y el sistema sabe a qué tabla de broker corresponde.
- [ ] **Demultiplexado cotización/ejecución (ADR-0155).** Cuando la fuente entrega ambos eventos mezclados en el mismo archivo, distinguidos por una bandera propia del terminal (MT5: bitmask `TICK_FLAG_BID`/`ASK` frente a `TICK_FLAG_LAST`/`VOLUME`), esta capa enruta cada registro crudo a `QuoteTick` o `TradeTick` según esa bandera de origen, nunca a un tipo mixto con columnas opcionales. Un mismo evento de fuente puede, en casos raros, generar ambas filas (un trade que también actualiza el BBO).

## Restricciones

- NUNCA se permiten símbolos duplicados con diferentes configuraciones de normalización.
- NUNCA se procesan barras que no hayan pasado por la normalización de tipado. **Corrección (ADR-0141):** los precios se normalizan a **`INTEGER` con escala fija ×10⁸**, NO a coma flotante — la referencia previa a `float64` contradecía la prohibición canónica de `REAL` para precios y queda derogada. El tiempo se normaliza a marca de tiempo UTC de resolución fija.
- La normalización debe ser "Zero-Loss": los datos originales crudos deben poder reconstruirse o estar disponibles en una columna de auditoría. **Esto no es solo sobre el valor, es sobre el conjunto de campos (ADR-0154):** está prohibido normalizar recortando al mínimo común denominador entre fuentes. Los campos que una fuente concreta no entrega quedan nulos; nunca se eliminan del esquema.

## Parámetros Configurables

| Parámetro | Default | Rango | Qué hace | FIJO/CONFIG |
|---|---|---|---|---|
| INTERNAL_TIMEZONE | UTC | - | Zona horaria para normalizar todos los timestamps | [FIJO] |
| PRICE_PRECISION | 8 | 0 - 15 | Decimales máximos para normalización de precio | CONFIG |
| SYMBOL_MAPPING | {} | - | Mapa de traducción BrokerSymbol -> InternalSymbol | CONFIG |

## Ciclo de Vida de la Feature — Data Normalization Layer

### Entrada
- DataFrame crudo desde Ingest (Polars).
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

### **TTR-003: Demultiplexador Cotización/Ejecución (ADR-0155)**
- **Qué tiene que pasar:** dado un archivo crudo que mezcla eventos de cotización y ejecución bajo una bandera propia de la fuente (MT5, bitmask interno del terminal), enrutar cada registro a `QuoteTick` (si trae `bid`/`ask`) o a `TradeTick` (si trae `price`/`trade_volume`) según esa bandera — nunca a un tipo intermedio con columnas opcionales de ambos.
- **Postcondición verificable:** dado un archivo mixto, la salida son dos flujos separados (`QuoteTick`, `TradeTick`), cada uno con sus campos obligatorios (`bid`/`ask` en el primero, `price`/`trade_volume` en el segundo) siempre poblados; ningún registro de salida tiene ambos grupos de campos ni ninguno de los dos.

## Puertos de Integración

> *(ADR-0137 / ADR-0155)* Esta feature es la dueña del demultiplexado cotización/ejecución. Recibe el archivo crudo íntegro descargado por `sovereign-data-fetcher` o importado por `data-import-wizard` (aún sin clasificar) y produce los dos tipos físicos separados.

| Puerto | ID de tipo | Dirección | Cardinalidad | Descripción |
|---|---|---|---|---|
| `raw_quote_in` | `QuoteTick` | Input | 0..N | Cotizaciones ya reconocidas como tales en la fuente (no requieren demux) |
| `raw_trade_in` | `TradeTick` | Input | 0..N | Ejecuciones ya reconocidas como tales en la fuente (no requieren demux) |
| `raw_bars_in` | `Bars` | Input | 0..N | Barras OHLCV crudas a normalizar (mapeo de símbolo/escala, sin demux — una barra no mezcla cotización/ejecución) |
| `quote_ticks_out` | `QuoteTick` | Output | 0..N | Cotizaciones normalizadas y demultiplexadas |
| `trade_ticks_out` | `TradeTick` | Output | 0..N | Ejecuciones normalizadas y demultiplexadas |
| `bars_out` | `Bars` | Output | 0..N | Barras normalizadas (símbolo interno, escala consistente) |

## Gobernanza y Estándares (Fijos)

- **Local-First (ADR-0016):** 100% Local (lógica de transformación).
- **Fidelidad (ADR-0017):** Alta (normalización sin pérdida de precisión).
## Persistencia (Inundación de Fundamentos — ADR-0020)

Cada mapeo de símbolo registra el set de relevancia técnica para Datos:

| Categoría | Campo | Descripción |
| :--- | :--- | :--- |
| **I. Identidad** | `id` | Identificador único del mapeo |
| | `created_at` | Timestamp de registro |
| | `updated_at` | Timestamp de última modificación del registro |
| | `audit_hash` | Hash de la configuración del símbolo |
| | `audit_chain_hash` | Hash encadenado del historial del registro |
| | `event_sequence_id` | Secuencia de recuperación (event-sourcing) |
| **III. Linaje** | `data_snapshot_id` | ID del broker fuente |
| | `transformation_id` | ID del mapeo institucional |
| | `logic_hash` | Hash del traductor de esquemas |
| **IV. Hardware** | `node_id` | ID del hardware físico |
| | `process_id` | PID del normalizador |
