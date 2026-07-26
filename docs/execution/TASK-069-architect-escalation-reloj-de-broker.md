# TASK-069 · Escalamiento al Architect — lo que se declara para un archivo completo no vale para todo su tramo

| Campo | Valor |
|---|---|
| **ID** | TASK-069 |
| **Tipo** | Task (escalamiento — vacío de diseño, decisión del Architect) |
| **Estado** | Pendiente de despacho al Architect |
| **Épica (Fase)** | EPIC-1 — Soberanía de Datos (`ingest`) |
| **Creada** | 2026-07-26 |
| **Origen** | El propietario identificó el bróker del archivo de 25 años (IC Markets); al confirmar su horario publicado, el Tech-Lead halló que el mecanismo de declaración de ADR-0162 no puede expresarlo |
| **Documento a decidir** | `docs/adr/ADR-0162.md` (Restricción 7 y el patrón de selector de zona con bandera DST) · `docs/features/data-import-wizard.md` · `docs/features/session-catalog.md` |
| **Bloquea** | La canonización de la zona horaria del archivo real de 25 años. **No bloquea** el lector ya construido (STORY-068, cerrada): hoy rechaza la importación si nadie declara zona, que es el comportamiento correcto. |

---

## 1. El hecho verificado

El archivo `datasets/EURUSD_M1_2000-01-01_2025-05-01.csv` es de **IC Markets**. Su horario de servidor está publicado por el propio bróker: **GMT+2 en invierno, GMT+3 en verano**.

Y el cambio **sigue las fechas de Estados Unidos, no las europeas**. Verificado en el aviso del propio bróker: el cambio a GMT+3 de 2025 fue el **domingo 9 de marzo** — segundo domingo de marzo, que es la fecha estadounidense. La fecha europea de ese año fue el 30 de marzo. El motivo que el bróker da es explícito: *"due to the US entering Daylight Savings"*.

## 2. El vacío de diseño

ADR-0162 fija que la zona horaria de origen se declara mediante un **selector de zona mundial con bandera de horario de verano**, y que jamás se infiere. La regla es correcta y no se discute. El problema es de expresividad del mecanismo:

**Ninguna zona del catálogo IANA es "GMT+2 en invierno, GMT+3 en verano, con fechas de cambio estadounidenses".**

- `Europe/Athens` / `Europe/Helsinki` tienen el par de offsets correcto (+2/+3) pero **fechas de cambio europeas**.
- `Etc/GMT-2` tiene el offset de invierno correcto pero **no cambia nunca**.

Si el usuario declara `Europe/Athens`, las barras quedan desfasadas **una hora** durante las ventanas en que las dos reglas no coinciden: aproximadamente del 9 al 30 de marzo (tres semanas) y del 26 de octubre al 2 de noviembre (una semana) de cada año. Sobre 25 años de historia, son unas cien semanas de barras mal colocadas — plausibles, silenciosas, y suficientes para invalidar cualquier estrategia sensible a la hora de sesión.

Si declara `Etc/GMT-2`, quedan desfasadas los ocho meses de horario de verano.

**Las dos opciones disponibles hoy son incorrectas, y ninguna avisa.**

## 3. Por qué no lo resuelve el Tech-Lead

El mecanismo de declaración de zona horaria es doctrina de ADR-0162, y ADR-0162 es del Architect. Corregir la forma de un contrato de declaración no es una corrección de implementación: cambia qué se le pide al usuario, qué se sella como evidencia en el ledger de mapeo, y cómo lo interpreta todo aguas abajo.

## 4. Lo que hay que decidir (sin precocinar la solución)

El Architect decide **cómo** se expresa un reloj de servidor de bróker de forma que el usuario pueda declararlo sin ambigüedad y sin que el sistema adivine. Las preguntas abiertas:

1. **¿Cuál es la forma correcta de la declaración?** Una zona del catálogo IANA no alcanza para este caso. Lo que sí parece haber es un ancla más un desplazamiento — pero la forma exacta, su vocabulario y su validación las decide el Architect.

   > **Hipótesis del Tech-Lead, a validar o refutar — no es el camino a seguir:** el reloj de IC Markets equivale exactamente a *hora de Nueva York más siete horas*, en las dos estaciones (invierno: Nueva York en UTC−5, más siete, da UTC+2; verano: UTC−4, más siete, da UTC+3). Si eso se generaliza, un reloj de bróker se declararía como **zona ancla del catálogo IANA + desplazamiento fijo en horas**, y el par de offsets observados dejaría de ser la declaración para volverse su consecuencia verificable. Sin verificar: si esta forma cubre a los brókers que anclan a Londres o a Sídney en vez de a Nueva York, y si algún bróker usa un tercer patrón que no encaje en "ancla + desplazamiento".

2. **¿Qué se sella en el ledger de mapeo?** Hoy se sella un identificador de zona IANA. Si la forma cambia, cambia la evidencia — y ese ledger es append-only, así que la decisión no se puede rehacer sobre las filas ya escritas.

3. **¿Se puede verificar la declaración contra el archivo, sin inferirla?** Declarar sigue siendo del usuario (ADR-0162 Restricción 7, y el propietario ya rechazó explícitamente inferir por volatilidad o noticias). Pero una declaración **incoherente con la huella del archivo** es detectable: si el usuario declara un reloj sin horario de verano y el archivo muestra el patrón de uno que sí lo tiene, eso es un rechazo legítimo, no una inferencia. ¿Entra esto en el Gate de Coherencia de Calendario que ADR-0162 ya define?

4. **¿Qué hace el catálogo de sesiones con esto?** La plantilla semilla de Forex se calibró sin este caso delante. Si la huella de un archivo real depende de las reglas de cambio de hora del bróker, la plantilla tiene que poder expresarlo o el Gate rechazará archivos correctos.

---

## 4bis. Segundo caso de la misma causa raíz — el archivo mezcla orígenes de dato a lo largo de sus 25 años

Hallado por el Quant-Engineer en la investigación de volumen del 2026-07-26 (`.agents/plans/quant-2026-07-26-volumen-forex-vs-futuros.md`), **verificado de forma independiente por el Tech-Lead sobre el archivo real**. Se añade a esta Task en vez de abrir otra porque comparte exactamente la causa raíz del problema del reloj: **la declaración se toma por archivo, y el archivo no es homogéneo.**

**Evidencia medida:**

- **IC Markets se fundó en 2007** (Sydney; licencia de la ASIC en 2009) y el archivo arranca el `2000.01.03`. Los primeros siete a nueve años **no pueden ser el feed de ejecución de ese bróker** — son histórico rellenado desde un proveedor, práctica habitual de los terminales de MetaTrader.
- **El campo de volumen real es intermitente, no una ventana:** poblado del `2010-06-03` al `2017-11-17`, pero con **2012 y 2013 completamente vacíos** (2011 tiene 4,505 filas y 2014 tiene 23,796).
- **Su magnitud salta sin transición:** media anual de ~137 millones en 2010, ~146 millones en 2015, ~26 millones en 2017, y el último valor del archivo es `159`. Seis órdenes de magnitud entre extremos del mismo campo, con una fractura limpia a mitad de 2017.

**Por qué es un problema de diseño y no un defecto del archivo:**

La identidad de una serie es la tripleta símbolo × temporalidad × bróker, y es doctrina sellada por el propietario que el EURUSD de un bróker no es el de otro y no se funden jamás. Este archivo **se declara íntegro como IC Markets** y, sobre buena parte de su rango, eso no es cierto. Un backtest de 25 años sobre él cruzaría en silencio la frontera entre el feed real del bróker y el histórico rellenado de un tercero — exactamente la fusión que la doctrina prohíbe, pero ocurrida **dentro** de una sola serie declarada, donde ninguna regla la vigila.

**Lo que hay que decidir (el Architect, no el Tech-Lead):**

1. **¿Cómo se representa un archivo cuya procedencia cambia a mitad de su rango?** ¿Se declara por tramos? ¿Se importa como series distintas? ¿Se sella la heterogeneidad como un hecho del linaje, para que quien construya sobre ella lo sepa?
2. **¿El campo que declara la unidad de volumen por bróker debe poder variar en el tiempo?** Hoy es estático, y la medición muestra que la característica del dato cambió a mitad de la relación con el mismo bróker. **Esta es la pregunta que el Quant dejó abierta expresamente para el Architect**, señalando la evidencia y sin resolverla.
3. **¿El reloj también puede variar por tramo?** Es la intersección con §1–§4 de esta Task: si el tramo pre-2007 viene de otro proveedor, su reloj de origen probablemente no es el de IC Markets — y hoy se declararía uno solo para los 25 años.
4. **¿Qué de esto es detectable y por tanto rechazable sin inferir?** Una discontinuidad de seis órdenes de magnitud en un mismo campo, o años enteros vacíos en medio de una ventana poblada, son hechos medibles. Declararlos sospechosos no es inferir la procedencia: es negarse a afirmar homogeneidad que no se observa.

---

## 5. Constancia de la corrección que originó esta Task

El informe del Quant de STORY-068 (`.agents/plans/quant-2026-07-25-viabilidad-lector-mt5.md` §3.5) concluyó que el patrón semanal idéntico en invierno y verano indicaba **un reloj de offset fijo sin horario de verano**. **La inferencia estaba invertida** y el informe ya lleva la corrección anotada.

El patrón se ve constante **precisamente porque** el reloj sí sigue el cambio: los eventos del mercado están fijos en hora de Nueva York y el reloj del servidor se mueve con ellos, así que desde ese reloj el cierre y la apertura caen siempre a la misma hora local. Un reloj de offset fijo produciría lo contrario — el patrón se correría una hora al cambiar la estación.

**La lección de método:** una huella constante entre estaciones no prueba ausencia de cambio de hora; prueba que el reloj de observación acompaña al fenómeno observado. Distinguir las dos cosas exige un dato externo — aquí, el bróker declarado por el propietario. **Ningún gate automático lo habría detectado**, porque la conclusión era internamente coherente con lo medido.
