# Data Import Wizard

**Carpeta:** `crates/features/data/data-import-wizard/`
**Estado:** 🟡 **Parcial — vía de plantilla de plataforma implementada, ya con declaración por tramo** (STORY-068, 2026-07-25, retrofit STORY-070, 2026-07-26 — QA pendiente de auditoría del Tech-Lead, gate de mutación pendiente sobre el código retrofit). Backend completo para la vía en que Drasus afirma la plantilla: lector del archivo de barras de MetaTrader, ledger append-only de declaración de mapeo por TRAMO (rango explícito, reloj de origen como ancla IANA + desplazamiento fijo en minutos) sellado antes de persistir dato, unidad de spread y de volumen declaradas o rechazo explícito, rechazo con motivo explícito de un tramo anterior a la fecha de operación declarada del bróker. **Diferido:** la vía de mapeo manual del usuario y su pantalla (arrastrar archivo, vista previa de dato transformado, confirmación) — es la que LDR-0008 exige que pase por vista previa, y va en la Story del asistente visual (`DEBT-035`). También diferido el lector de ticks, que vive en `data-normalization-layer`, y el resto del Gate de Homogeneidad de Procedencia (fractura de magnitud/disponibilidad), que queda como especificación de `pit-data-validator`, feature que no existe todavía en código.
**Última actualización:** 2026-07-26 (STORY-070 — retrofit de esquema: reloj de bróker por ancla+desplazamiento y declaración de mapeo por tramo, ejecutados)
**Decisión Arquitectónica Asociada:** ADR-0008 (Configurabilidad Universal) · ADR-0154 (vía 3 de ingesta: exportación manual desde el terminal del usuario) · ADR-0155 (esquema canónico de campos) · ADR-0156 (formato estándar de ingesta — rechazo por defecto, plantillas, mapeo declarado como evidencia) · ADR-0162 (patrón de selector de zona horaria con bandera DST; enmienda 2026-07-26: reloj por ancla+desplazamiento, declaración por tramo) · ADR-0163 (idempotencia de reimportación — esta feature es la vía manual, con confirmación explícita)

## ¿Qué es?

Es el componente que permite al usuario incorporar datos externos (EJ: CSV de MetaTrader, TXT de NinjaTrader) al sistema Drasus Engine. Aplica una plantilla de plataforma ya validada, o guía al usuario a declarar el mapeo de columnas contra el esquema canónico (ADR-0155) — nunca adivina en silencio.

> **Rol arquitectónico (ADR-0154):** esta feature materializa la **tercera vía de ingesta** — la única en la que Drasus no hace red. El usuario exporta desde su propio terminal y entrega el archivo; Drasus lee, valida, normaliza y particiona. Es la vía que garantiza el precio del bróker de ejecución real, con compra y venta, sin depender de que ese bróker publique volcado alguno.

**Problema:** Cada broker exporta CSVs con nombres de columnas y formatos de fecha distintos. Mapearlos a mano es lento y propenso a errores — pero adivinar el mapeo sin que el usuario lo confirme sobre datos reales es, además de un riesgo técnico, un riesgo legal (LDR-0008): Drasus terminaría afirmando una interpretación que nadie validó.
**Solución (ADR-0156):** dos vías, cada una con su responsable. **Plantilla de plataforma** (NinjaTrader, MetaTrader, Interactive Brokers…) — Drasus la declara, la valida contra exportaciones reales antes de publicarla, y responde si está mal construida. **Mapeo manual declarado** — el asistente puede pre-rellenar una propuesta (conserva el valor de la detección automática como *borrador*), pero el usuario debe confirmarla sobre una vista previa de sus propios datos ya transformados antes de que nada se persista. Sin plantilla y sin confirmación, el archivo se **rechaza** con motivo explícito — nunca se adivina ni se importa a medias.

---

## Narrativa de Producto

**Capacidad:** `CAP-01` — Datos en los que se puede confiar

**Historia de usuario:** Como usuario, quiero traer al sistema el archivo que exporté de mi propia plataforma y ver exactamente cómo quedó interpretado antes de que se guarde nada, para no descubrir después que una columna se leyó al revés.

**El problema (técnico):** cada plataforma exporta con nombres de columna, formatos de fecha y separadores distintos. Mapearlos a mano es lento y propenso a error. Pero la alternativa obvia —que el sistema adivine el mapeo— es peor: adivinar bien el 95% de las veces significa que el 5% restante entra corrompido sin que nadie se entere, y el sistema queda afirmando una interpretación que ninguna persona validó.

**La solución (técnica):** dos vías, cada una con un responsable declarado. La **plantilla de plataforma** la construye y valida Drasus contra exportaciones reales antes de publicarla — si está mal, es responsabilidad de Drasus. El **mapeo manual declarado** lo confirma el usuario sobre una vista previa de sus propios datos ya transformados; el sistema puede proponer un borrador, pero la propuesta no es la decisión. Sin plantilla y sin confirmación, el archivo se **rechaza con motivo explícito**. Nunca se importa a medias.

Esta es además la única vía de ingesta en la que Drasus no toca la red: el usuario exporta desde su terminal y entrega el archivo. Garantiza el precio de su bróker de ejecución real, sin depender de que ese bróker publique volcado alguno.

**Explicado sin tecnicismos:** cada plataforma exporta sus archivos a su manera: unas ponen la fecha como día/mes/año, otras al revés; unas separan con comas, otras con tabuladores.

El sistema **podría** adivinar. Y acertaría casi siempre. El problema es ese "casi": las veces que se equivoque, tus datos entran mal y nadie se entera hasta mucho después, cuando ya construiste todo encima.

Así que Drasus no adivina. O usa una plantilla que ya probó contra archivos reales de esa plataforma —y si está mal, es culpa de Drasus—, o **te enseña cómo quedarían tus datos y espera a que confirmes**. Si no hay ninguna de las dos, rechaza el archivo y te dice por qué. Prefiere decirte que no puede, antes que decirte que sí y estar equivocado.

## Comportamientos Observables

- [ ] El usuario arrastra un archivo CSV; el sistema previsualiza las primeras filas del archivo crudo.
- [ ] Si el archivo coincide con una plantilla de plataforma validada, el sistema la aplica y muestra la vista previa de **datos ya transformados** (no solo nombres de columna) para confirmación.
- [ ] Si no coincide con ninguna plantilla, el sistema propone un mapeo **borrador** (Columna 1 → Timestamp, Columna 2 → Open, etc.) que el usuario debe revisar y confirmar sobre esa misma vista previa de dato transformado real — la propuesta nunca se acepta por sí sola.
- [ ] Si el usuario no completa el mapeo, o el archivo mezcla cotización y ejecución sin que ninguna plantilla conocida lo explique, el sistema **rechaza** con un mensaje en lenguaje del usuario, nunca una importación parcial.
- [ ] Al confirmar, el sistema sella la declaración (archivo, mapeo, plantilla si hubo, vista previa, versión de esquema) en el ledger append-only de mapeo (ADR-0156 restricción 7) antes de persistir cualquier dato.
- [ ] El sistema valida el tipo de datos y corre sus propios guardarraíles (`pit-data-validator`) antes de aceptar el mapeo declarado — la declaración del usuario no los sustituye (ADR-0156 restricción 6).
- [ ] El sistema detecta automáticamente el delimitador (coma, punto y coma, tabulación) — esto es inferencia de formato de archivo, no de significado de columna; no está sujeto a la misma restricción que el mapeo semántico.
- [ ] **Declaración de zona horaria de origen (ADR-0114/TASK-064 §3, corregido por el propietario 2026-07-23; forma generalizada por ADR-0162 enmienda 2026-07-26).** Si el archivo no trae metadato de zona horaria explícito, el usuario declara el reloj de los timestamps crudos como **zona ancla del catálogo IANA + desplazamiento fijo en minutos** (mismo selector de zona mundial con bandera DST de `session-catalog`, más un campo de desplazamiento) — **nunca** se infiere de un pico de volatilidad ni de una noticia programada. Un reloj sin la anomalía de IC Markets se declara con desplazamiento `0`, sin caso especial. El selector se pre-rellena como borrador desde `broker_server_clock` (nueva, `instrument-catalog`) por `venue` — el usuario confirma o corrige, y **lo confirmado** (nunca lo que el borrador tenía) es lo que se sella. El desfase declarado queda sellado en el ledger de mapeo como evidencia (mismo tratamiento que el mapeo columna→columna). El hueco de fin de semana puede usarse como verificación estructural **opcional** de sanidad contra el reloj efectivo (ancla+desplazamiento en el instante observado), nunca como mecanismo primario.
- [ ] **Declaración por tramo, no por archivo (ADR-0162 enmienda 2026-07-26).** Un archivo físico puede producir **más de una fila** en el ledger de mapeo cuando su contenido no es homogéneo en todo su rango — ej. la unidad de un campo cambia a mitad del histórico, o un tramo no es tráfico genuino del bróker declarado (historial rellenado por un proveedor antes de que ese bróker existiera). Cada fila declara su propio rango de fechas (`declared_range_start_ns`/`declared_range_end_ns`) y su propia declaración completa (mapeo, reloj, unidades) — nunca se funde un tramo heterogéneo dentro de la declaración de otro. `pit-data-validator` corre el Gate de Homogeneidad de Procedencia antes de aceptar una fila: si detecta una fractura medible (magnitud, disponibilidad, o fecha anterior a la operación conocida del bróker) que el usuario no declaró como tramo separado, no deja sellar en silencio — el usuario segmenta la importación o confirma explícitamente que declara homogeneidad pese a la fractura mostrada.

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

## Cáscara Visual (Thin Shell)

> Autoridad: ADR-0106 · ADR-0136 · ADR-0117 · ADR-0156 · ADR-0162 (enmienda 2026-07-26) · LDR-0008 · `docs/DESIGN.md` · `docs/DESIGN.md §"Catálogo de Componentes"`

### Contexto de superficie (ADR-0136)
**Inspector Panel** — `data-import-wizard` es un feature node (leaf, sin sub-nodos) del pipeline `ingest` en la Vista Relacional del Canvas; un clic sobre ese nodo abre el panel lateral derecho con el asistente completo. Regla FIJO de ADR-0136 ("un feature node siempre abre inspector panel lateral, nunca pantalla full separada") gana sobre la tentación de producto de darle pantalla completa por ser la primera superficie de uso real — el ancho se resuelve con `scroll-area` horizontal en la tabla de vista previa (ver Layout), no rompiendo el panel lateral.

### Superficie y Densidad
- **Superficie principal:** panel lateral en `panel-glass` (`glassFill` + `glassRim`, radio 14–16) como chrome del Inspector Panel; el contenido interior de cada paso usa `panel-solid` (`panelSolid #0E1426`, radio 11) para las zonas de dato denso (tabla de mapeo, tabla de vista previa).
- **Densidad:** cómoda (Inspector Panel) — aire generoso entre pasos del stepper; la tabla de vista previa es la única zona de densidad alta dentro del panel.
- **Lienzo de fondo:** el Canvas detrás del panel sigue en `deepSpace #080A18` + `starField` tenue; el panel lo cubre parcialmente sin oscurecer el resto (sin `backdrop`/`scrim` de modal — es panel lateral, no diálogo).

### Componentes

| ID de catálogo (DESIGN.md §4–§9) | Rol en esta feature | Tokens clave | Estados semánticos |
|---|---|---|---|
| `stepper / wizard` | Cabecera fija del panel: 6 pasos — Archivo · Mapeo · Reloj de Origen · Unidades · Rango Declarado · Vista Previa y Confirmación | `optimaCyan` completado, `transitionIndigo` activo, `textMuted` pendiente | completado / actual / pendiente |
| `badge / tag / chip / pill` | Chip fijo bajo el stepper: declara la **vía activa** — nunca decorativo, es el requisito legal de LDR-0008 hecho visible | `optimaCyan` = Plantilla de Plataforma (Drasus afirma) · `transitionIndigo` = Mapeo Manual (el usuario declara) | óptimo / transición |
| `file-upload / dropzone` | Paso 1: zona de carga del archivo plano | `transitionIndigo` hover, `optimaCyan` cargando | reposo / hover / cargando |
| `table / data-grid` | Paso 1 (primeras filas crudas) y Paso 6 (`PreviewBarRow` ya transformado, condición técnica 1 de LDR-0008) | `panelSolid`, `dataMono` 13px, `surfaceRaised` en hover de fila | normal / fila con `is_activity_anomaly` |
| `select / dropdown` | Paso 2 (mapeo manual: columna origen → campo canónico, uno por campo) · Paso 3 (zona ancla IANA) · Paso 4 (unidad de volumen real) | `glassFill`, `surfaceRaised` | cerrado / abierto / selecto |
| `form-field` | Envoltura de cada control de Paso 2–5 (label + control + helper/error) | `textLabel`, `criticalCrimson` en error | normal / error |
| `number-input / stepper` | Paso 3: desplazamiento fijo en minutos sobre la zona ancla (default `0`) | `dataMono`, `glassFill` | default / focus |
| `date-range` | Paso 5: rango de fechas que este tramo declara (`declared_range_start_ns`/`end_ns`) | `glassFill` | default / seleccionando |
| `key-value-row` | Paso 6: resumen de la fila sellada del ledger (`mapping_declaration_id`, `event_sequence_id`) tras confirmar | `textLabel` + `dataMono` | normal |
| `button-primary` / `button-glass` / `button-danger` | Navegación del stepper (Atrás/Siguiente) y acción final (Confirmar e Importar / Rechazar) | `gradReactor`/`optimaCyan`, `glassFill`, `criticalCrimson` | default / hover / loading / disabled |
| `alert / banner / callout` | Rechazo de archivo (ADR-0156 Restricción 1), advertencia del Gate de Homogeneidad de Procedencia, éxito de sellado, reimportación no-op | `criticalCrimson` rechazo, `alertAmber` fractura detectada, `optimaCyan` éxito, `transitionIndigo` no-op informativo | info / óptimo / alerta / crítico |
| `empty-state` | Paso 1 antes de soltar archivo; Paso 2 si el mapeo manual no tiene ninguna columna asignada aún | `textMuted` | — |
| `tooltip` | Aclaración inline sobre "zona ancla" vs. "desplazamiento" y sobre qué es un "tramo" | `glassFill` | — |

**Deliberadamente NO se usa `heatmap`/`sparkline` (catálogo §10) para visualizar la fractura de homogeneidad** — ninguno de los dos existe como componente en `ui/lib/components/` ni siquiera como mockup de galería (verificado, cero resultados). La fractura se muestra con `table` (una fila por sub-ventana, columna de magnitud) + `chip` de color por sub-ventana (ver Paso 5 abajo) — mismo efecto de lectura visual, con componentes que ya existen y están wireados. Si el Tech-Lead prefiere el efecto de mapa de calor real, es una construcción nueva a escalar aparte, no un bloqueo de esta entrega.

### Estados Semánticos (Espectro de Vitalidad)

| Estado de negocio | Color token | Tratamiento visual completo |
|---|---|---|
| Plantilla de plataforma aplicada (Drasus afirma el mapeo) | `optimaCyan #54E8D0` | chip vía: texto `#54E8D0`, fondo `#08251F`, borde 1px `#1E5E4F`, radio 999px · `glow(optimaCyan)` fijo, sin parpadeo (es un hecho estable, no un proceso vivo) |
| Mapeo manual en curso / borrador sin confirmar (el usuario declara, aún no sellado) | `transitionIndigo #9A8CFF` | chip vía: texto `#9A8CFF`, fondo `#130F2A`, borde 1px `#3A2E6E` · `glow(transitionIndigo)` |
| Fractura de homogeneidad detectada / decisión pendiente del usuario | `alertAmber #FFC94D` | banner: borde izq. 3px `#FFC94D`, icono alerta, texto con la razón medida (ratio de magnitud u hueco de sub-ventana) en `dataMono` — nunca solo "archivo inconsistente" |
| Rechazo de archivo / tramo imposible (`venue_operational_since_ns`) / error de importación | `criticalCrimson #F0413F` | banner: borde izq. 3px `#F0413F`, mensaje literal devuelto por `ImportBarFileError`, botón "Volver" — nunca una importación parcial visible |
| Importación sellada con éxito | `optimaCyan #54E8D0` (banner) + `reactorGreen #7CF06A` (botón confirmar en su estado post-clic) | banner éxito + `key-value-row` con `mapping_declaration_id`/`event_sequence_id`/conteo de filas |
| Reimportación no-op (`was_noop_reimport: true`) | `transitionIndigo #9A8CFF` | banner informativo: "este archivo ya está importado — no se duplicó nada", con la declaración existente referenciada |

### Layout
- Panel lateral derecho, ancho fijo cómodo (no redimensionable por el usuario — evita que el Inspector Panel invada el Canvas).
- Estructura vertical fija: `stepper` (top, no scrollea) → chip de vía + banner contextual (top, no scrollea) → contenido del paso activo (`scroll-area` vertical) → barra de navegación Atrás/Siguiente/Confirmar (bottom, no scrollea).
- Paso 2 (Mapeo manual): grid de `form-field` de una columna, uno por campo canónico (Timestamp, Open, High, Low, Close, Volumen…), cada uno con su `select / dropdown` de columna origen — sin límite de campos porque el esquema es abierto por arriba (Restricción de la feature).
- Tablas de Paso 1/6: `scroll-area` horizontal propio dentro del panel — el panel no se ensancha, la tabla desplaza.
- Paso 5 (fractura, si el gate la señala): tabla de sub-ventanas (columnas: rango de la sub-ventana, magnitud observada, `chip` de severidad) inmediatamente debajo del `date-range`, seguida de dos `button-glass`/`button-danger`: "Segmentar en tramos" (regresa al Paso 5 con el rango acotado) y "Confirmar homogeneidad pese a la fractura" (avanza, la vista previa ya revisada por el usuario es la evidencia de esa confirmación — no se inventa un campo de formulario nuevo).
- Separación entre paneles: 24px (Inspector Panel es ceremonial/cómodo, no denso).

### Animaciones Aplicables
- [x] Dropdown: `AnimatedSize` + rotación del chevron 180° (zona ancla, unidad de volumen, columna origen)
- [x] Clic de botón: hundimiento (escala 0.96) + propagación de luz (~460ms) — Confirmar e Importar
- [x] Loader/progreso: el botón "Confirmar e Importar" pasa a estado `loading` (glow pulsante `optimaCyan`) mientras el `await` real corre — sin barra de progreso independiente, porque el backend no reporta progreso parcial (ver Notas de implementación)
- [x] Chip parpadeante: NO aplica al chip de vía (es un hecho estable, `glow` fijo) — SÍ aplica al banner `criticalCrimson` de rechazo (parpadeante, alerta activa)
- [x] Foco de input: borde 1.5px + glow limpio (~200ms) en cada `form-field`/`number-input`/`date-range`
- [ ] Zoom canvas — no aplica, el panel no hace zoom, es un overlay lateral fijo
- [ ] Switch — no hay switches en este diseño

### Notas de implementación

**Puente (Bridge).** El único punto de entrada real hoy es `crates/features/data/data-import-wizard/src/public_interface.rs::import_mt5_bar_file(pool, clock, ImportBarFileInput, bar_sink) -> Result<ImportBarFileOutcome, ImportBarFileError>` — una función `async` de una sola llamada que se espera (`await`) una vez y devuelve el resultado completo. **Nunca diseñar sondeo (`Timer.periodic`) ni streaming** para esta pantalla: no hay reporte de progreso parcial expuesto, ni lo va a haber salvo que una Story futura lo agregue — el botón "Confirmar e Importar" dispara una única llamada y espera su única respuesta. `bar_sink` es un closure interno que el Bridge compone al conectar con `hybrid-data-transformer`; Flutter nunca lo invoca por separado, es una sola operación de punta a punta desde su punto de vista.

**Bloqueos de backend para este diseño (no son bloqueos de componente de UI — son ausencia de superficie Rust que conectar):**
1. **Vía de mapeo manual: sin función pública.** `import_mt5_bar_file` solo sirve la vía de plantilla MT5 (detecta la cabecera, aplica `mt5_bar_column_mapping()` fijo). No existe ninguna función que reciba un mapeo columna→campo declarado por el usuario. El Paso 2 (rama manual) de este diseño no se puede cablear hasta que esa vía tenga su propia superficie pública — es la misma vía que la propia feature declara "Diferido" en su cabecera de estado.
2. **Reloj ancla+desplazamiento y rango declarado por tramo: RESUELTO (STORY-070, 2026-07-26).** `ImportBarFileInput` ya trae `source_clock_anchor_timezone_ref: Option<String>` + `source_clock_fixed_offset_minutes: i64` (sustituyen al antiguo `source_timezone_declared: Option<String>`), `declared_range_start_ns`/`declared_range_end_ns: i64` (rango explícito del tramo), y `venue_operational_since_ns: Option<i64>` (fecha de operación del bróker, inyectada — alimenta el rechazo del Gate de Homogeneidad; sufijo de unidad aplicado en TASK-072, 2026-07-27, retrofit de código ya completado). Los controles de Paso 3 (desplazamiento en minutos) y Paso 5 (rango del tramo) de este diseño ya tienen superficie Rust real que cablear — pendiente solo el cableado del Bridge mismo, que no entró en el alcance de STORY-070 (esquema y camino de importación, no la pantalla).
3. **Gate de Homogeneidad de Procedencia: sin veredicto que consumir.** El banner `alertAmber` de fractura y la tabla de sub-ventanas del Paso 5 están diseñados contra la forma que ADR-0162 (enmienda 2026-07-26) sella para esa extensión de `pit-data-validator`, que **tampoco está construida**. No hay hoy ningún `CalendarCoherenceVerdict`-equivalente para homogeneidad que el Bridge pueda transportar.
4. **Errores hoy no son "lenguaje del usuario" completo.** `ImportBarFileError` (vía `thiserror`) produce mensajes técnicos correctos pero orientados a desarrollador (ej. "el archivo no es UTF-8 válido", "error al parsear la línea de dato número N"). El banner de rechazo de este diseño muestra ese texto literal; traducirlo a lenguaje de negocio (ADR-0156 Restricción 1) es trabajo del Rust-Engineer sobre el enum de error, no algo que el Flutter Engineer deba reescribir client-side (sería lógica de dominio en el frontend, prohibido por ADR-0106).

**Flutter.**
- Import de la librería real: `import 'package:drasus_ui/components/components.dart' as custom_ui;` — todo widget de la tabla "Componentes" arriba se invoca `custom_ui.Stepper`, `custom_ui.Dropzone`, etc. (convención ADR-0138 enmienda 2026-06-29, namespace obligatorio por colisión con widgets de Material).
- `custom_ui.Dropzone.onTap` es `VoidCallback` sin payload — el componente **no recibe el archivo arrastrado como dato**, solo señala la intención de tocar. El flujo real es: `onTap` dispara el selector de archivos nativo (fuera de esta librería de componentes) → se leen los bytes → esos bytes se pasan al binding de importación. **El arrastre real no está implementado** (no hay `DropTarget`/`onDragDone` ni plugin de arrastre de escritorio): desde 2026-08-01 el `label` ya no lo insinúa, y la funcionalidad queda registrada en `DEBT-044`.
- **El selector nativo es `file_selector`, NO `file_picker` (fijado 2026-08-01 por fallo reproducido en producto).** En Linux `file_picker` no abre un diálogo propio: lanza el ejecutable externo `zenity` y falla con `Couldn't find the executable zenity in the path` si no está instalado — que es lo que ocurrió en Fedora Silverblue. Esa es una dependencia que no se le puede exigir a la máquina del usuario (el escritorio mínimo de KDE no trae `zenity`; una imagen atómica obliga a reiniciar para instalarlo). `file_selector_linux` compila el diálogo GTK dentro del propio plugin (`GtkFileChooserNative`, que además enruta al portal XDG cuando la app corre en sandbox), así que no depende de ningún binario externo. **Regla general que deja este caso: ningún camino de la interfaz puede depender de un ejecutable que el usuario tenga que instalar aparte.**
- `custom_ui.Table.rows` es `List<List<Widget>>` — cada celda es un widget arbitrario. Úsalo para meter un `custom_ui.Chip` con `ChipStatus.critical` en la celda de una fila cuyo `PreviewBarRow.is_activity_anomaly` sea `true`, en vez de una columna de texto plano.
- `custom_ui.Dropdown<T>.items` es `List<DropdownItem<T>>` con `label` de tipo `String` puro (sin slot para icono). La "bandera DST" del selector de zona ancla (mismo patrón que `session-catalog`) se compone como texto dentro del label (ej. `"America/New_York (UTC-5/-4, horario de verano)"`), no como un widget de bandera separado — el componente no lo admite hoy.
- `custom_ui.DateRangePicker.range` es `DateRange({start: DateTime, end: DateTime})` — la conversión a nanosegundos UTC (`declared_range_start_ns`/`end_ns`) es serialización de frontera FFI, no cálculo de dominio; ocurre en el binding, igual que cualquier otro campo numérico que cruza la frontera.
- Estado del wizard completo (paso actual, mapeo en construcción, selección de vía) vive en un único `StatefulWidget`/controller local al Inspector Panel — no hay necesidad de un state manager global, el ciclo de vida del wizard termina cuando el panel se cierra o la importación se confirma.

## Persistencia (Inundación de Fundamentos — ADR-0020)

**Ledger de declaración de mapeo (ADR-0156, nuevo — molde `.agents/knowledge/ledger-pattern.md`):** tabla append-only propia de esta feature, sellada ANTES de que cualquier dato del archivo se persista. **Grano de la fila: un tramo declarado, no un archivo completo** (ADR-0162 enmienda 2026-07-26) — un mismo archivo físico produce tantas filas como tramos heterogéneos el usuario declare, cada una con su propio `declared_range_start_ns`/`declared_range_end_ns` explícito. Cada fila guarda identidad del archivo (nombre + hash — el `UNIQUE` de idempotencia es `(hash, rango)`, no solo el hash, para no rechazar por error un segundo tramo del mismo archivo), el mapeo completo tal como se mostró al usuario, la plantilla aplicada y su versión (si hubo), la bandera `mapeo_manual_usuario`/`plantilla_plataforma`, la versión del motor de transformación y del esquema canónico (ADR-0155) vigentes al declarar, el instante de aceptación, la vista previa de dato transformado que el usuario confirmó, el **reloj de origen declarado** (`source_clock_anchor_timezone_ref` + `source_clock_fixed_offset_minutes` — zona ancla IANA más desplazamiento fijo en minutos, `0` reproduce sin caso especial una zona plana; sellado como evidencia inmutable, no como inferencia del sistema, TASK-064 §3.3 / ADR-0162 enmienda 2026-07-26), y el encadenado de hash de todo ledger. El `data_snapshot_id` de la partición resultante y la fila de `transformation_records` de la corrida referencian esta fila por identificador — nunca la duplican (ver ADR-0156 restricción 8).

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
- [`session-catalog`](./session-catalog.md)/[`instrument-catalog`](./instrument-catalog.md) — para la asociación bróker↔instrumento↔plantilla de sesión durante la importación (ADR-0162), y para el borrador de reloj de servidor por `venue` (`broker_server_clock`, ADR-0162 enmienda 2026-07-26) que pre-rellena el selector de zona ancla+desplazamiento.
- [`hybrid-data-transformer`](./hybrid-data-transformer.md) — ejecuta la reescritura atómica de partición Hive-Style una vez que el usuario confirma el solape (ADR-0163); este wizard nunca escribe Parquet directamente.

**Consumido por:**
- [`ingest`](../modules/ingest.md).

- **Contrato de Integración UI (ADR-0117) — Superficie propia:** Inspector Panel del feature node en el Canvas (ver `## Cáscara Visual` arriba). SVF: soltar el archivo de muestra embebido dispara `import_mt5_bar_file` real vía `public_interface`; el panel muestra la vista previa de datos ya transformados, sella la fila del ledger de mapeo, y el `mapping_declaration_id`/conteo de filas quedan visibles y persisten tras recargar.
