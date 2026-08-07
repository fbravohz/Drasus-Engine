# STORY-073 · La pantalla del asistente de importación — primera feature de uso real

> **Orden de trabajo — Spec-Driven.** Fuente de verdad de la ejecución. Vive en git, no en el chat.

| Campo | Valor |
|---|---|
| **ID** | STORY-073 |
| **Título** | La pantalla del asistente de importación — primera feature de uso real |
| **Tipo** | Story (hito) |
| **Épica (Fase)** | EPIC-1 — Soberanía de Datos (`ingest`) |
| **Sprint** | 2 |
| **Estado** | 🟡 **Construida y verificada — 13/14 criterios cumplidos.** Falta solo el criterio 13, que únicamente puede ejecutar el propietario (importar el archivo real desde la aplicación) |
| **Responsable** | Rust-Engineer + Bridge + Flutter-Engineer · auditan Tech-Lead + QA-Engineer |
| **Creada** | 2026-07-31 |
| **Completada** | — (a la espera del criterio 13) |

---

## 0. Resumen ejecutivo

Es la primera vez que el propietario va a **usar** Drasus en vez de verificarlo por terminal. Arrastra su archivo real de EURUSD (9,2 M de filas, 25 años), declara qué es cada cosa, **ve su propio dato ya transformado**, confirma, y el sistema sella esa declaración como evidencia inmutable y le devuelve un veredicto sobre lo que encontró.

Lo que **no** entra: que el dato quede guardado. Eso es la entrega inmediatamente siguiente (decisión del propietario, 2026-07-31). Al cerrar esta Story el camino completo funciona de punta a punta salvo el último tramo — la escritura a disco.

**Por qué el orden es este:** toda la incertidumbre está aquí (pantalla, puente FFI y cableado del lienzo no existen hoy); el almacenamiento es trabajo conocido. Se despeja primero lo que puede salir distinto de lo previsto.

---

## 1. Especificación de origen

- **Cáscara Visual sellada:** `docs/features/data-import-wizard.md` (Etapa 0.5, UI-Designer 2026-07-26). Clasificada **Inspector Panel** — ADR-0136 es FIJO: un feature node abre panel lateral, nunca pantalla completa. Seis pasos: Archivo → Mapeo → Reloj de Origen → Unidades → Rango Declarado → Vista Previa y Confirmación.
- **ADR-0136** (Canvas y paneles), **ADR-0137** (hexágonos y puertos), **ADR-0117** (Superficie de Verificación en la misma Story que el backend).
- **LDR-0008** — las cuatro condiciones de la protección de responsabilidad. La condición 1 es la que gobierna esta Story.
- **ADR-0162** — declaración por tramo, reloj ancla+desplazamiento, Gate de Homogeneidad.
- Deudas que se saldan aquí: [`DEBT-035`](../debt/DEBT-035.md), [`DEBT-037`](../debt/DEBT-037.md), [`DEBT-039`](../debt/DEBT-039.md).

---

## 2. Objetivo (una frase llana)

Que el propietario pueda importar su archivo real desde la interfaz, entendiendo y firmando lo que declara, sin leer una línea de código.

---

## 3. Agentes y Modo de Acompañamiento (ADR-0120)

| Etapa | Agente | Modo |
|---|---|---|
| 2 · Núcleo Rust | Rust-Engineer (Sonnet) | Docente |
| 3 · Puente FFI | Rust-Engineer (Sonnet) | Docente |
| 4 · Interfaz | Flutter-Engineer (Sonnet) | Docente |
| 5 · Gate | QA-Engineer (Sonnet) | — |

Etapas **0.3 (Quant)** y **0.4 (Architect)** declaradas cumplidas por trabajo previo: la viabilidad del lector y la coherencia del diseño se sellaron en STORY-068/070 y en las tres enmiendas de ADR-0162. Etapa **0.5 (UI-Designer)** entregada el 2026-07-26.

---

## 4. Instrucciones de despacho por agente

### 4.1 Rust-Engineer — la vía de mapeo manual y el cotejo del rango

**El problema, no el método.** Hoy Drasus solo sabe importar archivos cuyo formato **él mismo afirma** conocer (la plantilla de MetaTrader): el usuario no declara nada y Drasus responde por el resultado. Falta el caso inverso — un archivo que Drasus no conoce, donde **el usuario declara** qué es cada columna y responde él. Esa vía no existe en código.

Restricción que la gobierna, y no es técnica: **LDR-0008 condición 1** exige que lo que el usuario confirma sea una vista previa de **su propio dato ya transformado**, no una tabla de correspondencias entre nombres de columna. El motor debe poder producir esa vista previa **antes** de que nada se selle ni se persista, porque es lo que el usuario firma.

**Segundo problema, independiente:** hoy el rango que el usuario declara no se compara con lo que el archivo realmente contiene (`DEBT-039`). Se puede declarar un siglo sobre un archivo de un día y el sistema lo sella. El validador profundo (`pit-data-validator` TTR-005) no existe todavía y no entra aquí; lo que sí entra es el cotejo elemental que impide el caso grosero. Decide tú la forma —aviso o rechazo, y en qué punto del flujo— pero justifícala contra LDR-0008: lo que el usuario firma no debe poder contradecir lo que el archivo contiene.

**Lo que NO entra:** persistencia real de las barras. El puerto de salida sigue emitiendo a un consumidor en memoria (`DEBT-036` se salda en la entrega siguiente).

### 4.2 Bridge — el canal FFI

Superficie mínima que la pantalla necesita: seleccionar archivo, obtener las columnas detectadas, pedir la vista previa transformada bajo un mapeo propuesto, y confirmar la declaración. Sigue el patrón de los cinco módulos ya expuestos en `crates/bridge/src/api/`.

**Restricción de diseño:** la vista previa se pide y se descarta cuantas veces haga falta sin efecto alguno; **solo la confirmación escribe**. Si esa frontera no está clara en la forma del API, está mal.

### 4.3 Flutter-Engineer — el panel, y el cableado que lo hace alcanzable

Construye el Inspector Panel de seis pasos según la Cáscara Visual sellada, con los componentes ya existentes en `ui/lib/components/` (el UI-Designer verificó uno por uno los 14 que necesita).

**Y el cableado del lienzo, que es la mitad menos obvia y la más importante.** Hoy `ui/lib/tabs/canvas_tab.dart` permite soltar nodos pero **ningún nodo abre nada**: no hay manejador de pulsación. La prueba de que esto no es teórico es que ya existe un panel construido y referenciado desde ninguna parte — `ui/lib/tabs/hybrid_data_transformer/hybrid_data_transformer_inspector.dart`, 567 líneas inalcanzables. Sin este cableado, tu panel sería el segundo.

Diseña el mecanismo como **genérico**: pulsar un nodo abre el panel de esa feature. No lo resuelvas con un caso especial para el asistente — el problema no es abrir un panel, es que el lienzo no sabe abrir ninguno. **Adopta también el panel huérfano** en el mismo cableado: si tu mecanismo es genérico, adoptarlo no cuesta nada, y que siga huérfano después de esta Story sería la prueba de que no lo es.

**`DEBT-037`:** el mapa de componentes de `docs/DESIGN.md` nombra 56 veces clases retiradas por STORY-025/026. El UI-Designer corrigió las filas que su diseño usaba. Barre el resto.

**La mitad de `DEBT-039` que solo tú puedes cerrar.** La Etapa 2 cerró el caso estructural: si el contenido del archivo cae **fuera** del rango declarado, se rechaza. Lo que ninguna regla automática puede decidir es el caso inverso —un rango declarado mucho **más ancho** que el contenido— porque un tramo es una afirmación de *procedencia*, no de presencia de dato, y declarar un periodo con huecos internos es legítimo. **Lo decide el usuario si se lo enseñas:** el paso de confirmación debe mostrar **la primera y la última fecha que el archivo realmente contiene, junto a lo que el usuario acaba de declarar**. Ahí una discrepancia de tres años salta a la vista, y la resuelve quien responde por ella. Ver `docs/debt/DEBT-039.md`, sección "Qué quedó cerrado y qué no".

---

## 5. Criterio de aceptación (cada criterio ↔ su prueba)

| # | Criterio | Cómo se prueba |
|---|---|---|
| 1 | Un archivo sin plantilla conocida se puede importar declarando el mapeo manualmente | Prueba de dominio + de punta a punta con un CSV de columnas arbitrarias |
| 2 | La vista previa muestra **dato transformado**, no correspondencia de nombres | Prueba que afirma valores convertidos (escala entera, ns UTC), no cabeceras |
| 3 | Pedir la vista previa no escribe nada | Prueba: N vistas previas → 0 filas en el ledger |
| 4 | Un mapeo manual queda sellado con `MANUAL_USER`, distinguible del de plantilla | Prueba de persistencia sobre ambas vías |
| 5 | Contenido fuera del rango declarado → rechazo, en ambas vías | Prueba con rango deliberadamente falso; afirma el rechazo y que nada se selló |
| 5b | El paso de confirmación muestra la primera y última fecha reales del archivo junto al rango declarado | Prueba de widget: rango declarado tres años más ancho que el contenido → ambas fechas visibles y contrastables |
| 6 | Un campo sin declarar se rechaza, nunca se asume | Prueba por cada campo obligatorio ausente |
| 7 | El API del puente expone las cinco operaciones y la frontera lectura/escritura | Prueba de integración por el binario real |
| 8 | Pulsar un nodo del lienzo abre el panel de esa feature | Prueba de widget sobre el mecanismo genérico |
| 9 | El panel del transformador híbrido deja de estar huérfano | Prueba de widget: se alcanza desde el lienzo |
| 10 | El panel de importación recorre los seis pasos y bloquea el avance con datos incompletos | Prueba de widget por paso |
| 11 | El error de importación se muestra literal, nunca una importación parcial visible | Prueba de widget sobre el estado de rechazo |
| 12 | ✅ `docs/DESIGN.md` no presenta como usable ninguna clase que no exista | Cada fila del mapa contrastada contra el código real: o apunta a una clase existente con firma verificada, o dice explícitamente "Sin construir" / "clase inexistente". **Cumplido 2026-07-31**: 127 filas verificadas + dos comprobaciones automáticas (todo identificador existe / todo símbolo muerto aparece solo negado) |
| 13 | ⏳ El archivo real del propietario (9,2 M de filas) se importa desde la interfaz de principio a fin | Ejecución manual documentada, con el archivo real. **Solo lo puede ejecutar el propietario** — no hay forma de manejar la interfaz gráfica desde el entorno de agentes. **Recorrido literal en §6.1** |

---

## 6. Comandos de validación (para el usuario — copy/paste)

```bash
cd ui && flutter run -d linux      # arranca la aplicación de escritorio
cd ui && flutter test              # 30/30 en verde
cd ui && flutter analyze           # 0 errores
```

### 6.1 Recorrido del criterio 13 — paso a paso

**Por qué está escrito (2026-08-01):** este criterio se le asignó al propietario diciendo únicamente "solo lo puede ejecutar él", sin decirle **cómo**. Abrió el nodo vecino —el *Transformador de Datos de Mercado*, que consume datos ya importados y nunca toca archivos— y concluyó, con razón sobre lo que veía, que la pantalla estaba inservible. **Un criterio que el Tech-Lead no puede ejecutar es un criterio que debe poder dictar.** Regla nueva, en §8.

1. Arrancar la aplicación y abrir el tab **Canvas**.
2. En la lista de features de la izquierda, arrastrar al lienzo **"Asistente de Importación"** — *no* "Transformador de Datos de Mercado", que es la fila de abajo y hace otra cosa (ver `DEBT-045`).
3. Pulsar el nodo. Se abre el panel lateral; **la cabecera del panel dice qué feature se abrió** — debe leerse "Asistente de Importación".
4. **Paso 1 · Archivo:** pulsar el recuadro de carga. Se abre el diálogo de archivos del sistema, filtrado a `csv`/`txt`/`tsv`. Elegir `datasets/EURUSD_M1_2000-01-01_2025-05-01.csv`. Al volver aparecen nombre, tamaño, huella SHA-256 (copiable) y el número de columnas detectadas. Soltar el archivo arrastrándolo **no funciona todavía** (`DEBT-044`).
5. **Paso 2 · Mapeo:** declarar qué columna del archivo es cada campo canónico. El archivo real trae fecha y hora en columnas separadas (`<DATE>` y `<TIME>`) — hay un modo para eso. Ningún campo obligatorio se asume: si falta uno, "Siguiente" queda bloqueado.
6. **Paso 3 · Reloj de Origen**, **Paso 4 · Unidades**, **Paso 5 · Rango Declarado**: declarar el desplazamiento horario del bróker, las unidades de spread y volumen, y el tramo que se está importando.
7. **Paso 6 · Vista Previa y Confirmación:** aquí se comprueba lo que importa. La tabla muestra **dato ya transformado** (precios en escala entera, tiempos en ns UTC), no correspondencia de nombres; y la **primera y última fecha reales del archivo** aparecen junto al rango declarado, para contrastarlos antes de firmar. Confirmar sella la declaración.
8. **Si algo falla:** el mensaje de error es seleccionable y tiene botón de copiado — no hay que transcribirlo a mano.

**Qué significa que salió bien:** la confirmación reporta la importación sellada. Repetir el mismo tramo debe decir que ya estaba importado y **no duplicar nada**.

---

## 7. Registro de ejecución (bitácora cronológica)

- 2026-07-31 · Tech-Lead · **Gate de mutación relanzado tras los arreglos: `data-import-wizard` sigue APTO** — 160 mutantes, **137 cazados, 0 supervivientes**, 22 inviables, 1 *timeout*. El *timeout* está diagnosticado y **no es un superviviente**: `attempt += 1` → `attempt *= 1` en el bucle de reintento de `MappingDeclarationRepository::record` deja el contador clavado en 0, así que la condición de salida nunca se cumple y el bucle no termina. La mutación queda detectada por cuelgue en lugar de por aserción, que es detección igual — el código mutado no es equivalente al original. `flutter test`: **22/22**. `flutter analyze`: 0 errores.
- 2026-07-31 · Tech-Lead · **Criterio 12 CUMPLIDO — barrido de las 127 filas, ejecutado por el propio Tech-Lead** (los subagentes estaban topados por límite de sesión). El defecto era mayor y de otra naturaleza que lo registrado en `DEBT-037`, que hablaba de "56 menciones a un vocabulario retirado": **10 símbolos presentados como usables no existen en ninguna parte** (`GlassSurface`, `Gx.vDivider`, `_regimeChip`, `_kpi`, `_vitalityBar`, `_spinner`, `_scanRing`, `_organismCell`, `_vitalityLegend`, `_crystalOrb` con argumento inventado); **~30 filas apuntaban a archivos que no existen** (todo el bloque de tokens a `gallery/gallery_tokens.dart`, que es `theme/gx_tokens.dart`; los 14 painters de dominio a `gallery_painters.dart`, que viven en `sections/section_dataviz_quant.dart`); **5 firmas eran inventadas** en piezas que sí existen (`ZuiNavPill({segments})`, `ZuiZoomFrame({zoom, content})`, `ExpectationEnvelopeBadge({isInside})`, `FleetCommandPanel({source, items})`, `MultiSelect({value})` → es `selected`); y **3 componentes reales faltaban del mapa** (`Card`, `BentoCard`, `Sheet`). También `DrasusThemeState`, la clase nombrada en la nota de superficies, no existe: se llama `ThemeState`. **Causa raíz del patrón:** el mapa no distinguía **librería reutilizable** (`components/`) de **vitrina de exhibición** (`gallery/`: datos incrustados, constructor vacío). Las cuatro firmas inventadas son todas vitrinas que el mapa anunciaba como piezas capaces de recibir datos propios. Es la misma confusión que costó STORY-025 entera, esta vez en el documento en lugar del código; el mapa ahora la marca fila por fila. Verificado además con dos comprobaciones automáticas: todo identificador del mapa existe en `ui/lib/`, y todo símbolo inexistente aparece solo en contexto de negación explícita.
- 2026-07-31 · Tech-Lead · **El criterio de pago de `DEBT-037` era inservible y se sustituyó**, no se cumplió. Decía `grep -c "Glow" docs/DESIGN.md → 0`: imposible sin destruir contenido correcto (hay 51 apariciones legítimas — `Gx.glow`, `Gx.textGlow`, `HoverGlow`, la sección "Elevation y Glow"), y **premiaba borrar** las notas históricas que son justo lo que necesita quien recuerda el nombre viejo. Segunda vez en esta Story que un criterio verificable por `grep` resulta ser un criterio equivocado — la primera fue el criterio 12 original, escrito por mí. El patrón conviene nombrarlo: **cuando la verificación mecánica es más barata que pensar el criterio, se acaba midiendo lo que la herramienta sabe medir.**
- 2026-07-31 · Tech-Lead · **`flutter test` vuelve a verde entero (22/22) por primera vez desde STORY-023 — llevaba más de un mes en rojo permanente y nadie lo había registrado.** El barrido del criterio 12 destapó la causa: `docs/DESIGN.md` documenta tres familias tipográficas (Space Grotesk / Inter / JetBrains Mono) que **STORY-023 sustituyó por Rajdhani borrando sus `.ttf`**; `gallery_golden_test.dart` seguía cargándolos y reventaba en `setUpAll`. Es el mismo defecto que el criterio 12 persigue —el documento presenta como real algo que el código no tiene— solo que en la sección de Tipografía en vez de en el mapa de componentes, y con una consecuencia viva: **un suite que nunca puede ponerse verde convierte "Some tests failed" en ruido de fondo, y ahí se esconde la siguiente regresión de verdad.** Corregido en tres frentes: el documento declara el estado real (conservando el diseño de origen, que sigue fijando rol y carácter de cada voz), el test carga Rajdhani, y los PNG de referencia se regeneraron. **Segundo hallazgo al inspeccionar el PNG en vez de confiar en el "verde":** la mitad del texto salía como cajas grises. La escala tipográfica (`Gx.panelTitle`, `Gx.body`, `Gx.dataSmall`) vive en espejos estáticos que nacen **sin familia** y solo la reciben cuando `load()` dispara `_syncStyleScale()`, que el test nunca llamaba — así que el golden medía media pantalla real y media pantalla de tipografía de relleno. Con el tema inicializado en `setUpAll`, el golden es legible de punta a punta y por fin sirve de línea base visual. **Regenerar un golden sin mirarlo es bendecir lo que sea que haya salido.**
- 2026-07-31 · Tech-Lead · **Observación del QA sobre `PreviewBarRow` verificada y matizada, no aplicada tal cual.** El QA pedía añadir `spread_reported_unit` a cada fila de la vista previa. Comprobado: la unidad **ya está sellada** en la declaración que envuelve esas filas (`spread_reported_unit_declared`, dentro del mismo insumo de hash), que es su ámbito correcto — es propiedad de la corrida, no de cada fila. Copiarla fila por fila habría ido justo contra el principio de ADR-0156 restricción 8 (trazabilidad por referencia estable, nunca copiando la declaración en cada fila). **Lo que sí era gap real:** en la pantalla, el spread se imprimía "5 puntos" y el volumen se imprimía "1000" a secas. Corregido — el volumen ahora lleva su unidad declarada al lado, y cuando no hay unidad declarada no se inventa ninguna: se marca. Esta es la pantalla donde el usuario confirma su dato transformado (LDR-0008 condición 1) y un número desnudo no es confirmable.
- 2026-07-31 · Tech-Lead · **`DEBT-043` abierta** (🟡 Baja): los mensajes de rechazo conservan jerga de programador ("parsear", marcadores `{:?}` de depuración de Rust, "ledger") donde ADR-0156 restricción 1 exige lenguaje del usuario. Son comprensibles y accionables, así que no bloquea; es un barrido de redacción sobre todas las cadenas de la feature, no un arreglo puntual.
- 2026-07-31 · Tech-Lead · **Saneamiento de lenguaje mortuorio, hecho en el momento** (regla permanente, no se registra como deuda). `docs/DESIGN.md` y tres archivos Dart cargaban "autopsia", "reporte funerario", "muerte", "muerta": id `autopsy-header` → `ceremonial-header`, `_autopsyHeader()` → `_closingReportHeader()`, y los textos a "reporte de cierre" / "retiro definitivo". `flutter analyze`: 0 errores.
- 2026-07-31 · Tech-Lead · **Cadena de pánico cortada en los dos extremos** (hallazgo del gate de QA). Una fecha sintácticamente válida pero fuera del rango representable en nanosegundos (el año 9999 pasa cualquier validación de formato) hacía reventar el Núcleo con `expect()`; el pánico cruzaba la frontera FFI como excepción de Dart; y la pantalla no capturaba excepciones, así que el botón se quedaba en "Analizando…" **para siempre**. Con 9,2 M de filas basta **una** fecha corrupta. Arreglado en ambos lados: el Núcleo rechaza con `TimestampOutOfRepresentableRange` **sin sustituir por un valor por defecto** (un dato irrepresentable lo corrige el usuario, no lo adivina Drasus), y la pantalla captura vía `_runGuarded()` y muestra el fallo apagando el indicador de carga. Ambas pruebas **demostradas discriminantes**: se restauró el defecto en cada lado por separado y cada prueba se puso roja.
- 2026-07-31 · Tech-Lead · **Gate de mutación: `data-import-wizard` APTO** (160 mutantes, 0 supervivientes). **`bridge` NO APTO con 9 supervivientes, todos anteriores a esta Story** y en cinco módulos que no tocó → [`DEBT-042`](../debt/DEBT-042.md), con la justificación escrita de por qué se acepta. Los 2 que sí nacieron aquí (`describe_confirm_error`) se cerraron dentro de la Story. **Primera corrida del gate sobre `bridge` en la historia del proyecto**, y devolvió 118 supervivientes: **107 en `frb_generated.rs`, código que escribe el generador de FFI**. Mutarlo mide una herramienta de terceros y el archivo se reescribe entero en cada regeneración, así que ninguna prueba escrita contra él sobrevive al siguiente cambio. Excluido vía `.cargo/mutants.toml` (nuevo) — el gate pasó de 215 mutantes en 5 min a 45 en 2 min, y el ruido dejó de tapar los 11 reales.
- 2026-07-31 · Tech-Lead · **Criterio 12 reescrito, a petición del Flutter-Engineer y con razón.** Decía "`grep` sobre el vocabulario retirado → 0 resultados", que es **imposible de cumplir sin mutilar el documento**: `Gx.textGlow` y `HoverGlow` son código real y vigente, y el propio lenguaje visual del sistema usa esa palabra. Un `grep` no distingue entre nombrar una clase inexistente como usable y describir correctamente una que existe. El criterio correcto es el que él aplicó: cada fila contrastada contra el código, apuntando a una clase real con firma verificada o diciendo explícitamente "Sin construir". **Escribir un criterio verificable por `grep` no lo hace un criterio correcto** — si la verificación mecánica es más barata que pensar el criterio, se acaba midiendo lo que la herramienta sabe medir.
- 2026-07-31 · Tech-Lead · Orden creada. Alcance partido en dos por decisión del propietario: pantalla ahora, almacenamiento inmediatamente después. Etapas 0.3/0.4/0.5 declaradas cumplidas por trabajo previo.
- 2026-08-01 · Tech-Lead · **Primera prueba manual del propietario. Tres defectos reales, ninguno cazado por 30 pruebas automáticas ni por el gate de mutación — los tres viven en la frontera con el sistema operativo y con el cuerpo humano, que es donde no llega ningún test que tengamos hoy.**
  1. **El diálogo de archivos no abría en absoluto.** `file_picker` **no abre un diálogo propio en Linux**: lanza el ejecutable externo `zenity`. En Fedora Silverblue no está, y el resultado fue `Couldn't find the executable zenity in the path`. Es una dependencia inaceptable en producto: KDE mínimo no trae `zenity` y una imagen atómica obliga a reiniciar para instalarlo — le habría pasado a cualquier usuario con ese escritorio. Sustituido por `file_selector`, cuyo plugin de Linux **compila el diálogo GTK dentro de sí mismo** (`GtkFileChooserNative`, que además enruta al portal XDG en sandbox). Verificado de verdad: `flutter build linux` correcto, plugin registrado en `generated_plugin_registrant.cc` y `libfile_selector_linux_plugin.so` en el bundle. `file_picker` retirado del `pubspec`.
  2. **Parpadeo infinito de la zona de carga al acercar el cursor**, hasta el punto de no poder pulsarla. La geometría dependía del estado del puntero por dos vías: el texto cambiaba a uno más corto ("Suelta aquí") y el borde engordaba —y en Flutter el borde consume espacio interior—. La caja encogía, su borde derecho pasaba por debajo del cursor, se disparaba `onExit`, la caja crecía y el cursor volvía a quedar dentro. Bucle que se realimenta solo. Corregido con ancho fijo, grosor de borde constante y texto que no cambia con el puntero. **Prueba de regresión demostrada discriminante:** sobre el código anterior la caja mide 198,8 × 109 px en hover contra 400 × 128 px en reposo, y la prueba falla; con el arreglo el tamaño es idéntico en ambos estados. La regla quedó canonizada como irrompible en `docs/DESIGN.md`, junto con su corrección de la matriz de estados —que decía literalmente "hover = leve escala"—, y con las dos excepciones acotadas que sí son seguras (agrandar en hover, encoger al pulsar con el puntero capturado).
  3. **El mensaje de error no se podía copiar** — el propietario tuvo que transcribirlo a mano para reportarlo. Es el peor texto posible para obligar a transcribir: el que describe un fallo existe justamente para salir de la aplicación. Componentes nuevos `CopyButton` y `CopyableText`, más `copyable` en `Banner` y en `KeyValue` (donde copia el **valor**, nunca la etiqueta). Aplicado a los cinco banners de error del asistente y a la huella SHA-256. La regla —etiquetas y botones no seleccionables; errores, huellas, identificadores y rutas siempre seleccionables y copiables— quedó en `docs/DESIGN.md`. 5 pruebas nuevas.
- 2026-08-01 · Tech-Lead · **Deuda nueva `DEBT-044`** (🟡): el arrastre real nunca estuvo cableado — la zona parecía aceptar archivos soltados y no los acepta. El texto que lo prometía se saneó en el momento (prometer una interacción que no ocurre es defecto, no deuda); la funcionalidad queda debiendo. **Deuda nueva `DEBT-045`** (🟠): los nodos del lienzo son nombres pelados y no dicen qué consume ni qué produce cada feature — eso es lo que llevó al propietario a abrir el *Transformador de Datos de Mercado* en vez del asistente y a concluir que la pantalla estaba inservible. Le pasó a quien mandó construir ambas: no es falta de familiaridad, es información ausente de la pantalla.
- 2026-08-01 · Tech-Lead · **Verificación tras los tres arreglos:** `flutter test` **30/30** (25 previas + 3 de estabilidad de la zona de carga + 5 de copiado, con dos solapes de conteo), `flutter analyze` 0 errores, `flutter build linux --debug` correcto. Los goldens no se movieron. Filas de `DESIGN.md` actualizadas para las cuatro firmas que cambiaron — el criterio 12 exige que el mapa siga siendo verdad, y una firma nueva sin actualizar lo rompería el mismo día en que se cerró.

---

## 8. Pendientes derivados / decisiones

- **`DEBT-036` NO se salda aquí** — su disparador es la conexión con la persistencia real, que es la entrega siguiente.
- **`DEBT-038` no toca esta Story** — barrido de las siete columnas temporales sin sufijo de unidad, con fecha límite en el congelamiento a BROWNFIELD.
- **El Gate de Homogeneidad profundo (`pit-data-validator` TTR-005) sigue siendo especificación.** El cotejo de rango de esta Story es el mínimo que impide el caso grosero, no su sustituto.
- **Regla nueva del Tech-Lead (2026-08-01, permanente):** *todo criterio de aceptación cuyo ejecutor sea el propietario se entrega con el recorrido literal — dónde pulsar, qué debe aparecer, cómo saber que salió bien.* El criterio 13 se asignó con un "solo lo puede ejecutar él" y sin ninguna instrucción; el propietario abrió la feature vecina y perdió la sesión. **Un criterio que el Tech-Lead no puede ejecutar es un criterio que debe poder dictar.** El recorrido vive ahora en §6.1. Es el mismo patrón que ya falló tres veces en esta Story con los criterios verificables por `grep`: escribir el criterio salió más barato que pensar cómo se cumple.
- **Las tres etapas corrieron en modo Docente y no dejaron archivo de lección** (`docs/lessons/`), que ADR-0122/0124 exige. Pendiente de saldar antes de dar la Story por cerrada.

---

## 9. Cierre ejecutivo (para el usuario — CEO)

**Qué se entregó.** La pantalla con la que un usuario importa un archivo de datos que Drasus no reconoce: arrastra el archivo, declara qué es cada columna, **ve sus propios datos ya convertidos**, y confirma. Esa vista previa no es cosmética — es la pieza de la que depende que la responsabilidad por un mapeo equivocado sea de quien lo declaró. Sin ella no podíamos construir la vía manual en absoluto.

**Se puede llegar a ella.** Antes, un panel de feature no tenía forma de abrirse desde el lienzo; se cableaba a mano, uno por uno. Ahora hay un mecanismo genérico: dar de alta un panel cuesta una línea. El panel del transformador híbrido, que llevaba tiempo huérfano, quedó conectado con esa misma línea.

**Cuatro deudas saldadas**, dos de ellas las que motivaron el hito. Queda una a medias por decisión deliberada, no por olvido: cuando el rango que el usuario declara es **más ancho** que lo que el archivo contiene, no inventamos un umbral estadístico para juzgarlo — la pantalla le muestra la primera y última fecha reales al lado de lo que declaró, y decide él. Un rango declarado afirma de dónde vienen los datos, no cuántos hay.

**Dos hallazgos que nadie buscaba y valían más que el encargo:**

- Una sola fecha corrupta entre 9,2 millones dejaba el botón en "Analizando…" **para siempre**. Tres piezas distintas fallaban en cadena y cada una parecía razonable por separado. Cortada en los dos extremos.
- Las pruebas de interfaz llevaban **más de un mes en rojo permanente** y nadie lo había registrado: el documento de diseño describía tres tipografías que se habían sustituido hace un mes, y la prueba visual seguía buscando los archivos borrados. Ya está verde entero (22 de 22). Un conjunto de pruebas que nunca puede ponerse verde deja de avisar de nada.

**Lo que falta, y es tuyo.** Importar el archivo real de 9,2 millones de filas desde la aplicación. No puedo ejecutarlo yo: requiere manejar la interfaz gráfica. Todo lo demás está probado y verificado. Si esa corrida falla, se reabre la deuda correspondiente.

**Lo que viene después.** Conectar esta pantalla con el almacenamiento real. Hoy la importación confirma y sella la declaración, pero los datos no aterrizan todavía en su formato definitivo — esa es la entrega siguiente, y salda las dos deudas restantes de esta cadena.
