# TASK-064 · Política de spread nulo/cero, selector de fuente de spread, anclaje DST y plan de validación (CSV M1 EURUSD 25 años)

| Campo | Valor |
|---|---|
| **ID** | TASK-064 |
| **Tipo** | Decisión de arquitectura + doctrina de dominio + plan de validación |
| **Estado** | Abierta |
| **Abierta** | 2026-07-22 |
| **Origen** | Directiva del propietario, sesión 2026-07-22 — decisión ya validada por el Tech-Lead como doctrinalmente compatible |
| **Corrección** | 2026-07-23 (propietario + Tech-Lead) — ver banner abajo: se elimina la dependencia del dataset de abril 2025 para calibrar, y el método de anclaje DST por noticia/volatilidad. Ambos eran defectos inducidos por el despacho, no del dominio. |

> 🔴 **CORRECCIÓN DE FONDO (2026-07-23).** La primera redacción ató dos decisiones a insumos frágiles porque el despacho del Tech-Lead los precocinó. Se corrigen aquí:
> 1. **El piso de spread (§1.2) NO se calibra contra abril 2025.** Se calibra **por instrumento/símbolo**, desde dos fuentes que existen siempre: el `default_spread` declarado del instrumento en el catálogo (TASK-065) y la distribución empírica de los spreads no-cero del propio archivo. El mes de abril 2025 pasa a ser una **comprobación opcional de una sola vez**, de la que el sistema **no depende** — si el propietario borra ese dataset, nada se rompe.
> 2. **El anclaje DST (§3) NO usa la noticia de Nóminas ni el pico de volatilidad.** La zona horaria y su regla de DST **se declaran al importar** (selector de zona mundial con bandera DST por zona, patrón que SQX ya usa), nunca se infieren de la volatilidad. El hueco de fin de semana queda solo como verificación estructural opcional.
| **Destinatarios** | Quant-Engineer (Etapa 0.3, este documento) → Architect (Etapa 0.4, sellar enmienda) → **propietario (aprobación final del anclaje DST, §3.5)** |
| **Dominio** | Ingesta de precio/spread para backtesting — `data-import-wizard`, `data-sanitizer-pipeline`, motor de ejecución simulada (ADR-0114) |
| **Relacionadas** | ADR-0114 (regla c, Bid/Ask Exclusivo), ADR-0155 (`spread_reported`, `Bars`), `docs/features/slippage-models.md`, `docs/features/institutional-friction-modeling.md`, `docs/features/data-sanitizer-pipeline.md` (Capa 5), `docs/features/algorithmic-bars.md`, TASK-059 (contrato de momento de conocibilidad), TASK-060 (muestra real, bloqueada en el propietario), TASK-063/ADR-0156 (formato de ingesta) |

---

## 0. Búsqueda previa (Gate Doc-First) y clasificación de cada hallazgo

Búsqueda realizada antes de proponer nada nuevo: `grep -rniE "spread.{0,20}(nulo|null|cero|piso|floor)" docs/` → **0 coincidencias**. Lectura completa de `docs/adr/ADR-0114.md`, `docs/adr/ADR-0155.md`, `docs/features/slippage-models.md`, `docs/features/institutional-friction-modeling.md`, `docs/features/data-sanitizer-pipeline.md`, `docs/features/data-import-wizard.md`, `docs/adr/ADR-0156.md`.

| Punto de esta Task | Clasificación | Razón |
|---|---|---|
| §1 Política spread nulo/cero | **Vacío real** | Ninguno de los documentos leídos define esta política. `slippage-models.md` tiene spread dinámico por asset/broker (impacto de mercado, ATR) — dominio distinto: eso ajusta el fill *después* de tener un spread confiable, no decide qué hacer cuando el dato de origen falta o es sospechoso. `data-sanitizer-pipeline.md` Capa 5 compara `spread_reported` contra `ask − bid` **cuando existe `QuoteTick` nativo** — el CSV M1 no lo tiene, así que Capa 5 no puede correr aquí (ver §4). |
| §2 Selector de fuente de spread | **Vacío real** | Ningún parámetro configurable de este tipo existe en `slippage-models.md` ni en ningún otro feature doc leído. |
| §3 Anclaje DST | **Parcialmente resuelto + vacío real.** El **destino** ya está resuelto: todo se persiste como `event_timestamp_ns` en nanosegundos UTC (ADR-0141/ADR-0155) — STORY-051 lo verificó explícitamente como "agnóstico al calendario... sin deriva por horario de verano". El **vacío** es el **origen**: nada en `docs/` dice cómo determinar el offset UTC real de un CSV crudo sin metadato de huso horario explícito. |
| §4 Plan de validación abril 2025 | **Vacío real, con anclaje parcial.** ADR-0155 ya deja constancia de que la semántica exacta de `spread_reported` en `Bars` está **"sin verificar por bróker"** — este plan es exactamente el mecanismo que cierra esa nota, no una tarea inventada desde cero. |

---

## 1. Política de spread nulo vs. spread cero

**Son dos casos distintos y no se tratan igual** (comprobación 7, `critical-domain-reasoning.md` — distinguir tipos, no promediar).

### 1.1 Spread NULO (campo ausente)

**Recomendación: forward-fill acotado, con el mismo patrón ya existente en `data-sanitizer-pipeline.md` Capa 2 (Gap Infusion), no uno nuevo.**

- El sanitizador ya resuelve exactamente este problema para OHLC: gaps ≤ `MAX_GAP_FILL_LIMIT` (default 5 barras) se interpolan; gaps mayores se marcan `INVALID`. Para spread la interpolación lineal **no aplica** (el spread no es un precio continuo entre dos puntos conocidos — es una cotización discreta de un mercado que puede saltar por una noticia). Lo correcto es **carry-forward** (LOCF, *last observation carried forward*): arrastrar el último spread realmente observado.
- Por qué carry-forward y no promedio de N barras atrás: un promedio fabrica un número que **nunca existió** en ningún instante — viola el principio "guarda el hecho, calcula la interpretación" de ADR-0155 en su otra cara (`spread_reported` es una *observación*, no algo para sintetizar). Carry-forward reutiliza un hecho real ya observado; es la misma lógica que ya usa la Capa 2 para gaps de OHLC.
- **Reductio (comprobación 1):** si el hueco es de 1 barra, carry-forward es trivial y correcto. Si el hueco es sistemático y largo (ej. spread ausente en el 100% de las barras de un tramo de varios días), arrastrar indefinidamente un spread capturado en condiciones de mercado que ya no aplican (ej. baja volatilidad) hacia un tramo de alta volatilidad real subestima el riesgo — exactamente el optimismo sistemático que ADR-0114 regla (c) existe para prohibir. **Por eso el forward-fill de spread debe estar acotado por el mismo umbral (`MAX_GAP_FILL_LIMIT` o un parámetro hermano, ver §1.3) que ya gobierna los gaps de OHLC**; más allá del umbral, el tramo se marca `INVALID` para backtesting, igual que ya hace la Capa 2. No se inventa un mecanismo nuevo — se extiende el que ya existe al campo `spread_reported`.

### 1.2 Spread CERO (valor `0` explícito)

**Recomendación: no confiar en `0` a ciegas. Aplicar un piso calibrado POR INSTRUMENTO desde dos fuentes que existen siempre — el `default_spread` declarado del instrumento (catálogo, TASK-065) y la distribución empírica de los spreads no-cero del propio archivo — nunca desde un dataset externo fugaz ni una constante de biblioteca.**

- **Reductio (comprobación 1):** EURUSD es el par más líquido del mundo — un spread de 0 puntos ocurre de verdad, brevemente, en solapes de sesión de máxima liquidez (Londres/Nueva York) en brókers ECN Tier-1. Pero un CSV M1 de 25 años exportado de MetaTrader es exactamente el tipo de fuente con historial de reportar `spread=0` como valor por defecto en tramos antiguos donde la terminal no tenía granularidad de cotización para poblar el campo (defecto de exportador conocido en la plataforma, no una propiedad del mercado). **Si se empuja al límite** (una fracción no trivial de las ~13 millones de barras M1 de 25 años trae `0` por defecto de exportador, no por mercado real) y el sistema confía en ese `0` literalmente, el resultado es un backtest de 25 años operando parcialmente "gratis" — el defecto fundacional exacto que ADR-0114 regla (c) fue escrita para impedir. El modelo que confía en `0` sin verificar no sobrevive esta prueba.
- **Por qué no un piso fijo arbitrario tampoco:** un piso inventado sin evidencia (ej. "siempre mínimo 2 puntos" cableado en la librería) sería el mismo tipo de alucinación de cifra que este documento tiene prohibido cometer — sustituye un hecho no verificado por *otro* hecho no verificado, solo que pesimista en vez de optimista.
- **La solución correcta es empírica y POR INSTRUMENTO — no la misma para todos, no un dataset externo (corrección del propietario, 2026-07-23):** el piso de un `0` sospechoso sale de, en este orden de preferencia:
  1. **El `default_spread` declarado de ese instrumento/bróker** en el catálogo `broker_instrument_terms` (TASK-065). Es el valor que el propietario declara al importar — declarado, no adivinado (misma doctrina que ADR-0156). Un cripto, un índice, un futuro y un EURUSD tienen pisos distintos porque son instrumentos distintos: **el mismo piso para todos sería el error que el propietario señaló.**
  2. **La distribución de los spreads no-cero del propio archivo** (ej. percentil bajo robusto de los spreads reales del mismo instrumento) como calibración autocontenida y como rango de sanidad del valor declarado. Se autocalibra por dataset, sin depender de ningún otro archivo.
- **Abril 2025 NO es la fuente de calibración** — es, como mucho, una comprobación opcional de una sola vez (§4) de la que el sistema no depende. Si ese dataset se borra, la política de §1.2 sigue en pie completa.
- **Referencia de industria como sanidad, NO canon:** las cifras de SQX que citó el propietario ("Default spread" — Darwinex AUDCAD=2 pips, Dukascopy EURUSD=0.3 pips) sirven de rango de plausibilidad al declarar el `default_spread` del instrumento, no como valor cableado.

### 1.3 Parámetro que gobierna el forward-fill de spread

Propongo un parámetro hermano de `MAX_GAP_FILL_LIMIT` (Capa 2, `data-sanitizer-pipeline.md`), con nombre propio porque el spread y el OHLC pueden tener distinta tolerancia a estar desactualizados: `SPREAD_GAP_FILL_LIMIT` (CONFIG, default sugerido = mismo valor que `MAX_GAP_FILL_LIMIT`, 5 barras, hasta que haya evidencia de que deben divergir). Más allá del umbral, el tramo se marca `INVALID` para ese campo — el resto de la barra (OHLC) puede seguir siendo válido si su propio gap no excede su propio umbral; son dos relojes de vencimiento independientes sobre la misma fila.

---

## 2. Selector configurable del simulador — contrato

Tres modos, **CONFIG** (no FIJO — es "Umbral/Modo configurable", no un invariante físico, doctrina de CLAUDE.md §1):

| Parámetro | Tipo | Default propuesto | Qué hace |
|---|---|---|---|
| `SPREAD_SOURCE_MODE` | `enum { REPORTED_REAL, RANDOM_RANGE, FIXED }` | `REPORTED_REAL` | Selecciona de dónde sale el spread que el motor suma a `bid` para reconstruir `ask` (ADR-0114 regla c). Default en `REPORTED_REAL` porque Foundation Inundation (ADR-0020 §7.3, "ante duda genuina, incluir") + el principio de ADR-0155 mandan preferir el dato observado sobre el inventado siempre que exista. |
| `SPREAD_RANDOM_MIN` / `SPREAD_RANDOM_MAX` | `INTEGER` (puntos) | sin default — obligatorio si `RANDOM_RANGE` | Rango de muestreo uniforme por barra/operación. Solo aplica en modo `RANDOM_RANGE`. |
| `SPREAD_RANDOM_SEED` | `INTEGER` | sin default — obligatorio si `RANDOM_RANGE` | Semilla del generador. **No es opcional**: sin semilla fija, dos corridas del mismo backtest producen resultados distintos, violando el invariante de reproducibilidad ya establecido para `clock.md` ("misma entrada + mismo reloj = mismo output, siempre") y el precedente ya escrito en `institutional-friction-modeling.md` TTR-001 ("usa una semilla aleatoria (configurable)"). Mismo patrón, mismo motivo. |
| `SPREAD_FIXED_VALUE` | `INTEGER` (puntos) | sin default — obligatorio si `FIXED`, y también sirve como piso de §1.2 | Valor constante. Precedente directo: el campo "Default spread" de SQX que el propietario citó (Darwinex AUDCAD=2 pips, Dukascopy EURUSD=0.3 pips) — mismo concepto, a nivel de instrumento. |

### 2.1 Interacción con la política de nulo/cero de §1 (comprobación 5, separar primitivo de política)

El selector es el **primitivo** (de dónde sale el número); la política de §1 es la **política**, y solo aplica dentro de un modo concreto:

- **`REPORTED_REAL`:** la política de §1 completa (forward-fill acotado para NULO, piso calibrado para CERO) se ejecuta **dentro** de este modo. Es el único modo que lee el archivo.
- **`RANDOM_RANGE` / `FIXED`:** ignoran por completo el campo `spread_reported` del archivo — la política de §1 no aplica, porque no hay lectura del dato real que validar. `SPREAD_FIXED_VALUE` puede además reutilizarse como el piso de §1.2 cuando `REPORTED_REAL` cae en el caso degenerado "cero sin evidencia de ser real" — un solo campo, dos usos, en vez de duplicar el número en el contrato.

### 2.2 Cierre transitivo (comprobación 6)

El modo de spread NO relaja la regla (c) de ADR-0114 (Bid/Ask Exclusivo) en ningún caso: incluso en `RANDOM_RANGE`/`FIXED`, el motor sigue leyendo `bid` para vender y `ask = bid + spread_elegido` para comprar — nunca un punto medio. El selector cambia **de dónde sale el spread**, nunca cambia **que el spread siempre se suma al bid para formar el ask de compra**. Esto también es la razón por la que el "Filtro de Liquidez Pardo" (`slippage-models.md`) y el "Probabilistic Fill Engine" (`institutional-friction-modeling.md`) son ortogonales a este selector: friccionan la *cantidad ejecutable* y la *probabilidad de fill*, nunca el *precio* que ya fijó la doctrina bid/ask.

---

## 3. Horario de verano (DST) — explicación y método de anclaje

### 3.1 Qué es DST, en términos llanos

Muchos países adelantan el reloj 1 hora en primavera y lo atrasan 1 hora en otoño ("horario de verano") para aprovechar más luz de día. El problema para datos históricos: **Estados Unidos y Europa NO cambian de horario en la misma fecha del calendario.** Estados Unidos cambia el segundo domingo de marzo / primer domingo de noviembre; Europa cambia el último domingo de marzo / último domingo de octubre. Hay una ventana de 1 a 3 semanas cada primavera y cada otoño donde ambos continentes están "desfasados" un cambio de hora entre sí.

### 3.2 Por qué la etiqueta de Dukascopy que citó SQX es confusa a propósito de esa diferencia

La captura de SQX describe el dato de Dukascopy como **"(EST+07) New York hours, US DST"**. Decodificado:

- `EST` = hora estándar de Nueva York (UTC−5, invierno). `EDT` = hora de verano de Nueva York (UTC−4).
- `EST+07` significa literalmente **"toma la hora de Nueva York y súmale 7 horas, siempre"** — no es una zona horaria con nombre propio, es una regla aritmética sobre otra zona.
- En invierno: UTC−5 + 7 = **UTC+2**. En verano: UTC−4 + 7 = **UTC+3**.
- Numéricamente, UTC+2/UTC+3 **coincide** con EET/EEST (hora de Europa del Este) — pero es una coincidencia de magnitud, **no** identidad de regla.
- La diferencia crítica: el salto de 1 hora en esta convención ocurre **en la fecha en que Nueva York cambia de horario** (porque la regla está definida en función de la hora de Nueva York), no en la fecha en que Europa cambia. Eso es literalmente lo que dice la etiqueta ("US DST") y es la fuente exacta de la confusión del propietario.

**Consecuencia práctica:** si el pipeline de ingesta etiqueta ingenuamente este dato como "EET" y deja que una librería de zonas horarias aplique las reglas de DST de la Unión Europea, el salto de hora caerá en la fecha equivocada durante esas 1–3 semanas de desfase cada primavera y cada otoño — desplazando **todas** las barras de ese tramo por 1 hora completa, en silencio, sin error, sin advertencia. Es exactamente la clase de sesgo que TASK-059 ya documentó para `bar_timestamp_ns` vs. `bar_close_timestamp_ns`: "los resultados salen mejores o distintos, no peores de forma obvia — nadie sospecha".

**Reductio (comprobación 1):** una ventana de desfase de ~1–3 semanas por transición, 2 transiciones al año, sobre 25 años de historia, es del orden de **1 a 2 años de barras potencialmente mal ancladas en 1 hora** si el offset se infiere mal y no se corrige. No es una anécdota marginal — es una fracción del dataset lo bastante grande para sesgar sistemáticamente cualquier estrategia sensible a la hora del día (sesiones, aperturas, noticias).

### 3.3 Método correcto — zona horaria DECLARADA, no inferida (corrección del propietario, 2026-07-23)

El método de anclaje por noticia (Nóminas) cruzada con el pico de volatilidad **queda descartado**: inferir la hora de un archivo a partir de la volatilidad de una vela es ruidoso y frágil. La zona horaria **se declara**, siguiendo el patrón que SQX ya usa y que el propietario confirmó:

1. **La zona horaria y su regla de DST se declaran al importar.** El asistente ofrece un **selector de zona horaria mundial**, y cada zona trae su propia bandera de horario de verano (sí/no) — exactamente como el desplegable de SQX (ítem literal: *"(EST+07) New York Trading Hours, US DST, DST: Yes"*). El usuario elige la zona del dato; el sistema adopta la regla de DST estándar de esa zona (base de datos tz), sin calcular nada. Para orígenes que ya declaran su zona en el metadato de exportación (SQX/QDM la traen), se **lee**, no se pregunta.
2. **El desfase queda como dato declarado y aprobado (§3.5), no como una inferencia del sistema.** Esto es la misma doctrina de "mapeo declarado = evidencia inmutable" de ADR-0156/TASK-063, extendida a la zona horaria: el usuario declara, el sistema registra, la responsabilidad es del usuario.
3. **Verificación estructural OPCIONAL — el hueco de fin de semana, no la volatilidad.** Si se quiere cruzar la declaración contra el propio archivo, la señal robusta es puramente estructural: dónde empieza y termina el hueco semanal (presencia/ausencia de velas, no magnitud). El borde del hueco en la hora local del archivo confirma el offset, y si ese borde se corre 1 hora en la fecha de cambio de EE. UU. vs. la de Europa, eso revela qué régimen de DST rige — sin depender de ninguna noticia ni pico. Es una comprobación de sanidad, no la fuente de verdad.

### 3.4 Dónde vive esto en el código

Este anclaje resuelve el offset del **origen** (cómo interpretar los timestamps crudos del CSV); el **destino** ya está resuelto y no cambia — todo se convierte y persiste como `event_timestamp_ns` en nanosegundos UTC absolutos (ADR-0141/ADR-0155), exactamente como STORY-051 ya lo verificó para `hybrid-data-transformer`. Este trabajo es de `data-import-wizard`/`data-normalization-layer`, en la conversión de entrada, no un cambio a la convención de persistencia.

### 3.5 Aprobación obligatoria del propietario — responsabilidad explícita

**Esta inferencia es una propuesta metodológica, no una regla sellada.** El offset UTC final que se aplique a los 25 años del CSV **debe ser confirmado explícitamente por el propietario** antes de canonizarse — él puede conocer directamente el broker/exportador de origen y corregir esta inferencia sin necesidad de correr el procedimiento de §3.3. Al aprobar (o corregir) el offset, el propietario asume la decisión de forma expresa y auditable — mismo patrón de responsabilidad que ADR-0156/LDR-0008 ya establecieron para el mapeo declarado de columnas: una interpretación del dato que nadie validó no se canoniza como hecho.

---

## 4. Plan de validación — mes solapado de abril 2025 (ticks reales bid/ask)

### 4.1 Objetivo

Validar simultáneamente **dos hipótesis con un solo experimento**: (a) el modelo de fill `open = bid`, `ask = open + spread_reported`, y (b) que `spread_reported` en el CSV M1 realmente representa el spread vigente al **open** de la barra (no al cierre ni un promedio — riesgo que ADR-0155 ya deja anotado como "semántica exacta sin verificar por bróker").

**Naturaleza de este plan (corrección 2026-07-23):** es una comprobación de confianza **OPCIONAL y de una sola vez**, NO una dependencia de ninguna política de este documento. La política de spread (§1) y el anclaje DST declarado (§3) están completos sin él. Si el propietario borra el dataset de abril 2025, nada de lo canonizado aquí se rompe — solo se pierde este chequeo puntual de confianza sobre el modelo de fill. No se calibra nada contra este mes.

**Precondición (solo si se corre):** el offset DST de §3 debe estar resuelto y aprobado antes — comparar el minuto equivocado del tick contra el minuto equivocado de la barra invalidaría el resultado sin que nada lo advierta (mismo tipo de error silencioso que §3.2 describe).

### 4.2 Procedimiento

1. Tomar los ticks reales bid/ask de abril 2025 y las barras M1 del CSV para el mismo mes, ambos ya convertidos a `event_timestamp_ns` UTC.
2. Para cada barra M1: localizar el **último tick con timestamp ≤ `bar_open_timestamp_ns`** (nunca un tick posterior — usar "más cercano por distancia absoluta" violaría causalidad/PIT si el tick elegido cae después del open; regla explícita para quien implemente el validador).
3. Calcular `open_implicado_ask = bar.open + bar.spread_reported` y compararlo contra el `ask` real de ese tick. Registrar el residual (`ask_real − open_implicado_ask`) por barra.
4. Calcular la distribución del residual (media, mediana, desviación estándar, máximo, y sesgo direccional — ¿el error es simétrico o sistemáticamente hacia un lado?).
5. **Criterio de aceptación:** el CSV reporta spread en puntos enteros (resolución finita) — no se espera coincidencia exacta, se espera que el residual quede dentro de ±1 punto (la resolución de la columna) **sin sesgo direccional sistemático**. Un sesgo sistemático y no aleatorio es la señal de que algo estructural está mal, no ruido de redondeo — diagnosticar antes de aceptar:
   - Si el sesgo es constante y pequeño → candidato: el DST sigue ligeramente desalineado (revisar §3 primero).
   - Si el sesgo crece con la volatilidad de la barra → candidato: `spread_reported` no es el spread al open sino al cierre o un promedio de la barra (la nota ya anotada en ADR-0155 como sin verificar) — en ese caso, la doctrina D3/ADR-0114 regla (c) seguiría siendo correcta en el fondo, pero el campo a sumar tendría que re-especificarse (ej. usar el spread de la barra N−1 en vez de la N, si el reportado es de cierre).
6. Repetir el mismo procedimiento con el `bid` de la barra contra el `bid` real del tick (valida la hipótesis "open de barra MT5 = bid", independiente de la hipótesis del spread).
7. Usar el mismo mes para medir cuánto se mueve el bid/ask **dentro** de un mismo minuto M1 — esto no valida el modelo de fill, pero cuantifica cuánta precisión se pierde al usar alineación a apertura de barra (ADR-0114 regla a) en vez de tick real, insumo directo para el KPI de paridad simulación/real (SKILL.md §3).

### 4.3 Subproducto — construir otros tipos de barra sobre el mismo mes como prueba

`algorithmic-bars.md` tiene `TICK_COUNT`/`VOLUME`/`RENKO`/`RANGE` especificados pero **sin implementación ni fixture de ticks reales para probarlos**. Abril 2025 es, hasta hoy, el único mes con ticks reales disponible en el dominio Forex/CFD — es candidato natural a **oracle test fijo** (dataset congelado, resultado reproducible) para la futura Story de `algorithmic-bars`, evitando que esa implementación se apoye en datos sintéticos inventados para su primer test. Esto es una recomendación de reutilización, no una decisión de esta Task — la implementación de `algorithmic-bars` sigue bloqueada por su propia secuencia (TASK-061 para `DOLLAR`, sin bloqueo para los demás).

### 4.4 Salida esperada de este plan

Un veredicto reproducible: **APTO** (el modelo de fill se confirma dentro de tolerancia, sin sesgo) o **APTO-CON-CORRECCIÓN** (se identifica y corrige la semántica exacta de `spread_reported` — ej. "es spread de cierre, no de apertura" — antes de canonizar el modelo para los 25 años completos donde no hay ticks para verificar). No se anticipa aquí cuál de los dos resultados saldrá — sería alucinar un resultado sin haber corrido el experimento.

---

## 5. Enmienda propuesta a ADR-0114 / ADR-0155 (para que el Architect selle o corrija)

**No canonizo esto — lo propongo, con las 7 comprobaciones ya corridas arriba, para que el Architect decida dónde y cómo sellarlo (Mandato Documental, `.agents/skills/quant-engineer/SKILL.md` §1a).**

### 5.1 ADR-0114 — sub-cláusula a la regla (c) Bid/Ask Exclusivo

La regla (c) actual asume que `bid`/`ask` ya están dados como campos poblados. No cubre el caso de **reconstrucción** desde `spread_reported` (fuentes sin bid/ask nativo, como el CSV M1 de MetaTrader). Texto propuesto a insertar en la regla (c):

> *"Cuando el motor reconstruye `ask` a partir de `bid + spread_reported` (fuente sin cotización nativa de compra/venta), el `spread_reported` usado debe haber pasado primero por la Política de Integridad de Spread (nulo → forward-fill acotado; cero → piso calibrado POR INSTRUMENTO desde el `default_spread` declarado del catálogo y la distribución de spreads no-cero del propio archivo, nunca aceptado como `0` sin validar ni sustituido por una constante cableada) — sumar un spread crudo sin pasar por esa política es la misma clase de optimismo sistemático que esta regla ya prohíbe para el punto medio."*

### 5.2 ADR-0155 — nota sobre `spread_reported` en `Bars`

La tabla de `Bars` ya anota: *"semántica exacta sin verificar por bróker"*. Propongo que esa nota se resuelva —no se borre— citando este plan de validación (§4) como el mecanismo formal para cerrarla por broker/vendor, y que el resultado (spread al open/close/promedio, confirmado o corregido) se registre como un atributo del `Instrument` o de la fuente, no se asuma igual para todo proveedor futuro.

---

## 6. Preguntas abiertas para el Architect

1. **Ubicación de la Política de Integridad de Spread (§1):** ¿vive como extensión de `data-sanitizer-pipeline.md` (Capa 5, ya dueña de la comparación de spread) o merece su propia sub-sección en `data-import-wizard.md` (más cerca del punto de ingesta del CSV M1, antes de que exista un `QuoteTick` nativo contra el cual comparar)? Ambas encajan; la elección es de organización documental, no de sustancia.
2. **`SPREAD_GAP_FILL_LIMIT` (§1.3):** ¿parámetro propio o se reutiliza literalmente `MAX_GAP_FILL_LIMIT` de la Capa 2? Recomiendo propio (spread y OHLC pueden divergir en tolerancia), pero es una decisión de superficie de configuración, no de dominio cuantitativo.
3. **Alcance del selector de §2:** ¿es un parámetro por backtest, por instrumento, o ambos (nivel backtest anula nivel instrumento)? El precedente SQX citado por el propietario ("Default spread" vive en el instrumento) sugiere que al menos `SPREAD_FIXED_VALUE`/piso debería poder declararse a nivel `Instrument` como fallback, con el backtest pudiendo sobreescribirlo — decisión de arquitectura de configuración, no de estadística.
4. **Redacción final de la enmienda §5:** el texto propuesto es un borrador funcional, no prosa canónica — el Architect ajusta estilo/ubicación exacta siguiendo el patrón ya usado para la regla (c) original (TASK-063).

---

## 7. Conocimiento consultado (Gate de Reportaje)

`.agents/knowledge/governance.md` (gobernanza siempre activa) · `.agents/knowledge/critical-domain-reasoning.md` (las 7 comprobaciones, aplicadas en §1.1/§1.2/§2.1/§2.2/§3.2/§4.2 — esta decisión sella dominio cuantitativo) · `.agents/skills/quant-engineer/SKILL.md` (mandato documental y forma de entrega) · `.agents/knowledge/debt-management.md` (distinción DEBT vs. TASK — este hallazgo es "pregunta a decidir + trabajo a ejecutar", no un hueco en código ya construido, de ahí TASK y no DEBT) · `docs/adr/ADR-0114.md`, `docs/adr/ADR-0155.md`, `docs/adr/ADR-0156.md`, `docs/features/slippage-models.md`, `docs/features/institutional-friction-modeling.md`, `docs/features/data-sanitizer-pipeline.md`, `docs/features/data-import-wizard.md`, `docs/features/algorithmic-bars.md`, `docs/features/clock.md`, `docs/execution/TASK-059-contrato-momento-de-conocibilidad.md`, `docs/execution/TASK-063-formato-estandar-de-ingesta.md`, `.agents/plans/architect-2026-07-19-escala-barras-ingesta.md` (precedente de forma para esta misma familia de decisiones).
