# Data Sanitizer Pipeline (ETL Soberana — 6 Capas de Limpieza)

**Carpeta:** `./features/data-sanitizer-pipeline/`
**Estado:** En Diseño
**Última actualización:** 2026-07-23 (TASK-064 — Política de Integridad de Spread)
**Decisión Arquitectónica Asociada:** ADR-0037 (Protocolo de Calidad "The Sanitizer") · ADR-0155 (esquema canónico — `spread_reported` habilita la Capa 5) · ADR-0114 (sub-cláusula de regla (c), 2026-07-23 — esta feature es la dueña de la Política de Integridad de Spread que esa sub-cláusula exige) · ADR-0162 (`instrument-catalog`, fuente del piso de spread declarado por instrumento)

## ¿Qué es?

Es el guardian de la calidad de datos de Drasus Engine y el cerebro del Módulo Ingest. Implementa un protocolo institucional de 6 capas de limpieza profunda para transformar feeds de datos brutos (especialmente de proveedores como Tiingo, FirstRateData o Binance Vision) en datos "institucionales" antes de que lleguen al motor de backtest. Asegura la soberanía técnica eliminando outliers, rellenando huecos, validando la integridad del balance de precios y evitando que errores estructurales (gaps, precios anómalos) contaminen las estrategias.

**Problema:** Los datos crudos de brokers suelen tener huecos temporales o errores de OHLC que arruinan la fidelidad de las pruebas.
**Solución:** Un pipeline secuencial estricto de 6 capas que valida y repara la data bajo estándares profesionales.

## Comportamientos Observables (Pipeline de 6 Capas)

- [ ] **Capa 1: Ingesta Soberana.** Carga Parquet/CSV local con esquema estricto validado mediante schemas tipados nativos en Rust (Serde).
- [ ] **Capa 2: Gap Infusion (Detección y Relleno de Huecos).** Identifica huecos temporales en la serie (fines de semana, lag de red). Si el hueco es menor al umbral configurable (`MAX_GAP_FILL_LIMIT`), aplica interpolación lineal automática (Gap Auto-fill / Forward Fill); si lo excede, marca el tramo como `INVALID` para backtesting.
  > **Política de Integridad de Spread — caso NULO (ADR-0114 sub-cláusula, TASK-064 §1.1).** El campo `spread_reported` **no** se interpola linealmente como el OHLC — un spread es una cotización discreta que puede saltar por una noticia, no un precio continuo entre dos puntos conocidos. En su lugar se aplica **arrastre acotado** (`carry-forward`, LOCF): se propaga el último `spread_reported` realmente observado, hasta un umbral propio (`SPREAD_GAP_FILL_LIMIT`) — más allá del umbral, el campo se marca `INVALID` para ese tramo, mientras el OHLC de la misma barra puede seguir siendo válido si su propio gap no excede `MAX_GAP_FILL_LIMIT` (dos relojes de vencimiento independientes sobre la misma fila).
- [ ] **Capa 3: Outlier Scrubbing (Detección de Gaps de Precio).** Evalúa cada salto de precio entre barras consecutivas contra una escala **robusta y propia del instrumento**: la mediana de desviaciones absolutas (MAD) rodante de `|Δclose|` sobre una ventana de `OUTLIER_MAD_WINDOW_BARS` barras (nunca desviación estándar cruda — una sigma calculada incluyendo los propios outliers se infla y los esconde; nunca el archivo completo — mezclaría regímenes de volatilidad distintos), con un piso desde `canonical_pip_size` del catálogo cuando la MAD ronda cero. **Secuencia fija de evaluación, nunca al revés:** primero calendario, luego magnitud, luego reversión.
  1. **Contexto de calendario primero (ADR-0162).** Si la transición cae donde `session_template_window` declara un cierre (fin de semana Forex, mantenimiento de Futuros), el salto se presume gap legítimo y se exime del umbral estricto de sesión continua, sujeto solo al techo de cordura `OUTLIER_GAP_SANITY_CEILING` (mucho más laxo). En sesión líquida declarada, aplica el umbral estricto (`OUTLIER_MAD_MULTIPLIER_CONTINUOUS`) sin excepción. **Un instrumento sin plantilla de sesión resuelta (`INDETERMINADO` del Gate de Coherencia) nunca hereda la exención** — degrada siempre al régimen de sesión continua, el más estricto.
  2. **Magnitud normalizada.** `|close(N) − close(N−1)|` contra la escala MAD (o el techo de cordura si aplica la exención de cierre).
  3. **Reversión como señal de refuerzo, nunca decisoria por sí sola.** Si `close(N+1)` vuelve cerca de `close(N−1)` (dentro de `OUTLIER_REVERSION_TOLERANCE_PCT`), es indicio de impresión errónea que revierte — pero un movimiento real en V (caída-recuperación por una noticia genuina) también revierte, así que esta señal solo se cruza con las otras dos, nunca decide sola.
  4. **Razón de escala.** `price(N)/price(N−1)` cercano a una potencia de 10 (±`OUTLIER_SCALE_RATIO_EPSILON`) es indicio independiente de mezcla de fuentes o corrimiento decimal, sin importar la magnitud absoluta del salto.
  5. **Cruce con Capa 5.** Si `spread_reported` también se dispara y corrige junto con el precio, refuerza "evento real de liquidez" sobre "error puntual".
  - **Veredicto explícito por transición, nunca aprobación silenciosa:** `ACEPTADO` (dentro de umbral), `ACEPTADO` con etiqueta "gap de sesión" o "volatilidad verificada" (alimenta `ANALYZE_VOLATILITY` de [`quality-heatmap-generator`](quality-heatmap-generator.md)), `INVÁLIDO` (descartado, registrado en `error_log`), o `INDETERMINADO` (evidencia insuficiente — ventana MAD con menos de 2 barras — requiere revisión humana).
- [ ] **Capa 4: OHLC Sanity (Integrity Check).** Valida que `High >= (Open, Close, Low)` y `Low <= (Open, Close, High)`. Si una vela falla esta validación, la fila es descartada e incluida en el `error_log`, o reparada si es posible.
- [ ] **Capa 5: Volume & Spread Corroboration.** Filtra ticks con volumen cero o inconsistente (distinguiendo `volume_traded`/`volume_ticks`/`trade_count`/`volume_buy` — ADR-0155, nunca una columna `volume` genérica). Compara dos señales de spread: el derivado (`ask − bid` de `QuoteTick`) contra `spread_reported` cuando la fuente lo entrega como columna propia (ADR-0155 §"segunda observación independiente"); si la desviación entre ambos supera `SPREAD_SIGMA_THRESHOLD` (default 3σ), genera una alerta automática de anomalía — el defecto concreto que este esquema existe para atrapar es un bróker que reporta un spread que no coincide con su propia cotización.
  > **Política de Integridad de Spread — caso CERO (ADR-0114 sub-cláusula, TASK-064 §1.2).** Un `spread_reported = 0` explícito **no se acepta a ciegas** — un CSV M1 de 25 años exportado de un terminal minorista es exactamente el tipo de fuente con historial de reportar `0` por defecto de exportador en tramos antiguos, no por mercado real (aunque el `0` legítimo también existe, ej. solapes de sesión de máxima liquidez en brókers ECN Tier-1). Se aplica un **piso calibrado por instrumento**, en este orden: (1) `default_spread` declarado en el catálogo (`instrument-catalog`, puerto `resolved_terms_out` de ADR-0162) — el valor que el propietario declaró al configurar ese instrumento/bróker, nunca adivinado; (2) si no hay `default_spread` declarado, la distribución de los spreads no-cero del propio archivo (percentil bajo robusto) como calibración autocontenida. **Nunca** un piso fijo idéntico para todo instrumento, y **nunca** calibrado contra un dataset externo — ambos serían el mismo tipo de alucinación de cifra que esta política existe para evitar, solo que con signo distinto (uno optimista, uno pesimista arbitrario).
- [ ] **Capa 6: Point-in-Time Injection & Parquet Final Inundation.** Reconstruye el mercado ignorando eventos que ocurrieron "en el futuro" respecto a la fecha de simulación, y escribe el rastro auditable (ADR-0020) en el archivo de salida.

## Restricciones

- NUNCA un dato sin sanitizar puede guardarse como "Golden Source".
- NUNCA se realiza interpolación en gaps mayores a `MAX_GAP_FILL_LIMIT` (configurable).
- El pipeline debe ser determinista: misma entrada cruda siempre produce misma salida limpia.
- Si la Capa 4 (OHLC Sanity) falla y `CLEAN_FLOW_STRICT` está activo, el job completo falla.
- **(ADR-0114/TASK-064) NUNCA se interpola linealmente un spread nulo.** Se arrastra el último valor observado (`carry-forward`), acotado por `SPREAD_GAP_FILL_LIMIT`.
- **(ADR-0114/TASK-064) NUNCA un `spread_reported = 0` se acepta como hecho sin pasar por el piso calibrado por instrumento.** El piso nunca es una constante global ni se calibra contra un dataset externo.
- **(Capa 3, política Quant-Engineer 2026-07-24) NUNCA un salto de precio se evalúa contra una constante global.** La escala de referencia es la MAD rodante propia del instrumento (o el piso de `canonical_pip_size` del catálogo); nunca una desviación estándar cruda ni un umbral idéntico para todo instrumento.
- **(Capa 3) NUNCA se rechaza un salto por tamaño sin consultar antes el contexto de calendario declarado (ADR-0162).** La secuencia calendario → magnitud → reversión es fija; invertirla produce falsos positivos en cierres de sesión legítimos.
- **(Capa 3) NUNCA un instrumento sin plantilla de sesión resuelta hereda la exención de gap de cierre.** Sin calendario declarado, todo salto se evalúa bajo el régimen de sesión continua (el más estricto).

## Parámetros Configurables

| Parámetro | Default | Rango | Qué hace | FIJO/CONFIG |
|---|---|---|---|---|
| GAP_AUTOFILL_ENABLED | True | True/False | Activa la interpolación de micro-gaps | CONFIG |
| MAX_GAP_FILL_LIMIT | 5 | 1 - 60 (bins) | Máximo de barras consecutivas a interpolar | CONFIG |
| SPREAD_SIGMA_THRESHOLD | 3.0 | 1.0 - 10.0 | Umbral de desviación para alertas de spread | CONFIG |
| CLEAN_FLOW_STRICT | True | True/False | Si true, falla el job ante cualquier error OHLC | [FIJO] |
| SPREAD_GAP_FILL_LIMIT | 5 | 1 - 60 (bins) | Máximo de barras consecutivas para arrastrar (`carry-forward`) un `spread_reported` nulo antes de marcar el tramo `INVALID` — parámetro propio, no reutiliza `MAX_GAP_FILL_LIMIT` porque spread y OHLC pueden divergir en tolerancia (ADR-0114/TASK-064) | CONFIG |
| SPREAD_ZERO_FLOOR_PERCENTILE | percentil bajo (a fijar en Story de implementación) | 0.0 - 1.0 | Percentil de la distribución de spreads no-cero del propio archivo usado como piso cuando no hay `default_spread` declarado en el catálogo (ADR-0114/TASK-064 §1.2) | CONFIG |
| OUTLIER_MAD_MULTIPLIER_CONTINUOUS | 8.0 | 3.0 - 15.0 | Multiplicador de la MAD rodante que define el umbral de salto anómalo en sesión líquida (Capa 3) | CONFIG |
| OUTLIER_GAP_SANITY_CEILING | 40.0 | 15.0 - 100.0 | Techo de cordura (multiplicador de MAD) aplicado cuando la transición cae en un cierre de calendario declarado — más laxo que el umbral de sesión continua | CONFIG |
| OUTLIER_FLOOR_PIPS | 20 | por catálogo/clase de activo | Piso en pips (vía `canonical_pip_size` del catálogo) usado como escala mínima cuando la MAD ronda cero | CONFIG |
| OUTLIER_MAD_WINDOW_BARS | 500 | 50 - 5000 | Tamaño de la ventana rodante (barras) sobre la que se calcula la MAD — nunca el archivo completo, para no mezclar regímenes de volatilidad distintos | CONFIG |
| OUTLIER_REVERSION_TOLERANCE_PCT | 25% | 5% - 50% | Tolerancia para considerar que `close(N+1)` "revirtió" cerca de `close(N−1)` — señal de refuerzo, nunca decisoria por sí sola | CONFIG |
| OUTLIER_SCALE_RATIO_EPSILON | 2% | 0.5% - 10% | Banda alrededor de una potencia de 10 en `price(N)/price(N−1)` — indicio de mezcla de fuentes o corrimiento decimal | CONFIG |

## Ciclo de Vida de la Feature — Data Sanitizer Pipeline

### Entrada
- `Raw Data` (DataFrame de Polars).
- Configuración de filtros (Delisted, Corporate Events).

### Proceso
1. **Filtro de Delisting:** Elimina activos que ya no existían en el momento de la captura.
2. **Ajuste de Eventos Corporativos:** Aplica splits/dividendos para series históricas consistentes.
3. **Validador PIT:** Certifica que no hay look-ahead bias.
4. **Sanitización Física (6 Capas):** Ejecuta secuencialmente las Capas 1-6 descritas arriba para corregir OHLC y rellenar micro-gaps.

### Salida
- `Clean Data` (`Sanitized_Dataframe`, Parquet) lista para persistencia.
- Reporte detallado de anomalías detectadas (`error_log`, estadísticas de salud del dataset).

### Contextos de Uso

**Contexto 1: Ingesta ETL (Ingest)**
- Limpieza masiva de históricos tras descarga Bulk.

**Contexto 2: Feed Profesional (Live)**
- Validación en tiempo real de datos entrantes de proveedores institucionales.

## Tareas (TTRs)

### **TTR-001: Implementación de Pipeline Secuencial**
- Construye la lógica de flujo: `Raw → Delisted → Adjuster → PIT → Clean` ejecutando las 6 capas de limpieza en Polars para máxima velocidad.

### **TTR-002: Algoritmo de Gap Detection e Interpolación**
- Desarrolla la detección de discontinuidades temporales (Gap Infusion) y el motor de relleno lineal (Forward Fill).

### **TTR-003: Validador de Sanidad (Sanity Audit)**
- **Descripción:** Genera estadísticas de salud del dataset (ej: "% de outliers eliminados", "% de gaps rellenados").
- **Postcondición:** Persistencia de metadatos en `data_inventory` con hash de integridad.

### **TTR-004: Política de Integridad de Spread (ADR-0114/TASK-064)**
- **Descripción:** Extiende la Capa 2 (arrastre acotado para `spread_reported` nulo) y la Capa 5 (piso calibrado por instrumento para `spread_reported = 0`), consumiendo el `default_spread` resuelto de `instrument-catalog` como primera fuente de piso.
- **Reglas de Negocio:**
    * Nulo → `carry-forward` acotado por `SPREAD_GAP_FILL_LIMIT`, nunca interpolación lineal.
    * Cero → piso desde `default_spread` declarado (catálogo) o, en su ausencia, percentil bajo de la distribución de spreads no-cero del propio archivo — nunca constante global, nunca calibrado contra un dataset externo.
- **Entrada:** `spread_reported` crudo de la barra/tick, `ResolvedInstrumentTerms` (de `instrument-catalog`, opcional — su ausencia degrada al percentil del propio archivo).
- **Salida:** `spread_reported` saneado + marca `INVALID` en los tramos que excedieron el umbral de arrastre.
- **Precondición:** Capa 1 (ingesta) completada.
- **Postcondición:** El piso aplicado y su fuente (declarado vs. percentil del archivo) quedan registrados en el reporte de anomalías de la Capa 5, para auditoría.

### **TTR-005: Detección de Gaps de Precio por MAD Robusta + Exención de Calendario (Capa 3, ADR-0037 enmienda 2026-07-24)**
- **Descripción:** Implementa la secuencia fija calendario → magnitud → reversión → razón de escala descrita en Capa 3, con la MAD rodante como escala de referencia (nunca desviación estándar cruda ni constante global).
- **Reglas de Negocio:**
    * Calendario primero: consulta `calendar_verdict_in`; sin plantilla resuelta, régimen de sesión continua siempre.
    * Magnitud: MAD rodante de `OUTLIER_MAD_WINDOW_BARS` barras, piso `OUTLIER_FLOOR_PIPS` cuando MAD ronda cero.
    * Reversión y razón de escala son señales de refuerzo, nunca deciden solas.
- **Entrada:** `Bars` crudo (post Capas 1-2), `calendar_verdict_in` (opcional).
- **Salida:** cada transición con veredicto `ACEPTADO` / `ACEPTADO` etiquetado / `INVÁLIDO` / `INDETERMINADO`.
- **Precondición:** Capas 1-2 completadas (ingesta + gap temporal ya resuelto).
- **Postcondición:** los saltos `INVÁLIDO` quedan en `error_log`; los etiquetados alimentan `ANALYZE_VOLATILITY` de `quality-heatmap-generator`.

## Puertos de Integración (ADR-0137)

| Puerto | ID de tipo | Dirección | Cardinalidad | Descripción |
|---|---|---|---|---|
| `raw_data_in` | `Bars` | Input | 1 | Dataset crudo a sanitizar (Capas 1-6) |
| `resolved_terms_in` | `ResolvedInstrumentTerms` | Input | 0..1 | Términos resueltos del instrumento (de `instrument-catalog`), fuente primaria del piso de spread (Capa 5); su ausencia degrada al percentil del propio archivo |
| `calendar_verdict_in` | `CalendarCoherenceVerdict` | Input | 0..1 | Veredicto y huella de calendario resuelta (de `pit-data-validator`/`session-catalog`, ADR-0162), fuente de la exención de gap en Capa 3; su ausencia (`INDETERMINADO` o sin plantilla) fuerza el régimen de sesión continua |
| `sanitized_data_out` | `SanitizedDataframe` | Output | 1 | Dataset post-6 capas, incluida la Política de Integridad de Spread y la Detección de Gaps de Precio |

## Gobernanza y Estándares (Fijos)

- **Local-First (ADR-0016):** 100% Local.

## Persistencia (Inundación de Fundamentos — ADR-0020)

Toda sanitización registra el set de relevancia técnica para Datos:

| Categoría | Campo | Descripción |
| :--- | :--- | :--- |
| **I. Identidad** | `id` | Identificador único de la sanitización |
| | `created_at` | Timestamp de inicio del proceso |
| | `updated_at` | Timestamp de última modificación del registro |
| | `audit_hash` | Hash del dato limpio resultante |
| | `audit_chain_hash` | Hash de la secuencia de 6 capas de calidad |
| | `event_sequence_id` | Secuencia de recuperación (event-sourcing) |
| **III. Linaje** | `data_snapshot_id` | Ref al dataset crudo original (input) |
| | `transformation_id` | ID del paso de limpieza (Capa 1-6) |
| | `logic_hash` | Hash de la lógica del motor sanitizador |
| | `indicator_state_hash` | Estadísticas del dataset (outliers/gaps detectados) |
| **IV. Hardware** | `node_id` | ID del hardware físico |
| | `process_id` | PID del worker de limpieza |
| | `execution_latency_ms` | Tiempo total de sanitización |

## Dependencias
- [`pit-data-validator`](./pit-data-validator.md) — para la validación final post-limpieza (Validador PIT) y como fuente de `calendar_verdict_in` (Gate de Coherencia de Calendario, ADR-0162), que alimenta la exención de gap de Capa 3.
- [`instrument-catalog`](./instrument-catalog.md) — fuente del `default_spread` declarado que alimenta el piso de la Política de Integridad de Spread (ADR-0114/TASK-064); opcional, degrada al percentil del propio archivo si no está disponible.
- [`quality-heatmap-generator`](./quality-heatmap-generator.md) — consumidor de la etiqueta de anomalía de Capa 3 vía `ANALYZE_VOLATILITY`.
