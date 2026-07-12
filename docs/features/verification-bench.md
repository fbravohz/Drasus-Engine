# Verification Bench (Banco de Pruebas)

**Carpeta:** `./features/verification-bench/`
**Estado:** 🟡 Parcial — chasis nacido en EPIC-0/SPIKE-006 (tab maestro-detalle + registro extensible), despachador genérico y validación visual de forma pendientes.
**Última actualización:** 2026-07-12
**Decisión Arquitectónica Asociada:** ADR-0152 (Banco de Pruebas como Gate Permanente de Definición de Terminado), enmienda de ADR-0117.

## ¿Qué es esta feature?

El Banco de Pruebas es el harness genérico de verificación del Panel Operativo Fundacional: la superficie única en la interfaz de Flutter donde el propietario (perfil frontend, sin acceso práctico a terminal para verificar backend) confirma con el ratón que una feature funciona de verdad, contra el backend real de su propia máquina, sin datos simulados.

**Problema:** hoy, verificar que una feature de backend funciona exige dos caminos distintos según quién lo haga: el ingeniero corre `drasus verify <feature-id>` por CLI (Canal #2, ADR-0142); el propietario no tiene ese canal — su único canal práctico es la app Flutter. Cuando una tanda de features se construye "backend-first" (como ocurrió con los 14 cimientos del substrato de monetización, ADR-0144/0145/0146/0147/0148/0149), el propietario se queda sin forma de comprobarlas él mismo hasta que alguien construye, a mano, una sección de verificación por feature — trabajo que se pospone indefinidamente porque nada en el proceso de cierre lo exige (DEBT-005).

**Solución:** un único harness, construido una vez, que cualquier feature con superficie propia enchufa casi gratis: se añade una entrada a un registro central y automáticamente aparece en la lista de features verificables del Banco. El harness es la gemela gráfica de `drasus verify` — despacha a la misma función de verificación de la feature, nunca reimplementa su lógica.

**Resultado observable:** el propietario abre la app, elige una feature de una lista, edita o acepta un JSON de ejemplo, presiona un botón y ve, en tiempo real, la respuesta real del backend — incluyendo si el JSON que escribió era una estructura válida para esa feature antes de que la llamada real siquiera se disparara.

## Comportamientos Observables

- [ ] El usuario abre el Banco de Pruebas y ve una lista lateral de features verificables (una entrada por feature con superficie propia que ya está conectada).
  → Al seleccionar una, el panel de detalle muestra tres zonas: input JSON precargado y editable a la izquierda, un botón "Enviar" al centro, y un panel de respuesta vacío a la derecha.

- [ ] El usuario borra o corrompe un campo obligatorio del JSON de entrada y presiona "Enviar".
  → La Zona Derecha muestra de inmediato "Input inválido" con la razón concreta (ej. "falta el campo `email`", "el campo `days` no es un número") — **sin que la llamada real al backend llegue a dispararse**.

- [ ] El usuario corrige el JSON y presiona "Enviar" de nuevo.
  → La Zona Derecha muestra primero "Input válido", luego el resultado real de la operación (éxito/fallo) y el JSON de salida real devuelto por el backend de esa feature — el mismo que devolvería `drasus verify` para el mismo input.

- [ ] El usuario selecciona otra feature en la lista lateral.
  → La Zona Izquierda recarga con el input de ejemplo propio de esa nueva feature; la Zona Derecha se limpia hasta el próximo envío.

- [ ] Un ingeniero termina el backend de una feature nueva con superficie propia y quiere conectarla al Banco.
  → Añade una entrada al registro central (un solo punto de extensión) con el input de ejemplo de la feature; no toca el layout del tab ni el despachador genérico.

## Restricciones

- NUNCA la Zona Derecha muestra un resultado simulado, precalculado o hardcodeado — siempre es la respuesta real del backend por FFI, en la máquina del propio usuario.
- NUNCA se dispara la llamada real al backend si el input no pasó primero la validación de forma — evita que basura del usuario rompa el backend y le ahorra un fallo confuso.
- NUNCA conectar una feature nueva exige tocar más de un archivo de registro — el punto de extensión es único (mismo patrón que la galería de componentes, ADR-0117).
- NUNCA el Banco de Pruebas depende del servidor central del proveedor (Cabina de Mando) para operar — corre siempre contra el backend local real por FFI; el cable de red hacia la Cabina es un adaptador diferido, fuera del alcance de esta feature (ADR-0143).
- NUNCA una feature con superficie propia se declara "Terminada" (Etapa 5, QA) sin su conexión individual al Banco de Pruebas (ADR-0152).

## Parámetros Configurables

| Parámetro | Default | Rango | Qué hace | FIJO/CONFIG |
|---|---|---|---|---|
| INPUT_DE_EJEMPLO | El mismo default que usa `drasus verify <feature-id>` sin `--input` (ADR-0142) | Uno por feature registrada | JSON que aparece precargado y editable en la Zona Izquierda al abrir la sección de una feature | CONFIG (cada feature declara el suyo) |
| ORDEN_DEL_MENÚ_LATERAL | Orden de registro | Cualquier orden | En qué posición aparece cada feature en la lista lateral | CONFIG |

## Estructura Interna (FCIS — ADR-0002)

- **Core (Lógica Pura):** la comparación determinista, sin I/O, entre la forma del JSON que el usuario escribió y la forma que la feature seleccionada espera recibir. Esta comparación es lo único que decide si el input es "Válido" o "Inválido + razón" — nunca toca red, disco ni el backend real.
- **Shell (Infraestructura):** el despacho real hacia la función de verificación de la feature seleccionada (la misma que invoca el CLI, Canal #2/ADR-0142) — que sí hace I/O (base de datos temporal, reloj, adaptadores locales) — más la serialización del resultado y su transporte FFI hacia Flutter.
- **Frontera Pública:** dos puntos de entrada FFI, ambos plomería crosscutting sin tipo de dominio propio. (1) El despachador genérico, compartido por todas las features registradas, que recibe el identificador de la feature más el JSON de entrada y siempre devuelve el mismo paquete de cuatro piezas de información (ver "Ciclo de Vida" abajo). (2) Un catálogo de solo lectura que enumera las features verificables junto con su input de ejemplo — es lo que permite que la Zona Izquierda llegue precargada al abrir cualquier sección, y el único lugar que hay que tocar al conectar una feature nueva (junto con su brazo en el despachador). Ambos se apoyan en las funciones de verificación que cada feature ya expone en su propia `public_interface` — nunca las reimplementan.

## Ciclo de Vida de la Feature — Verification Bench

### Entrada
- El identificador de la feature elegida en la lista lateral.
- El JSON de entrada, tal como quedó tras la edición del usuario en la Zona Izquierda (o el input de ejemplo, si no lo tocó).

### Proceso
- Compara la forma del JSON contra lo que la feature seleccionada espera recibir — sin tocar el backend todavía.
- Si la forma es inválida, se detiene ahí y arma la razón concreta (qué campo falta, qué tipo no coincide).
- Si la forma es válida, despacha la llamada real hacia la función de verificación de esa feature — la misma que invoca `drasus verify` por CLI — y captura lo que el backend responde de verdad.

### Salida
Siempre las mismas cuatro piezas de información, mostradas en la Zona Derecha:

| Campo devuelto | Qué comunica |
|---|---|
| Estado del input | Si el JSON que el usuario escribió era una estructura válida para esa feature, o inválida — y en ese caso, la razón concreta. |
| Resultado general | Si la operación real contra el backend tuvo éxito o falló. |
| Salida real | El JSON real que el backend de la feature devolvió (idéntico al que devolvería `drasus verify` para el mismo input). |
| Error (si lo hay) | El mensaje de error real, presente únicamente cuando el resultado general falló. |

### Contextos de Uso

**Contexto 1: Verificación manual de un cimiento del substrato**
- Los 14 cimientos de monetización (ADR-0144 y siguientes) tienen backend con QA aprobado, pero hoy solo son verificables por CLI. Conectarlos aquí es la primera tanda del sprint que abre esta feature.

**Contexto 2: Verificación de cualquier feature futura con superficie propia**
- Toda feature nueva se enchufa añadiendo una entrada al registro — sin construir una sección de verificación a medida.

**Contexto 3: Detección temprana de contratos rotos durante el desarrollo**
- Antes de que un fallo llegue al backend, el propietario o el ingeniero ven de inmediato si el input tenía la forma correcta — sin necesidad de leer logs ni código.

## Tareas (TTRs)

### **TTR-001: Despachador Genérico de Verificación (FFI)**
- **Descripción:** punto de entrada único que recibe el identificador de una feature registrada más el JSON de entrada, valida su forma contra lo que esa feature espera, despacha a la función de verificación correspondiente cuando la forma es válida, y siempre devuelve el paquete de cuatro campos (estado del input, resultado general, salida real, error).
- **Regla de negocio:** el despachador nunca reimplementa la lógica de ninguna feature — solo la invoca. Añadir una feature nueva al despachador es registrar su función de verificación existente, no escribir lógica nueva.
- **Complemento:** un catálogo de solo lectura acompaña al despachador — enumera cada feature registrada con su identificador, nombre legible e input de ejemplo, para poblar el selector lateral y precargar la Zona Izquierda.

### **TTR-002: Validación Visual de Forma del Input**
- **Descripción:** la Zona Derecha distingue siempre, de forma inequívoca y antes de mostrar cualquier resultado de negocio, entre "input mal formado" (la llamada real nunca se disparó) e "input válido pero la operación falló" (la llamada real sí se disparó y el backend respondió con un error). Son dos estados distintos, con dos tratamientos visuales distintos — la mezcla de ambos es la causa raíz que este TTR corrige respecto del chasis parcial existente.

### **TTR-003: Conexión Retroactiva de los Cimientos del Substrato**
- **Descripción:** cada una de las funciones de verificación que hoy solo se invocan por CLI (los 14 cimientos del substrato de monetización, más cualquier otra feature con backend terminado sin conexión individual al Banco) obtiene su entrada en el registro central, con su input de ejemplo precargado. Esta tarea paga DEBT-005.

## Puertos de Integración (ADR-0137)

> El Banco de Pruebas es plomería crosscutting (mismo criterio bendecido que `clock`/`audit-log`/`telemetry` y que los 14 cimientos del substrato): no produce un tipo de dominio propio, no expone puerto de Alpha en el Canvas — no tiene nodo en el Canvas [Forge/Reactor], vive únicamente dentro del Banco de Verificación del Panel Operativo Fundacional. Declara igualmente sus puertos técnicos con el tipo de infraestructura ya catalogado en ADR-0137.

| Puerto | ID de tipo | Dirección | Cardinalidad | Descripción |
|---|---|---|---|---|
| `verification_request_in` | `textLabel` | Input | `1` | Identificador de la feature seleccionada + JSON de entrada editado por el usuario en la Zona Izquierda. |
| `verification_result_out` | `textLabel` | Output | `1` | El paquete de cuatro campos (estado del input, resultado general, salida real, error) que llena la Zona Derecha. |

> **Cardinalidad:** `1` = exactamente uno por cada envío (no hay envíos en lote ni streaming). El puerto de salida no requiere una nueva entrada en el catálogo de tipos de ADR-0137 — `textLabel` ya es el tipo canónico para plomería de infraestructura consumida transversalmente (mismo uso que en los cimientos #1–#14).

## Cáscara Visual (Thin Shell)

> Autoridad: ADR-0106 · ADR-0136 · ADR-0117 · `docs/DESIGN.md` · `docs/DESIGN.md §"Catálogo de Componentes"` · entregada por el skill `ui-designer` (Etapa 0.5, ADR-0135) el 2026-07-12, en paralelo a esta feature doc, sobre el modelo de tres zonas fijado por el propietario.

### Correcciones de violaciones (pre-diseño)

Ninguna. El chasis existente (`ui/lib/tabs/verification_bank/verification_bank_tab.dart`, `verification_registry.dart`) no usa WebGL/DOM/HTML/CSS/SVG, no calcula métricas en el frontend, no usa alias MACRO/MESO/MICRO ni tecnologías rechazadas. Consume `Gx.*` dinámico y sigue el patrón `gallery_tab.dart`.

### Contexto de superficie (ADR-0136)

**Plomería — harness SVF genérico.** El Banco de Pruebas no es una superficie de producto embebida en el Dashboard ni en el Canvas: es una **pestaña de plataforma de nivel superior**, hermana de `Dashboard`/`Auditoría`/`Trabajos`/`Components` en `panel_operativo.dart` (mismo patrón de `DefaultTabController` documentado en DESIGN.md §"Galería de Componentes — Implementación"). Es la gemela GUI de `drasus verify`: el propietario entra a probar una feature sin leer código, feature por feature, seleccionándola en el lateral. Por eso se diseña bajo la fila "Plomería" del catálogo de contextos de superficie (JSON precargado → enviar → respuesta real por FFI), no bajo los cuatro contextos de superficie de producto.

### Superficie y Densidad

- **Superficie principal:** `panel-solid` — el lateral usa `Gx.navRail` (ya implementado); el panel de detalle y sus 3 zonas usan `custom_ui.Surface` (delega a `panelSurface()`, consume el token dinámico `Gx.surfacePanel` — nunca `Gx.panelSolid` raw).
- **Densidad:** densa (cockpit) — es una superficie de trabajo diagnóstica, igual que `Jobs`/`Auditoría`, no una pantalla ceremonial.
- **Lienzo de fondo:** `Gx.canvasBase` (deepSpace dinámico de la paleta activa), heredado del `Container` raíz ya implementado en `_VerificationBankTabState.build()`. Sin telón cósmico (`galaxy-background`) adicional: es una herramienta de plataforma, no una vista MICRO/ceremonial.

### Componentes

| ID de catálogo (DESIGN.md §4–§11) | Rol en esta feature | Widget real / Tokens clave | Estados semánticos |
|---|---|---|---|
| `sidebar / nav-rail` `[CORE]` | Selector lateral de features (ya implementado) | `Gx.navRail`; entrada activa con barra de acento `gradAurora` + `glow(transitionIndigo)` (`_buildSidebarEntry`) | reposo (`textBaseLabel`) / seleccionado (`transitionIndigo` + glow) |
| `text-field / input` `[CORE]` | **Zona izquierda** — editor del JSON de input, precargado con un ejemplo válido por feature | `custom_ui.Textarea({controller, hint, onChanged, maxLines, enabled})` dentro de `custom_ui.Surface`; radio `Gx.rInput` (10px); glow de foco `Gx.glow(Gx.accentDynamic, blur: 18)` | reposo / foco (borde 1.5px `Gx.borderFocus` + glow) / deshabilitado mientras `enviando` (`enabled: false`) |
| `button-primary` `[CORE]` | **Zona centro** — botón "Enviar" | `custom_ui.Button({label: 'Enviar', onPressed, variant: ButtonVariant.primary, enabled, loading})`; `Gx.gradOptima` + `glowStrong`; pulso de propagación de luz al soltar (ya integrado en el componente) | default / hover / pressed / disabled (JSON vacío) / loading (`enviando`) |
| `spinner / loader` `[CORE]` | Indicador de trabajo en curso mientras se espera la respuesta FFI | `ScanRingWidget({color: Gx.transitionIndigo, maxRadius, period})` — primitivo de `gallery/gallery_fx.dart`, ya usado en producción en `sovereign_data_fetcher_section.dart` (`_buildJobEnCurso`) | activo solo durante `enviando`; envuelve un `custom_ui.Chip(status: ChipStatus.transition, pill: true)` con la etiqueta "Enviando…" |
| `alert / banner / callout` `[CORE]` | **REQUISITO CLAVE — Zona derecha, franja superior.** Indicador inequívoco de validez del input para el puente FFI | `custom_ui.Banner({message, type: BannerType})` — borde izquierdo 3px + icono + glow del color semántico (ya soporta `info/success/warning/error` de fábrica) | `success` = "Input válido." (`optimaCyan`) · `error` = "Input inválido: <razón del backend>" (`criticalCrimson`) · `warning` = "Error de backend: <detalle>" (`alertAmber`, ver Estados Semánticos) |
| `badge / tag / chip / pill` `[CORE]` | Refuerzo compacto del estado de validez, junto al banner (doble señal: color + texto + forma) | `custom_ui.Chip({label, status: ChipStatus, pill: true})` — `ChipStatus.optima` = "Válido" / `ChipStatus.critical` = "Rechazado" / `ChipStatus.alert` = "Error backend" | espejo del `Banner`, nunca contradictorio |
| `code-block` (composición) | **Zona derecha, cuerpo.** Respuesta REAL del backend por FFI, read-only, seleccionable para copiar/depurar | `SelectableText(json, style: Gx.dataMono(fontSize: 12, color: Gx.textBase))` dentro de `custom_ui.Surface` con `SingleChildScrollView`; mismo patrón ya usado para IDs/hashes truncados en `sovereign_data_fetcher_section.dart` | contenido normal; si la respuesta es un error, el bloque muestra el mensaje crudo del backend bajo el `Banner` correspondiente |
| `empty-state` `[CORE]` | Estado inicial de la Zona derecha antes de la primera petición | `custom_ui.Empty({message: 'Sin solicitud enviada.', subtitle: 'Edita el JSON y pulsa Enviar.', icon})` | idle — sin color semántico, orbe latente por defecto |
| `key-value-row` `[CORE]` | Metadatos opcionales de la respuesta (tiempo de respuesta, tamaño de payload) bajo el bloque JSON | `custom_ui.KeyValue({label, value, valueColor, mono: true})` | valor en `dataMono`, color = `Gx.textBase` salvo que se quiera resaltar una métrica |

### Estados Semánticos (Espectro de Vitalidad)

| Estado de negocio | Color token | Tratamiento visual completo |
|---|---|---|
| **Idle** (sin solicitud enviada) | `Gx.textBaseMuted` (neutro) | Zona derecha muestra `custom_ui.Empty`; sin banner; botón "Enviar" habilitado si el JSON no está vacío |
| **Enviando** (Future FFI en curso) | `transitionIndigo #9A8CFF` | `custom_ui.Button(loading: true)` (spinner interno) + `ScanRingWidget(color: transitionIndigo)` envolviendo `Chip(status: transition, pill: true, label: 'Enviando…')` en la Zona derecha; editor JSON deshabilitado (`enabled: false`) |
| **Éxito — input válido** | `optimaCyan #54E8D0` | `Banner(type: success, message: 'Input válido.')`: borde izq. 3px `optimaCyan`, icono check, `glow(optimaCyan, blur 14, opacity 0.2)`. `Chip(status: optima, pill: true, label: 'Válido')` espejo. Debajo, bloque JSON de la respuesta real en `dataMono` |
| **Input inválido para el puente FFI** | `criticalCrimson #F0413F` | `Banner(type: error, message: 'Input inválido: <razón devuelta por el backend>')`: borde izq. 3px `criticalCrimson`, icono de peligro, `glow(criticalCrimson)`. `Chip(status: critical, pill: true, label: 'Rechazado')` espejo. El mensaje de `<razón>` es el error real de deserialización/validación que devuelve el puente FFI (ej. campo faltante, tipo incorrecto) — nunca un texto genérico "algo salió mal" |
| **Error de backend** (fallo técnico distinto del rechazo de input: timeout, pánico, conexión) | `alertAmber #FFC94D` | `Banner(type: warning, message: 'Error de backend: <detalle>')`. Se distingue deliberadamente del rojo de "input inválido": **rojo = corrige tu JSON, ámbar = el sistema falló, no tu dato** — la distinción de color evita que el propietario edite un JSON que ya era correcto |

**Regla de la distinción roja/ámbar (razón de diseño):** el requisito pide un estado "inequívoco" de validez del input. Usar el mismo color para "tu JSON está mal formado" y "el backend se cayó" sería ambiguo y llevaría al propietario a editar un input que no tenía la culpa. Por eso el espectro de vitalidad se aplica con dos colores de fallo distintos, ambos ya definidos en DESIGN.md: `criticalCrimson` (fallo del dato, corregible por el usuario) y `alertAmber` (fallo del sistema, no corregible editando el JSON).

### Layout

- **Panel de detalle** (ya implementado en `_buildDetail()`): header de sección (barra `gradAurora` + título) seguido del cuerpo de 3 zonas.
- **Cuerpo de 3 zonas en `Row`:**
  - Zona izquierda (editor JSON): `Expanded(flex: 3)` — `custom_ui.Surface` con `custom_ui.Textarea(maxLines: 18)` a ancho completo.
  - Zona centro (botón Enviar): columna angosta de ancho fijo (~140px), `Center` vertical, contiene únicamente `custom_ui.Button` (+ el `ScanRingWidget`/`Chip` de "Enviando…" aparece en la Zona derecha, no aquí, para no desplazar el botón).
  - Zona derecha (respuesta real): `Expanded(flex: 4)` — `custom_ui.Surface` con, de arriba a abajo: `Banner` de validez, `Chip` espejo, bloque `SelectableText` con scroll, `KeyValue` de metadatos opcionales.
- **Separación entre zonas:** `Gx.space16` horizontal entre columnas, `Gx.space12` vertical entre elementos dentro de cada zona (densidad cockpit).
- El lateral (`sidebar`) mantiene su ancho fijo de 240px ya implementado; no cambia.

### Animaciones Aplicables

- [x] Clic de botón: hundimiento (escala 0.96) + propagación de luz (~460ms) — **ya integrado en `custom_ui.Button`**, sin trabajo adicional.
- [x] Foco de input: borde 1.5px + glow limpio (~200ms) — **ya integrado en `custom_ui.Textarea`**.
- [x] Chip parpadeante / glow pulsante: `ScanRingWidget` durante `enviando` — mismo patrón que `sovereign_data_fetcher_section.dart`.
- [ ] Transición Banner/Chip al cambiar de estado (idle→enviando→éxito/error): envolver el bloque de estado en `AnimatedSwitcher(duration: ~200ms)` para que el cambio de color/mensaje no sea un salto brusco — no bloqueante para la Historia 1, pero recomendado.
- [ ] Zoom canvas: no aplica (Plomería, sin Canvas).
- [ ] Dropdown / Switch / Slider: no aplica a esta feature (sin esos controles en el chasis).

### Notas de implementación para el Flutter Engineer

1. **Patrón de llamada FFI:** seguir el patrón G1 ya validado en `sovereign_data_fetcher_section.dart` — `await` directo sobre el binding generado, **sin** `Timer.periodic`. El botón se deshabilita (`enabled: false`) y `loading: true` mientras el `Future` no resuelve.
2. **Fuente de la "razón" del input inválido:** el mensaje de `Banner(type: error)` debe ser el texto de error real que devuelve el puente FFI (`DTO.error` o excepción de deserialización), nunca un texto genérico inventado en Dart. Verificar el binding real (`ui/lib/src/rust/api/<feature>.dart`) de cada feature antes de asumir el nombre del campo de error.
3. **Distinción input-inválido vs error-de-backend:** si el binding solo expone un único campo de error sin distinguir "rechazo de validación" de "fallo técnico", este es un gap de contrato FFI, no de diseño — escalar al Bridge Engineer antes de implementar el Banner ámbar. El diseño asume que el error trae información suficiente para clasificarlo (ej. un código o un `enum` de causa); si no la trae, se degrada a un único estado rojo (`criticalCrimson`) hasta que el contrato lo soporte.
4. **`VerificationEntry` necesita un campo de JSON de ejemplo:** el registro actual (`verification_registry.dart`) solo tiene `title/icon/builder`. Para que el editor de la Zona izquierda llegue "precargado", cada entrada del registro necesita aportar su JSON de muestra (vía el propio `builder`, que ya construye la sección — no requiere cambiar el modelo del registro, cada `builder` gestiona su propio `TextEditingController` con el texto inicial, igual que `_symbolCtrl` en el fetcher).
5. **Reutilización:** todo esto es composición sobre componentes que YA EXISTEN y tienen contrato funcional real (no showcase) — no hay que crear ningún widget nuevo para cumplir el layout de 3 zonas ni el indicador de validez.

### Gaps de catálogo (para el Tech Lead / próxima iteración de DESIGN.md)

1. **`custom_ui.Textarea` no tiene variante monoespaciada.** Su `TextField` interno usa `Gx.uiSans` fijo (`textarea.dart`, línea del `TextField.style`). Para que el editor de JSON se lea como dato (`dataMono`, alineado con la regla irrompible "todo número/dato va en mono"), se necesita extender el componente con un parámetro `monospace: bool` (mismo patrón que `KeyValue.mono`). Hasta que exista, el editor JSON se ve en `uiSans` — funcional pero no ideal tipográficamente.
2. **No existe un `custom_ui.CodeBlock`/visor JSON dedicado en la librería funcional.** El catálogo (`DESIGN.md §8`) tiene `code-block [CORE]` mapeado a `GlowCode` en `gallery/sections/section_data_display_extended.dart`, pero ese widget vive en la galería-showcase (STORY-025: render-only, no migrado a `ui/lib/components/`). Para esta feature se compone con `SelectableText` + `Gx.dataMono` dentro de `custom_ui.Surface` (patrón ya usado en el chasis existente para IDs truncados) — es una solución funcional válida hoy, pero si aparecen respuestas JSON grandes/anidadas, vale la pena promover `GlowCode` a un `custom_ui.CodeBlock` real con syntax highlighting mínimo.

Ninguno de los dos gaps bloquea la implementación: ambos tienen una solución funcional inmediata documentada arriba (registrar como deuda menor si el Tech-Lead lo considera, `docs/DEBT.md`).

## Gobernanza y Estándares (Fijos)

- **Local-First (ADR-0016):** 100% Local. El Banco de Pruebas corre siempre contra el backend real de la máquina del propio usuario por FFI; nunca depende de la Cabina de Mando Central del proveedor (Plano #3, ADR-0143) — esa conexión de red es, en el mejor de los casos, un adaptador diferido y fuera del alcance de esta feature.
- **Fidelidad (ADR-0017):** No aplica — el Banco de Pruebas no maneja datos de mercado. Su "fidelidad" propia es de otro tipo: la respuesta mostrada es siempre el dato real devuelto por el backend, nunca una simulación ni un mock (mismo estándar que exige ADR-0117 para toda SVF).
- **Inundación de Fundaciones (ADR-0020):** el Banco de Pruebas no crea tabla propia (ver "Persistencia" abajo), por lo que el Grupo I no aplica a una tabla que no existe. Si en el futuro se decide auditar las corridas del propio harness (quién verificó qué y cuándo), el perfil correspondiente sería **D — Ops/Auditoría** (Identidad + Soberanía + Hardware), mismo perfil que `clock` y `audit-log`, reutilizando la bitácora existente en vez de crear una tabla nueva.
- **Hooks Forenses:** ninguno propio. La evidencia de que una verificación ocurrió es la respuesta real mostrada en pantalla; si una feature específica necesita dejar rastro persistente de sus propias corridas de verificación, esa auditoría vive en la tabla de esa feature, no en el Banco de Pruebas.

## Persistencia (Inundación de Fundamentos — ADR-0020)

**Sin tabla propia.** El Banco de Pruebas no persiste estado — es un despachador y un shell de presentación. Lee y escribe únicamente a través de los puertos y tablas que cada feature verificada ya declara en su propio documento (ej. `sovereign_download_records` para `sovereign-data-fetcher`, las tablas de cada cimiento del substrato para sus respectivas features). Cerrar o reabrir la app no pierde nada del Banco en sí mismo, porque no hay nada propio que guardar.

**Rastro de Evidencia:** el Banco de Pruebas no emite rastro propio hacia `feedback`. La evidencia de causalidad de cada verificación individual la lleva la feature verificada, en su propia tabla, si esa feature declara auditoría de sus propias corridas.

## Dependencias y Bloqueantes

**Depende de:**
- Las funciones de verificación que cada feature con superficie propia ya expone en su `public_interface` (Canal #2, ADR-0142) — el Banco de Pruebas las invoca, nunca las reimplementa.
- El chasis ya nacido en EPIC-0/SPIKE-006: el tab maestro-detalle y el registro extensible del Banco de Verificación (ADR-0117), sobre el que esta feature añade el despachador genérico y la validación visual de forma.

**Bloquea:**
- El cierre de Etapa 5 (QA) de cualquier feature nueva con superficie propia declarada, a partir de ADR-0152: sin su entrada conectada aquí, esa feature no se considera Terminada.

**Consumido por:** Todos los módulos y features con superficie propia — es la realización canónica y permanente del Canal #1 de verificación (SVF) que ADR-0117 exige desde EPIC-0.

**Contrato de Integración UI (OBLIGATORIO — ADR-0117 / ADR-0152):**

- **Superficie propia:** el Banco de Pruebas **ES** el Banco de Verificación del Panel Operativo Fundacional — no es una sección más dentro de él, es el tab-anfitrión mismo (menú lateral + panel de detalle de tres zonas). No tiene, ni necesita, una entrada separada "verification-bench" en su propio menú lateral.
- **Su propia SVF es recursiva:** la prueba de que el harness funciona es que CUALQUIER sección registrada (cualquier feature conectada) dispare una verificación real por FFI y muestre la respuesta real, incluyendo el estado de validez del input. No existe una verificación aparte del harness distinta de verificar cualquiera de sus features conectadas.
- **No duplica SVF ni galería:** la galería de componentes (`ui/lib/gallery/`) muestra widgets con datos mock para explorar el catálogo visual; el Banco de Pruebas dispara operaciones reales por FFI contra el backend real. Comparten el mismo patrón de navegación (registro central + menú lateral + construcción bajo demanda) pero se estratifican por propósito: galería = catálogo visual; Banco de Pruebas = verificación funcional real (memoria `feedback-svf-galeria-transversal`).

---

> **Relación con DEBT-005:** esta feature es el "harness SVF genérico" que DEBT-005 identifica como pieza (a) de la tanda final de UI del substrato. Las piezas (b) — SVF retroactiva de los 14 cimientos — y (c) — arreglo/confirmación de la SVF de `sovereign-data-fetcher` — se pagan conectándolas aquí (TTR-003) y verificando, de paso, si el hallazgo de la auditoría retroactiva (Lote 5, 2026-07-10) sobre `sovereign-data-fetcher` sigue vigente. El cierre formal de DEBT-005 en `docs/DEBT.md` es responsabilidad del Tech-Lead al completarse el sprint (dominio de deuda técnica granular, `.agents/knowledge/base.md` §"Registro Obligatorio de Trabajo Diferido").
