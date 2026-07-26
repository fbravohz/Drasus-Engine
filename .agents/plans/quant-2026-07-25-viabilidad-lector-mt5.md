# Viabilidad del Lector de Archivos Reales de Mercado (vía manual, MT5) — Gate Etapa 0.3

**Encargo:** auditar si `datasets/EURUSD_M1_2000-01-01_2025-05-01.csv`, `datasets/EURUSD_TICK.csv` y los tres `.dat` (Dukascopy M1/H1, IC Markets M1) pueden llevarse al esquema canónico (ADR-0155) sin inventar dato, y qué se rompe en el intento.
**Método:** muestreo con `awk`/`grep`/`xxd` sobre los archivos reales (nunca carga completa), contraste contra ADR-0154/0155/0156/0114/0162/0163 y las features citadas.

---

## 1. Veredicto: APTO-CON-CONDICIONES

Los dos archivos de texto (`EURUSD_M1_...csv`, `EURUSD_TICK.csv`) son ingeribles bajo el esquema canónico. Los tres `.dat` binarios **no** son ingeribles hoy — no por defecto de diseño, sino porque su formato no está identificado ni documentado en ningún lugar del repo. Siete condiciones, cada una redactada como criterio de aceptación de una futura Orden de Trabajo:

1. **`data-normalization-layer` debe declarar, con ADR que enmiende o aclare ADR-0155 Restricción 3, cómo reconstruye un `QuoteTick` completo a partir de actualizaciones unilaterales reales** (ticks que traen `bid` o `ask`, nunca los dos a la vez). Criterio de aceptación: existe una regla FIJA escrita, con test de oráculo que reproduce la secuencia exacta de `datasets/EURUSD_TICK.csv` líneas con `FLAGS=2`/`FLAGS=4` (ver §2.2) y verifica el resultado esperado, incluida la marca de auditoría que distingue "observado en este tick" de "arrastrado del tick anterior".
2. **Los tres archivos `.dat` quedan fuera de alcance de cualquier Story de ingesta** hasta que el propietario identifique la herramienta/librería que los generó, o aporte su especificación. Criterio de aceptación: ninguna Story cita estos tres archivos como insumo sin una plantilla de plataforma validada (ADR-0156 restricción 4) que los respalde.
3. **La convención de unidad del campo `<VOL>` (volumen real MT5) se declara por bróker/feed antes de mapearse a `volume_traded`**, nunca se asume por defecto. Criterio de aceptación: `broker_instrument_terms` (o campo equivalente) registra la convención declarada; si no se declara, el importador marca la columna como "convención no verificada" en vez de persistir el valor sin marca.
4. **El parser de ambos CSV descarta el retorno de carro (`\r`) del último campo de cada línea antes de convertir a entero.** Criterio de aceptación: un test de oráculo alimenta una línea real del archivo (con su CRLF de Windows) y verifica que `SPREAD`/`FLAGS` se parsean como entero, no como error ni como `NULL` silencioso.
5. **El Gate de Coherencia de Calendario (ADR-0162) admite como huella válida de `FOREX_245` un hueco semanal viernes 23:56 → lunes 00:00 sin barras dominicales** (huella real observada en este archivo, ver §2.1) — no solo la huella "viernes→domingo" que la tabla de ADR-0162 usa como ejemplo. Criterio de aceptación: el archivo real se importa con la plantilla `FOREX_245` sin producir rechazo ni `INDETERMINADO` por el gate.
6. **Las dos filas con `TICKVOL=0` pero rango de precio no nulo** (línea 7187218/7187221 del M1, ver §3) pasan por `pit-data-validator` y quedan marcadas explícitamente, nunca persistidas como si fueran una vela ordinaria sin más.
7. **Se ejecuta la validación de semántica de `spread_reported` en `Bars`** que ADR-0155/TASK-064 §4 dejó pendiente, usando el propio solape real entre ambos archivos (abril de 2025, ver §3) — la evidencia preliminar de esta auditoría (§3) ya apunta a una respuesta concreta.

---

## 2. Contraste campo por campo

### 2.1 `EURUSD_M1_2000-01-01_2025-05-01.csv` → `Bars` (ADR-0155)

9,231,695 filas de dato + cabecera, separadas por tabulación, CRLF, 9 campos consistentes en el 100% de las filas verificado por `awk NF`.

| Columna del archivo | Campo canónico (`Bars`, ADR-0155) | Veredicto |
|---|---|---|
| `<DATE>`+`<TIME>` | `bar_open_timestamp_ns`/`bar_close_timestamp_ns` | **Encaja.** Formato `YYYY.MM.DD` + `HH:MM:SS`, monotónico y sin duplicados verificado (`sort \| uniq -d` vacío, 0 inversiones en el archivo completo). Falta la zona horaria de origen — **ya resuelto**: `data-import-wizard.md` §"Declaración de zona horaria de origen" exige que el usuario la declare vía selector DST (ADR-0162), el archivo no la trae ni tiene por qué traerla. |
| `<OPEN>/<HIGH>/<LOW>/<CLOSE>` | `open`/`high`/`low`/`close` | **Encaja.** 5 decimales, sin violación de `high≥low`, `close`/`open` dentro de rango en la muestra escaneada (3,000,000 filas, 0 violaciones). Escala a `INTEGER ×10⁸` sin pérdida (5 decimales caben sobrados). |
| `<TICKVOL>` | `volume_ticks` | **Encaja con una anomalía puntual** — ver hallazgo §3 (dos filas en `TICKVOL=0` con rango de precio real). |
| `<VOL>` | `volume_traded` | **Encaja conceptualmente, unidad sin declarar.** `.agents/plans/esquema-canonico-datos-mercado.md:204` ya cita `CopyRates` de MT5 (`tick_volume` vs. `real_volume`) como el origen de esta distinción — no es un campo ambiguo entre las 4 magnitudes de ADR-0155 Restricción 4. Pero el **valor concreto no tiene unidad declarada en el archivo**: 90.4% de las filas son `0` (sin volumen real disponible, típico de Forex OTC); el 9.6% restante arranca el 2010-06-03 con magnitudes de hasta 8 dígitos (ej. `93000000`) — condición 3. |
| `<SPREAD>` | `spread_reported` en `Bars` | **Encaja, ambigüedad de semántica exacta ya declarada como pendiente en el propio ADR-0155** ("open/close/promedio... sin verificar por bróker", línea 101 de ese ADR) — mecanismo de cierre ya diseñado (`TASK-064 §4`), condición 7 de este informe muestra que el dato para cerrarlo ya está disponible. 29.2% de las filas (2,693,274) traen `SPREAD=0`, máximo observado 99 puntos — sin outliers. |
| — | `venue`/`source_provider`/`instrument_id` | **Ninguno viene en el archivo** (no hay cabecera de bróker ni metadato). **Ya resuelto:** `data-import-wizard.md` exige "Metadatos manuales (Exchange, Symbol)" como entrada explícita del usuario — el archivo no necesita traerlos porque el wizard ya los captura. |
| — | `bar_type`/`timeframe_value` | No vienen en el archivo (es implícito por el nombre `_M1_`). **Ya resuelto** por el mismo mecanismo de metadatos manuales. |

### 2.2 `EURUSD_TICK.csv` → `QuoteTick` (ADR-0155)

4,998,198 filas de dato, tabulación, CRLF, siempre 7 campos, cubre exclusivamente abril de 2025 (un mes, no todo el histórico — coherente con ADR-0154: "los brókers conservan pocos ticks").

| Columna del archivo | Campo canónico (`QuoteTick`) | Veredicto |
|---|---|---|
| `<DATE>`+`<TIME>` (con milisegundos) | `event_timestamp_ns` | **Encaja.** `13:00:00.009` → nanosegundos triviales. |
| `<BID>` | `bid` (NOT NULL, ADR-0155 Restricción 3) | **En contradicción con el archivo real** — ver hallazgo crítico §3.1. |
| `<ASK>` | `ask` (NOT NULL) | Misma contradicción. |
| `<LAST>`/`<VOLUME>` | Campos de `TradeTick` | **Vacíos en el 100% de las 4,998,198 filas.** Confirmado por barrido completo: ninguna fila trae `LAST`/`VOLUME` poblados. Este archivo es puro `QuoteTick`; no hay demultiplexado que hacer (`data-normalization-layer` TTR-003 no aplica a este archivo concreto, aunque sigue aplicando a MT5 en general cuando el export incluye trades). |
| `<FLAGS>` | (no tiene campo canónico propio; gobierna cómo interpretar la fila) | Solo tres valores en todo el archivo: `2` (1,499,738 filas — bid solo), `4` (1,538,462 — ask solo), `6` (1,959,998 — ambos). La suma exacta iguala el total de filas y coincide bit a bit con el conteo de `BID`/`ASK` vacíos — confirma que `FLAGS` es el bitmask MT5 (`TICK_FLAG_BID=2`, `TICK_FLAG_ASK=4`). **Nunca aparecen `TICK_FLAG_LAST(8)`/`VOLUME(16)`** en este archivo. |

**Calidad verificada:** 0 cotizaciones cruzadas (`bid > ask`) en las filas donde ambos están presentes; 0 filas con ambos campos vacíos.

### 2.3 Los tres `.dat` (Dukascopy M1/H1, IC Markets M1)

Los tres comparten cabecera binaria idéntica en los primeros 96 bytes: la cadena ASCII `4.2 D ABCDEFGH` seguida de `SnRbTs` y una tabla de offsets. **No coincide con ningún formato documentado:**

- No es MT4 `.hst` (ese formato empieza con un entero de versión 400/401 y un campo de copyright de 64 bytes en texto plano — no aparece aquí).
- No es el `.bi5` oficial de Dukascopy (ese formato no lleva cabecera: es un flujo LZMA puro de registros fijos de 20 bytes — `.agents/plans/plan-decision-fuentes-datos-mercado.md:285` ya documenta esa estructura y no coincide).
- Búsqueda declarada: `grep -rli "dukascopy\|.hst\|.bi5\|icmarkets" docs/ .agents/` — los únicos dos resultados (`plan-decision-fuentes-datos-mercado.md`, `esquema-canonico-datos-mercado.md`) documentan el `.bi5` oficial y el CSV oficial de Dukascopy, ninguno de los dos coincide con la cabecera real observada. **Cero coincidencias que expliquen este formato concreto.**

Por ADR-0156 Restricción 1 (rechazo por defecto sin plantilla reconocida) y la restricción dura de este encargo ("nada se adivina"), tratar de inferir el layout de bytes sin especificación **sería exactamente el error que esa restricción prohíbe**. Veredicto: **vacío real, no NO-APTO** — la vía correcta no es reverse-engineering a ciegas, es preguntar al propietario qué herramienta generó estos tres archivos (ver pregunta abierta #1).

---

## 3. Hallazgos que solo el Quant-Engineer puede ver

### 3.1 Hallazgo crítico: `QuoteTick` NOT NULL choca con el tick real de MT5 (no es el mismo problema que ADR-0155 ya resolvió)

ADR-0155 Restricción 3 exige `bid`/`ask` NOT NULL en toda fila de `QuoteTick`. El razonamiento que sostiene esa restricción (`esquema-canonico-datos-mercado.md:31`) solo contempla la mezcla **cotización/ejecución** en un mismo archivo (el caso que `data-normalization-layer` demultiplexa) — nunca contempla que, **dentro del propio flujo de cotizaciones**, cada tick individual de MT5 suele traer un solo lado:

```
2025.04.01	13:05:00.102		1.08127			4   ← solo ASK (BID vacío)
2025.04.01	13:05:00.213		1.08125			4   ← solo ASK
2025.04.01	13:05:00.189	1.08125				2   ← solo BID (ASK vacío)
2025.04.01	13:05:00.834	1.08124	1.08124			6   ← ambos
```

1,499,738 filas traen solo `bid`, 1,538,462 solo `ask` — el 60.9% de las 4,998,198 filas del archivo **no puede escribirse directamente en un `QuoteTick` con NOT NULL en ambos campos sin inventar el lado ausente.** Este es exactamente el tipo de tensión que el principio "guarda el hecho, calcula la interpretación" (ADR-0155 Restricción 1) existe para resolver en otros campos (`spread_reported`, `vwap`) — pero aquí nadie lo resolvió, porque nadie verificó una exportación real de tick de MT5 columna por columna antes de sellar la Restricción 3. No es una contradicción del ADR consigo mismo: es un caso que el ADR no vio.

**Por qué esto no es un detalle menor:** la solución obvia — arrastrar (`carry-forward`) el último valor conocido del lado ausente, igual que ya se hace con `spread_reported` nulo (`data-sanitizer-pipeline` Capa 2, TASK-064 §1.3) — funciona para el 99.9% del archivo, pero **la primera fila del archivo completo ya es unilateral** (`2025.04.01 13:00:00.009`, solo `bid`, `ask` vacío desde el primer instante). No hay semilla de la que arrastrar al abrir un archivo o una sesión de importación nueva. Sin una regla explícita para ese borde, el ingeniero que construya esto tiene dos caminos igual de plausibles y con resultados distintos (rechazar la fila inicial hasta la primera cotización completa, o dejar el campo `NULL` violando la Restricción 3 con una excepción declarada) — y **nadie lo ha decidido**. Es dominio de Architect (enmienda de ADR-0155), no de ingeniería.

### 3.2 `TICKVOL=0` con rango de precio real — dos filas, un síntoma que se generaliza

```
7187218: 2019.11.01	16:58:00	1.11488	1.11496	1.11485	1.11486	0	0	0
7187221: 2019.11.01	17:01:00	1.11670	1.11686	1.11637	1.11650	0	0	0
```

`TICKVOL` cuenta cambios de precio dentro del minuto; si `open≠high≠low≠close`, hubo movimiento, y un movimiento sin ningún tick contado es matemáticamente imposible bajo la propia definición de `tick_volume` que MT5 documenta (`esquema-canonico-datos-mercado.md:125`). Solo dos filas en 9.2 millones (ambas el mismo día, 2019-11-01 — coincide con el cambio de hora de horario de verano en EE. UU., lo que sugiere un artefacto del propio proveedor del bróker al reconstruir la vela sintética de esa hora, no un error de exportación). El volumen es bajo, pero es exactamente el tipo de imposibilidad que ADR-0156 Restricción 6 exige que `pit-data-validator` detecte — y hoy nada en la documentación de ese validador enumera "rango de precio con contador de actividad en cero" como chequeo.

### 3.3 El solape real entre ambos archivos ya permite cerrar el pendiente de `spread_reported` — nadie lo ha usado

ADR-0155 deja abierta la semántica exacta de `Bars.spread_reported` (¿apertura, cierre o promedio del intervalo?) y asigna su cierre a "el plan de validación de mes solapado con ticks reales" (TASK-064 §4), calificándolo de comprobación diferida sin costo. **Ese mes solapado ya existe en `datasets/`:** el M1 cubre hasta 2025-05-01 y el tick cubre exactamente abril de 2025 completo. Una muestra de un minuto (`2025.04.01 13:05:00`, `SPREAD=0` en el bar) reconstruida desde los ticks reales de ese mismo minuto **da evidencia consistente con "spread de la última cotización completa conocida al cierre de la barra"**, no un promedio ni el spread de apertura: el último ask conocido antes del cierre fue `1.08091` (tick a las `13:05:59.170`) y el último bid conocido fue `1.08091` (tick a las `13:05:59.843`, unilateral) — spread implícito 0, igual al `SPREAD=0` de la barra. Es una sola muestra, no la validación exhaustiva que TASK-064 §4 exige — pero demuestra que el costo de cerrarla ya es cero: los datos para hacerlo están sobre la mesa hoy.

### 3.4 Sesgo latente: el `<VOL>` de MT5 aparece recién en 2010, no en todo el histórico

`<VOL>` es 0 en el 90.4% de las filas y arranca en 2010-06-03 (`93000000` como primer valor no nulo). Cualquier consumidor que calibre "cuánto volumen real hay disponible para EURUSD" contra este archivo sin fijarse en la fecha de corte concluiría erróneamente que el instrumento no tiene volumen real cuando en realidad **antes de 2010 el bróker/feed simplemente no lo reportaba** — un sesgo de "ausencia de dato" confundido con "ausencia de actividad" si alguien construye una métrica de liquidez histórica sobre esta columna sin segmentar por fecha de disponibilidad.

> 🔴 **CORRECCIÓN DEL TECH-LEAD (2026-07-26) — la inferencia de §3.5 sobre el reloj de origen está INVERTIDA y es peligrosa si se toma como está.**
>
> §3.5 concluye que el patrón semanal idéntico en invierno y en verano *"indica offset fijo sin DST en el reloj de origen"*. **Es exactamente al revés.** El propietario identificó el bróker (IC Markets) y su horario de servidor está publicado: **GMT+2 en invierno, GMT+3 en verano**, es decir el reloj **sí** sigue horario de verano.
>
> **Por qué el patrón se ve constante precisamente por eso:** los eventos del mercado están fijos en hora de Nueva York (el cierre del viernes es a las 17:00 de allá), y el reloj del servidor se mueve junto con ellos. Vistos desde un reloj que acompaña el cambio, el cierre y la apertura caen siempre a la misma hora local en las dos estaciones. Un reloj de offset **fijo** produciría lo contrario: el patrón se correría una hora al cambiar la estación. **La constancia observada es la huella de un reloj que SÍ sigue DST, no de uno que no lo sigue.**
>
> **Consecuencia si nadie corrige esto:** quien lea §3.5 declararía un offset fijo GMT+2 para los 25 años, y todas las barras de la mitad del año quedarían corridas una hora. Es el mismo modo de fallo que la unidad de spread — plausible, silencioso y masivo.
>
> **Y hay un segundo hallazgo que §3.5 no podía ver:** el cambio de IC Markets sigue las fechas de **Estados Unidos**, no las europeas (verificado en su propio aviso: el cambio de 2025 fue el 9 de marzo, segundo domingo, que es la fecha estadounidense; la europea de ese año fue el 30 de marzo). Eso significa que **ninguna zona del catálogo IANA expresa este reloj**: `Europe/Athens` es GMT+2/+3 pero con fechas europeas, y quedaría desfasada una hora unas cuatro semanas de cada año. Escalado al Architect como **TASK-069** — es un vacío del mecanismo de declaración de ADR-0162, no un error de este informe.

### 3.5 Ausencia de barras dominicales — variante real de `FOREX_245`

Verificado en invierno (2024-01-05→08) y verano (2024-07-05→08): el mercado cierra el viernes a las 23:56 (última barra) y reabre el lunes a las 00:00 (primera barra) — **cero barras el domingo en las dos ventanas muestreadas**, mismo patrón en ambas estaciones (indica offset fijo sin DST en el reloj de origen, no una zona que seguía horario de verano). Esto difiere de la huella de ejemplo que la tabla de ADR-0162 usa para Forex/CFD ("hueco semanal viernes→domingo"). No es una contradicción del ADR — ya declara que la tolerancia del gate es CONFIG y que el `SESSION_TEMPLATE_SEED_SET` es "representativo, editable" — pero es el dato concreto que demuestra que la plantilla semilla `FOREX_245` tal como esté calibrada por defecto **fallaría el Gate de Coherencia** contra este archivo real si asume domingo con barras.

---

## 4. Preguntas abiertas para el Architect

1. **¿Qué herramienta o librería generó los tres archivos `.dat`?** Su cabecera (`4.2 D ABCDEFGH`, `SnRbTs`) no coincide con MT4 `.hst` ni con el `.bi5` oficial de Dukascopy — no hay especificación en el repo. Sin esa identificación, construir una plantilla de plataforma para ellos violaría "nada se adivina". Ver §2.3.
2. **¿Cómo se reconstruye un `QuoteTick` NOT NULL cuando la fuente entrega actualizaciones unilaterales (solo `bid` o solo `ask`) dentro del propio flujo de cotización?** ADR-0155 Restricción 3 no contempla este caso — solo contempla la mezcla cotización/ejecución. ¿Se arrastra el lado ausente (con marca de auditoría explícita) o se declara una excepción a NOT NULL para el layer crudo, con el merge de estado ocurriendo en una vista de lectura? Ver §3.1 — es la pregunta de mayor impacto de este informe.
3. **¿Qué política rige la primera fila de un archivo/sesión de ticks cuando ningún lado tiene semilla previa?** Consecuencia directa de la pregunta 2: el arrastre no funciona sin un valor anterior, y el primer tick del archivo real muestreado ya es unilateral.
4. **¿Quién declara la convención de unidad de `<VOL>` (real_volume de MT5) por bróker** —¿`instrument-catalog`/`broker_instrument_terms` gana un campo nuevo, o se asume que el usuario ya sabe la convención de su propio bróker al importar? Hoy ningún documento fija dónde vive esa declaración.
5. **¿El `SESSION_TEMPLATE_SEED_SET` de `FOREX_245` (session-catalog, STORY-066) ya contempla brókers sin barras dominicales**, o hay que crear una plantilla separada (ej. `FOREX_245_NO_SUNDAY`) para no romper el Gate de Coherencia contra archivos reales como el de esta auditoría?
6. **¿`pit-data-validator` va a enumerar "rango de precio con `volume_ticks=0`" como chequeo de imposibilidad?** No aparece hoy en ningún feature doc revisado, y el archivo real trae dos casos concretos (§3.2).
7. **¿Quién ejecuta ahora la validación de semántica de `spread_reported`** que TASK-064 §4 dejó diferida, dado que el dataset que la desbloquea (mes de abril de 2025 solapado M1+tick) ya está en el repo? ¿Es una tarea de esta misma Story de ingesta o se abre aparte?

---

**Trazabilidad:** ADR-0154, ADR-0155, ADR-0156, ADR-0114 (enmiendas 2026-07-19/07-23), ADR-0162, ADR-0163, `docs/features/data-import-wizard.md`, `docs/features/data-normalization-layer.md`, `docs/features/instrument-catalog.md`, `docs/features/session-catalog.md`, `.agents/plans/esquema-canonico-datos-mercado.md`, `.agents/plans/plan-decision-fuentes-datos-mercado.md`, `docs/execution/TASK-064-politica-spread-fill-model-dst.md`.
