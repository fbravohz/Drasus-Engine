# Especificación Canónica de Campos — Tick y Barra de Mercado

> **Estado:** insumo técnico para que el Architect redacte el ADR correspondiente. No es un ADR — no se ha editado ningún ADR, feature ni el ROADMAP.
> **Autor:** Quant-Engineer · **Fecha:** 2026-07-18
> **Relacionado:** `.agents/plans/plan-decision-fuentes-datos-mercado.md` (TASK-052, Architect) ya delega en este skill la Q3 (forma del tick) y Q5 (origen del 1 minuto). Este documento las resuelve y las extiende a barra, microestructura y opciones.

**[knowledge consultado: `base.md` — gobernanza obligatoria de arranque]**
**[knowledge consultado: `critical-domain-reasoning.md` — decisión de dominio cuantitativo/microestructura que se va a canonizar; las 7 comprobaciones están en §7]**

---

## 0. Punto de partida verificado en código

`crates/shared/src/types/mod.rs:51` y `:59` declaran `pub struct Tick;` y `pub struct Bars;` — marcadores de puerto tipado (ADR-0137) **sin un solo campo**. Este documento es el contenido que llena esos dos structs (y sus vecinos de microestructura y opciones).

---

## 1. Principios rectores aplicados

- **Inundación de Fundaciones (ADR-0020 §7.3):** ante duda, incluir la columna ahora — es más barato que reescribir Parquet después (ADR-0141: solo se añaden columnas).
- **Molde de tipos (ADR-0141):** precio/volumen → `INTEGER` escala ×10⁸; timestamps → `INTEGER` nanosegundos UTC; enums discretos → `TEXT` con `CHECK (col IN (...))`; JSON → `TEXT` con `CHECK (json_valid(col))`. Toda desviación del molde se justifica explícitamente donde ocurre (§3, §4, §6).
- **Primitivo vs. atributo estático del instrumento:** lo que identifica QUÉ instrumento es (strike, vencimiento, multiplicador, contrato específico de futuro) vive **una sola vez** en el registro de instrumento (`Instrument`, ADR-0140), no repetido en cada tick/barra. Lo que CAMBIA con el tiempo (precio, interés abierto, IV, griegas) sí vive en el tick/barra. Repetir un atributo estático en cada fila de una serie de alta frecuencia es la misma clase de error que persistir un campo derivable — duplica una verdad que ya vive en otro lado.
- **Primitivo vs. política:** el dato crudo del contrato específico se captura tal cual llega; la política de cómo se agrega, ajusta o continúa (barra desde tick, contrato continuo de futuro, precio ajustado por split) es de una feature aparte, nunca se pre-calcula y persiste en la tabla cruda.

---

## 2. Decisión — Cotización vs. Ejecución: dos tipos, no un tipo con banderas

**Decisión: dos primitivos físicos distintos — `QuoteTick` (cotización) y `TradeTick` (ejecución).**

- **Por qué no un tipo con banderas:** MT5 exporta ambos eventos en el mismo archivo, distinguidos por un bitmask interno (`TICK_FLAG_BID`/`ASK` frente a `TICK_FLAG_LAST`/`VOLUME`) que el propio terminal calcula, no la fuente. Persistir eso como una fila con columnas nulables mezcladas (`bid`/`ask` a veces vacíos, `price`/`aggressor_side` a veces vacíos) rompe el `CHECK NOT NULL` que hace que un bug de relleno accidental sea detectable en vez de silencioso — exactamente el tipo de defecto que el checklist de sesgos de este skill audita (look-ahead por confundir "cambió el precio" con "alguien operó").
- **Aplicación de la comprobación #7 (distingue tipos, no promedies):** una cotización no tiene contraparte que cruzó el spread; una ejecución sí. Forzarlos al mismo esquema obliga a que el 50% de las columnas sean opcionales en cada fila, y cualquier consumidor (CVD, VWAP, motor de fills) tiene que volver a preguntar "¿esto es una cotización o un trade?" en tiempo de lectura — la misma pregunta que el bitmask ya respondía en origen.
- **Dónde vive el demux:** el `data-normalization-layer` (`docs/features/data-normalization-layer.md`, restricción "Zero-Loss") es el punto correcto para leer el archivo mixto de la fuente y enrutar cada registro crudo a `QuoteTick` o `TradeTick` según su bandera de origen. Un mismo evento de fuente puede, en casos raros, generar ambas filas (un trade que también actualiza el BBO).
- **Consecuencia para Q3 del plan de fuentes:** responde la pregunta del Architect con una forma concreta, no solo "compra y venta separadas" — son dos tipos de fila, ambos con `bid`/`ask` o `price`/`volumen` obligatorios, nunca nulos dentro de su propio tipo.

---

## 3. Tabla de campos — Tick

Columna **Tipo**: `Ambos` (campo común a `QuoteTick` y `TradeTick`) · `Cotización` (solo `QuoteTick`) · `Ejecución` (solo `TradeTick`).

El Grupo I de ADR-0020 (`id`, `created_at`, `audit_hash`, `audit_chain_hash`, `event_sequence_id`) es universal y no se repite aquí — aplica sin novedad.

| Campo | Significado | Tipo canónico (ADR-0141) | Oblig./Opc. | Tipo | Clases de activo |
|---|---|---|---|---|---|
| `instrument_id` | Instrumento negociado (FK al registro de instrumento) | `TEXT` | Obligatorio | Ambos | Todas |
| `venue` | Mercado o bróker de origen del dato (ej. `binance`, `ic-markets-mt5`, `cme`) | `TEXT` | Obligatorio | Ambos | Todas |
| `source_provider` | Proveedor concreto que entregó el archivo/feed (ej. `histdata`, `dukascopy`, `mt5-export-manual`) — distinto de `venue` cuando el proveedor no es el mercado (forex no tiene "el exchange") | `TEXT` | Obligatorio | Ambos | Todas |
| `event_timestamp_ns` | Instante del evento de mercado (no de procesamiento) | `INTEGER` ns UTC | Obligatorio | Ambos | Todas |
| `data_snapshot_id` | Partición Parquet de origen (linaje, formato canónico ADR-0141) | `TEXT` | Obligatorio | Ambos | Todas |
| `bid` | Mejor precio de compra cotizado en ese instante | `INTEGER` ×10⁸ | Obligatorio | Cotización | Forex/CFD, Acciones, Futuros, Cripto |
| `ask` | Mejor precio de venta cotizado en ese instante | `INTEGER` ×10⁸ | Obligatorio | Cotización | Forex/CFD, Acciones, Futuros, Cripto |
| `bid_size` | Tamaño disponible en el mejor bid (Nivel 1) | `INTEGER` ×10⁸ | Opcional — solo si la fuente lo publica (IB y Dukascopy sí; MT5 tick export no) | Cotización | Forex/CFD, Acciones, Futuros, Cripto |
| `ask_size` | Tamaño disponible en el mejor ask (Nivel 1) | `INTEGER` ×10⁸ | Opcional, igual condición que `bid_size` | Cotización | Forex/CFD, Acciones, Futuros, Cripto |
| `spread_reported` | Spread que la fuente reporta como columna propia (no el resultado de `ask − bid`) | `INTEGER` ×10⁸ | Opcional — ver decisión §6.1 | Cotización | Forex/CFD (MT5 lo trae) |
| `price` | Precio al que se ejecutó la operación | `INTEGER` ×10⁸ | Obligatorio | Ejecución | Todas |
| `trade_volume` | Tamaño de esa operación individual | `INTEGER` ×10⁸ | Obligatorio | Ejecución | Todas |
| `trade_id` | Identificador de la operación en la fuente (dedupe, reconciliación) | `TEXT` | Opcional — presente cuando la fuente lo expone (Binance aggTrades: `a`; IB: implícito en el stream) | Ejecución | Todas |
| `aggressor_side` | Quién cruzó el spread: `BUY` (agresor compró, golpeó el ask) / `SELL` (agresor vendió, golpeó el bid) / `UNKNOWN` | `TEXT` `CHECK (col IN ('BUY','SELL','UNKNOWN'))` | Opcional, `UNKNOWN` si no aplica | Ejecución | Todas |
| `exchange_code` | Venue específico donde se imprimió el trade (relevante cuando el mismo símbolo cotiza en múltiples mercados) | `TEXT` | Opcional | Ejecución | Acciones (multi-venue), Futuros |
| `trade_conditions` | Códigos de condición de la operación tal como los reporta la fuente (lote impar, subasta de apertura/cierre, precio promedio…) | `TEXT` JSON array, `CHECK (json_valid(col))` | Opcional | Ejecución | Acciones |

**Nota sobre opciones y microestructura de Nivel 2/3:** ambas amplían este primitivo sin romperlo — ver §7 (opciones) y §8 (profundidad de libro). Una cotización o ejecución de un contrato de opción usa exactamente esta tabla; lo que cambia es el instrumento al que apunta `instrument_id`.

---

## 4. Decisión — La barra: OHLCV con volumen desagregado, no OHLCV plano

Una barra no es solo "cuatro precios y un volumen". Es la agregación de un intervalo de `TradeTick` (nunca de `QuoteTick` — el volumen no existe en una cotización), y esa agregación pierde información salvo que el esquema capture explícitamente las distintas magnitudes que hoy se mezclan bajo la palabra "volumen" (ver decisión §6.3).

---

## 5. Tabla de campos — Barra

Columna **Clase**: `Común` (todas) · `Forex/CFD` · `Acciones` · `Futuros` · `Opciones`.

| Campo | Significado | Tipo canónico (ADR-0141) | Oblig./Opc. | Clase |
|---|---|---|---|---|
| `instrument_id` | Instrumento agregado | `TEXT` | Obligatorio | Común |
| `venue` / `source_provider` | Mismo significado que en Tick | `TEXT` | Obligatorio | Común |
| `bar_open_timestamp_ns` | Instante de apertura de la barra | `INTEGER` ns UTC | Obligatorio | Común |
| `bar_close_timestamp_ns` | Instante de cierre de la barra | `INTEGER` ns UTC | Obligatorio | Común |
| `bar_type` | Método de cierre: `TIME` / `TICK_COUNT` / `VOLUME` / `DOLLAR` / `RENKO` (`algorithmic-bars.md` ya cubre los no-temporales) | `TEXT` `CHECK (col IN (...))` | Obligatorio | Común |
| `timeframe_value` | Parámetro del `bar_type` (ej. `"1m"`, `"1000"` ticks, `"50000"` dólares) | `TEXT` | Obligatorio | Común |
| `open` / `high` / `low` / `close` | Primer, máximo, mínimo y último precio del intervalo | `INTEGER` ×10⁸ | Obligatorio | Común |
| `volume_traded` | Volumen real negociado en el intervalo, en unidades del activo base | `INTEGER` ×10⁸ | Opcional — obligatorio cuando la fuente lo distingue (ver §6.3) | Común |
| `volume_ticks` | Conteo de cambios de precio/cotización dentro del intervalo (proxy de actividad, NO volumen real — semántica de `tick_volume` de MT5) | `INTEGER` (contador, sin escala ×10⁸) | Opcional | Común |
| `trade_count` | Conteo de operaciones ejecutadas discretas dentro del intervalo (distinto de `volume_ticks`: cuenta fills, no cambios de cotización) | `INTEGER` (contador) | Opcional — Binance klines lo trae (`number_of_trades`) | Común |
| `volume_buy` | Volumen agredido por el lado comprador dentro del intervalo (solo si la fuente clasifica; `taker_buy_base_asset_volume` de Binance) | `INTEGER` ×10⁸ | Opcional | Común |
| `quote_asset_volume` | Volumen negociado expresado en la moneda de cotización (magnitud distinta de `volume_traded`, que está en el activo base) | `INTEGER` ×10⁸ | Opcional | Común |
| `spread_reported` | Spread que la fuente adjunta a la barra (MT5 `CopyRates.spread`) — semántica exacta (¿al abrir? ¿al cerrar? ¿promedio?) **sin verificar por bróker** | `INTEGER` ×10⁸ | Opcional | Forex/CFD |
| `vwap` | Precio promedio ponderado por volumen del intervalo | `INTEGER` ×10⁸ | Opcional — **solo si la fuente lo entrega**; nunca calculado y persistido por Drasus (ver §6.5) | Común |
| `open_interest` | Contratos abiertos no liquidados al cierre del intervalo | `INTEGER` ×10⁸ | Obligatorio para Futuros y Opciones, ausente en el resto | Futuros, Opciones |
| `settlement_price` | Precio de liquidación oficial del intervalo (distinto de `close`; ver ADR-0017 "Lógica de Settlement, Davey") | `INTEGER` ×10⁸ | Obligatorio para Futuros y Opciones | Futuros, Opciones |
| `implied_volatility_reported` | Volatilidad implícita tal como la reporta la fuente (fracción decimal, ej. 0.25 → `25000000`) | `INTEGER` ×10⁸ | Opcional — solo si la fuente la publica; convención de cálculo sin verificar (ver §7) | Opciones |
| `delta_reported` / `gamma_reported` / `theta_reported` / `vega_reported` / `rho_reported` | Sensibilidades tal como las reporta la fuente | `INTEGER` ×10⁸ | Opcional — mismo caveat que `implied_volatility_reported` | Opciones |
| `theoretical_price_reported` | Precio teórico de modelo que la fuente adjunta (no el `close` de mercado) | `INTEGER` ×10⁸ | Opcional | Opciones |

**Desviación explícita del molde ADR-0141:** dos timestamps (`bar_open_timestamp_ns` / `bar_close_timestamp_ns`) en vez del nombre singular `bar_timestamp_ns` que el ADR usa como ejemplo. Justificación: las barras algorítmicas (`algorithmic-bars.md`, Renko/volumen/dólar) tienen duración variable — sin el cierre explícito, dos barras consecutivas de duración distinta son indistinguibles en una consulta que solo indexe por apertura. Ambos campos siguen el molde `INTEGER` ns UTC sin excepción; el ADR-0006/0141 solo fija el tipo, no la cardinalidad de timestamps por fila.

**`contract_multiplier` y atributos estáticos de Opciones/Futuros:** deliberadamente NO están en esta tabla. `strike_price`, `expiration_date`, `option_type`, `option_style`, `underlying_instrument_id`, `contract_multiplier`, `occ_symbol` (contrato específico de futuro: `contract_code`, `contract_expiration_date`) identifican **qué instrumento es**, no un evento de mercado — pertenecen al registro de instrumento (`Instrument`, ADR-0140), creado una vez por contrato, referenciado por `instrument_id`. Repetirlos en cada fila de tick/barra es la misma duplicación de verdad que persistir un campo derivable (§6.5), solo que a nivel de identidad en vez de a nivel de cálculo.

---

## 6. Las cinco decisiones pedidas

### 6.1. El spread: ¿se persiste o se deriva?

**Decisión: se persisten `bid`/`ask` como obligatorios; `spread_reported` se persiste como columna opcional, poblada solo cuando la fuente entrega su propia columna de spread distinta de bid/ask.**

- El spread derivado (`ask − bid`) nunca se persiste como columna — se calcula en cualquier lectura, es determinista y gratis.
- `spread_reported` **no es el mismo dato** que `ask − bid`: MT5 reporta el spread del bróker en puntos, con redondeo y convenciones internas propias que pueden no coincidir bit a bit con la resta de los dos floats exportados. Persistirlo no duplica una verdad — captura una **segunda observación independiente** de la misma fuente.
- Esa segunda observación tiene un uso operativo concreto ya existente en el código: `data-sanitizer-pipeline.md` ya define `SPREAD_SIGMA_THRESHOLD` para alertar cuando el spread se desvía de forma anómala. Sin `spread_reported`, ese chequeo solo puede comparar la resta contra sí misma — no puede detectar el defecto exacto que motivó esta especificación (el spread fijo de StrategyQuant): un bróker que reporta un spread distinto del que sus propios bid/ask implican.
- **Reductio (comprobación #1):** si solo se persistiera `spread_reported` y se descartaran `bid`/`ask`, sería imposible reconstruir cualquier otra cosa (fills, mid-price, CVD). Si solo se persistieran `bid`/`ask` y se descartara `spread_reported`, sería imposible detectar cuándo el bróker reporta un spread que no coincide con su propia cotización — que es exactamente la anomalía que un motor de backtesting institucional necesita atrapar. Los dos extremos son degenerados; la posición correcta es capturar ambos con roles distintos.

### 6.2. Ticks de cotización frente a ticks de ejecución

Resuelto en §2: **dos tipos físicos**, no un tipo con banderas. Ver la justificación completa ahí.

### 6.3. Volumen: ¿de qué?

Tres magnitudes distintas, y confundirlas corrompe cualquier indicador de volumen — se persisten como columnas separadas, nunca colapsadas en un solo `volume`:

- **`volume_traded`** — volumen real negociado (unidades del activo base). Es la magnitud que un indicador de volumen institucional debe usar.
- **`volume_ticks`** — conteo de cambios de precio/cotización. Es un *proxy* de actividad, no de volumen. MT5 lo etiqueta `tick_volume` precisamente porque en MT4/MT5 legado, en ausencia de volumen real del bróker, esta era la única magnitud disponible y muchos usuarios la confundieron con volumen real durante años — el error que este esquema existe para prevenir.
- **`trade_count`** — conteo de operaciones ejecutadas discretas. Es sutilmente distinto de `volume_ticks`: un cambio de cotización (`volume_ticks`) puede no involucrar ninguna ejecución; una ejecución (`trade_count`) siempre representa un fill real. Tratarlos como sinónimos rompe cualquier métrica que dependa de "cuántas veces operó alguien" frente a "cuántas veces cambió el precio".
- **`volume_buy`** (y su complemento `volume_sell`, derivable — ver §6.5) — volumen clasificado por lado agresor, cuando la fuente lo clasifica en origen (Binance). Es una cuarta magnitud, no una variante de `volume_traded`.

**Regla de captura:** un campo se puebla solo si la fuente concreta lo distingue como tal. Si una fuente solo da un "volumen" sin aclarar cuál de los tres es, ese valor va en `volume_ticks` únicamente si la documentación de la fuente lo confirma como conteo de ticks, o se deja `NULL` en todos si la semántica es ambigua y no verificada — nunca se asume `volume_traded` por defecto.

### 6.4. Campos opcionales y particiones antiguas

- Todo campo marcado "Opcional" en §3 y §5 puede no existir en una fuente concreta desde el día uno. Bajo ADR-0141, eso es `NULL` explícito en las particiones donde la fuente no lo entrega, nunca error, nunca reescritura retroactiva.
- El esquema completo (incluidos los campos de Futuros y Opciones que hoy no tienen fuente construida — ver `plan-decision-fuentes-datos-mercado.md` §6.4) se canoniza **ahora**, aunque su población quede diferida. Es la aplicación directa de ADR-0020 §7.3: el costo de añadir la columna hoy es cero; el costo de añadirla dentro de dos años, cuando ya existan particiones sin ella, es una migración de esquema documentada con `schema_version` bump — evitable.
- Cada partición Parquet declara su `schema_version` (ADR-0141). Una consulta que dependa de `volume_buy` o `open_interest` debe verificar `schema_version` antes de asumir que el campo está poblado, no solo que la columna existe.

### 6.5. Qué NO entra al esquema (derivable sin pérdida = duplicación de verdad)

- **`spread` derivado** de `ask − bid` — se deriva en lectura (distinto de `spread_reported`, §6.1, que sí se persiste por ser una observación independiente).
- **`range` (high − low), `typical_price` ((H+L+C)/3), `mid_price` ((bid+ask)/2)** — aritmética trivial sobre columnas ya persistidas.
- **`vwap` calculado por Drasus** — si la fuente no lo entrega, se computa on-demand (`duckdb-resampler`, `temporal-aggregator`) y nunca se escribe de vuelta a la barra cruda. Solo se persiste cuando es la propia fuente quien lo reporta (magnitud distinta: el VWAP del vendor puede usar una ventana o convención propia no reproducible desde el tick).
- **`volume_sell`** cuando ya existen `volume_traded` y `volume_buy` — es el complemento aritmético (`volume_traded − volume_buy`); persistir los tres sería la misma clase de redundancia que el spread derivado, con la diferencia de que aquí no hay una segunda observación independiente que justifique la excepción del §6.1 (la fuente no reporta un `volume_sell` propio y distinto).
- **Etiqueta de sesión de mercado** (NY-AM, London-PM…) — ya resuelto en ADR-0141: se deriva en runtime desde el timestamp y el calendario, sin columna. Este esquema no reabre esa decisión, la hereda.
- **Precio de cierre ajustado por splits/dividendos (`adjusted_close`)** — **hallazgo de la comprobación #1 (límite degenerado):** si se persistiera como columna cruda, cada nuevo split o dividendo anunciado forzaría reescribir el factor de ajuste de **todas** las particiones históricas del instrumento — lo que ADR-0141 prohíbe explícitamente ("solo se añaden columnas; nunca se eliminan ni renombran… prohibido reescribir particiones históricas"). El primitivo correcto es el `close` crudo tal como lo imprimió el mercado, más un registro aparte de eventos corporativos (split ratio, monto de dividendo, fecha efectiva) con su propia cardinalidad (uno por evento, no uno por barra). El ajuste es una vista derivada que se calcula on-demand combinando ambos — nunca se pre-calcula y persiste por barra. Este esquema no define esa tabla de eventos corporativos hoy (no hay fuente de Acciones construida todavía); deja constancia del principio para que no se reinvente mal cuando llegue.
- **Contrato continuo / back-adjustado de futuros** — mismo argumento que el punto anterior, aplicado a futuros: el ajuste hacia atrás en cada rolo reescribe retroactivamente el histórico entero de la serie continua, lo que choca con append-only. El primitivo es el contrato específico (`ESZ26`, con su propio `instrument_id` y su propia fecha de vencimiento real) tal como cotizó; la política de "cómo encadenar contratos en una serie continua" es una feature futura de rolo, separada del ingest crudo (comprobación #5: separa el primitivo de la política).
- **Griegas/IV calculadas por Drasus** — si activa ADR-0140, el cálculo de sensibilidades a partir de un modelo de pricing (Black-Scholes, binomial) es responsabilidad de `option-pricing-engine`/`greeks-monitor` (moonshots ya declarados en ADR-0140), no de la capa de ingesta. Los campos `*_reported` de §5 capturan únicamente lo que la fuente ya trae calculado — no son el mismo dato que el que Drasus calculará después, y no deben confundirse ni fusionarse en la misma columna.

---

## 7. Opciones financieras — cobertura de esquema, no de implementación (ADR-0140)

ADR-0140 exige que el tipo `Instrument` reserve la variante `Option` sin sellar el conjunto, con costo de diseño cero. Este esquema cumple esa exigencia en la capa de datos de mercado:

- **Atributos estáticos del contrato** (van en el registro de instrumento, no en tick/barra — ver nota de cierre de §5): `strike_price` (`INTEGER` ×10⁸), `expiration_date` (`INTEGER` ns UTC), `option_type` (`TEXT CHECK (col IN ('CALL','PUT'))`), `option_style` (`TEXT CHECK (col IN ('AMERICAN','EUROPEAN','OTHER'))`), `underlying_instrument_id` (`TEXT`, FK), `contract_multiplier` (`INTEGER`, contador plano — típicamente 100, **no** es un precio ni un volumen y por tanto no lleva escala ×10⁸; se mapea a la fila "Contador / secuencia" del catálogo de tipos de ADR-0141), `occ_symbol` (`TEXT`, símbolo OSI/OCC de 21 caracteres: raíz de 6 + `YYMMDD` + `C`/`P` + strike ×1000 en 8 dígitos — formato verificado, ver fuentes §9).
- **Magnitudes que cambian con el tiempo** (van en `QuoteTick`/`TradeTick`/`Bar` de §3 y §5, apuntando a este `instrument_id`): `bid`/`ask`/`price`/volúmenes (mismos campos que cualquier otro instrumento — una opción cotiza y opera igual que cualquier activo), `open_interest`, `settlement_price`, `implied_volatility_reported`, `delta/gamma/theta/vega/rho_reported`, `theoretical_price_reported`.
- **Lo que este documento NO fija, y queda explícitamente pendiente:** la convención numérica exacta de cada sensibilidad reportada — theta por día calendario frente a por día de trading, vega por punto de volatilidad (1.0) frente a por 1%, delta como fracción frente a como porcentaje. Estas convenciones **varían por vendor** y no las verifiqué contra una fuente concreta porque ADR-0140 no tiene proveedor de opciones elegido todavía. Cuando se active el prerrequisito 4 de ADR-0140 (fuente de datos históricos de opciones identificada), el Quant-Engineer debe fijar la convención exacta contra esa fuente antes de poblar estas columnas — no antes, porque inventar la convención hoy sin fuente real sería alucinar un formato.
- **NPF (Net Premium Flow) y clasificación Lee-Ready de opciones** — ya diseñado en `order-flow-microstructure.md` como métrica derivada sobre `TradeTick`; no requiere columnas nuevas en este esquema, solo que `aggressor_side` y `price` de la ejecución existan (ya están en §3).

---

## 8. Order flow y microestructura — qué distingue Nivel 1, Nivel 2 y libro completo

Los tres niveles son primitivos de cardinalidad distinta — ninguno es un subconjunto de columnas del otro, cada uno necesita su propia tabla:

| Nivel | Qué captura | Cardinalidad | Primitivo |
|---|---|---|---|
| **Nivel 1 (BBO — Best Bid/Offer)** | Solo el mejor precio de compra y venta, con su tamaño agregado | 1 fila por actualización de cotización | Es exactamente `QuoteTick` (§3) — no necesita tabla nueva |
| **Nivel 2 (Market-by-Price / profundidad agregada)** | N niveles de precio por lado, con el tamaño total agregado de todas las órdenes en ese precio; algunos venues exponen además cuántas órdenes distintas componen ese agregado | 1 fila por (instantánea, lado, nivel) — muchas filas por cada instante de mercado | Tabla nueva: `DepthLevel` |
| **Nivel 3 (Market-by-Order / libro completo)** | Cada orden individual del libro, sin agregar — permite reconstruir el libro exacto en cualquier instante | 1 fila por evento de orden (alta/modificación/cancelación/ejecución) — la cardinalidad más alta de las tres | Tabla nueva: `OrderEvent` |

### Tabla de campos — `DepthLevel` (Nivel 2)

| Campo | Significado | Tipo canónico | Oblig./Opc. |
|---|---|---|---|
| `instrument_id` / `venue` / `event_timestamp_ns` | Igual que en Tick | ver §3 | Obligatorio |
| `book_snapshot_id` | Agrupa todas las filas (todos los niveles, ambos lados) que pertenecen a la misma instantánea del libro | `TEXT` (UUIDv7) | Obligatorio |
| `side` | Lado del libro | `TEXT CHECK (col IN ('BID','ASK'))` | Obligatorio |
| `level_index` | Posición del nivel, `0` = mejor precio | `INTEGER` (contador) | Obligatorio |
| `price` | Precio del nivel | `INTEGER` ×10⁸ | Obligatorio |
| `size` | Tamaño agregado de todas las órdenes en ese nivel | `INTEGER` ×10⁸ | Obligatorio |
| `order_count` | Número de órdenes distintas agregadas en ese nivel | `INTEGER` (contador) | Opcional — solo si el venue lo expone (no todos los feeds L2 lo hacen) |

### Tabla de campos — `OrderEvent` (Nivel 3)

| Campo | Significado | Tipo canónico | Oblig./Opc. |
|---|---|---|---|
| `instrument_id` / `venue` / `event_timestamp_ns` | Igual que en Tick | ver §3 | Obligatorio |
| `order_id` | Identificador de la orden individual en el venue | `TEXT` | Obligatorio |
| `side` | Lado de la orden | `TEXT CHECK (col IN ('BID','ASK'))` | Obligatorio |
| `event_type` | Qué pasó con la orden | `TEXT CHECK (col IN ('ADD','MODIFY','CANCEL','EXECUTE'))` | Obligatorio |
| `price` | Precio de la orden en el momento del evento | `INTEGER` ×10⁸ | Obligatorio |
| `size` | Tamaño de la orden en el momento del evento | `INTEGER` ×10⁸ | Obligatorio |
| `sequence` | Secuencia monótona del venue para detectar huecos (gap detection) | `INTEGER` (contador, `UNIQUE` por venue+instrumento) | Obligatorio |

`order-flow-microstructure.md` ya declara que la parte en vivo de L2 (OFI/DOM) es EPIC-6 y depende del feed de NautilusTrader. `DepthLevel` y `OrderEvent` quedan canonizados en este esquema (columna existe, ADR-0020 §7.3) aunque su población dependa de esa fase — coherente con la recomendación de diferir Nivel 2/3 histórico del plan de fuentes (`plan-decision-fuentes-datos-mercado.md` §6.5).

---

## 9. Anclaje empírico — formatos reales verificados

Cuatro fuentes reales, verificadas por búsqueda, no asumidas:

- **MetaTrader 5 — ticks:** exportación CSV con columnas `Fecha, Hora, Bid, Ask, Last, Volumen`; las banderas (`Flags`) las calcula el propio terminal al importar, no siempre están en el CSV exportado. [MQL5 forum — Flags en export CSV](https://www.mql5.com/en/forum/224009), [Exportar tick data de MT5](https://support.eareview.net/support/solutions/articles/19000064697-exporting-metatrader-5-tick-data).
- **MetaTrader 5 — barras:** `CopyRates` devuelve `time, open, high, low, close, tick_volume, spread, real_volume` — confirma la distinción entre `tick_volume` (conteo de cambios) y `real_volume` (volumen real) que este esquema adopta como `volume_ticks` y `volume_traded`. [MQL5 docs — copy_rates_from](https://www.mql5.com/en/docs/python_metatrader5/mt5copyratesfrom_py).
- **Dukascopy — ticks:** `timestamp, askPrice, bidPrice, askVolume, bidVolume` — confirma que compra/venta con volumen por lado es un formato real y no una invención de este documento. [Dukascopy — History ticks](https://www.dukascopy.com/wiki/en/development/strategy-api/historical-data/history-ticks/).
- **Interactive Brokers — ticks:** `BID_ASK` devuelve `time, priceBid, priceAsk, sizeBid, sizeAsk, atributos`; `TRADES` devuelve `time, price, size, exchange, condiciones especiales, atributos` — confirma la separación física cotización/ejecución de §2 en un feed institucional real, y respalda `exchange_code`/`trade_conditions` de §3. [IB TWS API — Tick-by-Tick Data](https://interactivebrokers.github.io/tws-api/tick_data.html).
- **Binance — barras (`klines`):** `open_time, open, high, low, close, volume, close_time, quote_asset_volume, number_of_trades, taker_buy_base_asset_volume, taker_buy_quote_asset_volume` — anclaje directo de `trade_count`, `quote_asset_volume` y `volume_buy` de §5; esta es además la fuente ya cableada en `sovereign-data-fetcher`. [Binance — Kline/Candlestick Data](https://developers.binance.com/docs/derivatives/usds-margined-futures/market-data/rest-api/Kline-Candlestick-Data).
- **Binance — ejecuciones agregadas (`aggTrades`):** `a` (id), `p` (precio), `q` (cantidad), `f`/`l` (primer/último trade id), `T` (timestamp), `m` (¿el comprador fue el maker?) — anclaje directo de `aggressor_side` (invertir `m` da el lado agresor). [Binance — Aggregate Trades](https://developers.binance.com/docs/alpha/market-data/rest-api/aggregated-trades).
- **OCC/OSI — símbolo de opciones:** raíz (6 car.) + `YYMMDD` + `C`/`P` + strike ×1000 en 8 dígitos, 21 caracteres totales. Ejemplo verificado: `AAPL230616C00150000`. [Fidelity — OSI](https://www.fidelity.com/webcontent/ap102701-quotes-content/16.10/shtml/osi.shtml), [Wikipedia — Option symbol](https://en.wikipedia.org/wiki/Option_symbol).
- **Campos sin verificar, marcados explícitamente en el cuerpo del documento:** semántica exacta de `spread` en `CopyRates` de MT5 (¿instante de apertura, cierre, o último conocido? — no lo confirmé contra documentación oficial de MetaQuotes); convención numérica de griegas/IV por vendor (§7); vocabulario exacto de códigos de `trade_conditions` en el tape consolidado de EE. UU. (SIP) — no hay fuente de Acciones construida todavía contra la cual verificarlo.

---

## 10. Comprobaciones de razonamiento crítico de dominio

Aplicadas según `.agents/knowledge/critical-domain-reasoning.md` — las siete, con constancia:

1. **Límite degenerado (obligatoria).** Aplicado dos veces: `adjusted_close` empujado al límite de "N splits futuros" fuerza reescritura retroactiva infinita de particiones append-only (§6.5); el contrato continuo de futuros empujado al límite de "N rolos futuros" tiene el mismo defecto. Ambos se resuelven igual: el primitivo es el evento/contrato crudo, el ajuste es una vista derivada nunca persistida.
2. **Gate de especialista.** Este documento ES el gate de especialista para Q3/Q5 del plan de fuentes del Architect — declarado en el encabezado, no salteado.
3. **Desconfianza del rigor impresionante.** La afirmación "el spread se deriva de bid−ask, persistirlo es redundante" suena rigurosa y es **parcialmente falsa**: es cierta para el spread derivado, falsa para `spread_reported` como observación independiente del bróker (§6.1). Tratarla como verdad completa habría eliminado la única columna capaz de detectar el defecto de StrategyQuant que originó este encargo.
4. **Re-derivación de supuestos.** El supuesto implícito de "un tick es un evento" se rompió al verificar que MT5 mezcla dos eventos distintos (cotización y ejecución) en el mismo archivo con un bitmask — de ahí la decisión de §2.
5. **Separar primitivo de política.** Aplicado tres veces: contrato continuo de futuros (§6.5), precio ajustado de acciones (§6.5), griegas calculadas por Drasus frente a griegas reportadas por la fuente (§6.5/§7).
6. **Cierre transitivo.** El concepto "qué instrumento es" (strike, vencimiento, multiplicador, contrato específico) toca tick, barra, Nivel 2 y Nivel 3 por igual — de ahí que se extrajera una vez al registro de instrumento en vez de repetirse en cada tabla (nota de cierre de §5).
7. **Distinguir tipos, no promediar.** Aplicado en §2 (cotización ≠ ejecución), §6.3 (volumen real ≠ volumen de ticks ≠ conteo de trades ≠ volumen por lado agresor) y §8 (Nivel 1 ≠ Nivel 2 ≠ Nivel 3, tres cardinalidades distintas, tres tablas distintas).

---

## 11. Qué canonizar ya frente a qué puede esperar

### Canonizar ya (bloquea Q3 del plan de fuentes; costo de diseño cero)

- El esquema completo de `QuoteTick` y `TradeTick` (§3), incluida la decisión de dos tipos físicos (§2).
- El esquema completo de `Bar` (§5), incluido el desglose de volumen en cuatro magnitudes (§6.3) aunque hoy solo Binance pueble `trade_count`/`quote_asset_volume`/`volume_buy`.
- Las columnas de Futuros y Opciones en `Bar` (`open_interest`, `settlement_price`, campos `*_reported`) y las tablas `DepthLevel`/`OrderEvent` — se definen ahora, se pueblan cuando exista fuente (Fase D del plan de fuentes / EPIC-6).
- Los atributos estáticos de instrumento para Opciones (§7) y el `contract_code`/`contract_expiration_date` de Futuros — el tipo `Instrument` de ADR-0140 debe reservarles espacio ya, sin sellar el enum.
- Los cinco principios de "qué NO entra" (§6.5) — se documentan ahora para que ninguna feature futura reinvente mal `adjusted_close` o el contrato continuo cuando llegue Acciones o Futuros.

### Puede esperar (requiere fuente real o decisión de negocio previa)

- Poblar `open_interest`/`settlement_price`/`DepthLevel`/`OrderEvent` con datos reales — depende del proveedor de Futuros (Databento, diferido en el plan de fuentes §6.4) y del feed L2/L3 de NautilusTrader (EPIC-6).
- Fijar la convención numérica exacta de griegas/IV (§7) — depende de que ADR-0140 tenga proveedor de opciones elegido (prerrequisito 4); fijarla hoy sin fuente sería inventar un formato.
- El vocabulario exacto de `trade_conditions` para Acciones (códigos SIP) — depende de que exista fuente de Acciones contra la cual verificar, hoy no construida.
- La tabla de eventos corporativos (splits/dividendos) — se diseña cuando Acciones entre en alcance; el principio que la gobierna (nunca persistir `adjusted_close`) ya queda fijado en §6.5 para que no se decida mal por prisa cuando llegue el momento.
