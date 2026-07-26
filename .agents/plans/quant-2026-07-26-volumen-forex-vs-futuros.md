# Volumen en Forex — unidad de la columna, frontera de uso, e hipótesis de futuros+contado

**Encargo:** el propietario pide, con evidencia dura y sin ajustarse a su hipótesis, (1) qué es la columna `<VOL>` de `datasets/EURUSD_M1_2000-01-01_2025-05-01.csv`, (2) qué preguntas admite el volumen de bróker Forex y cuáles no, (3) si volumen centralizado de futuros junto a precio Forex al contado tiene sentido, y con qué condiciones/modos de fallo.
**Método:** muestreo con `awk`/`wc` sobre el archivo real (9,231,695 filas, nunca cargado entero), contraste contra ADR-0154/0155/0158/0162/0114, `data-normalization-layer.md`, `order-flow-microstructure.md`, mi propio informe previo (`quant-2026-07-25-viabilidad-lector-mt5.md`) y dos búsquedas web puntuales para verificar hechos externos citados (fundación de IC Markets, semántica documentada de `real_volume` de MT5) — nunca como única fuente, siempre cruzadas contra la medición.

---

## 0. Corrección de encuadre antes de ejecutar (`GOVERNANCE-4.7`)

El encargo trae tres supuestos que no sobreviven la medición. Se reportan antes de responder, como exige el método.

1. **El mes de ticks de abril de 2025 no sirve para esta pregunta concreta.** El encargo lo propone como evidencia cruzada para resolver la unidad de volumen. Medido de forma independiente sobre `datasets/EURUSD_TICK.csv` (4,998,198 filas): las columnas `<LAST>`/`<VOLUME>` están vacías en el 100% de las filas — es un archivo puro de cotización (`bid`/`ask`), sin una sola ejecución ni volumen real. Y sobre el propio `EURUSD_M1_...csv`, la columna `<VOL>` es cero en el 100% de las filas de 2025 (ver §1). El solape "M1 + tick de abril 2025" es exactamente el que ya cerró la reconstrucción de `bid`/`ask` unilateral (ADR-0155 enmienda 2026-07-25) — para la pregunta de la unidad de volumen no aporta ni un solo dato, en ningún lado del cruce.
2. **`<VOL>` no "arranca en 2010-06-03 y sigue".** Es una ventana angosta y con una fractura interna: datos reales solo entre 2010-06-03 y 2017-11-17, y dentro de esa ventana hay un quiebre brusco de régimen en abril/mayo de 2017 (detalle en §1). Fuera de esa ventana —incluida toda la década 2018-2025, que es donde vive el mes de ticks que el encargo propone como ancla— la columna es cero sin excepción.
3. **El archivo no es una sola fuente continua "de IC Markets" a lo largo de sus 25 años.** IC Markets se fundó en 2007 (verificado por búsqueda externa, ver Fuentes). El archivo trae datos desde 2000-01-03 — siete años antes de que IC Markets existiera. Esos años no pueden ser historial de trading propio de ese bróker; son, casi con certeza, historial retro-cargado desde un vendedor de datos al configurar el terminal (práctica estándar en MT5: el bróker no fabrica el pasado, lo importa). Esto no invalida el archivo — ADR-0154 ya declara que la profundidad histórica nunca es responsabilidad de Drasus, y el usuario confirmó IC Markets como fuente de exportación, no de cada fila individual — pero sí obliga a tratar el archivo como una composición de al menos tres regímenes de generación de dato distintos (pre-2007 retro-cargado, 2010-06→2017-04 con volumen "limpio", 2017-05→2017-11 con volumen "roto"), nunca como una serie homogénea.

Estas tres correcciones acotan qué es alcanzable en §1: no hay ground truth independiente disponible en este repositorio para la unidad de volumen de este archivo, ni para el período que el encargo señala como validable.

---

> ✅ **VERIFICACIÓN INDEPENDIENTE DEL TECH-LEAD (2026-07-26) — reproducida sobre el archivo real, no aceptada de palabra.**
>
> Reproduje las tres mediciones que sostienen este informe. **Las tres se confirman**, con una corrección de precisión:
>
> - **Ventana del volumen real:** primera fila no nula `2010.06.03 10:25` con valor `93000000`; última `2017.11.17 23:56` con valor `159`. Confirmado.
> - **Divisibilidad por 1000 en el tramo limpio (2010-06 a 2016-12):** 533,852 filas no nulas, de las cuales **4 no son divisibles por 1000**. El informe afirma *"sin una sola excepción"* — son cuatro, el 0.0007%. **La conclusión se sostiene** (99.9993%), pero la afirmación absoluta no era exacta y así queda constando.
> - **La fractura de régimen se confirma y es peor que "una fractura":** el perfil año por año muestra que **2012 y 2013 no tienen ni una fila con volumen real**, mientras 2011 tiene 4,505 y 2014 tiene 23,796. El campo no es una ventana continua: es **intermitente**. Y la magnitud salta sin transición — media de ~137 millones en 2010, ~146 millones en 2015, ~26 millones en 2017, con el último valor del archivo en `159`. Seis órdenes de magnitud entre extremos del mismo campo.
> - **IC Markets se fundó en 2007** (Sydney, licencia de la ASIC en 2009) y el archivo arranca el `2000.01.03`. Confirmado por fuente externa. Los primeros siete a nueve años **no pueden ser el feed de ejecución de ese bróker**: son histórico rellenado desde un proveedor.
>
> **Conclusión del Tech-Lead:** el hallazgo de que el archivo mezcla orígenes a lo largo de sus 25 años es correcto y tiene consecuencias que exceden a este informe — la identidad declarada de la serie (símbolo × temporalidad × bróker) **no se sostiene uniformemente sobre todo el rango del archivo**. Escalado al Architect, añadido a **TASK-069** porque comparte causa raíz con el problema del reloj: en los dos casos, lo que se declara para un archivo completo no vale para todo su tramo.

## 1. Veredicto sobre la unidad de `<VOL>`

### 1.1 La corrección factual del propietario se confirma, medida de forma independiente

| Métrica | Medido sobre el archivo real |
|---|---|
| Filas de dato | 9,231,695 |
| `<TICKVOL>` = 0 | 2 filas (0.00002%) — las dos anomalías de 2019-11-01 ya documentadas en mi informe previo (§3.2 de `quant-2026-07-25-viabilidad-lector-mt5.md`) |
| `<VOL>` = 0 | 8,346,621 filas (90.41%) |
| Primera fila con `<VOL>` ≠ 0 | 2010-06-03 10:25:00, valor `93000000` |
| Última fila con `<VOL>` ≠ 0 | 2017-11-17 |

Coincide exactamente con lo que el propietario pidió confirmar: el contador de ticks está poblado casi siempre, y el volumen real es el que falta en la inmensa mayoría del archivo.

### 1.2 La ventana de datos reales, año por año

| Año | % de filas con `<VOL>` ≠ 0 |
|---|---|
| 2000–2009 | 0% |
| 2010 | 0.00% (9 filas sueltas) |
| 2011 | 1.22% |
| 2012–2013 | 0% |
| 2014 | 6.50% |
| 2015 | 44.14% |
| 2016 | **100.00%** |
| 2017 | 85.91% (se corta el 17 de noviembre) |
| 2018–2025 | 0% |

No es una rampa de adopción que se estabiliza — es una campana que sube, pasa por un año enteramente poblado (2016) y vuelve a cero por completo. Ese patrón es la firma de un cambio de fuente/feed en el tiempo, no de "el bróker decidió reportar volumen real desde tal fecha".

### 1.3 Dentro de la ventana poblada hay una fractura de régimen — no es una sola convención

Todo valor no-cero de `<VOL>` entre 2010 y marzo de 2017 es múltiplo exacto de 1,000 (0% de excepciones, verificado sobre las 401,336 filas de ese tramo). A partir de abril de 2017 ese patrón se rompe de golpe:

| Mes 2017 | % de valores NO múltiplos de 1,000 |
|---|---|
| enero–marzo | 0.0% |
| abril | 17.8% (mes de transición) |
| mayo–noviembre | 99.7%–100.0% |

Es un quiebre limpio en un mes concreto, no una degradación gradual. Confirma la lectura de §0.3: al menos dos generadores de dato distintos conviven bajo la misma columna con el mismo nombre.

### 1.4 Qué es la magnitud, con las hipótesis descartadas por mecanismo

Sobre el tramo limpio (2010-06→2017-04, régimen "múltiplo de 1,000"): media `≈124,448,000`, mínimo `23,000`, máximo `2,846,814,000` (unidades crudas, sin escala).

- **Descartado — conteo de operaciones ejecutadas.** Un valor de 93 millones en una barra de un minuto implicaría 1.5 millones de ejecuciones por segundo en un solo símbolo de un bróker retail — físicamente imposible.
- **Descartado — número de lotes enteros.** 930,000 lotes en un minuto equivalen a 93,000 millones de EUR de nocional en 60 segundos en un solo bróker — órdenes de magnitud por encima de lo que factura globalmente el mercado interbancario de EURUSD en esa ventana.
- **Debilitado — centésimas de lote (convención estándar de volumen de orden en MT5, donde 100 = 1.00 lote).** Bajo esa convención, el mínimo observado (`23,000`) implicaría 230 lotes estándar como la operación más pequeña detectada en siete años de datos — no hay valores pequeños en absoluto en 373,136 filas de 2016 pese a que la actividad varía fila a fila (noches, fines de semana, sesión activa). Si la unidad fuera centésimas de lote, deberían aparecer sumas pequeñas en las horas de baja liquidez; no aparecen ni una vez.
- **Mejor sostenida — unidades de moneda base (EUR), con talla mínima de negociación de 1,000 unidades (0.01 lote estándar).** Es la única hipótesis compatible con: (a) el 100% de divisibilidad exacta por 1,000 en el tramo limpio, sin una sola excepción en más de 500,000 filas; (b) un orden de magnitud (decenas de millones a bajos miles de millones de EUR por minuto, agregando toda la actividad de un bróker ECN grande en el par más líquido del mundo) plausible para volumen agregado propio de un bróker, no para volumen de mercado mundial; (c) la correlación con `<TICKVOL>` es fuerte pero incompleta (`r = 0.79`, razón media `VOL/TICKVOL ≈ 1.25M` con desviación estándar `≈615K`) — es información parcialmente independiente del conteo de cambios de cotización, consistente con ser una segunda magnitud real y no una copia derivada de la primera.

**No se puede distinguir, con este archivo, moneda base (EUR) de moneda de cotización (USD) como denominación exacta** — EURUSD cotiza siempre cerca de 1:1 en todo el período, así que ambas lecturas producen magnitudes casi idénticas. Un par con relación de precio muy distinta de 1 (ej. USDJPY) sí permitiría esa distinción; EURUSD es un mal candidato para esa prueba concreta.

### 1.5 Veredicto y grado de certeza

- **Para el tramo 2010-06→2017-04:** la hipótesis mejor sostenida es volumen real agregado por el bróker/feed, en unidades de moneda (base o cotización, indistinguibles aquí), con talla mínima de 0.01 lote. **Certeza: media.** Es la lectura mecánicamente más consistente con la evidencia, pero no hay una segunda fuente independiente en este repositorio que la confirme con datos de ejecución reales de ese período — el mes de tick de 2025 (§0) no cubre esa ventana ni contiene volumen.
- **Para el tramo 2017-05→2017-11-17:** unidad **sin resolver**. El quiebre de divisibilidad por 1,000 indica un generador de dato distinto; no hay evidencia suficiente en este archivo para caracterizarlo.
- **Para todo lo demás (2000-2010, 2018-2025, incluido el archivo de 2025 completo):** la columna es cero. No hay unidad que declarar porque no hay dato que interpretar.
- **Qué haría falta para cerrarlo del todo:** un dataset de ejecuciones reales (trade tape) del mismo bróker que solape con 2016 específicamente (el único año 100% poblado), o una respuesta directa de soporte de IC Markets sobre qué generó el campo `real_volume` de su terminal en ese período concreto — el propio MQL5 documenta `real_volume` como dependiente del bróker, nunca de convención universal, así que la respuesta vive en su infraestructura, no en la documentación pública de MetaQuotes.

**Consecuencia práctica inmediata:** lo que STORY-068 ya implementó —persistir el valor crudo y marcar la corrida como `volume_unit_unverified` cuando `real_volume_unit_declared` no está declarado (ADR-0162)— es la decisión correcta con la evidencia de hoy. Declarar `BASE_ASSET_UNITS` para todo el archivo sería alucinar certeza sobre un tramo (2017-05 en adelante) que la propia medición contradice.

---

## 2. La frontera de uso del volumen de bróker Forex

Forex spot es un mercado OTC sin cinta consolidada: no existe un "volumen total de EURUSD" observable por nadie, ni siquiera por el BIS en tiempo real. Un bróker retail ve solo el flujo que pasa por su propio libro y sus propios proveedores de liquidez (LP) — una muestra no aleatoria, sesgada hacia el perfil de cliente de ese bróker concreto.

### 2.1 Qué preguntas admite (con mecanismo)

- **Actividad relativa dentro de la misma serie, en el tiempo.** `volume_ticks` (conteo de cambios de cotización, ya persistido y sin ambigüedad de unidad — es un contador, no una magnitud física) es un proxy legítimo de "cuándo está despierto el mercado" para ESE bróker: más cotizaciones por minuto en la sesión Londres-NY que de madrugada es un hecho verificable sin necesitar saber qué es un "lote" ni cuánto pesa. Válido porque la comparación es contra sí mismo, no contra un universo externo.
- **Detección de anomalías propias de la ejecución en ESE bróker.** Si el volumen de ESE feed se dispara justo en zonas de caza de stops, es una señal sobre el comportamiento de ese bróker/LP concreto (microestructura defensiva, cuestión de calidad de ejecución) — no una señal sobre el mercado interbancario.
- **Correlación autorreferenciada entre actividad propia y movimiento propio.** Preguntar "¿en mis propios datos, más `volume_ticks` precede mayores rangos de vela?" es una pregunta válida sobre la propia serie, sujeta al mismo rigor de sobreajuste (WFA/CPCV) que cualquier otra señal — pero no exige que el volumen represente nada fuera de esa serie.

### 2.2 Qué preguntas NO admite (con mecanismo, no adjetivo)

- **"¿Cuánto interés global hay en EURUSD a este nivel de precio?"** No: el flujo de un bróker retail es una fracción minúscula y sistemáticamente sesgada (perfil apalancado, estrategias de ruptura/momentum típicas de la comunidad de EAs retail) del ~1 billón de dólares diarios que mueve el mercado interbancario real. Usarlo como proxy de interés agregado importa ese sesgo de selección directo a cualquier señal derivada.
- **"¿Dónde está posicionado el dinero institucional?"** No: el flujo institucional real (banco a banco, fondos, bancos centrales) se ejecuta en plataformas interbancarias (EBS, Reuters Matching) y ECNs primarios que un feed de bróker retail nunca ve. Lo que el bróker agrega, cuando lo agrega, es su propia posición neta frente a sus LPs — no una ventana al libro institucional.
- **CVD/OFI (Order Flow Imbalance) sobre este archivo, en absoluto.** No es una cuestión de si el dato "es bueno": es que la fuente no tiene el campo que la técnica exige. `order-flow-microstructure.md` (línea 21) fija que el CVD histórico "requiere datos a nivel de operación (ej. Binance aggTrades)" — clasificación compra/venta por operación individual. La exportación de MT5 (barras y ticks, verificado en ambos archivos) nunca trae esa clasificación: no hay `aggressor_side`, no hay `volume_buy`/`volume_sell` para Forex. **Ya resuelto** por el propio esquema canónico — ADR-0155 marca `volume_buy` como "Opcional... Binance klines lo trae", nunca como campo esperado de un CSV de MT5. No hay nada que decidir aquí: la puerta está cerrada por ausencia de dato, no por juicio de calidad.
- **Comparaciones de magnitud absoluta, entre brókers o en el tiempo, sin renormalizar.** El propio archivo lo demuestra: el mismo bróker, el mismo símbolo, dos regímenes de magnitud/granularidad incompatibles con solo un mes de diferencia (§1.3). Cualquier característica que use "volumen crudo de este archivo" como si fuera comparable año a año está, por construcción, mezclando convenciones distintas bajo el mismo nombre de columna.
- **Cualquier backtest que "descubra" que una señal de volumen funcionaba en 2016.** Riesgo de minería de sesgo de selección (checklist §2 de este skill): 2016 es el único año 100% poblado del archivo completo — es un hecho de la tubería de datos (qué vendor/feed estaba activo ese año), no un hecho del mercado. Un hallazgo "rentable" anclado a esa ventana no generaliza porque la característica que lo sostiene (disponibilidad del campo) desapareció después, dos veces, dentro del mismo archivo.

---

## 3. Veredicto sobre la hipótesis de futuros centralizados + contado Forex

### 3.1 Por qué la intuición del propietario tiene una base real

El volumen de un futuro en una bolsa centralizada (ej. 6E, EUR/USD de CME) es un hecho auditable de un único libro de órdenes — exactamente el tipo de medición que §2 acaba de descartar para Forex spot. La vinculación económica entre ambos no es una coincidencia estadística que haya que descubrir: futuro y contado están atados por paridad de tasas de interés/costo de acarreo, y mesas de arbitraje mantienen esa relación cointegrada de forma casi continua. Hay precedente académico serio de estudios de descubrimiento de precio futuro↔contado en índices y en FX — la pregunta no es ilegítima.

### 3.2 Condiciones de validez

1. El futuro se modela como **instrumento propio** (`asset_class = FUTURE`, `venue = CME`, su propia plantilla de sesión) — nunca fusionado con `EURUSD-1M-ICMARKETS`. La tripleta de identidad de ADR-0155 y la regla de no-fusión ya lo exigen sin excepción.
2. El uso del volumen de futuro como insumo de una estrategia de contado se modela como **Fuente de Señal** distinta del **Destino de Ejecución** (ADR-0158) — no como una fila fusionada de dos series.
3. Toda lectura del futuro respeta Alineación por Reloj generalizada a toda fuente (ADR-0158, que ya extiende ADR-0114 Bar-Open Alignment): solo se puede usar una barra de futuro cuyo cierre sea `≤ t`, nunca la barra en formación ni un agregado diario reportado con retraso.
4. La política de rolo/contrato continuo del futuro se resuelve como cálculo de lectura sobre el contrato específico, nunca como columna materializada — exactamente lo que ADR-0155 ya fija como principio general ("el primitivo es el contrato específico... la política de encadenamiento... es una feature futura de rolo, separada del ingest crudo").

### 3.3 Modos de fallo concretos, con mecanismo

1. **Desajuste de calendario por plantilla de sesión equivocada.** 6E tiene un hueco diario de mantenimiento que Forex no tiene. Si el futuro hereda o comparte plantilla de sesión con el instrumento Forex en vez de tener la suya propia (`session-catalog`, ADR-0162), el Gate de Coherencia de Calendario puede leer el hueco de mantenimiento del futuro como si fuera una anomalía del contado, o viceversa — falso positivo o falso negativo según el sentido del error.
2. **Contaminación por rolo de contrato.** El volumen de futuros se concentra en el contrato del frente y migra abruptamente al vencimiento (trimestral en 6E). Una serie de "volumen de futuro" construida sin lógica de rolo mostraría picos/caídas artificiales cada ~3 meses que no tienen nada que ver con actividad real en EURUSD — el mecanismo exacto que ADR-0155 ya previno al prohibir persistir un contrato continuo materializado.
3. **Confusión de factor de riesgo (base de carry).** El precio del futuro difiere del contado por el diferencial de tasas. Un pico de volumen de futuro puede originarse en una mesa rebalanceando cobertura de tasas antes de una decisión de un banco central, sin ningún contenido direccional sobre EURUSD — una estrategia que "compra contado cuando el volumen de 6E se dispara" puede estar leyendo pura actividad de carry trade, no una señal de dirección.
4. **Look-ahead por desajuste de granularidad/latencia de reporte.** El volumen de futuros suele reportarse agregado (diario, con liquidación) con una cadencia distinta a la barra de 1 minuto de la estrategia. Usar "el volumen total del día de hoy" para decidir intradía en contado es mirar al futuro dentro del propio día — exactamente el caso que la Alineación por Reloj de ADR-0158 existe para bloquear, y el punto donde es más fácil introducirlo sin darse cuenta porque el volumen agregado "se siente" como un dato pasivo, no como una fuente con su propio reloj.
5. **Minería de correlación sin mecanismo.** Buscar "¿el volumen de 6E en el lag N predice el retorno de EURUSD?" barriendo lags/ventanas es caza de sesgo de selección igual que cualquier otra búsqueda de alfa — que el dato de entrada sea confiable no exime del mismo rigor de DSR/CPCV (ADR-0067/0063) que cualquier señal.

### 3.4 El caso "no tiene sentido en absoluto" — por qué es un contendiente serio, no un hombre de paja

Si el arbitraje futuro↔contado es lo bastante rápido (sub-segundo, como es habitual en FX/futuros líquidos hoy), toda la información que el volumen de futuros pudiera aportar ya está reflejada en el precio y el spread del propio contado dentro de la misma barra de 1 minuto — el volumen de futuro y el retorno de contado se moverían juntos (correlación visible) sin que el primero añada nada que el segundo no already contuviera (información incremental nula, condicional al precio ya observado). Esta hipótesis nula —"el contado ya lo sabe"— debe tratarse como el punto de partida por defecto, no como el caso raro: es exactamente el resultado que predice un mercado eficiente a esta granularidad. La carga de la prueba está en demostrar mejora fuera de muestra sobre un modelo que solo usa el propio contado, no en asumir que "más dato correlacionado" implica "más señal".

### 3.5 Qué se necesita para probarla o descartarla — y qué ya existe/falta

- **Ya resuelto arquitectónicamente:** el mecanismo de cablear una fuente de señal externa hacia un destino de ejecución distinto, con reloj propio y sin relleno invisible de huecos, ya está sellado en ADR-0158 — nada que inventar en este eje.
- **Bloqueo de secuenciación declarado, no de esta pregunta:** ADR-0158 depende de TASK-059 (contrato del momento de conocibilidad), que hoy está **abierta, pendiente de despacho** (`docs/execution/TASK-059-contrato-momento-de-conocibilidad.md`) — la topología cross-market no es construible hasta cerrarla, para esta hipótesis o para cualquier otra que la use.
- **Falta por completo:** una fuente real de datos de futuros (CME 6E vía CQG/NinjaTrader, que el propietario ya anticipa) — hoy no hay ni un `instrument_master` de futuro EUR/USD, ni plantilla de sesión de futuro, ni política de rolo construida (diferida por diseño en ADR-0155).
- **Prueba empírica mínima, en orden:** (1) construir el instrumento de futuro con su propia identidad y sesión; (2) resolver rolo de contrato como feature de lectura; (3) correr la hipótesis como **una candidata más** dentro del guantelete de robustez (WFA/CPCV/PBO/DSR) contra una línea base de "solo contado" — nunca con exención por venir de una fuente centralizada; (4) declarar el resultado, incluida la posibilidad honesta de "sin mejora fuera de muestra" (§3.4).

---

## 4. Implicaciones para el diseño ya sellado

1. **`real_volume_unit_declared` (ADR-0162) ya está bien diseñado para el caso general — nada que abrir por eso.** La regla "`NULL` = convención no declarada, nunca asumir por defecto" es exactamente la postura correcta frente a la evidencia de este archivo. No corresponde ampliarlo por esta auditoría.
2. **Vacío real, señalado, no resuelto por mí (Architect decide):** `real_volume_unit_declared` es un campo estático por `(instrument_id, venue)` en `broker_instrument_terms` — una convención fija en el tiempo. La evidencia de §1.3 (el mismo bróker, el mismo símbolo, dos regímenes de granularidad de volumen incompatibles con un mes de diferencia en 2017) plantea una pregunta que ningún documento revisado responde: **si la convención de un campo cambia a mitad de la relación con un bróker, un campo estático por bróker×instrumento la captura mal** — aplicaría retroactivamente la última declaración a todo el histórico, o dejaría todo sin declarar por prudencia (que es lo que hoy ocurre, y es lo correcto mientras no se decida lo contrario). Búsqueda declarada: `grep -rli "vigencia\|effective_from\|versionado.*broker_instrument_terms" docs/adr/ docs/features/` — cero coincidencias que resuelvan esto. No propongo el mecanismo (versionado temporal de términos, o exigir un campo de vigencia) porque no es evidencia clara todavía de que la unidad real cambió versus que cambió la granularidad/redondeo de la misma unidad — es del Architect decidir si esto merece una enmienda o si el `NULL` conservador ya basta como respuesta permanente.
3. **La política de rolo de futuros ya está fijada como diferida — nada que abrir.** ADR-0155 ya declara que el primitivo es el contrato específico y que el encadenamiento continuo es una feature de lectura futura, separada del ingest crudo. La hipótesis de §3 no exige ninguna decisión nueva ahí; solo exige construirla cuando llegue el conector de futuros.
4. **El cableado cross-market de la hipótesis de §3 ya está sellado (ADR-0158) — el bloqueo es de secuenciación, no de diseño.** Vale la pena que el Tech-Lead lo tenga presente al priorizar: TASK-059 pasa de "prerequisito abstracto" a "prerequisito concreto de una hipótesis de producto ya nombrada por el propietario" en el momento en que llegue el conector de CQG/NinjaTrader.

---

## 5. Las siete comprobaciones de razonamiento crítico de dominio, aplicadas a §2 y §3

1. **Límite degenerado (`critical-domain-reasoning.md` #1).** Empujar "volumen de bróker retail = proxy de mercado" al límite (un bróker con cero clientes seguiría "teniendo volumen de mercado" bajo esa lectura errónea) muestra el absurdo: el volumen de un bróker tiende, en el límite de tamaño de bróker→0, a cero mientras el mercado sigue ahí — la lectura correcta es que mide participación propia, no interés de mercado. Para §3: empujar "el futuro y el contado están perfectamente arbitrados" al límite (velocidad de arbitraje→∞) hace que la información incremental del volumen de futuro tienda a cero exactamente — es el caso §3.4, no un caso borde ignorable, es el límite físico del mecanismo mismo.
2. **Gate de especialista.** Este informe es del Quant-Engineer, como exige el dominio (estadística/finanzas cuantitativas). Las dos correcciones de encuadre de §0 no dependen de otro especialista — son medición directa. La pregunta de vigencia temporal de §4.2 se remite explícitamente al Architect, que es quien decide esquema, no yo.
3. **Desconfianza del rigor impresionante.** La cita "en futuros el volumen no varía de bróker a bróker" suena sólida y **es cierta** — pero de ahí NO se sigue que "por tanto sirve automáticamente para Forex": la solidez del dato de futuros no transfiere solidez a la relación económica entre ambos mercados, que es una hipótesis aparte y debe probarse aparte (§3.4). Tratar "el input es confiable" como sinónimo de "la estrategia funciona" es exactamente el salto que este punto prohíbe.
4. **Re-derivación de supuestos.** El supuesto que sostenía el encargo — "hay un mes de ticks reales solapado que sirve para validar la unidad de volumen" — se rompió al verificar los dos archivos por separado (§0.1): el tick no tiene volumen, y el M1 no tiene volumen en ese mes. El supuesto que sostiene §3 — "futuro y contado se mueven juntos, por tanto uno informa al otro" — se re-derivó en §3.4: se mueven juntos precisamente PORQUE están arbitrados, lo cual puede significar que uno ya contiene toda la información del otro, no que aporte información nueva.
5. **Separar primitivo de política.** El primitivo (persistir `<VOL>` crudo, tal como llega) nunca cambia — es lo que STORY-068 ya hizo bien. La política (qué significa ese número, si es usable, si se combina con futuros) es intercambiable y depende de evidencia que puede actualizarse sin tocar el dato guardado. Este informe es enteramente sobre política, nunca propone tocar el primitivo ya persistido.
6. **Cierre transitivo.** El fenómeno "volumen sin unidad confiable" no toca solo esta columna: toca cualquier feature futura que consuma `volume_traded`/`volume_ticks` de Forex — `order-flow-microstructure.md` (ya se autoexcluye por diseño, §2.2), cualquier indicador de liquidez, cualquier filtro de "operar solo con volumen alto". Todas heredan la misma frontera de §2 si alguna vez leen esta columna.
7. **Distinguir tipos, no promediar.** Este informe trata como cosas distintas, sin promediar: `volume_ticks` (contador, sin ambigüedad de unidad) ≠ `volume_traded` (magnitud física, unidad en disputa); régimen 2010-06→2017-04 (múltiplo limpio de 1,000) ≠ régimen 2017-05→2017-11 (roto) ≠ resto del archivo (ausente); volumen de bróker (muestra sesgada, OTC) ≠ volumen de futuro (población completa, centralizado); correlación (§3.1, medible) ≠ información incremental fuera de muestra (§3.4, lo único que de verdad importa para una estrategia).

---

**Fuentes externas citadas (verificadas por búsqueda, nunca como única evidencia — cruzadas contra la medición de §1):**
- IC Markets, año de fundación (2007) — usado en §0.3 para fechar la imposibilidad de que el archivo sea una serie homogénea de esa fuente.
- Documentación general de `real_volume` en MT5 (depende del bróker; típicamente vacío en Forex spot por ser mercado OTC sin cinta centralizada) — usado en §1.5 y §2 como respaldo de que la ambigüedad de unidad es estructural del formato, no un defecto de este archivo concreto.

**Trazabilidad:** ADR-0154, ADR-0155 (incl. enmienda 2026-07-25), ADR-0158, ADR-0162 (incl. enmienda 2026-07-25), ADR-0114, `docs/features/data-normalization-layer.md`, `docs/features/order-flow-microstructure.md`, `docs/execution/STORY-068-mt5-bar-reader.md`, `docs/execution/TASK-059-contrato-momento-de-conocibilidad.md`, `.agents/plans/quant-2026-07-25-viabilidad-lector-mt5.md` (informe previo del mismo autor), `.agents/plans/esquema-canonico-datos-mercado.md`.
