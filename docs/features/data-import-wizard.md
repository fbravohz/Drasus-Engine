# Data Import Wizard

**Carpeta:** `crates/features/data/data-import-wizard/`
**Estado:** 🟡 **Parcial — vía de plantilla de plataforma implementada** (STORY-068, 2026-07-25 — QA APTO, gate de mutación 0 supervivientes). Backend completo para la vía en que Drasus afirma la plantilla: lector del archivo de barras de MetaTrader, ledger append-only de declaración de mapeo sellado antes de persistir dato, unidad de spread y de volumen declaradas o rechazo explícito. **Diferido:** la vía de mapeo manual del usuario y su pantalla (arrastrar archivo, vista previa de dato transformado, confirmación) — es la que LDR-0008 exige que pase por vista previa, y va en la Story del asistente visual (`DEBT-035`). También diferido el lector de ticks, que vive en `data-normalization-layer`.
**Última actualización:** 2026-07-25 (STORY-068 — lector de barras MT5, ledger de mapeo, unidad de spread sellada)
**Decisión Arquitectónica Asociada:** ADR-0008 (Configurabilidad Universal) · ADR-0154 (vía 3 de ingesta: exportación manual desde el terminal del usuario) · ADR-0155 (esquema canónico de campos) · ADR-0156 (formato estándar de ingesta — rechazo por defecto, plantillas, mapeo declarado como evidencia) · ADR-0162 (patrón de selector de zona horaria con bandera DST) · ADR-0163 (idempotencia de reimportación — esta feature es la vía manual, con confirmación explícita)

## ¿Qué es?

Es el componente que permite al usuario incorporar datos externos (EJ: CSV de MetaTrader, TXT de NinjaTrader) al sistema Drasus Engine. Aplica una plantilla de plataforma ya validada, o guía al usuario a declarar el mapeo de columnas contra el esquema canónico (ADR-0155) — nunca adivina en silencio.

> **Rol arquitectónico (ADR-0154):** esta feature materializa la **tercera vía de ingesta** — la única en la que Drasus no hace red. El usuario exporta desde su propio terminal y entrega el archivo; Drasus lee, valida, normaliza y particiona. Es la vía que garantiza el precio del bróker de ejecución real, con compra y venta, sin depender de que ese bróker publique volcado alguno.

**Problema:** Cada broker exporta CSVs con nombres de columnas y formatos de fecha distintos. Mapearlos a mano es lento y propenso a errores — pero adivinar el mapeo sin que el usuario lo confirme sobre datos reales es, además de un riesgo técnico, un riesgo legal (LDR-0008): Drasus terminaría afirmando una interpretación que nadie validó.
**Solución (ADR-0156):** dos vías, cada una con su responsable. **Plantilla de plataforma** (NinjaTrader, MetaTrader, Interactive Brokers…) — Drasus la declara, la valida contra exportaciones reales antes de publicarla, y responde si está mal construida. **Mapeo manual declarado** — el asistente puede pre-rellenar una propuesta (conserva el valor de la detección automática como *borrador*), pero el usuario debe confirmarla sobre una vista previa de sus propios datos ya transformados antes de que nada se persista. Sin plantilla y sin confirmación, el archivo se **rechaza** con motivo explícito — nunca se adivina ni se importa a medias.

## Comportamientos Observables

- [ ] El usuario arrastra un archivo CSV; el sistema previsualiza las primeras filas del archivo crudo.
- [ ] Si el archivo coincide con una plantilla de plataforma validada, el sistema la aplica y muestra la vista previa de **datos ya transformados** (no solo nombres de columna) para confirmación.
- [ ] Si no coincide con ninguna plantilla, el sistema propone un mapeo **borrador** (Columna 1 → Timestamp, Columna 2 → Open, etc.) que el usuario debe revisar y confirmar sobre esa misma vista previa de dato transformado real — la propuesta nunca se acepta por sí sola.
- [ ] Si el usuario no completa el mapeo, o el archivo mezcla cotización y ejecución sin que ninguna plantilla conocida lo explique, el sistema **rechaza** con un mensaje en lenguaje del usuario, nunca una importación parcial.
- [ ] Al confirmar, el sistema sella la declaración (archivo, mapeo, plantilla si hubo, vista previa, versión de esquema) en el ledger append-only de mapeo (ADR-0156 restricción 7) antes de persistir cualquier dato.
- [ ] El sistema valida el tipo de datos y corre sus propios guardarraíles (`pit-data-validator`) antes de aceptar el mapeo declarado — la declaración del usuario no los sustituye (ADR-0156 restricción 6).
- [ ] El sistema detecta automáticamente el delimitador (coma, punto y coma, tabulación) — esto es inferencia de formato de archivo, no de significado de columna; no está sujeto a la misma restricción que el mapeo semántico.
- [ ] **Declaración de zona horaria de origen (ADR-0114/TASK-064 §3, corregido por el propietario 2026-07-23).** Si el archivo no trae metadato de zona horaria explícito, el usuario declara la zona horaria de los timestamps crudos mediante el mismo selector de zona mundial con bandera de horario de verano (DST) que usa `session-catalog` (ADR-0162) — **nunca** se infiere de un pico de volatilidad ni de una noticia programada. El desfase declarado queda sellado en el ledger de mapeo como evidencia (mismo tratamiento que el mapeo columna→columna). El hueco de fin de semana puede usarse como verificación estructural **opcional** de sanidad, nunca como mecanismo primario.

## Restricciones

- NUNCA se descarta un campo que el archivo de origen entregue. El esquema de importación es **abierto por arriba**: marca de tiempo y precio son el mínimo exigido, pero todo lo demás que traiga la fuente —compra, venta, volúmenes por lado, banderas de sesión, identificadores de orden— se conserva y se persiste (ADR-0154, ADR-0020). Reducir la importación a "OHLCV + Timestamp" está **prohibido**: es lo que convierte el spread simulado en un parámetro inventado en vez de un dato observado.
  > ✅ **Pendiente resuelto — ADR-0155 (2026-07-18):** el esquema canónico de campos de la operación y de la vela ya existe (`QuoteTick`/`TradeTick`/`Bars`/`DepthLevel`/`OrderEvent`). Un CSV de MT5 que mezcla cotización y ejecución en las mismas columnas (`Bid, Ask, Last, Volumen`) se importa crudo aquí tal cual llega; el demultiplexado a `QuoteTick`/`TradeTick` es responsabilidad de [`data-normalization-layer`](data-normalization-layer.md) aguas abajo, no de este wizard. El mapeo de columnas del asistente debe reconocer los nombres canónicos de ADR-0155 (`bid`/`ask`/`price`/`trade_volume`/`aggressor_side`/`spread_reported`/`volume_traded`/`volume_ticks`/`trade_count`/`volume_buy`) además de los ya cubiertos (OHLCV/Timestamp).
- **(ADR-0163) NUNCA se sobreescriben datos existentes sin una confirmación explícita del usuario.** A diferencia de la vía automatizada (`sovereign-data-fetcher`), aquí el usuario está presente en el momento de la importación: si el archivo se solapa con un rango ya persistido y el contenido diverge (hash distinto), el asistente muestra una vista previa de qué se sobrescribiría y cuál se conservaría — mismo patrón de confirmación sobre dato transformado real que ya usa para el mapeo de columnas (ADR-0156) — y no persiste nada hasta que el usuario confirme. Si el contenido solapado es idéntico (hash igual), es un no-op silencioso sin fricción.
- El proceso de importación masiva debe ejecutarse en segundo plano (vía Download Manager) para no bloquear la UI.
- **(ADR-0156) Rechazo por defecto, nunca adivinanza silenciosa.** Un archivo sin plantilla reconocida y sin mapeo confirmado por el usuario se rechaza — nunca se importa con una interpretación que Drasus inventó sin mostrarla.
- **(ADR-0156) La confirmación del usuario exige vista previa de dato transformado real**, nunca solo la tabla de correspondencias columna→columna. Un mapeo aceptado sobre una lista abstracta de encabezados no protege a Drasus (LDR-0008 condición 1).
- **(ADR-0156) Toda declaración de mapeo se sella en el ledger append-only antes de persistir dato alguno.** Sin excepción, incluida la vía de plantilla de plataforma (que también registra cuál plantilla y qué versión).
- **(ADR-0156) Payload mixto (cotización + ejecución sin mecanismo de plataforma conocido) se rechaza con error de formato explícito**, nunca se consume parcialmente. El caso MT5 (bitmask documentado) no es este caso — cae bajo plantilla de plataforma.

## Parámetros Configurables

| Parámetro | Default | Rango | Qué hace | FIJO/CONFIG |
|---|---|---|---|---|
| AUTO_DETECT_COLUMNS | True | True/False | Pre-rellena un mapeo **borrador** por nombre de columna, para acelerar la revisión del usuario — **nunca se acepta sin que el usuario lo confirme sobre la vista previa de dato transformado** (ADR-0156 restricción 2) | CONFIG |
| PREVIEW_ROWS | 10 | 5 - 100 | Cuántas filas de dato transformado mostrar en la vista previa de confirmación | CONFIG |
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

**Ledger de declaración de mapeo (ADR-0156, nuevo — molde `.agents/knowledge/ledger-pattern.md`):** tabla append-only propia de esta feature, sellada ANTES de que cualquier dato del archivo se persista. Guarda identidad del archivo (nombre + hash), el mapeo completo tal como se mostró al usuario, la plantilla aplicada y su versión (si hubo), la bandera `mapeo_manual_usuario`/`plantilla_plataforma`, la versión del motor de transformación y del esquema canónico (ADR-0155) vigentes al declarar, el instante de aceptación, la vista previa de dato transformado que el usuario confirmó, la **zona horaria de origen declarada** (identificador IANA, mismo mecanismo de captura que `session-catalog`, ADR-0162 — sellada como evidencia inmutable, no como inferencia del sistema, TASK-064 §3.3), y el encadenado de hash de todo ledger. El `data_snapshot_id` de la partición resultante y la fila de `transformation_records` de la corrida referencian esta fila por identificador — nunca la duplican (ver ADR-0156 restricción 8).

Cada importación manual registra además el set de relevancia técnica:

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

## Puertos de Integración (ADR-0137)

Completados por el Tech-Lead en el cierre de STORY-068 (la sección faltaba; el QA lo señaló en su gate final). Tipos del catálogo de ADR-0137.

| Puerto | Tipo | Dirección | Cardinalidad | Qué transporta |
|---|---|---|---|---|
| `bars_out` | `Bars` | Salida | 0..N | Filas de barra ya canónicas (precio en entero ×10⁸, tiempo en ns UTC, identidad de serie completa, unidad de spread declarada), listas para que [`hybrid-data-transformer`](./hybrid-data-transformer.md) las escriba en Parquet. Esta feature **nunca** escribe Parquet directamente (ADR-0163). |
| `mapping_declaration_out` | `AuditEvent` | Salida | 1 por importación | La fila del ledger append-only que sella qué declaró el usuario sobre el archivo, antes de que exista dato. Es la evidencia de LDR-0008, no configuración. |
| `calendar_verdict_in` | `CalendarCoherenceVerdict` | Entrada | 0..1 | Veredicto del Gate de Coherencia de Calendario de [`pit-data-validator`](./pit-data-validator.md), que corre ANTES de aceptar el mapeo declarado (ADR-0156 restricción 6). |

**Deuda de puerto conocida (registrada como `DEBT-036`):** `bars_out` no transporta hoy el identificador de la declaración sellada. Mientras el destino sea un consumidor en memoria, la trazabilidad de ADR-0156 restricción 8 se sostiene por convención de aplicación; en cuanto ese puerto se conecte con la persistencia real, su forma debe cargar la referencia o la cadena de trazabilidad se rompe físicamente.

## Dependencias

**Depende de:**
- [`pit-data-validator`](./pit-data-validator.md) — corre sus guardarraíles (incl. el Gate de Coherencia de Calendario, ADR-0162) antes de aceptar el mapeo declarado (ADR-0156 restricción 6).
- [`session-catalog`](./session-catalog.md)/[`instrument-catalog`](./instrument-catalog.md) — para la asociación bróker↔instrumento↔plantilla de sesión durante la importación (ADR-0162).
- [`hybrid-data-transformer`](./hybrid-data-transformer.md) — ejecuta la reescritura atómica de partición Hive-Style una vez que el usuario confirma el solape (ADR-0163); este wizard nunca escribe Parquet directamente.

**Consumido por:**
- [`ingest`](../modules/ingest.md).
