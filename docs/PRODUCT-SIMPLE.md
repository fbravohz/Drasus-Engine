# Drasus Engine — Explicado Sin Tecnicismos

El mismo producto que describe [`PRODUCT.md`](./PRODUCT.md), contado para alguien que **nunca ha oído hablar de trading algorítmico ni de estadística avanzada**, pero terminó la preparatoria y sabe quién fue Darwin.

**Regla de este documento:** cero palabras que haya que buscar en internet. Cuando un concepto es inevitable, se explica con algo que la persona ya conoce.

**Gemelo técnico:** [`PRODUCT.md`](./PRODUCT.md). Los identificadores `CAP-01`…`CAP-10` son **los mismos en los dos documentos**. Quien quiera profundizar en cualquier punto, salta al gemelo por su número y encuentra exactamente el mismo tema con el vocabulario completo.

---

## 0. La imagen que sostiene todo el documento

Antes de cualquier detalle, una sola imagen. Todo lo demás proviene de aquí:

> **Drasus Engine es el centro de diseño, fabricación, validación y ejecución más avanzado del mercado para sistemas automáticos de inversión.**
> **Explora y afina** constantemente el universo de posibilidades para encontrar candidatos viables, los **somete a pruebas extremas** para descartar a los frágiles, explora combinaciones óptimas con los supervivientes para **ensamblar portafolios sólidos**, **gestiona sus operaciones** en el mercado real y aplica **medidas de proteccion extremas para salvaguardar el capital**.

Nadie más ofrece el ciclo completo bajo el mismo techo. Te venden soluciones en fragmentos: el motor por un lado, el validador anticuado por otro y te dejan solo en la ejecución. Drasus es la única infraestructura completa y sin fisuras del mercado, unificando la cadena completa y entregando el estándar más exigente y definitivo de punta a punta. **El producto es la solución definitiva, la ventaja injusta de tenerlo todo**.

---

## 1. Nivel General — Qué es esto

### 1.1 En una frase

Un programa que **crea estrategias automaticas para operar en los mercados financieros, y luego intenta destruirlas** para quedarse solo con las que sobreviven.

### 1.2 En un párrafo

Imagina que quieres contratar a alguien que compre y venda por ti en los mercados financieros mientras duermes. Podrías entrevistar a cinco candidatos y elegir al que mejor te parezca. Drasus hace otra cosa: genera **millones** de candidatos de manera automatica, y luego los somete a una batería de exámenes brutales —diseñados para reprobarlos y para descubrir a los que solo tuvieron suerte—. Los poquísimos que pasan todos los exámenes se van a un período de prueba con dinero simulado. Los que también sobreviven ahí, y solo esos, empiezan a operar de verdad. Y mientras operan, el sistema los vigila; si uno se empieza a echar a perder, lo jubila y usa lo aprendido para generar mejor a la siguiente generación. **Todo esto corre en tu propia computadora.** Nada se ejecuta en servidores externos.

### 1.3 El problema que estamos resolviendo

Aquí está el corazón del asunto, y se entiende con un ejemplo cotidiano.

**Imagina que lanzas una moneda diez veces y sale cara diez veces seguidas.** Impresionante, ¿no? Pensarías que la moneda está cargada.

Ahora imagina que **mil personas** lanzan una moneda diez veces cada una. Es prácticamente seguro que a alguna le salgan diez caras seguidas. Esa persona no tiene una moneda mágica. Tuvo suerte, y la suerte era esperable porque hubo mil intentos.

**Eso, exactamente eso, es lo que pasa en el trading automático.** Si pruebas diez mil estrategias en datos del pasado, algunas se verán espectaculares por pura casualidad. Y como se ven espectaculares, la gente les mete dinero real. Y pierden, porque nunca hubo nada ahí — solo fueron el afortunado de los mil lanzamientos.

Los cuatro problemas concretos de la industria hoy:

- **Casi nadie cuenta cuántas veces intentó.** Y sin ese número, no hay forma de saber si el resultado es mérito o suerte.
- **Las herramientas existentes se enfocan en producir más candidatos**, cuando producir candidatos ya es lo fácil. Lo difícil es descartarlos bien.
- **Las simulaciones hacen trampa sin querer.** Suponen que compras al precio exacto que querías, sin comisiones realistas y sin los retrasos que existen en la vida real. Un resultado bonito en esas condiciones no significa nada.
- **Tu trabajo no es tuyo.** Casi todas las plataformas corren en la nube: tus ideas, tus resultados y tus operaciones viven en el servidor de otro, que puede verlos.

### 1.4 Nuestra apuesta

**El valor no está en generar ideas. Está en descartarlas bien.**

Drasus invierte el esfuerzo de la industria. Generar es una etapa; examinar es el producto. Y el examen parte de una postura hostil: **toda estrategia es culpable hasta que demuestre lo contrario**, y demostrarlo significa sobrevivir a una docena de intentos independientes de romperla.

### 1.5 Lo que Drasus **no** es

Decirlo evita malentendidos:

- **No vende datos del mercado.** Trae los tuyos, de tu proveedor o tu casa de bolsa, y los limpia.
- **No maneja tu dinero.** Tú operas tu propia cuenta a través de Drasus. Es una herramienta, no un fondo de inversión.
- **No promete que vas a ganar.** Promete un proceso de evaluación riguroso y honesto. Lo que ese proceso encuentre depende del mercado.
- **No es una caja negra.** Cada estrategia que produce se puede leer, como una lista de reglas. No es "la inteligencia artificial dijo que compres".
- **No corre en la nube.** Corre en tu computadora.

### 1.6 Para quién es

Cada fila describe **lo que la persona hace**, no cómo la llama el sector. Nadie se presenta diciendo "soy trader retail"; sí se reconoce en "invierto mi propio dinero siguiendo reglas".

| Quién | Qué se lleva |
|---|---|
| Alguien a quien le gusta probar ideas con datos y ver si de verdad funcionan | Un laboratorio completo, sin pagar la infraestructura que usa un banco |
| Alguien que invierte su propio dinero, por su cuenta, siguiendo reglas en vez de corazonadas | Estrategias con evidencia real detrás, y frenos de emergencia que le cuidan la cuenta |
| Alguien que quiere que una empresa le preste capital para operar y tiene que pasar una prueba para conseguirlo (lo llaman "cuenta fondeada") | Un simulacro que aplica las reglas exactas de esa prueba **antes** de pagarla |
| Alguien que ya maneja dinero de otras personas, o varias cuentas a la vez | Control de la flota completa, con permisos por persona y reportes auditables |

---

## 2. Nivel Medio — Las diez piezas

Cada pieza responde tres preguntas: **¿qué le duele al usuario?**, **¿qué hace Drasus?** y **¿por qué así?**. Los números `CAP-NN` son los mismos que en el documento técnico.

---

### CAP-01 — Que los datos no mientan

**Lo que duele.** Todo lo demás depende de esto. Si mides con una báscula descompuesta, no importa qué tan buena sea tu receta.

Los datos del mercado llegan sucios: horas faltantes, precios imposibles, husos horarios revueltos. Pero el error verdaderamente grave es otro, y es tan sutil que casi nadie lo ve:

> **Hacer trampa con información del futuro.**
>
> Imagina que pruebas tu estrategia sobre el año pasado. Y sin darte cuenta, usas el dato de desempleo **corregido**, el que publicaron tres meses después. En su momento nadie tenía ese número. Tu estrategia se ve brillante, pero solo porque le diste las respuestas del examen antes de aplicárselo.

Esto le pasa a casi todo el mundo, y explica una enorme parte de las estrategias que funcionan en la prueba y fracasan en la vida real.

**Qué hace Drasus.** Guarda cada dato con la fecha exacta en que estuvo disponible por primera vez, y guarda las correcciones posteriores **por separado**. Cuando la simulación pregunta "¿qué se sabía el 3 de marzo?", solo puede ver lo que existía el 3 de marzo. No es una revisión al final: es cómo está construido el archivero.

Además revisa los datos en seis capas de limpieza y avisa por adelantado cuando tu proveedor simplemente no tiene algo, en vez de fallar con un error incomprensible a media descarga.

**Por qué así.** Porque este es el único error del que no se puede regresar: si los datos mienten, todos los exámenes posteriores están evaluando una fantasía y ni siquiera lo saben.

**En el documento técnico:** `CAP-01` · **Cuándo se construye:** ahora mismo, es la fase en curso.

---

### CAP-02 — Un simulador que no hace trampa

**Lo que duele.** Simular es fácil. Simular honestamente es difícil, y casi nadie lo hace.

La mayoría de las simulaciones suponen que compras exactamente al precio que querías, en el instante que querías, sin costos. En la vida real: pagas comisión, el precio se te mueve entre que decides y ejecutas, y —el detalle más traicionero— tus órdenes "a precio fijo" **solo se llenan cuando el mercado va en tu contra**. Si iba a tu favor, se te escapó.

Y hay un error todavía peor, que aparece constantemente: **decidir con el precio de cierre de una hora y comprar a ese mismo precio de cierre.** En la realidad, cuando ya conoces el cierre, ese momento ya pasó. Es como apostar al caballo cuando ya cruzó la meta.

**Qué hace Drasus.** Dos simuladores. Uno rápido, para descartar miles de candidatos a toda velocidad. Y uno lento y minucioso, que reproduce el mercado paso a paso con comisiones, retrasos y el costo real de ejecutar. Con una regla de honestidad entre ambos: **el rápido nunca puede reportar mejores resultados que el minucioso.** Si hay diferencia, gana el pesimista.

Y una regla que no se puede desactivar: lo que decides con el cierre de una hora se ejecuta en la apertura de la siguiente. Siempre.

**Por qué así.** Porque toda la ambición del proyecto se apoya aquí. Si el simulador miente aunque sea un poco, los mil exámenes que vienen después están calificando mentiras con mucha precisión.

**En el documento técnico:** `CAP-02` · **Cuándo se construye:** la fase siguiente.

---

### CAP-03 — Buscar estrategias automáticamente (empezando por Darwin)

**Lo que duele.** Una persona se puede imaginar veinte estrategias. El universo de combinaciones posibles es de miles de millones. Las buenas casi con seguridad no están entre las veinte que se te ocurrieron.

**Qué hace Drasus.** Selección natural, literalmente el mecanismo de Darwin, aplicado a programas de trading:

1. **Nacen** miles de estrategias con reglas al azar.
2. **Compiten**: cada una se prueba contra la historia del mercado.
3. **Las mejores se reproducen**: se cruzan entre sí y sus "hijas" heredan partes de cada padre.
4. **Hay mutaciones**: pequeños cambios al azar, que a veces resultan ser mejoras.
5. **Se repite** por cientos de generaciones.

Esto tiene nombre técnico —**NSGA-II**— pero lo que importa es su diferencia con la evolución "normal", y es una idea preciosa:

> **No hay un solo "mejor".**
>
> En la naturaleza, ¿quién es mejor: el guepardo o la tortuga? La pregunta no tiene sentido. El guepardo es rapidísimo y frágil. La tortuga es lentísima y vive doscientos años. **Cada uno es imbatible en su propia dimensión.** No compiten en la misma escala.
>
> Con las estrategias pasa igual. Una gana mucho pero tiene sustos enormes. Otra gana poco pero es tranquilísima. ¿Cuál es mejor? **Depende de ti, no del algoritmo.**
>
> Casi todas las herramientas te obligan a decidir esto **antes** de buscar: "dime cuánto vale un punto de ganancia contra un punto de susto". Y con esa fórmula arbitraria buscan. El problema es evidente: elegiste a ciegas, antes de ver qué había.
>
> Drasus no te obliga. Te entrega **todo el abanico**: el guepardo, la tortuga, y todo lo que hay en medio. Tú eliges después, viendo las opciones reales.

**Darwin es la puerta de entrada, no la única puerta.** La evolución se explica primero porque es la más fácil de entender, pero Drasus no está casado con ella. Buscar estrategias es como buscar algo perdido en una casa: hay muchas maneras, y la buena depende de qué buscas.

- **Revisar cuarto por cuarto, sin saltarse ninguno.** Lento y exhaustivo, pero no se te escapa nada.
- **Ir aprendiendo dónde conviene buscar.** Cada intento te dice algo sobre dónde vale la pena mirar después, y no repites las zonas vacías.
- **Mapear la forma del terreno.** En vez de buscar el punto más alto, buscas la meseta ancha — porque una cima de aguja se cae con el primer viento.
- **Ir probando y apostar más a lo que va funcionando.** Repartes el esfuerzo sobre la marcha en vez de decidirlo todo al principio.

Cada método viene de un mundo distinto —la biología, la ingeniería, la logística, el diseño de medicamentos, hasta la física— y Drasus está construido para poder **enchufar cualquiera de ellos sin rehacer nada**. Esto no es un adorno: es infraestructura de investigación, y dejar fuera un método que funciona porque "aquí siempre se usó evolución" sería tirar dinero.

**Y un detalle que casi nadie tiene:** desde el primerísimo intento, y **sin importar qué método se use**, el sistema **lleva la cuenta de cuántas veces buscó**. Es el número de los mil lanzamientos de moneda de §1.3. Sin él, el examen de la siguiente pieza es imposible. Y no se puede reconstruir después: o lo cuentas desde el principio, o lo perdiste. De hecho es el requisito de entrada: un método de búsqueda que no sepa decir cuántas veces intentó, no se admite.

**Por qué así.** Porque probar todo a lo bruto y quedarse con lo mejor es exactamente la máquina de producir suerte disfrazada de talento. Lo que salva no es el método concreto: es buscar con varios objetivos a la vez, ser honesto sobre los compromisos, y **contar los intentos**.

**En el documento técnico:** `CAP-03`.

---

### CAP-04 — El examen diseñado para reprobarte

Esta es **la pieza que justifica el proyecto entero**. Si solo puedes explicar una, explica esta.

**Lo que duele.** Volvemos a las mil monedas. Después de generar miles de estrategias, algunas se ven espectaculares por casualidad. Y una gráfica que sube es **hipnótica**: la gente le cree, le mete dinero y pierde.

**Qué hace Drasus.** Una serie de exámenes independientes, cada uno atacando una forma distinta de haber tenido suerte. Van del más barato al más caro: quien reprueba uno, ya no presenta los siguientes.

- **La prueba del profesor tramposo.** Se parte la historia en muchos pedazos y se estudian todas las combinaciones posibles de "con estos pedazos estudias, con estos otros te examino". Al final se calcula algo demoledor: *la probabilidad de que la estrategia que se ve mejor en el estudio sea, en realidad, del montón cuando la examinas*.
- **La prueba de avanzar en el tiempo.** Se afina la estrategia con lo que se sabía hasta enero, se le deja operar febrero, y se avanza. Repetido durante años. Mide si sirve cuando solo puede ver el pasado — que es la única situación real.
- **La prueba de las mil vidas paralelas.** Se revuelven y se alteran sus operaciones miles de veces, para ver el abanico completo de resultados posibles. La pregunta: *lo que obtuvo, ¿está en el centro de ese abanico, o fue la vida afortunada?*
- **El descuento por haber intentado mucho.** Aquí se usa el número que se venía contando desde `CAP-03`. Si probaste diez mil configuraciones, el resultado de la ganadora se castiga en proporción. **Es el examen que corrige el problema de las mil monedas, y es el que casi ninguna herramienta del mercado aplica.**
- **La prueba del mercado hermano.** Se corre la misma estrategia en otro mercado parecido, **sin ajustarle nada**. Si aprendió algo real del comportamiento humano, aguanta. Si solo memorizó las casualidades de un gráfico, se derrumba.
- **La prueba de quitar piezas.** Se le apagan sus reglas una por una. Si apagar una regla no cambia nada, esa regla era adorno inventado para que el pasado cuadrara. Se corta.
- **La prueba contra el azar.** Se compara contra una versión de sí misma que decide lanzando una moneda. Sorprende de verdad cuántas no le ganan.
- **El equipo rojo.** Un componente cuyo único trabajo es ensuciar los datos a propósito buscando romperla.
- **La prueba del terreno firme.** Si mover una perilla un 1% derrumba el resultado, la estrategia está parada en la punta de un alfiler y el mercado real la va a tirar. Se buscan las que están paradas en una meseta ancha.
- **El examinador de cuenta fondeada.** Aplica las reglas exactas de la empresa que te va a evaluar (cuánto puedes perder al día, cuánto en total) **antes** de que pagues por presentar esa evaluación.

Al final, todo se resume en **una calificación y un veredicto escrito en español**: en qué examen falló, y qué significa.

**Por qué así.** Porque generar es fácil y descartar es difícil, y todo el mundo invierte en lo fácil. **Aquí es donde Drasus le gana a las herramientas que ya existen**, y es la razón por la que el proyecto tiene sentido.

**En el documento técnico:** `CAP-04`.

---

### CAP-05 — El período de prueba, sin dinero

**Lo que duele.** Una estrategia puede pasar todos los exámenes sobre el pasado y aun así fallar hoy. Porque el pasado no contiene el presente: no sabe cómo está la liquidez esta semana ni cómo se está portando tu casa de bolsa este mes.

**Qué hace Drasus.** Antes de un solo peso real, la estrategia opera **en vivo pero en falso** durante un período de prueba (por defecto una semana, ajustable). Compra y vende contra el mercado real, en tiempo real, con dinero imaginario. Y mientras tanto, el sistema compara *sin parar* lo que está haciendo contra lo que su examen predijo que haría.

Si se desvía demasiado, **se retira antes de que el período termine**. El sistema no espera cortésmente a que acabe la semana para admitir lo que ya vio.

**Por qué así.** Es la clase de manejo antes de que te den la licencia. Descubrir el problema aquí cuesta cero. Descubrirlo el lunes siguiente cuesta dinero real.

**En el documento técnico:** `CAP-05`.

---

### CAP-06 — Los frenos, el cinturón y el botón rojo

**Lo que duele.** Los programas que manejan dinero fallan de maneras que otros programas no: se desconectan a media operación, mandan la orden dos veces, calculan mal el tamaño. Y fallan **a velocidad de máquina**, no a velocidad de humano. Para cuando lo notas, ya pasó cincuenta veces.

**Qué hace Drasus.** Varias capas de protección, cada una funcionando por separado:

- **Once revisiones antes de cada orden**, en menos de una milésima de segundo. Tamaño, exposición, coherencia con lo que ya tienes abierto.
- **Un vigilante que vive fuera del programa principal.** Esto es clave: si el vigilante viviera *dentro* del programa que vigila, se colgaría junto con él. Vive aparte, revisa el pulso del sistema, y **corta todo** si detecta que algo se rompió.
- **Recuperación tras un apagón.** Si se va la luz a media operación, al reencender no adivina: le pregunta a la casa de bolsa qué pasó realmente y reconstruye el estado.
- **Tus contraseñas nunca salen de tu computadora.** En ningún plan, ni el gratuito. Ni siquiera cifradas. No salen.
- **Límites globales que ninguna estrategia puede saltarse**, sin importar lo que su propia lógica diga.

**Por qué así.** Estas piezas no ganan dinero: **evitan la ruina**. Y se construyen en la misma etapa en que el sistema empieza a arriesgar, nunca después. Ponerle el cinturón al coche cuando ya vas en la carretera no cuenta.

**En el documento técnico:** `CAP-06`.

---

### CAP-07 — El equipo, no la estrella

**Lo que duele.** Aquí hay una trampa que arruina cuentas y casi nadie ve venir.

> Tienes diez estrategias, todas rentables. Te sientes diversificado.
>
> Pero si las diez ganan los mismos días y pierden los mismos días, **no tienes diez apuestas: tienes una sola apuesta grande, disfrazada de diez pequeñas.** El día que le va mal, le va mal a todo a la vez.

**Qué hace Drasus.** Analiza cómo se comportan las estrategias **entre sí**, no solo cada una por su lado. Las agrupa por parecido de comportamiento —las que se mueven juntas quedan en la misma familia— y reparte el dinero por familias, no por estrategia. Así una familia de siete clones no acapara siete veces el capital de una estrategia genuinamente distinta.

Y hay una vuelta de tuerca: **el criadero de `CAP-03` sabe qué tienes operando**. En vez de producir más de lo mismo, produce candidatas que *complementen* lo que ya tienes. La fábrica sabe qué le falta al equipo.

**Por qué así.** Porque aquí es donde el trabajo de investigación se convierte —o no— en resultado. La mayoría de las herramientas producen a ciegas y te dejan a ti el problema de armar el equipo.

**En el documento técnico:** `CAP-07`.

---

### CAP-08 — La fábrica que aprende sola

**Lo que duele.** Ninguna estrategia funciona para siempre. El mercado cambia y la ventaja se gasta. La pregunta real no es *"¿funciona?"*, es *"¿sigue funcionando, y me voy a enterar antes de que me duela?"*.

**Qué hace Drasus.** Todos los días compara lo que creía contra lo que la casa de bolsa dice que realmente pasó. Detecta cuando una estrategia se está degradando y la jubila —con pausa reversible, no borrada— guardando su historial completo.

Y luego **lo que casi ninguna herramienta hace**: toma la lección del fracaso y la manda **de regreso al criadero** como una restricción nueva. "Esto que acabas de ver fallar, deja de producirlo."

**Por qué así.** Es la diferencia entre una herramienta y una fábrica. Una herramienta que descubre pero no aprende repite sus errores en serie. La meta declarada: **treinta días operando sola, sin que nadie intervenga de emergencia.**

**En el documento técnico:** `CAP-08`.

---

### CAP-09 — Un tablero donde se ve el razonamiento

**Lo que duele.** Las herramientas de este mundo te dan a elegir entre dos cosas malas: una pantalla bonita que esconde todo detrás de botones (no puedes auditar nada), o una pantalla de programador donde todo es posible y nada se entiende.

Y hay un problema peor. Cuando el criadero produce una estrategia con cuarenta reglas encadenadas, **nadie la entiende**. Ni quien la generó.

**Qué hace Drasus.** Un lienzo visual donde cada pieza del sistema es un bloque que se conecta con otros, como un diagrama. Cada bloque se puede abrir y ver por dentro.

Y dos capacidades que resuelven el problema de la caja negra:

- **El traductor.** Convierte esas cuarenta reglas encadenadas en **un párrafo en español** que cualquiera puede leer. "Compra cuando el precio lleva tres horas subiendo y el volumen es mayor al promedio de la semana, salvo los viernes por la tarde."
- **El rebobinador.** Puedes tomar cualquier simulación ya terminada, rebobinarla, plantar el cursor en cualquier instante y ver **exactamente** qué veía la estrategia y por qué decidió lo que decidió, en ese segundo.

**Por qué así.** Porque un sistema que te dice "confía en mí" no se distingue de la superstición. Poder auditar no es un lujo de la interfaz: es la razón de ser del producto.

**En el documento técnico:** `CAP-09`.

---

### CAP-10 — De quién es tu trabajo

**Lo que duele.** Hoy tienes que elegir entre dos cosas incómodas: o corres todo en tu máquina y no tienes ningún servicio, o subes todo a la nube y el dueño de la nube ve tus ideas, tus resultados y tus operaciones.

**Qué hace Drasus.** Divide el mundo en tres partes, y dice en voz alta qué pasa en cada una:

| Parte | Qué hace | Dónde está |
|---|---|---|
| **La pantalla** | Lo que ves y tocas | Tu computadora |
| **El motor** | Todos los cálculos y todas las órdenes | Tu computadora o tu servidor privado |
| **La oficina central** | Cuentas, licencias, cobros. **Nunca calcula nada tuyo.** | Servidores de Drasus |

Y la regla de privacidad, dicha sin letra chica:

- **Plan gratuito:** tu trabajo (estrategias, resultados) viaja a Drasus y es de Drasus, según los términos que aceptas. **Pagas con datos, y se te dice de frente.**
- **Plan de pago al corriente:** ese envío **se apaga en tu computadora**. No es "prometemos no mirar": es que no sale.
- **Plan de pago vencido:** el envío se reactiva. Tu trabajo nunca se borra.
- **En todos los planes, sin excepción:** las contraseñas de tu casa de bolsa **jamás salen de tu máquina**.

Debajo hay catorce piezas de infraestructura —cuentas, licencias, planes, medición de uso, consentimientos, respaldos cifrados, permisos por persona, portabilidad de tus datos— construidas **antes** de tener un solo cliente.

**Por qué así.** Porque instalar la plomería antes de construir la casa cuesta una fracción de lo que cuesta romper los muros después. Y porque cobrar sin volverse el custodio del trabajo ajeno es una decisión de producto, no un detalle técnico.

**En el documento técnico:** `CAP-10`.

---

## 3. En qué orden se construye

La regla es una sola: **acortar lo más posible el camino hasta que el sistema opere dinero real.** Cada etapa entrega algo completo y utilizable, no un pedazo.

| Etapa | Qué se termina | Estado |
|---|---|---|
| Cimientos | El esqueleto, y la plomería del negocio | ✅ y 🟡 |
| 1 | Que los datos no mientan | 🟡 en curso |
| 2 | El simulador honesto | pendiente |
| 3 | El criadero (Darwin) | pendiente |
| 4 | El examen brutal | pendiente |
| 5 | **Primer dinero real** | pendiente |
| 6 | Manejar la flota completa | pendiente |
| 7 | Que funcione sola 24/7 | pendiente |
| 8 | El tablero visual completo | pendiente |

**Por qué en ese orden y no en otro:**

- Si los datos mienten, el simulador miente. **Primero los datos.**
- Si el simulador miente, el criadero produce basura muy convincente. **Primero el simulador.**
- El examen del descuento por intentos necesita que alguien **haya contado los intentos**. El criadero tiene que existir antes que el examen.

### 3.1 Ideas en exploración (todavía no son promesas)

Hay unas cuarenta líneas de investigación documentadas: redes neuronales profundas, minería de estrategias entre varias computadoras, un mercado donde comprar y vender estrategias cifradas, opciones financieras, incluso computación cuántica. **No son parte del plan comprometido.** Son ideas con estudio previo, que solo avanzan si demuestran valer la pena.

---

## 4. El guión

**Uno solo, válido para cualquier persona.** No hay versión para médicos, para abogados ni para ingenieros: si hiciera falta adaptarlo a la profesión de quien escucha, el guión estaría mal escrito.

Funciona porque se apoya en dos cosas que todo el mundo ya trae puestas: **el azar de las monedas** y **Darwin**. Nadie necesita saber de mercados para seguirlo.

> ¿Sabes que si mil personas lanzan una moneda diez veces, a alguna le salen diez caras seguidas? Su moneda no es mágica. Hubo mil intentos.
>
> En invertir con programas pasa exactamente eso: se prueban miles de estrategias sobre el pasado, una se ve preciosa, le meten dinero y pierden. Nunca hubo nada ahí. Fue la afortunada.
>
> Yo construyo la máquina que detecta a esa afortunada y la reprueba. Cría miles de estrategias por selección natural, como Darwin, y luego las mete a un examen diseñado para tumbarlas. Solo las que sobreviven tocan dinero.
>
> Y corre en tu computadora. No en la nube de nadie.

**Si la persona quiere más, no cambies de guión: profundiza por donde le picó.** Cada quien pregunta por lo suyo, y cada pregunta ya tiene su pieza en §2:

| Si pregunta por… | Sigue por |
|---|---|
| "¿y cómo sabes que los datos son buenos?" | `CAP-01` |
| "¿cómo simulas algo que no pasó?" | `CAP-02` |
| "¿cómo las inventa?" — o si le interesó Darwin | `CAP-03` |
| "¿y cómo sabes que no fue suerte?" | `CAP-04` |
| "¿y si falla con dinero real?" | `CAP-05` y `CAP-06` |
| "¿quién ve mis datos?" | `CAP-10` |

---

## 5. Mantenimiento

**Dueño:** el Architect. Se edita junto con [`PRODUCT.md`](./PRODUCT.md), **nunca por separado**.

**Reglas de este documento:**

- **Dos vías, nunca más.** Existe este documento y existe el técnico. **Prohibido añadir versiones por profesión, industria o perfil de interlocutor** — ni aquí, ni en el técnico, ni en material derivado. Si una explicación solo funciona para un gremio, la analogía está mal elegida: se corrige la analogía, no se añade una variante. La prueba es simple: si al conocer a alguien de un oficio nuevo hiciera falta escribir una sección nueva, este documento ya falló.
- **Etiqueta por conducta, no por gremio.** Al describir a quién sirve algo, se dice lo que la persona hace, no cómo la clasifica el sector. Nadie se auto-asigna "trader retail"; todos se reconocen en "invierto mi propio dinero siguiendo reglas". Si un término del sector aporta, va entre paréntesis después de la descripción llana, jamás como título.
- Cada `CAP-NN` de aquí tiene su gemelo con el mismo número en `PRODUCT.md`. Uno sin el otro es un error.
- Cero jerga sin traducir. Si aparece un término técnico, se explica en la misma línea con algo que la persona ya conoce.
- Cero promesas de rentabilidad, aquí y en cualquier material derivado.
- Las ideas en exploración se comunican como exploración, nunca como plan.
- Las analogías se sostienen hasta el final de su sección. Cambiar de metáfora a media explicación obliga al lector a reconstruir todo desde cero, y es la forma más común de perder a alguien.
